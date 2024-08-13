use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use proc_macro2::Span;
use syn::{Ident, spanned::Spanned, Data, DeriveInput, LitByteStr, Meta, PathArguments};

enum Feature {
    None,
    Instance,
    Context,
}

enum Entry {
    None {
        ident: Ident
    },
    Instance {
        ident: Ident,
        instance: Option<Ident>,
    },
    Context {
        ident: Ident,
        instance: Ident,
        context: Ident,
    }
}

#[proc_macro_derive(ParseEntries, attributes(entry))]
pub fn parse_entries(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let ident = input.ident;

    let data = match input.data {
        Data::Enum(data) => data,
        _ => return syn::Error::new(Span::call_site(), "ParseEntries may only be used on an enum").into_compile_error().into(),
    };

    let mut entries: HashMap<LitByteStr, Entry> = HashMap::new();

    for variant in data.variants {
        let entry = match variant.fields.len() {
            0 => Feature::None,
            1 => Feature::Instance,
            _ => Feature::Context,
        };

        let meta = match variant.attrs.into_iter().find_map(|attr| {
            let ident = attr.meta.path().get_ident()?;
            if ident != "entry" {
                return None;
            }

            Some(attr.meta)
        }) {
            Some(value) => value,
            None => continue,
        };

        
    }

    todo!()
}
