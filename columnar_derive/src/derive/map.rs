use crate::args::{Args, FieldArgs};
use syn::{DeriveInput, Generics};
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use super::utils::{add_generics_clause_to_where, is_field_type_is_can_copy, process_strategy};

pub fn generate_derive_hashmap_row_ser(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_len = field_args.len();
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut impl_generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) =
        process_map_generics(&generics_params_to_modify, &mut impl_generics, true);

    // generate ser columns
    let columns = generate_with_map_per_columns(field_args)?;
    let ser_quote = encode_map_per_column_to_ser(field_args)?;
    // let de_columns = generate_map_per_column_to_de_columns(field_args)?;

    let ret = quote::quote!(
        const _:()={
            use serde::ser::SerializeTuple;
            use columnar::MultiUnzip;
            #[automatically_derived]
            impl #impl_generics ::columnar::KeyRowSer<K, IT> for #struct_name_ident #ty_generics #where_clause {
                const FIELD_NUM: usize = #fields_len;
                fn serialize_columns<S>(rows: &IT, ser: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    #columns
                    #ser_quote
                }
            }
        };
    );
    Ok(ret)
}

pub fn generate_derive_hashmap_row_de(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_len = field_args.len();
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut impl_generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) =
        process_map_generics(&generics_params_to_modify, &mut impl_generics, false);

    let de_columns = generate_map_per_column_to_de_columns(field_args)?;

    let ret = quote::quote!(
        const _:()={
            use serde::ser::SerializeTuple;
            #[automatically_derived]
            impl #impl_generics ::columnar::KeyRowDe<'de, K, IT> for #struct_name_ident #ty_generics #where_clause {
                const FIELD_NUM: usize = #fields_len;
                fn deserialize_columns<D>(de: D) -> Result<IT, D::Error>
                where D: serde::Deserializer<'de>{
                    #de_columns
                }
            }
        };
    );
    Ok(ret)
}

