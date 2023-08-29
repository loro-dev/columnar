use quote::ToTokens;
use syn::{parse_quote, GenericArgument, Generics, PathArguments, Type, WherePredicate};

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

pub fn generate_generics_phantom(generics: &Generics) -> proc_macro2::TokenStream {
    let mut phantom_data_fields = proc_macro2::TokenStream::new();
    for param in generics.params.iter() {
        match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                let field = quote::quote! {
                    std::marker::PhantomData::<#ident>,
                };
                phantom_data_fields.extend(field);
            }
            syn::GenericParam::Lifetime(lifetime_def) => {
                let lifetime = &lifetime_def.lifetime;
                let field = quote::quote! {
                    std::marker::PhantomData::<&#lifetime ()>,
                };
                phantom_data_fields.extend(field);
            }
            _ => {}
        }
    }
    phantom_data_fields
}
