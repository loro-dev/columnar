use crate::args::{Args, FieldArgs};
use syn::{DeriveInput, Generics};
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use super::utils::{add_generics_clause_to_where, generate_generics_phantom};

pub fn generate_derive_hashmap_row_ser(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut impl_generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = process_map_generics(
        struct_name_ident,
        &generics_params_to_modify,
        &mut impl_generics,
        true,
    );

    // generate ser columns
    let columns = generate_with_map_per_columns(field_args)?;
    let ser_quote = encode_map_per_column_to_ser(field_args)?;

    let ret = quote::quote!(
        const _:()={
            use ::serde::ser::Error;
            use ::serde::ser::SerializeSeq;
            use ::serde_columnar::MultiUnzip;
            use ::serde_columnar::ColumnTrait;
            #[automatically_derived]
            impl #impl_generics ::serde_columnar::KeyRowSer<__K, __IT> for #struct_name_ident #ty_generics #where_clause {

                fn serialize_columns<__S>(rows: &__IT, ser: __S) -> std::result::Result<__S::Ok, __S::Error>
                where
                    __S: ::serde::Serializer,
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
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut impl_generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = process_map_generics(
        struct_name_ident,
        &generics_params_to_modify,
        &mut impl_generics,
        false,
    );
    let mut generics_params_add_it_k = input.generics.clone();
    generics_params_add_it_k
        .params
        .push(syn::parse_quote! { __IT });
    generics_params_add_it_k
        .params
        .push(syn::parse_quote! { __K });
    let (_, visitor_ty_generics, _) = generics_params_add_it_k.split_for_impl();
    let phantom_data_fields = generate_generics_phantom(&generics_params_add_it_k);

    let de_columns = generate_map_per_column_to_de_columns(field_args, input)?;

    let ret = quote::quote!(
        const _:()={
           use ::serde::de::Error as DeError;
            use ::serde::de::Visitor;
            use ::std::collections::HashMap;
            use ::serde_columnar::ColumnTrait;
            #[automatically_derived]
            impl #impl_generics ::serde_columnar::KeyRowDe<'__de, __K, __IT> for #struct_name_ident #ty_generics #where_clause {
                fn deserialize_columns<__D>(de: __D) -> Result<__IT, __D::Error>
                where __D: ::serde::Deserializer<'__de>{
                    struct DeVisitor #visitor_ty_generics ((#phantom_data_fields));
                    impl #impl_generics Visitor<'__de> for DeVisitor #visitor_ty_generics #where_clause{
                        type Value = __IT;
                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("Map de")
                        }

                        fn visit_seq<__A>(self, mut seq: __A) -> Result<Self::Value, __A::Error>
                        where
                            __A: ::serde::de::SeqAccess<'__de>,
                        {
                            #de_columns
                        }
                    }
                    let visitor = DeVisitor(Default::default());
                    de.deserialize_seq(visitor)
                }
            }
        };
    );
    Ok(ret)
}

