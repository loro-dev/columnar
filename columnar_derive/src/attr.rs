use std::rc::Rc;

use proc_macro2::Ident;
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use crate::{
    args::{DeriveArgs, FieldArgs},
    ast::struct_from_ast,
    de::{borrowed_lifetimes, BorrowedLifetimes},
};

/// ```rust
/// #[columnar(vec, map, ser, de)]
/// #[derive(Debug)]
/// pub struct Data {
///     #[columnar(strategy = "DeltaRle")]
///     id: ID,
///     #[columnar(strategy = "Rle")]
///     name: String,
///     #
///     bytes: &'a [u8]
/// }
/// ```

pub enum Data {
    Enum,
    Struct(Style, Vec<FieldArgs>),
}

impl Data {
    pub fn fields(&self) -> &[FieldArgs] {
        match &self {
            Data::Enum => unimplemented!("unsupported enum for now"),
            Data::Struct(Style::Struct, fields) => fields,
            _ => unimplemented!("only support named struct for now"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Style {
    /// Named fields.
    Struct,
    /// Many unnamed fields.
    Tuple,
    /// One unnamed field.
    Newtype,
    /// No fields.
    Unit,
}

pub struct Context<'a> {
    pub ident: Ident,
    /// The contents of the struct or enum.
    pub data: Data,
    /// Any generics on the struct or enum.
    pub generics: &'a syn::Generics,
    /// Original input.
    pub original: &'a syn::DeriveInput,
    pub derive_args: DeriveArgs,
    pub borrowed: BorrowedLifetimes,
}

impl<'a> Context<'a> {
    pub fn new(input: &'a syn::DeriveInput, derive_args: DeriveArgs) -> syn::Result<Self> {
        let data = match &input.data {
            syn::Data::Enum(_) => {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "columnar does not support enum type",
                ))
            }
            syn::Data::Struct(data) => {
                let (style, fields) = struct_from_ast(&data.fields)?;
                Data::Struct(style, fields)
            }
            syn::Data::Union(_) => {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "columnar does not support unions type",
                ))
            }
        };
        let borrowed = borrowed_lifetimes(data.fields())?;
        Ok(Context {
            ident: input.ident.clone(),
            data,
            generics: &input.generics,
            original: input,
            derive_args,
            borrowed,
        })
    }

    pub fn fields(&self) -> &[FieldArgs] {
        self.data.fields()
    }
}

pub trait ColumnarSerAttributes {
    fn impl_generics(&self) -> ImplGenerics;
    fn ty_generics(&self) -> TypeGenerics;
    fn where_clause(&self) -> WhereClause;
}
