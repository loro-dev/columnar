use proc_macro2::TokenStream;
use syn;

fn has_attribute(field: &syn::Field, attribute_name: &str) -> bool {
    for attribute in &field.attrs {
        if attribute.path().is_ident(attribute_name) {
            return true;
        }
    }
    false
}

pub fn expand_derive_analyze(st: &mut syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let struct_name = st.ident.clone();
    match &mut st.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            let content = derive_analyze_struct(named)?;

            // println!("{}", content);
            let derive = quote::quote! {
                    #[automatically_derived]
                    impl ::serde_columnar::FieldAnalyze for #struct_name {
                        fn analyze(&self) -> ::serde_columnar::AnalyzeResults {
                            #content
                        }
                    }
            };
            // println!("{}", derive);
            Ok(derive)
        }
        _ => Err(vec![syn::Error::new_spanned(
            st,
            "only supported named struct",
        )]),
    }
}

fn derive_analyze_struct(
    fields: &mut syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Result<TokenStream, Vec<syn::Error>> {
    let mut per_field_size = Vec::new();
    let field_names = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().clone())
        .collect::<Vec<_>>();
    for field in fields.iter().filter(|f| has_attribute(f, "analyze")) {
        per_field_size.push(generate_per_field_result(field, &field_names)?)
    }
    Ok(quote::quote! {
        let mut ans = Vec::new();
        let total_size = serde_columnar::to_vec(&self).unwrap().len();
        #(#per_field_size)*
        ans.into()
    })
}

fn generate_per_field_result(
    field: &syn::Field,
    field_names: &[syn::Ident],
) -> Result<TokenStream, Vec<syn::Error>> {
    let field_ident = field.ident.as_ref().unwrap().clone();
    let field_name = syn::Ident::new(
        &format!("field_{}_size", field_ident),
        proc_macro2::Span::call_site(),
    );
    let other_fields = field_names
        .iter()
        .filter(|f| *f != &field_ident)
        .map(|f| {
            quote::quote! {
                #f: self.#f.clone(),
            }
        })
        .collect::<Vec<_>>();
    Ok(quote::quote! {
        let #field_name = {
            let new_self = Self {
                #field_ident: Default::default(),
                #(#other_fields)*
            };
            let new_size = ::serde_columnar::to_vec(&new_self).unwrap().len();
            total_size - new_size
        };
        ans.push(::serde_columnar::AnalyzeResult {
            field_name: stringify!(#field_ident).to_string(),
            binary_size: #field_name,
        });
    })
}
