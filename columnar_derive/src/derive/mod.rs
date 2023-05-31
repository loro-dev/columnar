mod map;
mod serde;
mod utils;
mod vec;
use crate::args::{DeriveArgs, FieldArgs};
use darling::Error as DarlingError;
use syn::DeriveInput;

use self::{
    map::{generate_derive_hashmap_row_de, generate_derive_hashmap_row_ser},
    vec::{generate_derive_vec_row_de, generate_derive_vec_row_ser},
};

pub fn process_derive_args(
    derive_args: &DeriveArgs,
    input: &DeriveInput,
    field_args: &Vec<FieldArgs>,
) -> Result<proc_macro2::TokenStream, DarlingError> {
    let mut tokens = proc_macro2::TokenStream::new();

    if derive_args.compatible {
        if derive_args.ser {
            let compatible_ser = serde::generate_compatible_ser(input, field_args)?;
            tokens.extend(compatible_ser);
        }
        if derive_args.de {
            let compatible_de = serde::generate_compatible_de(input, field_args)?;
            tokens.extend(compatible_de);
        }
    }

    if derive_args.vec {
        if derive_args.ser {
            let vec = generate_derive_vec_row_ser(input, field_args)?;
            tokens.extend(vec);
        }
        if derive_args.de {
            let vec = generate_derive_vec_row_de(input, field_args)?;
            tokens.extend(vec);
        }
    }
    if derive_args.hashmap {
        if derive_args.ser {
            let map = generate_derive_hashmap_row_ser(input, field_args)?;
            tokens.extend(map);
        }
        if derive_args.de {
            let map = generate_derive_hashmap_row_de(input, field_args)?;
            tokens.extend(map);
        }
    }
    Ok(tokens)
}
