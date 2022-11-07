//! proc-macro extensions for [`columnar`].
//!
//! This crate should **NEVER** be used alone.
//! All macros **MUST** be used via the re-exports in the [`columnar`] crate.
//!
//! [`columnar`]: <https://github.com/loro-dev/columnar/>
extern crate darling;
extern crate quote;

extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

use darling::Error as DarlingError;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemStruct};

mod attr;
mod derive;
use crate::derive::process_derive_args;

///
/// Convenience macro to use the [`columnar`] system.
///
/// Each field of a struct can be annotated with `#[columnar(...)]` to specify which transformations should be applied.
/// `columnar` is *not* supported on enum and tuple struct temporarily.
///
/// [`columnar`]: <https://github.com/loro-dev/columnar/>
///
/// # Example:
///
/// ```rust, ignore
/// use columnar::{columnar, to_vec, from_bytes};
/// use serde::{Serialize, Deserialize};
///
/// // This struct will be serialized as a columnar format in another collection container.
///
/// // the `vec` represents this struct will derive `VecRow` trait by macro
/// // so that this struct can be used in some container like `Vec<Data>` etc. .
///
/// // the `map` represents this struct will derive `MapRow` trait by macro.
/// // so that this struct can be used in some container like `HashMap<K, Data>` etc. .
///
/// #[columnar(vec, map)]
/// #[derive(Serialize, Deserialize)]
/// struct Data{
///     // in `columnar` system, this field will be considered as a `Vec<Cow<T>>` type with
///     // index 1, and using `Rle` strategy to encode it.
///     #[columnar(index = 1, strategy = "Rle")]
///     id: u64,
/// }
///
/// // The container of `Data` struct.
/// // This struct need also be annotated with `#[columnar]` to use the `columnar` attributes.
///
/// #[columnar]
/// struct Store{
///     // this attribute represents this field will use `VecRow` trait to serialize and deserialize it by columnar format.
///     // this line is equivalent to `#[serde(serialize_with = "VecRow::serialize_columns")]` and #[serde(deserialize_with = "VecRow::deserialize_columns")].
///     // more details about `serde` attributes, please refer to the [`serde`](https://serde.rs) crate.
///     #[columnar(type="vec")]
///     data: Vec<Data>,
/// }
///
///
/// let store = Store{...};
/// let bytes = to_vec(&store).unwrap();
/// let store2: Store = from_bytes(&bytes).unwrap();
/// assert_eq!(store, store2);
///
#[proc_macro_attribute]
pub fn columnar(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args: AttributeArgs = parse_macro_input!(attr as AttributeArgs);
    let input = match add_consume_columnar_attribute(&input) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    let st = parse_macro_input!(input as DeriveInput);
    match expand_columnar(args, st) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

/// [`columnar_derive`] mainly does two things:
///
/// 1. iterate all fields to check if there is any `columnar` attribute and parse all fields' `columnar` attributes to [`FieldArgs`].
///    if there is a `type` attribute, the field will be added `#[serde(serialize_with=..., deserialize_with=...)]`.
/// 2. generate `VecRow` and `MapRow` trait implementations for the struct.
///
fn expand_columnar(args: AttributeArgs, mut st: DeriveInput) -> syn::Result<TokenStream> {
    check_derive_serde(&st)?;
    // iterate all fields to check if there is any `columnar` attribute
    // and parse all fields' `columnar` attributes to [`FieldArgs`].
    let field_args = attr::get_fields_add_serde_with_to_field(&mut st)?;

    let derive_trait_tokens = process_derive_args(&args, &st, &field_args)?;
    let input = quote::quote!(#st);
    Ok(quote!(#input #derive_trait_tokens ).into())
}

/// The struct annotated with `columnar` *MUST* be derived with `Serialize` and `Deserialize` trait.
#[allow(dead_code)]
fn check_derive_serde(_: &DeriveInput) -> syn::Result<()> {
    // TODO: adjust this whether derive Serialize and Deserialize
    Ok(())
}

/// Add [`__private_consume_columnar_attributes`] derive attribute to the input struct.
///
/// In order to use `columnar(...)` attribute, we add a private derive macro with `columnar` attributes annotation.
/// So if a struct is annotated with `columnar`, it will be expanded to a struct with `__private_consume_columnar_attributes` derive attribute.
///
/// Like this:
///
/// ```rust, ignore
/// #[columnar]
/// #[derive(__private_consume_columnar_attributes)]
/// struct Data{...}
/// ```
///
fn add_consume_columnar_attribute(input: &TokenStream) -> Result<TokenStream, DarlingError> {
    let consume_columnar_attribute = syn::parse_quote!(
        #[derive(::columnar::__private_consume_columnar_attributes)]
    );
    if let Ok(mut input) = syn::parse::<ItemStruct>(input.clone()) {
        input.attrs.push(consume_columnar_attribute);
        Ok(quote!(#input).into())
    } else {
        // TODO: add support for enums
        Err(DarlingError::unsupported_shape("expected struct"))
    }
}

#[doc(hidden)]
/// Private function. Not part of the public API
///
/// More details about the use-cases in the GitHub discussion: <https://github.com/jonasbb/serde_with/discussions/260>.
#[proc_macro_derive(__private_consume_columnar_attributes, attributes(columnar))]
pub fn __private_consume_columnar_attributes(_: TokenStream) -> TokenStream {
    TokenStream::new()
}
