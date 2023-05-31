use crate::args::FieldArgs;
use syn::{DeriveInput, Generics, ImplGenerics, TypeGenerics, WhereClause};

pub fn generate_compatible_ser(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_len = field_args.len();
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics_params_to_modify.split_for_impl();
    let per_field_ser = generate_per_element_ser(field_args)?;
    let ret = quote::quote!(
        const _:()={
            use ::serde::ser::SerializeSeq;
            impl #impl_generics ::serde::ser::Serialize for #struct_name_ident #ty_generics #where_clause {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::ser::Serializer,
                {
                    // TODO: don't serialize the field if its value is default
                    let mut seq = serializer.serialize_seq(Some(#fields_len))?;
                    #(#per_field_ser)*
                    seq.end()
                }
            }
        };
    );

    Ok(ret)
}

fn generate_per_element_ser(
    field_args: &Vec<FieldArgs>,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let field_len = field_args.len();
    let mut elements = Vec::with_capacity(field_len);
    // if some fields is not optional, but it appears after some optional fields, then we need to throw error
    let mut start_optional = false;
    for args in field_args {
        let field_name = &args.ident;
        let field_type = &args.ty;
        let optional = args.optional;
        let index = args.index;
        // TODO: type vec map need wrapper
        let class = &args.type_;
        if start_optional && !optional {
            return Err(syn::Error::new_spanned(
                field_name,
                "optional field must be placed after non-optional field",
            ));
        }

        let after_wrapper_field = if let Some(type_) = class {
            match type_.as_str() {
                "vec" => {
                    quote::quote!(&::serde_columnar::ColumnarVec::<_, #field_type>::new(&self.#field_name))
                }
                "map" => {
                    quote::quote!(&::serde_columnar::ColumnarMap::<_, _, #field_type>::new(&self.#field_name))
                }
                _ => return Err(syn::Error::new_spanned(class, "unsupported type")),
            }
        } else {
            quote::quote!(&self.#field_name)
        };

        let e = if !optional {
            quote::quote!(
                seq.serialize_element(#after_wrapper_field)?;
            )
        } else {
            start_optional = true;
            if let Some(index) = index {
                quote::quote!(
                    seq.serialize_element(&(#index, ::postcard::to_allocvec(#after_wrapper_field).map_err(S::Error::custom)?))?;
                )
            } else {
                return Err(syn::Error::new_spanned(
                    field_name,
                    "optional field must have index",
                ));
            }
        };
        elements.push(e);
    }
    Ok(elements)
}

pub fn generate_compatible_de(
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> syn::Result<proc_macro2::TokenStream> {
    // let fields_len = field_args.len();
    let struct_name_ident = &input.ident;
    let generics_params_to_modify = input.generics.clone();
    let mut g_clone = input.generics.clone();
    let (_, ty_generics, where_clause) = generics_params_to_modify.split_for_impl();
    let impl_generics = add_de(&mut g_clone);
    let per_field_de = generate_per_element_de(field_args)?;
    let field_names = field_args.iter().map(|args| &args.ident);
    let ret = quote::quote!(
        const _:()={
            use ::std::collections::HashMap;
            use ::serde::de::Visitor;
            use ::serde::de::Error as DeError;
            impl #impl_generics ::serde::de::Deserialize<'de> for #struct_name_ident #ty_generics #where_clause {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct DeVisitor;
                    impl<'de> Visitor<'de> for DeVisitor {
                        type Value = #struct_name_ident #ty_generics;
                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("a sequence")
                        }
                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            #(#per_field_de)*
                            Ok(#struct_name_ident {
                                #(#field_names),*
                            })
                        }
                    }
                    deserializer.deserialize_seq(DeVisitor)
                }
            }
        };
    );
    Ok(ret)
}

fn add_de(impl_generics: &mut Generics) -> ImplGenerics {
    impl_generics.params.push(syn::parse_quote! { 'de });
    let (impl_generics, _, _) = impl_generics.split_for_impl();
    impl_generics
}

fn generate_per_element_de(
    field_args: &Vec<FieldArgs>,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let field_len = field_args.len();
    let mut elements = Vec::with_capacity(field_len);
    let mut start_optional = false;
    let mut add_mapping = false;
    for args in field_args {
        let field_name = &args.ident;
        let optional = args.optional;
        let index = args.index;
        let field_type = &args.ty;
        let class = &args.type_;
        if start_optional && !optional {
            return Err(syn::Error::new_spanned(
                field_name,
                "optional field must be placed after non-optional field",
            ));
        }
        let e = if !optional {
            if let Some(type_) = class {
                match type_.as_str() {
                    "vec" => {
                        quote::quote!(
                            let wrapper: ::serde_columnar::ColumnarVec<_, #field_type> = seq.next_element()?.ok_or_else(|| A::Error::custom("DeserializeUnexpectedEnd"))?;
                            let #field_name = wrapper.into_vec();
                        )
                    }
                    "map" => {
                        quote::quote!(
                            let wrapper: ::serde_columnar::ColumnarMap<_, _, #field_type> = seq.next_element()?.ok_or_else(|| A::Error::custom("DeserializeUnexpectedEnd"))?;
                            let #field_name = wrapper.into_map();
                        )
                    }
                    _ => return Err(syn::Error::new_spanned(class, "unsupported type")),
                }
            } else {
                quote::quote!(
                let #field_name = seq.next_element()?.ok_or_else(|| A::Error::custom("DeserializeUnexpectedEnd"))?;
                )
            }
        } else {
            start_optional = true;
            if !add_mapping {
                elements.push(quote::quote!(
                    let mut mapping = HashMap::new();
                    while let Ok(Some((index, bytes))) = seq.next_element::<(u8, Vec<u8>)>() {
                        // ignore
                        mapping.insert(index, bytes);
                    }

                ));
                add_mapping = true;
            }
            if let Some(index) = index {
                if let Some(type_) = class {
                    match type_.as_str() {
                        "vec" => {
                            quote::quote!(
                                let #field_name = if let Some(bytes) = mapping.remove(&#index){
                                    let wrapper: ::serde_columnar::ColumnarVec<_, #field_type> = ::postcard::from_bytes(&bytes).map_err(A::Error::custom)?;
                                    wrapper.into_vec()
                                }else{
                                    Default::default()
                                };
                            )
                        }
                        "map" => {
                            quote::quote!(
                                let #field_name = if let Some(bytes) = mapping.remove(&#index){
                                    let wrapper: ::serde_columnar::ColumnarMap<_, _, #field_type> = ::postcard::from_bytes(&bytes).map_err(A::Error::custom)?;
                                    wrapper.into_map()
                                }else{
                                    Default::default()
                                };
                            )
                        }
                        _ => return Err(syn::Error::new_spanned(class, "unsupported type")),
                    }
                } else {
                    quote::quote!(
                        let #field_name = if let Some(bytes) = mapping.remove(&#index){
                            ::postcard::from_bytes(&bytes).map_err(A::Error::custom)?
                        }else{
                            Default::default()
                        };
                    )
                }
            } else {
                return Err(syn::Error::new_spanned(
                    field_name,
                    "optional field must have index",
                ));
            }
        };
        elements.push(e);
    }
    Ok(elements)
}
