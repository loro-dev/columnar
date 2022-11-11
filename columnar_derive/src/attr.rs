use syn::{parse_quote, Field};

use crate::args::{DeriveArgs, FieldArgs};

pub fn add_serde_skip(field: &mut Field, args: &FieldArgs) -> syn::Result<()> {
    if args.skip {
        let attr = parse_quote! {
            #[serde(skip)]
        };
        field.attrs.push(attr);
    }
    Ok(())
}

pub fn add_serde_with(
    field: &mut Field,
    args: &FieldArgs,
    derive_args: &DeriveArgs,
) -> syn::Result<()> {
    if let Some(as_arg) = &args._type {
        if as_arg == "vec" {
            if derive_args.ser {
                field.attrs.push(parse_quote! {
                    #[serde(serialize_with = "::columnar::RowSer::serialize_columns")]
                });
            }
            if derive_args.de {
                field.attrs.push(parse_quote! {
                    #[serde(deserialize_with = "::columnar::RowDe::deserialize_columns")]
                });
            }
        } else if as_arg == "map" {
            if derive_args.ser {
                field.attrs.push(parse_quote! {
                    #[serde(serialize_with = "::columnar::KeyRowSer::serialize_columns")]
                });
            }
            if derive_args.de {
                field.attrs.push(parse_quote! {
                    #[serde(deserialize_with = "::columnar::KeyRowDe::deserialize_columns")]
                });
            }
        } else {
            return Err(syn::Error::new_spanned(
                field,
                "expected `vec` or `map` as value of `type`",
            ));
        }
    }
    Ok(())
}
