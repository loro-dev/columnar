use proc_macro2::{Ident, TokenStream};
use syn::{Generics, Type};

use crate::attr::Context;

struct SerFieldAttrs {
    name: Ident,
    ty: Type,
    optional: bool,
    index: Option<usize>,
    class: Option<String>,
    skip: bool,
}

/// All the parameters of `Serialize`
pub struct SerParameter {
    ident: Ident,
    generics: Generics,
    field_attrs: Vec<SerFieldAttrs>,
}

impl SerParameter {
    pub fn from_ctx(ctx: &Context) -> Self {
        Self {
            ident: ctx.ident.clone(),
            generics: ctx.generics.clone(),
            field_attrs: ctx
                .fields()
                .iter()
                .map(|f| SerFieldAttrs {
                    name: f.ident.clone().unwrap(),
                    ty: f.ty.clone(),
                    optional: f.optional,
                    index: f.index,
                    class: f.type_.clone(),
                    skip: f.skip,
                })
                .collect(),
        }
    }

    fn field_length_without_skip(&self) -> usize {
        self.field_attrs.iter().filter(|f| !f.skip).count()
    }

    fn per_field_ser(&self, field: &SerFieldAttrs) -> syn::Result<TokenStream> {
        let field_name = &field.name;
        let field_type = &field.ty;

        let field_token = if let Some(class) = &field.class {
            match class.as_str() {
                "vec" => {
                    quote::quote!(&::serde_columnar::ColumnarVec::<_, #field_type>::new(&self.#field_name))
                }
                "map" => {
                    quote::quote!(&::serde_columnar::ColumnarMap::<_, _, #field_type>::new(&self.#field_name))
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        class,
                        "only support `vec` or `map` columnar class",
                    ))
                }
            }
        } else {
            quote::quote!(&self.#field_name)
        };
        let ans = if !field.optional {
            quote::quote!(
                seq.serialize_element(#field_token)?;
            )
        } else {
            let index = field.index.ok_or(syn::Error::new(
                field_name.span(),
                "field with `index` must be `optional` ",
            ))?;
            quote::quote!(
                seq.serialize_element(&(#index, ::postcard::to_allocvec(#field_token).map_err(S::Error::custom)?))?;
            )
        };
        Ok(ans)
    }

    pub fn derive_ser(&self) -> syn::Result<TokenStream> {
        let struct_name_ident = &self.ident;
        let field_length = self.field_length_without_skip();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let mut per_element_body = Vec::with_capacity(field_length);
        for field in &self.field_attrs {
            if !field.skip {
                per_element_body.push(self.per_field_ser(field)?)
            }
        }

        let ans = quote::quote!(
            const _:()={
                use ::serde::ser::SerializeSeq;
                use ::serde::ser::Error;
                impl #impl_generics ::serde::ser::Serialize for #struct_name_ident #ty_generics #where_clause {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: ::serde::ser::Serializer,
                    {
                        let mut seq = serializer.serialize_seq(Some(#field_length))?;
                        #(#per_element_body)*
                        seq.end()
                    }
                }
            };
        );
        Ok(ans)
    }
}
