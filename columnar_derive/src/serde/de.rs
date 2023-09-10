use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{punctuated::Punctuated, ExprPath, Generics, LifetimeParam, Path, Type};

use crate::{
    attr::Context,
    de::{borrowed_lifetimes, BorrowedLifetimes},
    utils::{is_cow, is_slice_u8, is_str},
};

struct DeFieldAttrs {
    name: Ident,
    ty: Type,
    optional: bool,
    index: Option<usize>,
    class: Option<String>,
    skip: bool,
}

const DE_LIFETIME: &str = "'de";

impl DeFieldAttrs {
    fn generate_vec_wrapper(&self) -> TokenStream {
        let field_type = &self.ty;
        let field_name = &self.name;
        quote::quote!(
            let wrapper: ::serde_columnar::ColumnarVec<_, #field_type> = seq.next_element()?.ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
            let #field_name = wrapper.into_vec();
        )
    }

    fn generate_map_wrapper(&self) -> TokenStream {
        let field_type = &self.ty;
        let field_name = &self.name;
        quote::quote!(
            let wrapper: ::serde_columnar::ColumnarMap<_, _, #field_type> = seq.next_element()?.ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
            let #field_name = wrapper.into_map();
        )
    }

    fn generate_vec_wrapper_from_mapping(&self) -> TokenStream {
        let field_type = &self.ty;
        let field_name = &self.name;
        let index = self.index.unwrap();
        quote::quote!(
            let #field_name = if let Some(bytes) = mapping.remove(&#index){
                let wrapper: ::serde_columnar::ColumnarVec<_, #field_type> = ::postcard::from_bytes(bytes).map_err(__A::Error::custom)?;
                wrapper.into_vec()
            }else{
                Default::default()
            };
        )
    }
    fn generate_map_wrapper_from_mapping(&self) -> TokenStream {
        let field_type = &self.ty;
        let field_name = &self.name;
        let index = self.index.unwrap();
        quote::quote!(
            let #field_name = if let Some(bytes) = mapping.remove(&#index){
                let wrapper: ::serde_columnar::ColumnarMap<_, _, #field_type> = ::postcard::from_bytes(bytes).map_err(__A::Error::custom)?;
                wrapper.into_map()
            }else{
                Default::default()
            };
        )
    }

    fn generate_normal_field(&self, params: &DeParameter) -> TokenStream {
        let field_name = &self.name;
        if let Some(path) = self.borrow_with() {
            let ty = &self.ty;
            let (wrapper, wrapper_ty) = wrap_deserialize_with(params, &quote::quote!(#ty), &path);
            quote::quote!(
                let #field_name = {
                    #wrapper
                    ::serde::__private::Option::map(
                        ::serde::de::SeqAccess::next_element::<#wrapper_ty>(&mut seq)?,
                        |__wrap| __wrap.value).ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?
                };
            )
        } else {
            quote::quote!(
                let #field_name = seq.next_element()?.ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
            )
        }
    }

    fn generate_normal_field_from_mapping(&self, params: &DeParameter) -> TokenStream {
        let field_name = &self.name;
        let index = self.index.unwrap();
        if let Some(path) = self.borrow_with() {
            let ty = &self.ty;
            let (wrapper, wrapper_ty) = wrap_deserialize_with(params, &quote::quote!(#ty), &path);

            quote::quote!(
                let #field_name = if let Some(bytes) = mapping.remove(&#index){
                    #wrapper
                    ::postcard::from_bytes::<#wrapper_ty>(&bytes).map_err(__A::Error::custom)?.value
                }else{
                    Default::default()
                };
            )
        } else {
            quote::quote!(
                let #field_name = if let Some(bytes) = mapping.remove(&#index){
                    ::postcard::from_bytes(&bytes).map_err(__A::Error::custom)?
                }else{
                    Default::default()
                };
            )
        }
    }

    fn borrow_with(&self) -> Option<ExprPath> {
        if is_cow(&self.ty, is_str) {
            let mut path = syn::Path {
                leading_colon: None,
                segments: Punctuated::new(),
            };
            let span = Span::call_site();
            path.segments.push(Ident::new("serde", span).into());
            path.segments.push(Ident::new("__private", span).into());
            path.segments.push(Ident::new("de", span).into());
            path.segments
                .push(Ident::new("borrow_cow_str", span).into());
            let ans = syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            };
            Some(ans)
        } else if is_cow(&self.ty, is_slice_u8) {
            let mut path = syn::Path {
                leading_colon: None,
                segments: Punctuated::new(),
            };
            let span = Span::call_site();
            path.segments.push(Ident::new("serde", span).into());
            path.segments.push(Ident::new("__private", span).into());
            path.segments.push(Ident::new("de", span).into());
            path.segments
                .push(Ident::new("borrow_cow_bytes", span).into());
            let ans = syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            };
            Some(ans)
        } else {
            None
        }
    }
}

