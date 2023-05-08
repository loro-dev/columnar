use crate::args::{Args, FieldArgs};
use syn::{DeriveInput, Generics};
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use super::utils::{add_generics_clause_to_where, is_field_type_is_can_copy};

pub fn generate_derive_vec_row_ser(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_len = field_args.len();
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut impl_generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) =
        process_vec_generics(&generics_params_to_modify, &mut impl_generics, true);

    // generate ser columns
    let mut columns_quote = Vec::with_capacity(fields_len);
    for args in field_args {
        let col = generate_per_field_to_column(args)?;
        columns_quote.push(col);
    }
    // generate ser
    let ser_quote = encode_per_column_to_ser(field_args)?;

    let ret = quote::quote!(
        const _:()={
        use serde::ser::SerializeTuple;
        #[automatically_derived]
        impl #impl_generics ::serde_columnar::RowSer<IT> for #struct_name_ident #ty_generics #where_clause {
            const FIELD_NUM: usize = #fields_len;
            fn serialize_columns<S>(rows: &IT, ser: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                #(#columns_quote)*
                #ser_quote
            }
         }
        };
    );
    Ok(ret)
}

fn process_vec_generics<'a>(
    generics_params_to_modify: &'a Generics,
    impl_generics: &'a mut Generics,
    is_ser: bool,
) -> (ImplGenerics<'a>, TypeGenerics<'a>, WhereClause) {
    let (_, ty_generics, where_clause) = generics_params_to_modify.split_for_impl();
    let (impl_generics, where_clause) = match is_ser {
        true => {
            let where_clause = add_generics_clause_to_where(
                vec![syn::parse_quote! {for<'c> &'c IT: IntoIterator<Item = &'c Self>}],
                where_clause,
            );
            impl_generics.params.push(syn::parse_quote! { IT });
            let (impl_generics, _, _) = impl_generics.split_for_impl();
            (impl_generics, where_clause)
        }
        false => {
            let where_clause = add_generics_clause_to_where(
                vec![syn::parse_quote! {IT: FromIterator<Self> + Clone}],
                where_clause,
            );
            impl_generics.params.push(syn::parse_quote! { 'de });
            impl_generics.params.push(syn::parse_quote! { IT });
            let (impl_generics, _, _) = impl_generics.split_for_impl();
            (impl_generics, where_clause)
        }
    };
    (impl_generics, ty_generics, where_clause)
}

fn generate_per_field_to_column(field_arg: &FieldArgs) -> syn::Result<proc_macro2::TokenStream> {
    if field_arg.skip {
        return Ok(quote::quote! {});
    }
    let field_name = &field_arg.ident;
    let field_type = &field_arg.ty;
    let field_attr_ty = &field_arg._type;
    let index_num = field_arg.index.unwrap();
    let column_index = syn::Ident::new(
        &format!("column{}", index_num),
        proc_macro2::Span::call_site(),
    );
    #[cfg(feature = "compress")]
    let compress_quote = field_arg.compress_args()?;
    let can_copy = is_field_type_is_can_copy(field_arg)?;
    let row_content = if can_copy {
        quote::quote!(row.#field_name)
    } else if field_attr_ty.is_some() {
        match field_attr_ty.as_ref().unwrap_or(&"".to_string()).as_str() {
            "vec" => {
                quote::quote!(::serde_columnar::ColumnarVec::<_, #field_type>::new(&row.#field_name))
            }
            "map" => {
                quote::quote!(::serde_columnar::ColumnarMap::<_, _, #field_type>::new(&row.#field_name))
            }
            _ => return Err(syn::Error::new_spanned(field_attr_ty, "unsupported type")),
        }
    } else {
        quote::quote!(std::borrow::Cow::Borrowed(&row.#field_name))
    };
    let this_ty = if can_copy{
        quote::quote!(#field_type)
    }else{
        quote::quote!(std::borrow::Cow<#field_type>)
    };
    let column_type_token = field_arg.get_strategy_column(this_ty)?;
    #[cfg(feature = "compress")]
    let column_content_token = if field_arg.strategy.is_none() {
        quote::quote!()
    } else {
        quote::quote!(let #column_index = 
            #column_type_token::new(
            #column_index,
            ::serde_columnar::ColumnAttr{
                index: #index_num,
                // strategy: #strategy,
                
                compress: #compress_quote,
            }
        );)
        
    };
    #[cfg(not(feature = "compress"))]
    let column_content_token = if field_arg.strategy.is_none() {
        quote::quote!()
    } else {
        quote::quote!(let #column_index = 
            #column_type_token::new(
            #column_index,
            ::serde_columnar::ColumnAttr{
                index: #index_num,
            }
        );)
        
    };

    let ret = quote::quote!(
        let #column_index = rows.into_iter().map(
            |row| #row_content
        ).collect::<::std::vec::Vec<_>>();

        #column_content_token

    );
    Ok(ret)
}

fn encode_per_column_to_ser(field_args: &Vec<FieldArgs>) -> syn::Result<proc_macro2::TokenStream> {
    let mut field_len = field_args.len();
    let mut ser_elements = Vec::with_capacity(field_len);
    for field_arg in field_args.iter() {
        if field_arg.skip {
            field_len -= 1;
            continue;
        }
        let index = field_arg.index.unwrap();
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

// Deserialize
pub fn generate_derive_vec_row_de(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_len = field_args.len();
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut impl_generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) =
        process_vec_generics(&generics_params_to_modify, &mut impl_generics, false);
    // generate de columns
    let de = generate_per_column_to_de_columns(field_args)?;

    let ret = quote::quote!(
        const _:()={
        use serde::ser::SerializeTuple;
        #[automatically_derived]
        impl #impl_generics ::serde_columnar::RowDe<'de, IT> for #struct_name_ident #ty_generics #where_clause {
            const FIELD_NUM: usize = #fields_len;
            fn deserialize_columns<D>(de: D) -> Result<IT, D::Error>
            where
                D: serde::Deserializer<'de>
            {
                #de
            }
         }
        };
    );
    Ok(ret)
}

fn generate_per_column_to_de_columns(
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let field_len = field_args.len();

    let mut columns_quote = Vec::with_capacity(field_len);
    let mut columns_types = Vec::with_capacity(field_len);
    let mut into_iter_quote = Vec::with_capacity(field_len);
    let mut field_names = Vec::with_capacity(field_len);
    let mut field_names_build = Vec::with_capacity(field_len);
    for (_, args) in field_args.iter().enumerate() {
        let field_name = &args.ident;
        if args.skip {
            field_names_build.push(quote::quote!(
                #field_name: ::std::default::Default::default()
            ));
            continue;
        }
        let field_attr_ty = &args._type;
        let index = args.index.unwrap();
        let column_index =
            syn::Ident::new(&format!("column{}", index), proc_macro2::Span::call_site());
        columns_quote.push(quote::quote!(#column_index));
        let field_type = &args.ty;
        let is_num = is_field_type_is_can_copy(args)?;
        let column_type_token = args.get_strategy_column(quote::quote!(#field_type))?;
        let row_content = if is_num {
            column_type_token
        } else if field_attr_ty.is_some() {
            match field_attr_ty.as_ref().unwrap_or(&"".to_string()).as_str() {
                "vec" => args.get_strategy_column(
                    quote::quote!(::serde_columnar::ColumnarVec<_, #field_type>),
                )?,
                "map" => {
                    args.get_strategy_column(
                        quote::quote!(::serde_columnar::ColumnarMap<_, _, #field_type>),
                    )?
                    // quote::quote!(::serde_columnar::Column<::serde_columnar::ColumnarMap<_, _, #field_type>>)
                }
                _ => return Err(syn::Error::new_spanned(field_attr_ty, "unsupported type")),
            }
        } else {
            args.get_strategy_column(quote::quote!(::std::borrow::Cow<#field_type>))?
            // quote::quote!(::serde_columnar::Column<::std::borrow::Cow<#field_type>>)
        };
        columns_types.push(row_content);
        let into_element = if args.strategy.is_none() {
            quote::quote!(#column_index.into_iter())
        } else {
            quote::quote!(
                #column_index.data.into_iter()
            )
        };

        into_iter_quote.push(into_element);

        field_names.push(field_name);
        let field_name_build = if is_num {
            quote::quote!(
                #field_name: #field_name
            )
        } else if field_attr_ty.is_some() {
            match field_attr_ty.as_ref().unwrap_or(&"".to_string()).as_str() {
                "vec" => {
                    quote::quote!(#field_name: #field_name.into_vec())
                }
                "map" => {
                    quote::quote!(#field_name: #field_name.into_map())
                }
                _ => return Err(syn::Error::new_spanned(field_attr_ty, "unsupported type")),
            }
        } else {
            quote::quote!(
                #field_name: #field_name.into_owned()
            )
        };

        field_names_build.push(field_name_build);
    }

    // generate
    let ret = quote::quote!(
        let (#(#columns_quote),*):(#(#columns_types),*) = serde::de::Deserialize::deserialize(de)?;
        let ans = ::serde_columnar::izip!(#(#into_iter_quote),*)
                    .map(|(#(#field_names),*)| Self{
                        #(#field_names_build),*
                    }).collect();
        Ok(ans)
    );
    Ok(ret)
}