fn process_map_generics<'a>(
    generics_params_to_modify: &'a Generics,
    impl_generics: &'a mut Generics,
    is_ser: bool,
) -> (ImplGenerics<'a>, TypeGenerics<'a>, WhereClause) {
    let (_, ty_generics, where_clause) = generics_params_to_modify.split_for_impl();
    let (impl_generics, where_clause) = match is_ser {
        true => {
            let where_clause = add_generics_clause_to_where(
                vec![
                    syn::parse_quote! {for<'c> &'c IT: IntoIterator<Item = (&'c K, &'c Self)>},
                    syn::parse_quote! {K: Serialize + Eq + Clone},
                ],
                where_clause,
            );
            impl_generics.params.push(syn::parse_quote! { K });
            impl_generics.params.push(syn::parse_quote! { IT });
            let (impl_generics, _, _) = impl_generics.split_for_impl();
            (impl_generics, where_clause)
        }
        false => {
            let where_clause = add_generics_clause_to_where(
                vec![
                    syn::parse_quote! {IT: FromIterator<(K, Self)> + Clone},
                    syn::parse_quote! {K: Deserialize<'de> + Eq + Clone},
                ],
                where_clause,
            );
            impl_generics.params.push(syn::parse_quote! { 'de });
            impl_generics.params.push(syn::parse_quote! { K });
            impl_generics.params.push(syn::parse_quote! { IT });
            let (impl_generics, _, _) = impl_generics.split_for_impl();
            (impl_generics, where_clause)
        }
    };
    (impl_generics, ty_generics, where_clause)
}

fn generate_with_map_per_columns(
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut columns_quote = Vec::with_capacity(field_args.len());
    let mut columns_types = Vec::with_capacity(field_args.len());
    let mut cow_columns_fields = Vec::with_capacity(field_args.len());
    let mut real_columns = Vec::with_capacity(field_args.len());

    for args in field_args.iter() {
        if args.skip {
            continue;
        }
        let field_name = &args.ident;
        let field_type = &args.ty;
        let field_attr_ty = &args._type;
        let ori_ty = &args.original_type;
        let compress_quote = &args.compress_args()?;
        let strategy = process_strategy(&args.strategy, field_type, ori_ty)?;
        let index = args.index.unwrap();
        let index_num = syn::LitInt::new(&index.to_string(), proc_macro2::Span::call_site());
        let column_index =
            syn::Ident::new(&format!("column{}", index), proc_macro2::Span::call_site());
        columns_quote.push(quote::quote!(#column_index));
        let columns_type = quote::quote!(::std::vec::Vec<_>);
        columns_types.push(columns_type);
        let cow_columns_field = if is_field_type_is_can_copy(args)? {
            quote::quote!(v.#field_name)
        } else if field_attr_ty.is_some() {
            match field_attr_ty.as_ref().unwrap_or(&"".to_string()).as_str() {
                "vec" => {
                    quote::quote!(::columnar::ColumnarVec::<_, #field_type>::new(&v.#field_name))
                }
                "map" => {
                    quote::quote!(::columnar::ColumnarMap::<_, _, #field_type>::new(&v.#field_name))
                }
                _ => return Err(syn::Error::new_spanned(field_attr_ty, "unsupported type")),
            }
        } else {
            quote::quote!(::std::borrow::Cow::Borrowed(&v.#field_name))
        };
        cow_columns_fields.push(cow_columns_field);

        // real columns
        real_columns.push(quote::quote!(
            let #column_index = ::columnar::Column::new(
                #column_index,
                ::columnar::ColumnAttr{
                    index: #index_num,
                    strategy: #strategy,
                    compress: #compress_quote,
                }
            );
        ));
    }

    let mut ret = quote::quote!(
        let (vec_k, #(#columns_quote),*): (::std::vec::Vec<_>, #(#columns_types),*) = rows
        .into_iter()
        .map(|(k, v)| (::std::borrow::Cow::Borrowed(k), #(#cow_columns_fields),*))
        .multiunzip();
    );
    ret.extend(real_columns);
    Ok(ret)
}

fn encode_map_per_column_to_ser(
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut field_len = field_args.len();
    let mut ser_elements = Vec::with_capacity(field_len);
    for args in field_args {
        if args.skip {
            field_len -= 1;
            continue;
        }
        let index = args.index.unwrap();
        let column_index =
            syn::Ident::new(&format!("column{}", index), proc_macro2::Span::call_site());
        let ser_element = quote::quote!(
            seq_encoder.serialize_element(&#column_index)?;
        );
        ser_elements.push(ser_element);
    }

    let ret = quote::quote!(
        let mut seq_encoder = ser.serialize_tuple(#field_len + 1)?;
        seq_encoder.serialize_element(&vec_k)?;
        #(#ser_elements)*
        seq_encoder.end()
    );
    Ok(ret)
}

fn generate_map_per_column_to_de_columns(
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let field_len = field_args.len();
    let mut columns_quote = Vec::with_capacity(field_len);
    let mut columns_types = Vec::with_capacity(field_len);
    let mut field_names = Vec::with_capacity(field_len);
    let mut field_names_build = Vec::with_capacity(field_len);
    let mut into_iter_quote = Vec::with_capacity(field_len);
    for (_, args) in field_args.iter().enumerate() {
        let field_name = &args.ident;
        // TODO: no named struct
        if args.skip {
            field_names_build.push(quote::quote!(#field_name: ::std::default::Default::default()));
            continue;
        }
        let field_type = &args.ty;
        let field_attr_ty = &args._type;
        let index = args.index.unwrap();
        let column_index =
            syn::Ident::new(&format!("column{}", index), proc_macro2::Span::call_site());
        columns_quote.push(quote::quote!(#column_index));
        field_names.push(quote::quote!(#field_name));
        let is_num = is_field_type_is_can_copy(args)?;
        let column_type = if is_num {
            quote::quote!(::columnar::Column<#field_type>)
        } else if field_attr_ty.is_some() {
            match field_attr_ty.as_ref().unwrap_or(&"".to_string()).as_str() {
                "vec" => {
                    quote::quote!(::columnar::Column<::columnar::ColumnarVec<_, #field_type>>)
                }
                "map" => {
                    quote::quote!(::columnar::Column<::columnar::ColumnarMap<_, _, #field_type>>)
                }
                _ => return Err(syn::Error::new_spanned(field_attr_ty, "unsupported type")),
            }
        } else {
            quote::quote!(::columnar::Column<::std::borrow::Cow<#field_type>>)
        };
        columns_types.push(column_type);

        let field_name_build = if is_num {
            quote::quote!(#field_name: #field_name)
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
            quote::quote!(#field_name: #field_name.into_owned())
        };
        field_names_build.push(field_name_build);

        let into_element = quote::quote!(
            #column_index.data.into_iter()
        );
        into_iter_quote.push(into_element);
    }

    let ret = quote::quote!(
        let (vec_k, #(#columns_quote),*): (::std::vec::Vec<_>, #(#columns_types),*) =
            ::serde::de::Deserialize::deserialize(de)?;
        let ans: ::std::vec::Vec<_> = ::columnar::izip!(#(#into_iter_quote),*)
            .map(|(#(#field_names),*)| Self{
                #(#field_names_build),*
            }).collect();
        let ans = vec_k.into_iter().zip(ans).collect();
        Ok(ans)
    );
    Ok(ret)
}
