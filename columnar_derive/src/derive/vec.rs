use crate::args::{Args, FieldArgs};
use syn::{DeriveInput, Generics};
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use super::utils::{add_generics_clause_to_where, generate_generics_phantom};

pub fn generate_derive_vec_row_ser(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_len = field_args.len();
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut impl_generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = process_vec_generics(
        struct_name_ident,
        &generics_params_to_modify,
        &mut impl_generics,
        true,
    );

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
            use ::serde::ser::Error;
            use ::serde::ser::SerializeSeq;
            #[automatically_derived]
            impl #impl_generics ::serde_columnar::RowSer<__IT> for #struct_name_ident #ty_generics #where_clause {
                fn serialize_columns<S>(rows: &__IT, ser: S) -> std::result::Result<S::Ok, S::Error>
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
                    syn::parse_quote! {for<'c> &'c __IT: IntoIterator<Item = &'c #struct_name #ty_generics>},
                ],
                where_clause,
            );
            impl_generics.params.push(syn::parse_quote! { __IT });
            let (impl_generics, _, _) = impl_generics.split_for_impl();
            (impl_generics, where_clause)
        }
        false => {
            let where_clause = add_generics_clause_to_where(
                vec![syn::parse_quote! {__IT: FromIterator<#struct_name #ty_generics> + Clone}],
                where_clause,
            );
            impl_generics.params.push(syn::parse_quote! { '__de });
            impl_generics.params.push(syn::parse_quote! { __IT });
            let (impl_generics, _, _) = impl_generics.split_for_impl();
            (impl_generics, where_clause)
        }
    };
    (impl_generics, ty_generics, where_clause)
}

fn generate_per_field_to_column(field_arg: &FieldArgs) -> syn::Result<proc_macro2::TokenStream> {
    // if field_arg.skip {
    //     return Ok(quote::quote! {});
    // }
    let field_name = &field_arg.ident;
    let field_type = &field_arg.ty;
    let field_attr_ty = &field_arg.type_;
    let column_name = syn::Ident::new(
        &format!("column_{}", field_name.as_ref().unwrap()),
        proc_macro2::Span::call_site(),
    );
    #[cfg(feature = "compress")]
    let compress_quote = field_arg.compress_args()?;
    let can_copy = field_arg.strategy == Some("DeltaRle".to_string())
        || field_arg.strategy == Some("BoolRle".to_string()); //is_field_type_is_can_copy(field_arg)?;
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
    let this_ty = if can_copy {
        quote::quote!(#field_type)
    } else {
        quote::quote!(std::borrow::Cow<#field_type>)
    };
    let column_type_token = field_arg.get_strategy_column(this_ty)?;
    #[cfg(feature = "compress")]
    let column_content_token = if field_arg.strategy.is_none() {
        quote::quote!()
    } else {
        quote::quote!(let #column_name = 
            #column_type_token::new(
            #column_name,
            ::serde_columnar::ColumnAttr{
                index: None,
                // strategy: #strategy,
                
                compress: #compress_quote,
            }
        );)
    };
    #[cfg(not(feature = "compress"))]
    let column_content_token = if field_arg.strategy.is_none() {
        quote::quote!()
    } else {
        quote::quote!(let #column_name = 
            #column_type_token::new(
            #column_name,
            ::serde_columnar::ColumnAttr{
                index: None,
            }
        );)
    };

    let ret = quote::quote!(
        let #column_name = rows.into_iter().map(
            |row| #row_content
        ).collect::<::std::vec::Vec<_>>();

        #column_content_token
    );
    Ok(ret)
}

fn encode_per_column_to_ser(field_args: &Vec<FieldArgs>) -> syn::Result<proc_macro2::TokenStream> {
    let field_len = field_args.len();
    let mut ser_elements = Vec::with_capacity(field_len);
    for args in field_args.iter() {
        let field_name = &args.ident;
        let optional = args.optional;
        let index = args.index;
        // TODO: feat skip
        // if args.skip {
        //     field_len -= 1;
        //     continue;
        // }
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
                {let bytes = ::postcard::to_allocvec(&#column_index).map_err(S::Error::custom)?;
                seq_encoder.serialize_element(&(#index , bytes))?;}
            )
        };
        ser_elements.push(ser_element);
    }

    let ret = quote::quote!(
        let mut seq_encoder = ser.serialize_seq(Some(#field_len))?;
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
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut impl_generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = process_vec_generics(
        struct_name_ident,
        &generics_params_to_modify,
        &mut impl_generics,
        false,
    );
    let mut generics_params_add_it = input.generics.clone();
    generics_params_add_it
        .params
        .push(syn::parse_quote! { __IT });
    let (_, visitor_ty_generics, _) = generics_params_add_it.split_for_impl();
    let phantom_data_fields = generate_generics_phantom(&generics_params_add_it);
    // generate de columns
    let de = generate_per_column_to_de_columns(field_args, input)?;

    let ret = quote::quote!(
        const _:()={
            use ::serde::de::Error as DeError;
            use ::serde::de::Visitor;
            use ::std::collections::HashMap;
            #[automatically_derived]
            impl #impl_generics ::serde_columnar::RowDe<'__de, __IT> for #struct_name_ident #ty_generics #where_clause {
                fn deserialize_columns<__D>(de: __D) -> Result<__IT, __D::Error>
                where
                    __D: serde::Deserializer<'__de>
                {
                    struct DeVisitor #visitor_ty_generics ((#phantom_data_fields));
                    impl #impl_generics Visitor<'__de> for DeVisitor #visitor_ty_generics #where_clause{
                        type Value = __IT;
                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("Vec de")
                        }

                        fn visit_seq<__A>(self, mut seq: __A) -> Result<Self::Value, __A::Error>
                        where
                            __A: ::serde::de::SeqAccess<'__de>,
                        {
                            #de
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

fn generate_per_column_to_de_columns(
    field_args: &Vec<FieldArgs>,
    input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let field_len = field_args.len();
    let mut elements = Vec::with_capacity(field_len);
    let mut add_mapping = false;
    let mut columns_quote = Vec::with_capacity(field_len);
    let mut columns_types = Vec::with_capacity(field_len);
    let mut into_iter_quote = Vec::with_capacity(field_len);
    let mut field_names = Vec::with_capacity(field_len);
    let mut field_names_build = Vec::with_capacity(field_len);
    for (_, args) in field_args.iter().enumerate() {
        let field_name = &args.ident;
        let optional = args.optional;
        let index = args.index;
        let field_type = &args.ty;
        let class = &args.type_;

        // if args.skip {
        //     field_names_build.push(quote::quote!(
        //         #field_name: ::std::default::Default::default()
        //     ));
        //     continue;
        // }

        let column_index = syn::Ident::new(
            &format!("column_{}", field_name.as_ref().unwrap()),
            proc_macro2::Span::call_site(),
        );
        columns_quote.push(quote::quote!(#column_index));
        let is_num = args.strategy == Some("DeltaRle".to_string())
            || args.strategy == Some("BoolRle".to_string());
        let column_type_token = args.get_strategy_column(quote::quote!(#field_type))?;
        let column_type = if is_num {
            column_type_token
        } else if class.is_some() {
            match class.as_ref().unwrap_or(&"".to_string()).as_str() {
                "vec" => args.get_strategy_column(
                    quote::quote!(::serde_columnar::ColumnarVec<_, #field_type>),
                )?,
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
            // quote::quote!(::serde_columnar::Column<::std::borrow::Cow<#field_type>>)
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
            // have checked before
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
            quote::quote!(
                #field_name: #field_name.into_owned()
            )
        };

        field_names_build.push(field_name_build);
    }

    // generate
    let ret = quote::quote!(
        let mut column_data_len: usize = 0;
        #(#elements)*;
        let ans = ::serde_columnar::izip!(
            #(#into_iter_quote),*
        ).map(|(#(#field_names),*)| #struct_name{
            #(#field_names_build),*
        }).collect();
        Ok(ans)
    );
    Ok(ret)
}
