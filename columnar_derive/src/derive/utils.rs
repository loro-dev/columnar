use crate::args::FieldArgs;
use syn::WherePredicate;

/// For example
///
/// ```
/// type ID = u64;
/// struct Data{
///     id: u32,   // this is type literal
///     id2: ID,   // this is type literal
///     bool: bool, // this is type literal
/// }
/// ```
// pub fn get_without_generic_type_literal_by_syn_type(ty: &syn::Type) -> syn::Result<Option<String>> {
//     let mut type_literal = String::new();
//     match ty {
//         syn::Type::Path(syn::TypePath { path, .. }) => {
//             if let Some(ident) = path.get_ident() {
//                 type_literal.push_str(&ident.to_string());
//             } else {
//                 return Ok(None);
//             }
//         }
//         _ => return Ok(None),
//     }
//     Ok(Some(type_literal))
// }

/// For example
/// ```rust, ignore
/// struct Data{
///     id: u32,   // this is num type
///     id2: ID,   // this is num type too, but it need to be annotated with #[columnar(original_type = "u64")]
/// }
/// ```
pub fn is_field_type_is_can_copy(field_arg: &FieldArgs) -> syn::Result<bool> {
    let mut field_type = &field_arg.ty;
    if let Some(original_type) = &field_arg.original_type {
        field_type = original_type;
    };
    if let syn::Type::Path(syn::TypePath { path, .. }) = field_type {
        let path = path.clone();
        if path.is_ident("i8")
            || path.is_ident("i16")
            || path.is_ident("i32")
            || path.is_ident("i64")
            || path.is_ident("i128")
            || path.is_ident("u8")
            || path.is_ident("u16")
            || path.is_ident("u32")
            || path.is_ident("u64")
            || path.is_ident("u128")
            || path.is_ident("f32")
            || path.is_ident("f64")
            || path.is_ident("isize")
            || path.is_ident("usize")
            || path.is_ident("bool")
        {
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

pub fn add_generics_clause_to_where(
    generics: Vec<WherePredicate>,
    where_clause: Option<&syn::WhereClause>,
) -> syn::WhereClause {
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    for generic in generics {
        where_clause.predicates.push(generic);
    }
    where_clause
}
