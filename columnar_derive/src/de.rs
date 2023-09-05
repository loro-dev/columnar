use std::collections::BTreeSet;

use proc_macro2::Span;

use crate::args::{Args, FieldArgs};

pub enum BorrowedLifetimes {
    Borrowed(BTreeSet<syn::Lifetime>),
    Static,
}

impl BorrowedLifetimes {
    pub fn de_lifetime(&self) -> syn::Lifetime {
        match *self {
            BorrowedLifetimes::Borrowed(_) => syn::Lifetime::new("'de", Span::call_site()),
            BorrowedLifetimes::Static => syn::Lifetime::new("'static", Span::call_site()),
        }
    }

    pub fn de_lifetime_param(&self) -> Option<syn::LifetimeParam> {
        match self {
            BorrowedLifetimes::Borrowed(bounds) => Some(syn::LifetimeParam {
                attrs: Vec::new(),
                lifetime: syn::Lifetime::new("'de", Span::call_site()),
                colon_token: None,
                bounds: bounds.iter().cloned().collect(),
            }),
            BorrowedLifetimes::Static => None,
        }
    }
}

// The union of lifetimes borrowed by each field of the container.
//
// These turn into bounds on the `'de` lifetime of the Deserialize impl. If
// lifetimes `'a` and `'b` are borrowed but `'c` is not, the impl is:
//
//     impl<'de: 'a + 'b, 'a, 'b, 'c> Deserialize<'de> for S<'a, 'b, 'c>
//
// If any borrowed lifetime is `'static`, then `'de: 'static` would be redundant
// and we use plain `'static` instead of `'de`.
pub fn borrowed_lifetimes(fields: &[FieldArgs]) -> syn::Result<BorrowedLifetimes> {
    let mut lifetimes = BTreeSet::new();
    for field in fields {
        if !field.skip && field.has_borrow_lifetime() {
            lifetimes.extend(field.borrow_lifetimes()?.unwrap().iter().cloned());
        }
    }
    if lifetimes.iter().any(|b| b.to_string() == "'static") {
        Ok(BorrowedLifetimes::Static)
    } else {
        Ok(BorrowedLifetimes::Borrowed(lifetimes))
    }
}
