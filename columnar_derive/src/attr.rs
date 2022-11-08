use syn::{DeriveInput, Field};

use darling::FromField;

#[derive(FromField, Debug)]
#[darling(attributes(columnar))]
pub struct FieldArgs {
    pub ident: Option<syn::Ident>,
    pub vis: syn::Visibility,
    pub ty: syn::Type,
    pub attrs: Vec<syn::Attribute>,
    // custom attributes
    pub index: Option<usize>,
    pub strategy: Option<String>,
    pub item: Option<syn::Type>,
    pub original_type: Option<syn::Type>,
    #[darling(rename = "type")]
    pub _type: Option<String>,
}

pub fn get_fields_add_serde_with_to_field(st: &mut DeriveInput) -> syn::Result<Vec<FieldArgs>> {
    let fields = match &mut st.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => return Err(syn::Error::new_spanned(st, "expected struct")),
    };
    let mut index = 1usize;
    let mut fields_args = Vec::with_capacity(fields.len());
    for field in fields.iter() {
        let mut field_args = FieldArgs::from_field(field)?;
        if let Some(_index) = field_args.index {
            index = _index + 1;
        } else {
            field_args.index = Some(index);
            index += 1;
        }
        fields_args.push(field_args);
    }
    for (idx, field) in fields.iter_mut().enumerate() {
        add_serde_with_to_field(field, &fields_args[idx])?;
    }
    Ok(fields_args)
}

fn add_serde_with_to_field(field: &mut Field, args: &FieldArgs) -> syn::Result<()> {
    if let Some(as_arg) = &args._type {
        if as_arg == "vec" {
            field.attrs.extend([
                syn::parse_quote!(#[serde(serialize_with = "::columnar::VecRow::serialize_columns")]),
                syn::parse_quote!(#[serde(deserialize_with = "::columnar::VecRow::deserialize_columns")]),
            ]);
        } else if as_arg == "map" {
            field.attrs.extend([
                syn::parse_quote!(#[serde(serialize_with = "::columnar::MapRow::serialize_columns")]),
                syn::parse_quote!(#[serde(deserialize_with = "::columnar::MapRow::deserialize_columns")]),
            ]);
        } else {
            return Err(syn::Error::new_spanned(
                field,
                "expected `Vec` or `Map` as value of `as`",
            ));
        }
    }
    Ok(())
}
