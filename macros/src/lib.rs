use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use proc_macro2::Span;
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, ExprClosure, Ident, LitByteStr, Meta, Path, PathArguments, Token};

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
    callback: Option<Callback>,
}

struct Attr {
    indicator: LitByteStr,
    callback: Option<Callback>,
}

impl Parse for Attr {
    fn parse(mut input: syn::parse::ParseStream) -> syn::Result<Self> {
        let indicator: LitByteStr = input.parse()?;
        let callback = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;

            if input.peek(syn::Ident) {
                let path = Path::parse(input)?;
                Some(Callback::Label(path))
            } else {
                todo!()
            }
        } else {
            None
        };

        Ok(Attr {
            indicator, callback,
        })
    }
}

enum Callback {
    Label(Path),
    Inline {
        i_ident: Ident,
        bytes_ident: Ident,
        body: proc_macro2::TokenStream,
    }
}

impl ToTokens for Callback {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Callback::Label(label) => quote!{ #label },
            Callback::Inline { i_ident, bytes_ident, body } => quote! {
                (|#i_ident, #bytes_ident| #body)
            },
        })
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
        let attr: Attr = match meta {
            Meta::List(list) => {
                match list.parse_args() {
                    Ok(indicator) => indicator,
                    Err(_) => return syn::Error::new(list.span(), "the `entry` attribute expects a single byte literal and an optional callback").into_compile_error().into(),
                }
            },
            _ => return syn::Error::new(s, "the `entry` attribute requires the entry indicator as an argument").into_compile_error().into(),
        };

        let entry = Entry {ident: variant.ident, fields, callback: attr.callback};

        let s = attr.indicator.span();
        if let Some(_) = entries.insert(attr.indicator, entry) {
            return syn::Error::new(s, "duplicate indicator found").into_compile_error().into();
        }
    }

    let mut key_pairs: Vec<(LitByteStr, Entry)> = entries.into_iter().collect();
    key_pairs.sort_by(|(a, _), (b, _)| a.value().len().cmp(&b.value().len()));

    let mut parse = proc_macro2::TokenStream::new();
    for (i, (indicator, entry)) in key_pairs.into_iter().enumerate() {
        let if_stmt = if i == 0 {
            quote! { if }
        } else {
            quote! { else if }
        };

        // Gather the bytes of the indicator into a TokenStream array to compare
        let mut indicator_bytes = proc_macro2::TokenStream::new();
        for byte in indicator.value() {
            let byte_token = quote! { #byte, };
            indicator_bytes.extend(byte_token);
        }
        let indicator_array = quote!{ [#indicator_bytes] };

        let entry_ident = entry.ident;

        let parse_block = match entry.fields {
            Fields::None => {
                let callback = if let Some(callback) = entry.callback {
                    quote! {
                        entries.push((#callback)(&mut i, &bytes));
                    }
                } else {
                    quote! {
                        entries.push(#ident::#entry_ident);
                    }
                };

                quote! {
                    #if_stmt bytes[i..].starts_with(&#indicator_array) {
                        #callback
                    }
                }
            },
            Fields::Instance { ident: field_ident } => {
                let get_entry = if let Some(callback) = entry.callback {
                    quote! {
                        let entry = match #callback(&mut i, &bytes) {
                            Some(entry) => entry,
                            None => {
                                i += 1;
                                continue;
                            }
                        };
                    }
                } else if let Some(field_ident) = field_ident {
                    quote! {
                        let instance = match Self::parse_instance(&mut i, &bytes) {
                            Some(instance) => instance,
                            None => {
                                i += 1;
                                continue;
                            }
                        };

                        let entry = #ident::#entry_ident {
                            #field_ident: instance,
                        };
                    }
                } else {
                    quote! {
                        let instance = match Self::parse_instance(&mut i, &bytes) {
                            Some(instance) => instance,
                            None => {
                                i += 1;
                                continue;
                            }
                        };

                        let entry = #ident::#entry_ident(instance);
                    }
                };

                quote! {
                    #if_stmt bytes[i..].starts_with(&#indicator_array) {
                        #get_entry
                        entries.push(entry);
                    }
                }
            },
            Fields::Context { instance: instance_ident, context: context_ident } => {
                let get_entry = if let Some(callback) = entry.callback {
                    quote! {
                        let entry = match #callback(&mut i, &bytes) {
                            Some(entry) => entry,
                            None => {
                                i += 1;
                                continue;
                            }
                        };
                    }
                } else {
                    quote! {
                        let instance = match Self::parse_instance(&mut i, &bytes) {
                            Some(instance) => instance,
                            None => {
                                i += 1;
                                continue;
                            }
                        };

                        let context = match Self::parse_context(&mut i, &bytes) {
                            Some(context) => context,
                            None => {
                                i += 1;
                                continue;
                            }
                        };

                        let entry = #ident::#entry_ident {
                            #instance_ident: instance,
                            #context_ident: context,
                        };
                    }
                };


                quote! {
                    #if_stmt bytes[i..].starts_with(&#indicator_array) {
                        #get_entry
                        entries.push(entry);
                    }
                }
            }
        };

        parse.extend(parse_block);
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

            fn parse_instance<Dst: From<u8>>(i: &mut usize, bytes: &[u8]) -> Option<Dst> {
                *i += 1;
                bytes.get(*i).map(|byte| Dst::from(*byte))
            }

            fn parse_context<Dst: From<u8>>(i: &mut usize, bytes: &[u8]) -> Option<Dst> {
                if bytes[*i + 1] == b':' {
                    *i += 2;
                    bytes.get(*i).map(|byte| Dst::from(*byte))
                } else {
                    None
                }
            }
        }
    }.into()
}
