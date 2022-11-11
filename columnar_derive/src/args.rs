use darling::{Error as DarlingError, FromField, FromMeta};
use syn::{AttributeArgs, DeriveInput};

use crate::attr::{add_serde_skip, add_serde_with};

#[derive(Debug, FromMeta)]
pub struct DeriveArgs {
    #[darling(default)]
    pub(crate) vec: bool,
    #[darling(rename = "map", default)]
    pub(crate) hashmap: bool,
    #[darling(default)]
    pub(crate) ser: bool,
    #[darling(default)]
    pub(crate) de: bool,
}

#[derive(FromField, Debug)]
#[darling(attributes(columnar))]
pub struct FieldArgs {
    pub ident: Option<syn::Ident>,
    pub vis: syn::Visibility,
    pub ty: syn::Type,
    pub attrs: Vec<syn::Attribute>,
    // custom attributes
    /// The index of the field in the struct, starts from 0 default.
    pub index: Option<usize>,
    /// the strategy to convert the field values to a column.
    pub strategy: Option<String>,
    /// the original type of the field, in order to adjust whether the field type is number type, if field type is alias.
    pub original_type: Option<syn::Type>,
    /// the type of the column format, vec or map.
    #[darling(rename = "type")]
    pub _type: Option<String>,
    /// If skip, this field will be ignored.
    #[darling(default)]
    pub skip: bool,
}

pub fn get_derive_args(args: AttributeArgs) -> syn::Result<DeriveArgs> {
    match DeriveArgs::from_list(&args) {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!("get_derive_args error: {}", e);
            Err(DarlingError::unsupported_format(
                "columnar only supports attributes with `vec`, `map` and `ser`, `de`",
            )
            .into())
        }
    }
}

pub fn get_field_args_add_serde_with_to_field(
    st: &mut DeriveInput,
    derive_args: &DeriveArgs,
) -> syn::Result<Vec<FieldArgs>> {
    let fields = match &mut st.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => {
            return Err(syn::Error::new_spanned(
                st,
                "only supported named struct type",
            ))
        }
    };
    let mut index = 0;
    let mut fields_args = Vec::with_capacity(fields.len());
    for field in fields.iter_mut() {
        let mut field_args = FieldArgs::from_field(field)?;
        add_serde_skip(field, &field_args)?;
        if field_args.skip {
            fields_args.push(field_args);
            continue;
        }
        add_serde_with(field, &field_args, derive_args)?;
        if let Some(_index) = field_args.index {
            index = _index + 1;
        } else {
            field_args.index = Some(index);
            index += 1;
        }
        fields_args.push(field_args);
    }
    Ok(fields_args)
}
