use darling::ast::NestedMeta;
#[allow(unused_imports)]
use darling::{util::Override, Error as DarlingError, FromField, FromMeta, FromVariant};
#[allow(unused_imports)]
use proc_macro2::{Span, TokenStream};
use syn::DeriveInput;

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
    #[darling(default)]
    pub(crate) compatible: bool,
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
                        let level = compress.level.unwrap();
                        Ok(quote::quote! {
                            Some(::serde_columnar::CompressConfig::from_level(#threshold, #level))
                        })
                    } else {
                        let method = compress.method.unwrap();
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

    #[cfg(feature = "compress")]
    fn compress(&self) -> Option<Override<CompressArgs>> {
        self.compress.clone()
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
    #[cfg(feature = "compress")]
    fn compress(&self) -> Option<Override<CompressArgs>> {
        self.compress.clone()
    }
}

pub fn get_derive_args(args: &[NestedMeta]) -> syn::Result<DeriveArgs> {
    match DeriveArgs::from_list(args) {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!("get_derive_args error: {}", e);
            Err(DarlingError::unsupported_format(
                "columnar only supports attributes with `compatible`, `vec`, `map` and `ser`, `de`",
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

// fn process_named_struct(
//     fields: &mut syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
//      derive_args: &DeriveArgs,
// ) -> syn::Result<Vec<FieldArgs>> {
//     let mut index = 0;
//     let mut fields_args = Vec::with_capacity(fields.len());
//     for field in fields.iter_mut() {
//         let mut field_args = FieldArgs::from_field(field)?;
//         // skip
//         add_serde_skip(field, &field_args)?;
//         if field_args.skip {
//             fields_args.push(field_args);
//             continue;
//         }
//         // serde with
//         add_serde_with(field, &field_args, derive_args)?;
//         if let Some(_index) = field_args.index {
//             index = _index + 1;
//         } else {
//             field_args.index = Some(index);
//             index += 1;
//         }
//         fields_args.push(field_args);
//     }
//     Ok(fields_args)
// }

fn process_enum_variants(
    variants: &mut syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    derive_args: &DeriveArgs,
) -> syn::Result<Vec<VariantArgs>> {
    let mut fields_args = Vec::with_capacity(variants.len());
    for variant in variants.iter_mut() {
        let field_args = VariantArgs::from_variant(variant)?;
        // skip
        add_serde_skip(variant, &field_args)?;
        // if field_args.skip {
        //     fields_args.push(field_args);
        //     continue;
        // }
        // serde with
        add_serde_with(variant, &field_args, derive_args)?;
        fields_args.push(field_args);
    }
    Ok(fields_args)
}

pub fn check_args_validate(field_args: &[FieldArgs]) -> syn::Result<()> {
    // if some fields is not optional, but it appears after some optional fields, then we need to throw error
    let mut start_optional = false;
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
