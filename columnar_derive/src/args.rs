use std::collections::BTreeSet;

use darling::ast::NestedMeta;
#[allow(unused_imports)]
use darling::{util::Override, Error as DarlingError, FromField, FromMeta, FromVariant};
use proc_macro2::{Spacing, TokenTree};
#[allow(unused_imports)]
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{parse::ParseStream, spanned::Spanned, DeriveInput, Lifetime, LitStr, Token, Type};

#[derive(Debug, Clone, Copy, FromMeta)]
pub struct DeriveArgs {
    #[darling(default)]
    pub(crate) vec: bool,
    #[darling(rename = "map", default)]
    pub(crate) hashmap: bool,
    #[darling(default)]
    pub(crate) ser: bool,
    #[darling(default)]
    pub(crate) de: bool,
    // only row struct
    #[darling(default)]
    pub(crate) iterable: bool,
}

#[derive(FromField, Debug, Clone)]
#[darling(attributes(columnar))]
pub struct FieldArgs {
    /// the name of field
    pub ident: Option<syn::Ident>,
    // pub vis: syn::Visibility,
    /// the type of field
    pub ty: Type,
    // custom attributes
    /// The index of the field in the struct, starts from 0 default.
    pub index: Option<usize>,
    /// If optional, this field need to be compatible with the old or new version.
    #[darling(default)]
    pub optional: bool,
    /// the strategy to convert the field values to a column.
    pub strategy: Option<String>,
    /// the type of the column format, vec or map.
    pub class: Option<String>,
    /// Same as the `borrow` of `serde`
    pub borrow: Option<Override<LitStr>>,
    /// Same as the `skip` of serde
    #[darling(default)]
    pub skip: bool,
    pub iter: Option<Type>,
}

#[derive(Debug, Clone, Copy)]
pub enum Strategy {
    Rle,
    DeltaRle,
    BoolRle,
    DeltaOfDelta,
    None,
}

impl Strategy {
    fn from_str(s: Option<String>) -> Self {
        if let Some(s) = s {
            match s.as_str() {
                "Rle" => Self::Rle,
                "DeltaRle" => Self::DeltaRle,
                "BoolRle" => Self::BoolRle,
                "DeltaOfDelta" => Self::DeltaOfDelta,
                _ => unreachable!("strategy should be Rle, BoolRle or DeltaRle"),
            }
        } else {
            Self::None
        }
    }
}

pub enum AsType {
    Vec,
    Map,
    Other,
}

pub trait Args {
    // fn ident(&self) -> Option<syn::Ident>;
    fn ty(&self) -> Option<syn::Type>;
    // fn index(&self) -> Option<usize>;
    // fn optional(&self) -> bool;
    fn strategy(&self) -> Strategy;
    fn can_copy(&self) -> bool {
        match self.strategy() {
            Strategy::BoolRle | Strategy::DeltaRle | Strategy::DeltaOfDelta => true,
            Strategy::Rle | Strategy::None => false,
        }
    }
    fn class(&self) -> Option<AsType>;
    fn has_borrow_lifetime(&self) -> bool;
    fn borrow_lifetimes(&self) -> syn::Result<Option<BTreeSet<Lifetime>>>;
    fn self_lifetime(&self) -> syn::Result<BTreeSet<Lifetime>>;
    fn lifetime(&self) -> syn::Result<BTreeSet<Lifetime>>;
    fn get_strategy_column(&self, ty: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
        match self.strategy() {
            Strategy::Rle => Ok(quote::quote!(::serde_columnar::RleColumn::<#ty>)),
            Strategy::BoolRle => Ok(quote::quote!(::serde_columnar::BoolRleColumn)),
            Strategy::DeltaRle => Ok(quote::quote!(::serde_columnar::DeltaRleColumn::<#ty>)),
            Strategy::DeltaOfDelta => {
                Ok(quote::quote!(::serde_columnar::DeltaOfDeltaColumn::<#ty>))
            }
            Strategy::None => {
                if self.class().is_some() {
                    let self_ty = &self.ty();

                    let ans = match self.class().unwrap() {
                        AsType::Map => {
                            quote::quote!(::serde_columnar::GenericColumn::<::serde_columnar::ColumnarMap::<_, _, #self_ty>>)
                        }
                        AsType::Vec => {
                            quote::quote!(::serde_columnar::GenericColumn::<::serde_columnar::ColumnarVec::<_, #self_ty>>)
                        }
                        _ => unreachable!(),
                    };

                    Ok(ans)
                } else {
                    Ok(quote::quote!(::serde_columnar::GenericColumn::<#ty>))
                }
            }
        }
    }
}

impl Args for FieldArgs {
    fn ty(&self) -> Option<syn::Type> {
        Some(self.ty.clone())
    }

    fn strategy(&self) -> Strategy {
        Strategy::from_str(self.strategy.clone())
    }

    fn class(&self) -> Option<AsType> {
        match self.class.as_deref() {
            Some("vec") => Some(AsType::Vec),
            Some("map") => Some(AsType::Map),
            Some(_) => Some(AsType::Other),
            None => None,
        }
    }

    fn lifetime(&self) -> syn::Result<BTreeSet<Lifetime>> {
        if self.has_borrow_lifetime() {
            Ok(self.borrow_lifetimes()?.unwrap())
        } else {
            self.self_lifetime()
        }
    }

    fn self_lifetime(&self) -> syn::Result<BTreeSet<Lifetime>> {
        let mut lifetimes = BTreeSet::new();
        collect_lifetimes(&self.ty, &mut lifetimes);

        Ok(lifetimes)
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

                    if let Ok(field_lifetimes) = self.self_lifetime() {
                        for l in &lifetimes {
                            if !field_lifetimes.contains(l) {
                                return Err(syn::Error::new(
                                    self.ident.span(),
                                    format!(
                                        "field `{}` does not have lifetime {}",
                                        self.ident.as_ref().unwrap(),
                                        l,
                                    ),
                                ));
                            }
                        }
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

    fn has_borrow_lifetime(&self) -> bool {
        self.borrow.is_some()
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

pub fn parse_field_args(st: &mut DeriveInput) -> syn::Result<Option<Vec<FieldArgs>>> {
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
        syn::Data::Enum(syn::DataEnum { variants: _, .. }) => Err(syn::Error::new_spanned(
            st,
            "only supported named struct type",
        )),
        _ => Err(syn::Error::new_spanned(
            st,
            "only supported named struct type",
        )),
    }
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
        let class = &args.class;
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