pub struct DeParameter {
    ident: Ident,
    ty: Path,
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
                class: f.class.clone(),
                skip: f.skip,
            };
            field_attrs.push(attr);
        }

        let ans = Self {
            ident: ctx.ident.clone(),
            ty: Path::from(ctx.ident.clone()),
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
        let e = if !field.optional {
            if let Some(class) = field.class.as_ref() {
                match class.as_str() {
                    "vec" => field.generate_vec_wrapper(),
                    "map" => field.generate_map_wrapper(),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            class,
                            "only support `vec` or `map` columnar class",
                        ))
                    }
                }
            } else {
                field.generate_normal_field(self)
            }
        } else {
            if !init_hashmap {
                elements.push(quote::quote!(
                    let mut mapping = HashMap::new();
                    while let Ok(Some((index, bytes))) = seq.next_element::<(usize, &'de [u8])>() {
                        // ignore
                        mapping.insert(index, bytes);
                    }

                ));
                init_hashmap = true;
            }
            // have checked before
            if field.index.is_none() {
                return Err(syn::Error::new(
                    field.name.span(),
                    "field with `index` must be `optional`",
                ));
            }

            if let Some(class) = field.class.as_ref() {
                match class.as_str() {
                    "vec" => field.generate_vec_wrapper_from_mapping(),
                    "map" => field.generate_map_wrapper_from_mapping(),
                    _ => return Err(syn::Error::new_spanned(class, "unsupported type")),
                }
            } else {
                field.generate_normal_field_from_mapping(self)
            }
        };
        Ok((init_hashmap, e))
    }

    pub fn derive_de(&self) -> syn::Result<TokenStream> {
        let struct_name_ident = &self.ident;
        let this_type = Path::from(self.ident.clone());
        let (de_impl_generics, de_ty_generics, ty_generics, where_clause) =
            split_with_de_lifetime(self);
        let delife = self.borrow.de_lifetime(DE_LIFETIME);
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

pub trait WithGenericsBorrow {
    fn generics(&self) -> Generics;
    fn generics_borrow(&self) -> &Generics;
    fn de_lifetime_param(&self) -> Option<LifetimeParam>;
    fn de_lifetime(&self) -> &str;
}

pub struct DeImplGenerics<'a, P>(&'a P);
pub struct DeTypeGenerics<'a, P>(&'a P);

impl<'a, P: WithGenericsBorrow> ToTokens for DeImplGenerics<'a, P> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut generics = self.0.generics();
        if let Some(de_lifetime) = self.0.de_lifetime_param() {
            generics.params = Some(syn::GenericParam::Lifetime(de_lifetime))
                .into_iter()
                .chain(generics.params)
                .collect();
        }
        let (impl_generics, _, _) = generics.split_for_impl();
        impl_generics.to_tokens(tokens);
    }
}
impl<'a, P: WithGenericsBorrow> ToTokens for DeTypeGenerics<'a, P> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut generics = self.0.generics();
        if self.0.de_lifetime_param().is_some() {
            let def = syn::LifetimeParam {
                attrs: Vec::new(),
                lifetime: syn::Lifetime::new(self.0.de_lifetime(), Span::call_site()),
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

pub fn split_with_de_lifetime<P: WithGenericsBorrow>(
    params: &P,
) -> (
    DeImplGenerics<P>,
    DeTypeGenerics<P>,
    syn::TypeGenerics,
    Option<&syn::WhereClause>,
) {
    let de_impl_generics = DeImplGenerics(params);
    let de_ty_generics = DeTypeGenerics(params);
    let (_, ty_generics, where_clause) = params.generics_borrow().split_for_impl();
    (de_impl_generics, de_ty_generics, ty_generics, where_clause)
}

/// This function wraps the expression in `#[serde(deserialize_with = "...")]`
/// in a trait to prevent it from accessing the internal `Deserialize` state.
fn wrap_deserialize_with(
    params: &DeParameter,
    value_ty: &TokenStream,
    deserialize_with: &syn::ExprPath,
) -> (TokenStream, TokenStream) {
    let this_type = &params.ty;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) =
        split_with_de_lifetime(params);
    let delife = params.borrow.de_lifetime(DE_LIFETIME);
    let wrapper = quote::quote! {
        #[doc(hidden)]
        struct __DeserializeWith #de_impl_generics #where_clause {
            value: #value_ty,
            phantom: ::serde::__private::PhantomData<#this_type #ty_generics>,
            lifetime: ::serde::__private::PhantomData<&#delife ()>,
        }

        impl #de_impl_generics serde::Deserialize<#delife> for __DeserializeWith #de_ty_generics #where_clause {
            fn deserialize<__D>(__deserializer: __D) -> ::serde::__private::Result<Self, __D::Error>
            where
                __D: serde::Deserializer<#delife>,
            {
                ::serde::__private::Ok(__DeserializeWith {
                    value: #deserialize_with(__deserializer)?,
                    phantom: ::serde::__private::PhantomData,
                    lifetime: ::serde::__private::PhantomData,
                })
            }
        }
    };

    let wrapper_ty = quote:: quote!(__DeserializeWith #de_ty_generics);

    (wrapper, wrapper_ty)
}

impl WithGenericsBorrow for DeParameter {
    fn de_lifetime_param(&self) -> Option<LifetimeParam> {
        self.borrow.de_lifetime_param(DE_LIFETIME)
    }

    fn generics(&self) -> Generics {
        self.generics.clone()
    }

    fn generics_borrow(&self) -> &Generics {
        &self.generics
    }

    fn de_lifetime(&self) -> &str {
        DE_LIFETIME
    }
}
