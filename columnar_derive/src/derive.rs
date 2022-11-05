use darling::{Error as DarlingError, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn};

#[derive(Debug, FromMeta)]
pub struct DeriveArgs {
    #[darling(default)]
    vec: bool,
    #[darling(rename = "map", default)]
    hashmap: bool,
}

impl DeriveArgs {
    pub fn derive_vec(&self, input: &TokenStream) -> syn::Result<Option<TokenStream>> {
        if !self.vec {
            return Ok(None);
        }
        todo!()
    }
}

pub fn process_derive_args(args: &AttributeArgs) -> Result<proc_macro2::TokenStream, DarlingError> {
    let derive_args = match DeriveArgs::from_list(args) {
        Ok(v) => v,
        Err(_) => {
            return Err(DarlingError::custom(
                "columnar only supports attributes with `vec` and `map`",
            ))
        }
    };
    Ok(quote::quote!())
}

pub fn generate_derive_vec_row_for_struct(
    input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name_ident = &input.ident;
    let mut generics_params_to_modify = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics_params_to_modify.split_for_impl();
    let ret = quote::quote!(
        // impl #impl_generics ::columnar::VecRow for #struct_name_ident #ty_generics #where_clause {
        //     type Input
        // }
    );
    Ok(ret)
}
