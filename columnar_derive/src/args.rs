use std::collections::BTreeSet;

use darling::ast::NestedMeta;
#[allow(unused_imports)]
use darling::{util::Override, Error as DarlingError, FromField, FromMeta, FromVariant};
use proc_macro2::{Spacing, TokenTree};
#[allow(unused_imports)]
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{parse::ParseStream, DeriveInput, Lifetime, LitStr, Token, Type};

use crate::attr::{add_serde_skip, add_serde_with};

#[derive(Debug, FromMeta)]
pub struct DeriveArgs {
    #[darling(default)]
    pub(crate) vec: bool,
    #[darling(rename = "map", default)]
    pub(crate) hashmap: bool,
    #[darling(default)]
    pub(crate) ser: bool,
    #[darling(default)]
    pub(crate) de: bool,
}
#[cfg(feature = "compress")]
#[derive(FromMeta, Debug, Clone)]
pub struct CompressArgs {
    pub min_size: Option<usize>,
    // 0~9
    pub level: Option<u32>,
    // "best"(9), "fast"(1), "default"(6)
    pub method: Option<String>,
}

#[cfg(feature = "compress")]
#[derive(FromField, Debug)]
#[darling(attributes(columnar))]
pub struct FieldArgs {
    pub ident: Option<syn::Ident>,
    pub vis: syn::Visibility,
    pub ty: syn::Type,
    pub attrs: Vec<syn::Attribute>,
    // custom attributes
    /// The index of the field in the struct, starts from 0 default.
    pub index: Option<usize>,
    /// If optional, this field need to be compatible with the old or new version.
    #[darling(default)]
    pub optional: bool,
    /// the strategy to convert the field values to a column.
    pub strategy: Option<String>,
    /// the type of the column format, vec or map.
    #[darling(rename = "class")]
    pub type_: Option<String>,
    pub compress: Option<Override<CompressArgs>>,
    pub borrow: Option<Override<LitStr>>,
}

#[cfg(not(feature = "compress"))]
#[derive(FromField, Debug)]
#[darling(attributes(columnar))]
pub struct FieldArgs {
    pub ident: Option<syn::Ident>,
    pub vis: syn::Visibility,
    pub ty: syn::Type,
    pub attrs: Vec<syn::Attribute>,
    // custom attributes
    /// The index of the field in the struct, starts from 0 default.
    pub index: Option<usize>,
    /// If optional, this field need to be compatible with the old or new version.
    #[darling(default)]
    pub optional: bool,
    /// the strategy to convert the field values to a column.
    pub strategy: Option<String>,
    /// the type of the column format, vec or map.
    #[darling(rename = "class")]
    pub type_: Option<String>,
    pub borrow: Option<Override<LitStr>>,
}

#[cfg(feature = "compress")]
#[derive(FromVariant, Debug)]
#[darling(attributes(columnar))]
pub struct VariantArgs {
    // pub ident: syn::Ident,
    // pub vis: syn::Visibility,
    // pub ty: syn::Type,
    pub attrs: Vec<syn::Attribute>,
    /// the type of the column format, vec or map.
    #[darling(rename = "class")]
    pub type_: Option<String>,
    /// If skip, this field will be ignored.
    #[darling(default)]
    pub skip: bool,
}

#[cfg(not(feature = "compress"))]
#[derive(FromVariant, Debug)]
#[darling(attributes(columnar))]
pub struct VariantArgs {
    // pub ident: syn::Ident,
    // pub vis: syn::Visibility,
    // pub ty: syn::Type,
    pub attrs: Vec<syn::Attribute>,
    /// the type of the column format, vec or map.
    #[darling(rename = "class")]
    pub type_: Option<String>,
    /// If skip, this field will be ignored.
    #[darling(default)]
    pub skip: bool,
}

pub enum AsType {
    Vec,
    Map,
    Other,
}

