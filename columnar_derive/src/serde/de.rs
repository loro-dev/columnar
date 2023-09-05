use std::collections::BTreeSet;

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{punctuated::Punctuated, Generics, Lifetime, Path, Type};

use crate::{
    args::Args,
    attr::Context,
    de::{borrowed_lifetimes, BorrowedLifetimes},
};

struct DeFieldAttrs {
    name: Ident,
    ty: Type,
    optional: bool,
    index: Option<usize>,
    class: Option<String>,
    skip: bool,
    borrow: Option<BTreeSet<Lifetime>>,
}

pub struct DeParameter {
    ident: Ident,
    generics: Generics,
    field_attrs: Vec<DeFieldAttrs>,
    borrow: BorrowedLifetimes,
}

impl DeParameter {
    pub fn from_ctx(ctx: &Context) -> syn::Result<Self> {
        let borrow = borrowed_lifetimes(ctx.fields())?;
        let mut field_attrs = Vec::with_capacity(ctx.fields().len());
        for f in ctx.fields() {
            let attr = DeFieldAttrs {
                name: f.ident.clone().unwrap(),
                ty: f.ty.clone(),
                optional: f.optional,
                index: f.index,
                class: f.type_.clone(),
                skip: f.skip,
                borrow: f.borrow_lifetimes()?,
            };
            field_attrs.push(attr);
        }

        let ans = Self {
            ident: ctx.ident.clone(),
            generics: ctx.generics.clone(),
            field_attrs,
            borrow,
        };
        Ok(ans)
    }

    fn field_length(&self) -> usize {
        self.field_attrs.len()
    }

    fn per_field_de(
        &self,
        field: &DeFieldAttrs,
        mut init_hashmap: bool,
        elements: &mut Vec<TokenStream>,
    ) -> syn::Result<(bool, TokenStream)> {
        let field_name = &field.name;
        let field_type = &field.ty;

        let e = if !field.optional {
            if let Some(class) = field.class.as_ref() {
                match class.as_str() {
                    "vec" => {
                        quote::quote!(
                            let wrapper: ::serde_columnar::ColumnarVec<_, #field_type> = seq.next_element()?.ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
                            let #field_name = wrapper.into_vec();
                        )
                    }
                    "map" => {
                        quote::quote!(
                            let wrapper: ::serde_columnar::ColumnarMap<_, _, #field_type> = seq.next_element()?.ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
                            let #field_name = wrapper.into_map();
                        )
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            class,
                            "only support `vec` or `map` columnar class",
                        ))
                    }
                }
            } else {
                quote::quote!(
                let #field_name = seq.next_element()?.ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
                )
            }
        } else {
            if !init_hashmap {
                elements.push(quote::quote!(
                    let mut mapping = HashMap::new();
                    while let Ok(Some((index, bytes))) = seq.next_element::<(usize, Vec<u8>)>() {
                        // ignore
                        mapping.insert(index, bytes);
                    }

                ));
                init_hashmap = true;
            }
            // have checked before
            let index = field.index.ok_or(syn::Error::new(
                field_name.span(),
                "field with `index` must be `optional` ",
            ))?;
            if let Some(class) = field.class.as_ref() {
                match class.as_str() {
                    "vec" => {
                        quote::quote!(
                            let #field_name = if let Some(bytes) = mapping.remove(&#index){
                                let wrapper: ::serde_columnar::ColumnarVec<_, #field_type> = ::postcard::from_bytes(&bytes).map_err(__A::Error::custom)?;
                                wrapper.into_vec()
                            }else{
                                Default::default()
                            };
                        )
                    }
                    "map" => {
                        quote::quote!(
                            let #field_name = if let Some(bytes) = mapping.remove(&#index){
                                let wrapper: ::serde_columnar::ColumnarMap<_, _, #field_type> = ::postcard::from_bytes(&bytes).map_err(__A::Error::custom)?;
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
                        ::postcard::from_bytes(&bytes).map_err(__A::Error::custom)?
                    }else{
                        Default::default()
                    };
                )
            }
        };
        Ok((init_hashmap, e))
    }

    pub fn derive_de(&self) -> syn::Result<TokenStream> {
        let struct_name_ident = &self.ident;
        let this_type = Path::from(self.ident.clone());
        let (de_impl_generics, de_ty_generics, ty_generics, where_clause) =
            split_with_de_lifetime(self);
        let delife = self.borrow.de_lifetime();
        let field_names = self.field_attrs.iter().map(|args| &args.name);
        let mut init_hashmap = false;
        let mut per_field_de = Vec::with_capacity(self.field_length());
        for field in &self.field_attrs {
            if !field.skip {
                let (flag, field_token) =
                    self.per_field_de(field, init_hashmap, &mut per_field_de)?;
                init_hashmap = flag;
                per_field_de.push(field_token);
            } else {
                let field_name = &field.name;
                per_field_de.push(quote::quote!(
                    let #field_name = Default::default();
                ))
            }
        }

        let ans = quote::quote!(
            const _:()={
                use ::std::collections::HashMap;
                use ::serde::de::Visitor;
                use ::serde::de::Error as DeError;
                impl #de_impl_generics ::serde::de::Deserialize<'de> for #struct_name_ident #ty_generics #where_clause {
                    fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
                    where
                        __D: serde::Deserializer<'de>,
                    {
                        struct DeVisitor #de_impl_generics #where_clause {
                            marker: std::marker::PhantomData::<#this_type #ty_generics>,
                            lifetime: std::marker::PhantomData<&#delife ()>,
                        };
                        impl #de_impl_generics Visitor<#delife> for DeVisitor #de_ty_generics #where_clause {
                            type Value = #struct_name_ident #ty_generics;
                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str("a sequence")
                            }
                            fn visit_seq<__A>(self, mut seq: __A) -> Result<Self::Value, __A::Error>
                            where
                                __A: serde::de::SeqAccess<'de>,
                            {
                                #(#per_field_de)*
                                Ok(#struct_name_ident {
                                    #(#field_names),*
                                })
                            }
                        }
                        deserializer.deserialize_seq(DeVisitor{
                            marker: Default::default(),
                            lifetime: Default::default(),
                        })
                    }
                }
            };
        );
        Ok(ans)
    }
}

