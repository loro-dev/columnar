use std::collections::BTreeSet;

use proc_macro2::{Ident, TokenStream};
use syn::{parse_quote, GenericArgument, Generics, Lifetime, Path, Type};

use crate::{
    args::{Args, Strategy},
    attr::Context,
    de::{borrowed_lifetimes, BorrowedLifetimes},
    serde::de::{split_with_de_lifetime, WithGenericsBorrow},
    utils::add_lifetime_to_type,
};

// TODO: map-like support
// TODO: lifetime

const ITER_LIFETIME: &str = "'__iter";

pub struct TableIterFieldAttr {
    name: Ident,
    ty: Type,
    class: Option<String>,
    iter_item: Option<Type>,
    strategy: Strategy,
    lifetime: BTreeSet<Lifetime>,
}

impl TableIterFieldAttr {
    // data: IterableData<'a>,
    fn generate_class_iter_field(&self) -> syn::Result<TokenStream> {
        if self.iter_item.is_some() && self.class.is_none() {
            return Err(syn::Error::new_spanned(
                self.ty.clone(),
                "only fields marked `class` can be marked `iter`",
            ));
        }
        assert!(self.class.is_some());
        assert!(self.iter_item.is_some());
        let class = self.class.clone().unwrap();
        if class.eq("map") {
            unimplemented!("map")
        }
        let mut iter_item = self.iter_item.clone().unwrap();
        add_lifetime_to_type(
            &mut iter_item,
            GenericArgument::Lifetime(parse_quote!('__iter)),
            Some(|ident: &mut Ident| {
                *ident = Ident::new(&format!("Iterable{}", ident), ident.span())
            }),
        )?;
        let name = &self.name;

        let ans = quote::quote!(
            pub #name: #iter_item
        );
        Ok(ans)
    }

    /// id: u32
    fn generate_normal_field(&self) -> syn::Result<TokenStream> {
        let name = &self.name;
        let ty = &self.ty;
        let ans = quote::quote!(
            pub #name: #ty
        );
        Ok(ans)
    }

    // =============row iter
    /// a: DeltaRleIter<'i, u32>,
    fn generate_row_iter_field(&self) -> syn::Result<TokenStream> {
        let name = &self.name;
        let ty = &self.ty;
        let iter_ident = match self.strategy {
            Strategy::Rle => quote::quote!(AnyRleIter),
            Strategy::BoolRle => quote::quote!(BoolRleIter),
            Strategy::DeltaRle => quote::quote!(DeltaRleIter),
            Strategy::None => parse_quote!(GenericIter),
        };
        let ans = quote::quote!(
            #name: #iter_ident<'__iter, #ty>
        );
        Ok(ans)
    }
}

// info that table struct needs
pub struct TableIterParameter {
    ident: Ident,
    ty: Path,
    generics: Generics,
    iterable: bool,
    field_attrs: Vec<TableIterFieldAttr>,
    borrow: BorrowedLifetimes,
}

impl TableIterParameter {
    pub fn from_ctx(ctx: &Context) -> syn::Result<Self> {
        let borrow = borrowed_lifetimes(ctx.fields())?;
        let mut field_attrs = Vec::with_capacity(ctx.fields().len());
        for f in ctx.fields() {
            let name = f.ident.clone().ok_or(syn::Error::new_spanned(
                ctx.ident.clone(),
                "only support `vec` or `map` columnar class",
            ))?;
            let tf = TableIterFieldAttr {
                name,
                ty: f.ty.clone(),
                class: f.class.clone(),
                iter_item: f.iter.clone(),
                strategy: f.strategy(),
                lifetime: f.self_lifetime()?,
            };
            field_attrs.push(tf);
        }
        let ans = Self {
            ident: ctx.ident.clone(),
            ty: Path::from(ctx.ident.clone()),
            generics: ctx.generics.clone(),
            field_attrs,
            iterable: ctx.derive_args.iterable,
            borrow,
        };
        Ok(ans)
    }

    pub fn generate_iterable(&self) -> syn::Result<TokenStream> {
        let table_tokens = self.generate_table_iter()?;
        let row_token = self.generate_row_iter()?;
        Ok(quote::quote!(
            #table_tokens
            #row_token
        ))
    }

    fn generate_table_iter(&self) -> syn::Result<TokenStream> {
        if !(self.field_attrs.iter().any(|f| f.iter_item.is_some())) {
            return Ok(quote::quote!());
        }

        let struct_name_ident = &self.ident;
        let mut per_field = Vec::with_capacity(self.field_attrs.len());
        for f in self.field_attrs.iter() {
            let ans = self.generate_table_per_field(f)?;
            per_field.push(ans);
        }

        let this_table_iter_struct_name = syn::Ident::new(
            &format!("TableIter{}", struct_name_ident),
            proc_macro2::Span::call_site(),
        );
        let (de_impl_generics, de_ty_generics, ty_generics, where_clause) =
            split_with_de_lifetime(self);

        let ans = quote::quote!(
            use ::serde_columnar::iterable::TableIter;
            #[columnar(de)]
            pub struct #this_table_iter_struct_name #de_ty_generics #where_clause{
                #(#per_field),*
            }

            impl #de_impl_generics TableIter<'__iter> for #struct_name_ident #ty_generics #where_clause{
                type Iter = #this_table_iter_struct_name #de_ty_generics;
            }
        );
        println!("#####table iter {:?}", ans.to_string());

        Ok(ans)
    }

    fn generate_table_per_field(&self, field: &TableIterFieldAttr) -> syn::Result<TokenStream> {
        if field.class.is_some() && field.iter_item.is_some() {
            field.generate_class_iter_field()
        } else {
            field.generate_normal_field()
        }
    }

    fn generate_row_iter(&self) -> syn::Result<TokenStream> {
        if !self.iterable {
            return Ok(quote::quote!());
        }
        let struct_name_ident = &self.ident;
        let mut per_field = Vec::with_capacity(self.field_attrs.len());
        for f in self.field_attrs.iter() {
            let ans = self.generate_row_per_field(f)?;
            per_field.push(ans);
        }

        let this_row_iter_struct_name = syn::Ident::new(
            &format!("Iterable{}", struct_name_ident),
            proc_macro2::Span::call_site(),
        );
        let (iter_impl_generics, iter_ty_generics, ty_generics, where_clause) =
            split_with_de_lifetime(self);

        let ans = quote::quote!(
            #[columnar(de)]
            pub struct #this_row_iter_struct_name #iter_ty_generics #where_clause{
                #(#per_field),*
            }
        );

        println!("!!!!!row {:?}", ans.to_string());
        Ok(ans)
    }

    fn generate_row_per_field(&self, field: &TableIterFieldAttr) -> syn::Result<TokenStream> {
        field.generate_row_iter_field()
    }
}

impl WithGenericsBorrow for TableIterParameter {
    fn de_lifetime_param(&self) -> Option<syn::LifetimeParam> {
        self.borrow.de_lifetime_param(ITER_LIFETIME)
    }

    fn generics(&self) -> Generics {
        self.generics.clone()
    }

    fn generics_borrow(&self) -> &Generics {
        &self.generics
    }

    fn de_lifetime(&self) -> &str {
        ITER_LIFETIME
    }
}
