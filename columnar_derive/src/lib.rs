extern crate darling;
extern crate quote;

extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

mod attr;
mod derive;

use darling::Error as DarlingError;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemStruct};

use crate::derive::process_derive_args;

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

fn expand_columnar(args: AttributeArgs, mut st: DeriveInput) -> syn::Result<TokenStream> {
    check_derive_serde(&st)?;
    let field_args = attr::get_fields_add_serde_with_to_field(&mut st)?;
    let derive_trait_tokens = process_derive_args(&args, &st, &field_args)?;
    let input = quote::quote!(#st);
    Ok(quote!(#input #derive_trait_tokens ).into())
}

#[allow(dead_code)]
fn check_derive_serde(_: &DeriveInput) -> syn::Result<()> {
    // TODO: adjust this whether derive Serialize and Deserialize
    Ok(())
}

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
