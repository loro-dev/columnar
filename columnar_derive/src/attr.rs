use syn::parse_quote;

use crate::args::{Args, AsType, DeriveArgs};

pub trait ContainerFieldVariant: quote::ToTokens {
    fn get_attr_mut(&mut self) -> &mut Vec<syn::Attribute>;
    fn add_attr(&mut self, attr: syn::Attribute) {
        self.get_attr_mut().push(attr);
    }
}

impl ContainerFieldVariant for syn::Field {
    fn get_attr_mut(&mut self) -> &mut Vec<syn::Attribute> {
        &mut self.attrs
    }
}

impl ContainerFieldVariant for syn::Variant {
    fn get_attr_mut(&mut self) -> &mut Vec<syn::Attribute> {
        &mut self.attrs
    }
}

pub fn add_serde_skip<C: ContainerFieldVariant, A: Args>(cfv: &mut C, args: &A) -> syn::Result<()> {
    if args.skip() {
        let attr = parse_quote! {
            #[serde(skip)]
        };
        cfv.add_attr(attr);
    }
    Ok(())
}

pub fn add_serde_with<C: ContainerFieldVariant, A: Args>(
    cfv: &mut C,
    args: &A,
    derive_args: &DeriveArgs,
) -> syn::Result<()> {
    if let Some(as_arg) = &args._type() {
        match as_arg {
            AsType::Vec => {
                if derive_args.ser {
                    cfv.add_attr(parse_quote! {
                        #[serde(serialize_with = "::serde_columnar::RowSer::serialize_columns")]
                    });
                }
                if derive_args.de {
                    cfv.add_attr(parse_quote! {
                        #[serde(deserialize_with = "::serde_columnar::RowDe::deserialize_columns")]
                    });
                }
            }
            AsType::Map => {
                if derive_args.ser {
                    cfv.add_attr(parse_quote! {
                        #[serde(serialize_with = "::serde_columnar::KeyRowSer::serialize_columns")]
                    });
                }
                if derive_args.de {
                    cfv.add_attr(parse_quote! {
                        #[serde(deserialize_with = "::serde_columnar::KeyRowDe::deserialize_columns")]
                    });
                }
            }
            AsType::Other => {
                return Err(syn::Error::new_spanned(
                    cfv,
                    "expected `vec` or `map` as value of `type`",
                ));
            }
        }
    }
    Ok(())
}
