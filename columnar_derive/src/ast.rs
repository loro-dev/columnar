use darling::FromField;
use syn::{punctuated::Punctuated, Token};

use crate::{args::FieldArgs, attr::Style};

pub fn struct_from_ast(fields: &syn::Fields) -> syn::Result<(Style, Vec<FieldArgs>)> {
    let ans = match fields {
        syn::Fields::Named(fields) => (Style::Struct, fields_from_ast(&fields.named)?),
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            (Style::Newtype, fields_from_ast(&fields.unnamed)?)
        }
        syn::Fields::Unnamed(fields) => (Style::Tuple, fields_from_ast(&fields.unnamed)?),
        syn::Fields::Unit => (Style::Unit, Vec::new()),
    };
    Ok(ans)
}

fn fields_from_ast(fields: &Punctuated<syn::Field, Token![,]>) -> syn::Result<Vec<FieldArgs>> {
    let mut ans = Vec::with_capacity(fields.len());
    for field in fields {
        let args = FieldArgs::from_field(field)?;
        ans.push(args);
    }
    Ok(ans)
}
