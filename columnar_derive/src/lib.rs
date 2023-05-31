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

use darling::{export::NestedMeta, Error};
use derive::process_derive_args;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Item};

mod args;
use args::{get_derive_args, get_field_args_add_serde_with_to_field};
#[cfg(feature = "analyze")]
mod analyze;
mod attr;
mod derive;

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
    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let input = match add_consume_columnar_attribute(&input) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };
    let st = parse_macro_input!(input as DeriveInput);
    match expand_columnar(attr_args, st) {
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
fn expand_columnar(args: Vec<NestedMeta>, mut st: DeriveInput) -> syn::Result<TokenStream> {
    check_derive_serde(&st)?;
    let derive_args = get_derive_args(&args)?;
    // iterate all fields to check if there is any `columnar` attribute
    // and parse all fields' `columnar` attributes to [`FieldArgs`].
    // add [`serde_with`] attributes to the fields.
    let field_args = get_field_args_add_serde_with_to_field(&mut st, &derive_args)?;
    let input = quote::quote!(#st);
    if let Some(field_args) = field_args {
        let derive_trait_tokens = process_derive_args(&derive_args, &st, &field_args)?;
        Ok(quote!(#input #derive_trait_tokens).into())
    } else {
        // enum 情况
        Ok(input.into())
    }
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
fn add_consume_columnar_attribute(input: &TokenStream) -> syn::Result<TokenStream> {
    let consume_columnar_attribute = syn::parse_quote!(
        #[derive(::serde_columnar::__private_consume_columnar_attributes)]
    );
    let item: Item = syn::parse(input.clone()).unwrap();
    match item {
        Item::Struct(st) => {
            let mut st = st;
            st.attrs.push(consume_columnar_attribute);
            Ok(quote!(#st).into())
        }
        Item::Enum(en) => {
            let mut en = en;
            en.attrs.push(consume_columnar_attribute);
            Ok(quote!(#en).into())
        }
        _ => Err(syn::Error::new(
            Span::call_site(),
            "columnar only support struct and enum",
        )),
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

#[cfg(feature = "analyze")]
fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}

#[cfg(feature = "analyze")]
#[proc_macro_derive(FieldAnalyze, attributes(analyze))]
pub fn derive_field_analyze(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    analyze::expand_derive_analyze(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}