pub trait Args {
    fn ident(&self) -> Option<syn::Ident>;
    fn ty(&self) -> Option<syn::Type>;
    fn attrs(&self) -> &[syn::Attribute];
    fn index(&self) -> Option<usize>;
    fn optional(&self) -> bool;
    fn strategy(&self) -> &Option<String>;
    fn type_(&self) -> Option<AsType>;
    fn borrow_lifetimes(&self) -> syn::Result<Option<BTreeSet<Lifetime>>>;
    #[cfg(feature = "compress")]
    fn compress(&self) -> &Option<Override<CompressArgs>>;
    #[cfg(feature = "compress")]
    fn compress_args(&self) -> syn::Result<proc_macro2::TokenStream> {
        if let Some(compress) = self.compress() {
            match compress {
                Override::Explicit(compress) => {
                    if compress.level.is_some() && compress.method.is_some() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "columnar only support struct and enum",
                        ));
                    }
                    const DEFAULT_COMPRESS_THRESHOLD: usize = 256;
                    let threshold = compress.min_size.unwrap_or(DEFAULT_COMPRESS_THRESHOLD);
                    if compress.level.is_some() {
                        let level = compress.level.as_ref().unwrap();
                        Ok(quote::quote! {
                            Some(::serde_columnar::CompressConfig::from_level(#threshold, #level))
                        })
                    } else {
                        let method = compress.method.as_ref().unwrap();
                        // TODO: check method is best fast default
                        Ok(quote::quote! {
                            Some(::serde_columnar::CompressConfig::from_method(#threshold, #method.to_string()))
                        })
                    }
                }
                Override::Inherit => {
                    Ok(quote::quote! { Some(::serde_columnar::CompressConfig::default()) })
                }
            }
        } else {
            Ok(quote::quote!(None))
        }
    }
    fn get_strategy_column(&self, ty: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
        if let Some(strategy) = self.strategy() {
            match strategy.as_str() {
                "Rle" => Ok(quote::quote!(::serde_columnar::RleColumn::<#ty>)),
                "BoolRle" => Ok(quote::quote!(::serde_columnar::BoolRleColumn)),
                "DeltaRle" => Ok(quote::quote!(::serde_columnar::DeltaRleColumn::<#ty>)),
                _ => unreachable!("strategy should be Rle, BoolRle or DeltaRle"),
            }
        } else {
            Ok(quote::quote!(::std::vec::Vec::<#ty>))
        }
    }
}

impl Args for FieldArgs {
    fn ident(&self) -> Option<syn::Ident> {
        self.ident.clone()
    }
    fn ty(&self) -> Option<syn::Type> {
        Some(self.ty.clone())
    }
    fn attrs(&self) -> &[syn::Attribute] {
        &self.attrs
    }
    fn index(&self) -> Option<usize> {
        self.index
    }
    fn optional(&self) -> bool {
        self.optional
    }
    fn strategy(&self) -> &Option<String> {
        &self.strategy
    }

    fn type_(&self) -> Option<AsType> {
        match self.type_.as_deref() {
            Some("vec") => Some(AsType::Vec),
            Some("map") => Some(AsType::Map),
            Some(_) => Some(AsType::Other),
            None => None,
        }
    }

    fn borrow_lifetimes(&self) -> syn::Result<Option<BTreeSet<Lifetime>>> {
        if self.borrow.is_none() {
            return Ok(None);
        }

        match self.borrow.as_ref().unwrap() {
            Override::Inherit => {
                let mut lifetimes = BTreeSet::new();
                collect_lifetimes(&self.ty, &mut lifetimes);
                if lifetimes.is_empty() {
                    Err(syn::Error::new_spanned(
                        self.ty.clone().into_token_stream(),
                        "at least one lifetime must be borrowed",
                    ))
                } else {
                    Ok(Some(lifetimes))
                }
            }
            Override::Explicit(string) => {
                if let Ok(lifetimes) = string.parse_with(|input: ParseStream| {
                    let mut set = BTreeSet::new();
                    while !input.is_empty() {
                        let lifetime: Lifetime = input.parse()?;
                        if !set.insert(lifetime.clone()) {
                            return Err(syn::Error::new_spanned(
                                string.clone().into_token_stream(),
                                format!("duplicate borrowed lifetime `{}`", lifetime),
                            ));
                        }
                        if input.is_empty() {
                            break;
                        }
                        input.parse::<Token![+]>()?;
                    }
                    Ok(set)
                }) {
                    if lifetimes.is_empty() {
                        return Err(syn::Error::new_spanned(
                            string.clone().into_token_stream(),
                            "at least one lifetime must be borrowed",
                        ));
                    }
                    Ok(Some(lifetimes))
                } else {
                    Err(syn::Error::new_spanned(
                        string.clone().into_token_stream(),
                        format!("failed to parse borrowed lifetimes: {:?}", string.value()),
                    ))
                }
            }
        }
    }

    #[cfg(feature = "compress")]
    fn compress(&self) -> &Option<Override<CompressArgs>> {
        &self.compress
    }
}

impl Args for VariantArgs {
    fn ident(&self) -> Option<syn::Ident> {
        None
    }
    fn ty(&self) -> Option<syn::Type> {
        None
    }
    fn attrs(&self) -> &[syn::Attribute] {
        &self.attrs
    }
    fn index(&self) -> Option<usize> {
        None
    }
    fn optional(&self) -> bool {
        false
    }
    fn strategy(&self) -> &Option<String> {
        &None
    }
    fn type_(&self) -> Option<AsType> {
        match self.type_.as_deref() {
            Some("vec") => Some(AsType::Vec),
            Some("map") => Some(AsType::Map),
            Some(_) => Some(AsType::Other),
            None => None,
        }
    }
    fn borrow_lifetimes(&self) -> syn::Result<Option<BTreeSet<Lifetime>>> {
        Ok(None)
    }
    #[cfg(feature = "compress")]
    fn compress(&self) -> &Option<Override<CompressArgs>> {
        &None
    }
}

pub fn get_derive_args(args: &[NestedMeta]) -> syn::Result<DeriveArgs> {
    match DeriveArgs::from_list(args) {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!("get_derive_args error: {}", e);
            Err(DarlingError::unsupported_format(
                "columnar only supports attributes with `vec`, `map` and `ser`, `de`",
            )
            .into())
        }
    }
}

pub fn get_field_args_add_serde_with_to_field(
    st: &mut DeriveInput,
    derive_args: &DeriveArgs,
) -> syn::Result<Option<Vec<FieldArgs>>> {
    match &mut st.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            let mut args = Vec::with_capacity(named.len());
            for field in named.iter() {
                let field_args = FieldArgs::from_field(field)?;
                args.push(field_args);
            }
            check_args_validate(&args)?;
            Ok(Some(args))
        }
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            process_enum_variants(variants, derive_args)?;
            Ok(None)
        }
        _ => Err(syn::Error::new_spanned(
            st,
            "only supported named struct or enum type",
        )),
    }
}

fn process_enum_variants(
    variants: &mut syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    derive_args: &DeriveArgs,
) -> syn::Result<Vec<VariantArgs>> {
    let mut fields_args = Vec::with_capacity(variants.len());
    for variant in variants.iter_mut() {
        let field_args = VariantArgs::from_variant(variant)?;
        // skip
        add_serde_skip(variant, &field_args)?;
        if field_args.skip {
            fields_args.push(field_args);
            continue;
        }
        // serde with
        add_serde_with(variant, &field_args, derive_args)?;
        fields_args.push(field_args);
    }
    Ok(fields_args)
}

pub fn check_args_validate(field_args: &[FieldArgs]) -> syn::Result<()> {
    // if some fields is not optional, but it appears after some optional fields, then we need to throw error
    let mut start_optional = false;
    let mut indexes = std::collections::HashSet::new();
    for args in field_args {
        let field_name = &args.ident;
        let optional = args.optional;
        let index = args.index;
        if start_optional && !optional {
            return Err(syn::Error::new_spanned(
                field_name,
                "optional field must be placed after non-optional field",
            ));
        }
        if optional {
            start_optional = true;
            if index.is_none() {
                return Err(syn::Error::new_spanned(
                    field_name,
                    "optional field must have index",
                ));
            }
            if indexes.contains(&index.unwrap()) {
                return Err(syn::Error::new_spanned(
                    field_name,
                    "index cannot have duplicate values",
                ));
            }
            indexes.insert(index.unwrap());
        };

        let strategy = &args.strategy;
        let class = &args.type_;
        if strategy.is_some() && class.is_some() {
            return Err(syn::Error::new_spanned(
                field_name,
                "strategy and class cannot be set at the same time",
            ));
        }
    }
    Ok(())
}

fn collect_lifetimes(ty: &syn::Type, out: &mut BTreeSet<syn::Lifetime>) {
    match ty {
        syn::Type::Slice(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Array(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Ptr(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Reference(ty) => {
            out.extend(ty.lifetime.iter().cloned());
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Tuple(ty) => {
            for elem in &ty.elems {
                collect_lifetimes(elem, out);
            }
        }
        syn::Type::Path(ty) => {
            if let Some(qself) = &ty.qself {
                collect_lifetimes(&qself.ty, out);
            }
            for seg in &ty.path.segments {
                if let syn::PathArguments::AngleBracketed(bracketed) = &seg.arguments {
                    for arg in &bracketed.args {
                        match arg {
                            syn::GenericArgument::Lifetime(lifetime) => {
                                out.insert(lifetime.clone());
                            }
                            syn::GenericArgument::Type(ty) => {
                                collect_lifetimes(ty, out);
                            }
                            syn::GenericArgument::AssocType(binding) => {
                                collect_lifetimes(&binding.ty, out);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        syn::Type::Paren(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Group(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Macro(ty) => {
            collect_lifetimes_from_tokens(ty.mac.tokens.clone(), out);
        }
        syn::Type::BareFn(_)
        | syn::Type::Never(_)
        | syn::Type::TraitObject(_)
        | syn::Type::ImplTrait(_)
        | syn::Type::Infer(_)
        | syn::Type::Verbatim(_) => {}

        #[cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
        _ => {}
    }
}
fn collect_lifetimes_from_tokens(tokens: TokenStream, out: &mut BTreeSet<syn::Lifetime>) {
    let mut iter = tokens.into_iter();
    while let Some(tt) = iter.next() {
        match &tt {
            TokenTree::Punct(op) if op.as_char() == '\'' && op.spacing() == Spacing::Joint => {
                if let Some(TokenTree::Ident(ident)) = iter.next() {
                    out.insert(syn::Lifetime {
                        apostrophe: op.span(),
                        ident,
                    });
                }
            }
            TokenTree::Group(group) => {
                let tokens = group.stream();
                collect_lifetimes_from_tokens(tokens, out);
            }
            _ => {}
        }
    }
}

// Whether the type looks like it might be `std::borrow::Cow<T>` where elem="T".
// This can have false negatives and false positives.
//
// False negative:
//
//     use std::borrow::Cow as Pig;
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         #[serde(borrow)]
//         pig: Pig<'a, str>,
//     }
//
// False positive:
//
//     type str = [i16];
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         #[serde(borrow)]
//         cow: Cow<'a, str>,
//     }
fn is_cow(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    let path = match ungroup(ty) {
        syn::Type::Path(ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg,
        None => {
            return false;
        }
    };
    let args = match &seg.arguments {
        syn::PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    seg.ident == "Cow"
        && args.len() == 2
        && match (&args[0], &args[1]) {
            (syn::GenericArgument::Lifetime(_), syn::GenericArgument::Type(arg)) => elem(arg),
            _ => false,
        }
}

fn is_option(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    let path = match ungroup(ty) {
        syn::Type::Path(ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg,
        None => {
            return false;
        }
    };
    let args = match &seg.arguments {
        syn::PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    seg.ident == "Option"
        && args.len() == 1
        && match &args[0] {
            syn::GenericArgument::Type(arg) => elem(arg),
            _ => false,
        }
}

// Whether the type looks like it might be `&T` where elem="T". This can have
// false negatives and false positives.
//
// False negative:
//
//     type Yarn = str;
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         r: &'a Yarn,
//     }
//
// False positive:
//
//     type str = [i16];
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         r: &'a str,
//     }
fn is_reference(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    match ungroup(ty) {
        syn::Type::Reference(ty) => ty.mutability.is_none() && elem(&ty.elem),
        _ => false,
    }
}

fn is_str(ty: &syn::Type) -> bool {
    is_primitive_type(ty, "str")
}

fn is_slice_u8(ty: &syn::Type) -> bool {
    match ungroup(ty) {
        syn::Type::Slice(ty) => is_primitive_type(&ty.elem, "u8"),
        _ => false,
    }
}

fn is_primitive_type(ty: &syn::Type, primitive: &str) -> bool {
    match ungroup(ty) {
        syn::Type::Path(ty) => ty.qself.is_none() && is_primitive_path(&ty.path, primitive),
        _ => false,
    }
}

fn is_primitive_path(path: &syn::Path, primitive: &str) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].ident == primitive
        && path.segments[0].arguments.is_empty()
}

pub fn ungroup(mut ty: &Type) -> &Type {
    while let Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}
