#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::DeriveInput;

mod internals;
mod row;

#[proc_macro_derive(Row, attributes(columnar))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    todo!()
}