fn process_map_generics<'a>(
    struct_name: &proc_macro2::Ident,
    generics_params_to_modify: &'a Generics,
    impl_generics: &'a mut Generics,
    is_ser: bool,
) -> (ImplGenerics<'a>, TypeGenerics<'a>, WhereClause) {
    let (_, ty_generics, where_clause) = generics_params_to_modify.split_for_impl();
    let (impl_generics, where_clause) = match is_ser {
        true => {
            let where_clause = add_generics_clause_to_where(
                vec![
                    syn::parse_quote! {for<'c> &'c __IT: IntoIterator<Item = (&'c __K, &'c #struct_name #ty_generics)>},
                    syn::parse_quote! {__K: ::serde::ser::Serialize + Eq + Clone},
                ],
                where_clause,
            );
            impl_generics.params.push(syn::parse_quote! { __K });
            impl_generics.params.push(syn::parse_quote! { __IT });
            let (impl_generics, _, _) = impl_generics.split_for_impl();
            (impl_generics, where_clause)
        }
        false => {
            let where_clause = add_generics_clause_to_where(
                vec![
                    syn::parse_quote! {__IT: FromIterator<(__K, #struct_name #ty_generics)> + Clone},
                    syn::parse_quote! {__K: ::serde::de::Deserialize<'__de> + Eq + Clone},
                ],
                where_clause,
            );
            impl_generics.params.push(syn::parse_quote! { '__de });
            impl_generics.params.push(syn::parse_quote! { __K });
            impl_generics.params.push(syn::parse_quote! { __IT });
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
        // if args.skip {
        //     continue;
        // }
        let field_name = &args.ident;
        let field_type = &args.ty;
        let field_attr_ty = &args.class;
        #[cfg(feature = "compress")]
        let compress_quote = &args.compress_args()?;
        let column_name = syn::Ident::new(
            &format!("column_{}", field_name.as_ref().unwrap()),
            proc_macro2::Span::call_site(),
        );
        columns_quote.push(quote::quote!(#column_name));
        let columns_type = quote::quote!(::std::vec::Vec<_>);
        columns_types.push(columns_type);
        let can_copy = args.strategy == Some("DeltaRle".to_string())
            || args.strategy == Some("BoolRle".to_string()); //is_field_type_is_can_copy(args)?;
        let cow_columns_field = if can_copy {
            quote::quote!(v.#field_name)
        } else if field_attr_ty.is_some() {
            match field_attr_ty.as_ref().unwrap_or(&"".to_string()).as_str() {
                "vec" => {
                    quote::quote!(::serde_columnar::ColumnarVec::<_, #field_type>::new(&v.#field_name))
                }
                "map" => {
                    quote::quote!(::serde_columnar::ColumnarMap::<_, _, #field_type>::new(&v.#field_name))
                }
                _ => return Err(syn::Error::new_spanned(field_attr_ty, "unsupported type")),
            }
        } else {
            quote::quote!(::std::borrow::Cow::Borrowed(&v.#field_name))
        };
        cow_columns_fields.push(cow_columns_field);
        let this_ty = if can_copy {
            quote::quote!(#field_type)
        } else {
            quote::quote!(std::borrow::Cow<#field_type>)
        };
        // real columns
        let column_type_token = args.get_strategy_column(this_ty)?;
        #[cfg(feature = "compress")]
        let column_content_token = quote::quote!(let #column_name = #column_type_token::new(
                #column_name,
                ::serde_columnar::ColumnAttr{
                    index: None,
                    compress: #compress_quote
                }
            ););
        #[cfg(not(feature = "compress"))]
        let column_content_token = quote::quote!(let #column_name = #column_type_token::new(
                #column_name,
                ::serde_columnar::ColumnAttr{
                    // TODO: index
                    index: None,
                }
            ););

        real_columns.push(column_content_token);
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
    let field_len = field_args.len();
    let mut ser_elements = Vec::with_capacity(field_len);
    for args in field_args {
        let field_name = &args.ident;
        let optional = args.optional;
        let index = args.index;
        let column_index = syn::Ident::new(
            &format!("column_{}", field_name.as_ref().unwrap()),
            proc_macro2::Span::call_site(),
        );
        let ser_element = if !optional {
            quote::quote!(
                seq_encoder.serialize_element(&#column_index)?;
            )
        } else {
            let index = index.unwrap();
            quote::quote!(
                {
                    let bytes = ::postcard::to_allocvec(&#column_index).map_err(__S::Error::custom)?;
                    seq_encoder.serialize_element(&(#index , bytes))?;
                }

            )
        };
        ser_elements.push(ser_element);
    }

    let ret = quote::quote!(
        let mut seq_encoder = ser.serialize_seq(Some(#field_len + 1))?;
        seq_encoder.serialize_element(&vec_k)?;
        #(#ser_elements)*
        seq_encoder.end()
    );
    Ok(ret)
}

fn generate_map_per_column_to_de_columns(
    field_args: &Vec<FieldArgs>,
    input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let field_len = field_args.len();
    let mut add_mapping = false;
    let mut elements = Vec::with_capacity(field_len);
    let mut columns_quote = Vec::with_capacity(field_len);
    let mut columns_types = Vec::with_capacity(field_len);
    let mut field_names = Vec::with_capacity(field_len);
    let mut field_names_build = Vec::with_capacity(field_len);
    let mut into_iter_quote = Vec::with_capacity(field_len);
    for (_, args) in field_args.iter().enumerate() {
        let field_name = &args.ident;
        let optional = args.optional;
        let index = args.index;
        let field_type = &args.ty;
        let class = &args.class;
        // TODO: no named struct
        // if args.skip {
        //     field_names_build.push(quote::quote!(#field_name: ::std::default::Default::default()));
        //     continue;
        // }
        let column_index = syn::Ident::new(
            &format!("column_{}", field_name.as_ref().unwrap()),
            proc_macro2::Span::call_site(),
        );
        columns_quote.push(quote::quote!(#column_index));
        field_names.push(quote::quote!(#field_name));
        let is_num = args.strategy == Some("DeltaRle".to_string())
            || args.strategy == Some("BoolRle".to_string()); //is_field_type_is_can_copy(args)?;
        let column_type = if is_num {
            args.get_strategy_column(quote::quote!(#field_type))?
        } else if class.is_some() {
            match class.as_ref().unwrap_or(&"".to_string()).as_str() {
                "vec" => {
                    args.get_strategy_column(
                        quote::quote!(::serde_columnar::ColumnarVec<_, #field_type>),
                    )?
                    // quote::quote!(::serde_columnar::Column<::serde_columnar::ColumnarVec<_, #field_type>>)
                }
                "map" => {
                    args.get_strategy_column(
                        quote::quote!(::serde_columnar::ColumnarMap<_, _, #field_type>),
                    )?
                    // quote::quote!(::serde_columnar::Column<::serde_columnar::ColumnarMap<_, _, #field_type>>)
                }
                _ => return Err(syn::Error::new_spanned(class, "unsupported type")),
            }
        } else {
            args.get_strategy_column(quote::quote!(::std::borrow::Cow<#field_type>))?
        };

        let q = if !optional {
            quote::quote!(
                let #column_index: #column_type = seq.next_element()?.ok_or_else(||__A::Error::custom("DeserializeUnexpectedEnd"))?;
                column_data_len = ::std::cmp::max(column_data_len, #column_index.len());
            )
        } else {
            if !add_mapping {
                elements.push(quote::quote!(
                    let mut mapping = HashMap::new();
                    while let Ok(Some((index, bytes))) = seq.next_element::<(usize, Vec<u8>)>() {
                        // ignore
                        mapping.insert(index, bytes);
                    }
                ));
                add_mapping = true;
            }
            let index = index.unwrap();
            quote::quote!(
                let #column_index: #column_type = if let Some(bytes) = mapping.remove(&#index){
                    postcard::from_bytes(&bytes).map_err(__A::Error::custom)?
                }else{
                    vec![Default::default(); column_data_len].into()
                };
            )
        };
        elements.push(q);

        columns_types.push(column_type);

        let field_name_build = if is_num {
            quote::quote!(#field_name: #field_name)
        } else if class.is_some() {
            match class.as_ref().unwrap_or(&"".to_string()).as_str() {
                "vec" => {
                    quote::quote!(#field_name: #field_name.into_vec())
                }
                "map" => {
                    quote::quote!(#field_name: #field_name.into_map())
                }
                _ => return Err(syn::Error::new_spanned(class, "unsupported type")),
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
        let mut column_data_len: usize = 0;
        let vec_k: ::std::vec::Vec<_> = seq.next_element()?.ok_or_else(||__A::Error::custom("DeserializeUnexpectedEnd"))?;
        #(#elements)*;
        let ans: ::std::vec::Vec<_> = ::serde_columnar::izip!(#(#into_iter_quote),*)
            .map(|(#(#field_names),*)| #struct_name{
                #(#field_names_build),*
            }).collect();
        let ans = vec_k.into_iter().zip(ans).collect();
        Ok(ans)
    );
    Ok(ret)
}
