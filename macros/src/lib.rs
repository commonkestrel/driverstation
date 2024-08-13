use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
use proc_macro2::Span;
use syn::{Ident, spanned::Spanned, Data, DeriveInput, LitByteStr, Meta, PathArguments};

enum Fields {
    None,
    Instance {
        ident: Option<Ident>,
    },
    Context {
        instance: Ident,
        context: Ident,
    },
}

struct Entry {
    ident: Ident,
    fields: Fields,
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
        let fields = match variant.fields.len() {
            0 => Fields::None,
            1 => Fields::Instance {
                ident: variant.fields.into_iter().next().unwrap().ident,
            },
            _ => {
                let mut fields = variant.fields.into_iter();
                let instance_field = fields.next().unwrap();

                let instance = match instance_field.ident {
                    Some(ident) => ident,
                    None => return syn::Error::new(instance_field.span(), "dual-field entries must have named fields").into_compile_error().into(),
                };

                let context_field = fields.next().unwrap();

                let context = match context_field.ident {
                    Some(ident) => ident,
                    None => return syn::Error::new(context_field.span(), "dual-field entries must have named fields").into_compile_error().into(),
                };

                Fields::Context {instance, context}
            },
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

        let s = meta.span();
        let indicator: LitByteStr = match meta {
            Meta::List(list) => {
                match list.parse_args() {
                    Ok(indicator) => indicator,
                    Err(_) => return syn::Error::new(list.span(), "the `entry` attribute expects a single byte literal").into_compile_error().into(),
                }
            },
            _ => return syn::Error::new(s, "the `entry` attribute requires the entry indicator as an argument").into_compile_error().into(),
        };

        let entry = Entry {ident: variant.ident, fields};

        let s = indicator.span();
        if let Some(_) = entries.insert(indicator, entry) {
            return syn::Error::new(s, "duplicate indicator found").into_compile_error().into();
        }
    }

    let mut key_pairs: Vec<(LitByteStr, Entry)> = entries.into_iter().collect();
    key_pairs.sort_by(|(a, _), (b, _)| a.value().len().cmp(&b.value().len()));

    let mut parse = proc_macro2::TokenStream::new();
    for (indicator, entry) in key_pairs {
        let parse_block = match entry.fields {
            Fields::None => {
                quote! {

                }
            },
            Fields::Instance { ident } => {
                todo!()
            },
            Fields::Context { instance, context } => {
                todo!()
            }
        };

        parse = quote! {
            #parse
            #parse_block
        }
    }

    quote! {
        impl #ident {
            fn parse_entries(source: ::std::ffi::CString) -> ::std::vec::Vec<#ident> {
                let bytes = source.into_bytes();

                let mut i = 0;
                let mut entries = ::std::vec::Vec::new();
                while i < bytes.len() {
                    #parse

                    i += 1;
                }

                entries
            }
        }
    }.into()
}