struct DeImplGenerics<'a>(&'a DeParameter);
struct DeTypeGenerics<'a>(&'a DeParameter);

impl<'a> ToTokens for DeImplGenerics<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut generics = self.0.generics.clone();
        if let Some(de_lifetime) = self.0.borrow.de_lifetime_param() {
            generics.params = Some(syn::GenericParam::Lifetime(de_lifetime))
                .into_iter()
                .chain(generics.params)
                .collect();
        }
        let (impl_generics, _, _) = generics.split_for_impl();
        impl_generics.to_tokens(tokens);
    }
}
impl<'a> ToTokens for DeTypeGenerics<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut generics = self.0.generics.clone();
        if self.0.borrow.de_lifetime_param().is_some() {
            let def = syn::LifetimeParam {
                attrs: Vec::new(),
                lifetime: syn::Lifetime::new("'de", Span::call_site()),
                colon_token: None,
                bounds: Punctuated::new(),
            };
            // Prepend 'de lifetime to list of generics
            generics.params = Some(syn::GenericParam::Lifetime(def))
                .into_iter()
                .chain(generics.params)
                .collect();
        }
        let (_, ty_generics, _) = generics.split_for_impl();
        ty_generics.to_tokens(tokens);
    }
}

fn split_with_de_lifetime(
    params: &DeParameter,
) -> (
    DeImplGenerics,
    DeTypeGenerics,
    syn::TypeGenerics,
    Option<&syn::WhereClause>,
) {
    let de_impl_generics = DeImplGenerics(params);
    let de_ty_generics = DeTypeGenerics(params);
    let (_, ty_generics, where_clause) = params.generics.split_for_impl();
    (de_impl_generics, de_ty_generics, ty_generics, where_clause)
}
