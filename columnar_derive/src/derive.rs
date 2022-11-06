use darling::{Error as DarlingError, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn};

use crate::attr::FieldArgs;

#[derive(Debug, FromMeta)]
pub struct DeriveArgs {
    #[darling(default)]
    vec: bool,
    #[darling(rename = "map", default)]
    hashmap: bool,
}

impl DeriveArgs {
    pub fn derive_vec(&self, input: &TokenStream) -> syn::Result<Option<TokenStream>> {
        if !self.vec {
            return Ok(None);
        }
        todo!()
    }
}

pub fn process_derive_args(args: &AttributeArgs) -> Result<proc_macro2::TokenStream, DarlingError> {
    let derive_args = match DeriveArgs::from_list(args) {
        Ok(v) => v,
        Err(_) => {
            return Err(DarlingError::custom(
                "columnar only supports attributes with `vec` and `map`",
            ))
        }
    };
    Ok(quote::quote!())
}

pub fn generate_derive_vec_row_for_struct(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_len = field_args.len();
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics_params_to_modify.split_for_impl();
    let where_clause = add_it_clause_to_where(where_clause);

    // generate ser columns
    let mut columns_quote = Vec::with_capacity(fields_len);
    for args in field_args {
        let col = generate_per_field_to_column(&args)?;
        columns_quote.push(col);
    }
    // generate ser
    let ser_quote = encode_per_column_to_ser(&field_args)?;

    // generate de columns
    let de_columns = generate_per_column_to_de_columns(&field_args)?;

    let ret = quote::quote!(
        const _:()={
            use serde::ser::SerializeTuple;

        impl<IT> #impl_generics ::columnar::VecRow<IT> for #struct_name_ident #ty_generics #where_clause {
            const FIELD_NUM: usize = #fields_len;
            fn serialize_columns<S>(rows: &IT, ser: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                #(#columns_quote)*
                #ser_quote
            }

            fn deserialize_columns<'de, D>(de: D) -> std::result::Result<IT, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                #de_columns
            }
         }
        };
    );
    Ok(ret)
}

fn add_it_clause_to_where(where_clause: Option<&syn::WhereClause>) -> syn::WhereClause {
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    let it_clause = syn::parse_quote!(IT: FromIterator<Self> + Clone);
    let into_clause = syn::parse_quote!(for<'c> &'c IT: IntoIterator<Item = &'c Self>);
    where_clause.predicates.push(it_clause);
    where_clause.predicates.push(into_clause);
    where_clause
}

fn generate_per_field_to_column(field_arg: &FieldArgs) -> syn::Result<proc_macro2::TokenStream> {
    let field_name = &field_arg.ident;
    let field_type = &field_arg.ty;
    let strategy = process_strategy(&field_arg.strategy)?;
    let index_num = field_arg.index.unwrap();
    let column_index = syn::Ident::new(
        &format!("column{}", index_num),
        proc_macro2::Span::call_site(),
    );
    let ret = quote::quote!(
        let #column_index = rows.into_iter().map(
            |row| std::borrow::Cow::Borrowed(&row.#field_name)
        ).collect::<Vec<_>>();
        let #column_index = ::columnar::Column::new(
            #column_index,
            ::columnar::ColumnAttr{
                index: #index_num,
                strategy: #strategy,
            }
        );
    );
    Ok(ret)
}

fn process_strategy(strategy: &Option<String>) -> syn::Result<proc_macro2::TokenStream> {
    if let Some(strategy) = strategy {
        let strategy = syn::Ident::new(&strategy, proc_macro2::Span::call_site());
        let ret = quote::quote!(
            std::option::Option::Some(
                ::columnar::Strategy::#strategy
            )
        );
        Ok(ret)
    } else {
        Ok(quote::quote!(std::option::Option::None))
    }
}

fn encode_per_column_to_ser(field_args: &Vec<FieldArgs>) -> syn::Result<proc_macro2::TokenStream> {
    let field_len = field_args.len();
    let indexes = field_args.iter().map(|args| args.index.unwrap());
    let mut ser_elements = Vec::with_capacity(field_len);
    for index in indexes {
        let column_index =
            syn::Ident::new(&format!("column{}", index), proc_macro2::Span::call_site());
        let ser_element = quote::quote!(
            seq_encoder.serialize_element(&#column_index)?;
        );
        ser_elements.push(ser_element);
    }

    let ret = quote::quote!(
        let mut seq_encoder = ser.serialize_tuple(#field_len)?;
        #(#ser_elements)*
        seq_encoder.end()
    );
    Ok(ret)
}

fn generate_per_column_to_de_columns(
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let field_len = field_args.len();
    // let indexes = field_args.iter().map(|args| args.index.unwrap());

    let mut columns_quote = Vec::with_capacity(field_len);
    let mut columns_types = Vec::with_capacity(field_len);
    let mut into_iter_quote = Vec::with_capacity(field_len);
    let mut field_names = Vec::with_capacity(field_len);
    let mut field_names_build = Vec::with_capacity(field_len);
    for (idx, args) in field_args.iter().enumerate() {
        let index = args.index.unwrap();
        let column_index =
            syn::Ident::new(&format!("column{}", index), proc_macro2::Span::call_site());
        columns_quote.push(quote::quote!(#column_index));
        let field_type = &args.ty;
        columns_types.push(quote::quote!(::columnar::Column<::std::borrow::Cow<#field_type>>));
        let into_element = if idx == 0 {
            quote::quote!(
                #column_index.data.into_iter()
            )
        } else {
            quote::quote!(
                .zip(#column_index.data.into_iter())
            )
        };
        into_iter_quote.push(into_element);

        let field_name = &args.ident;
        field_names.push(field_name);
        field_names_build.push(quote::quote!(
            #field_name: #field_name.into_owned()
        ));
    }

    // generate
    let ret = quote::quote!(
        let (#(#columns_quote),*):(#(#columns_types),*) = serde::de::Deserialize::deserialize(de)?;
        let ans = #(#into_iter_quote)*
                    .map(|(#(#field_names),*)| Self{
                        #(#field_names_build),*
                    }).collect();
        Ok(ans)
    );
    Ok(ret)
}
