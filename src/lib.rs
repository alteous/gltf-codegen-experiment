#![allow(bare_trait_objects)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;

fn extract_documentation(attributes: &[syn::Attribute]) -> Option<String> {
    for attr in attributes {
        let mut stream = proc_macro2::TokenStream::new();
        attr.path.to_tokens(&mut stream);
        let key = stream.to_string();
        if key.starts_with("doc") {
            let tts = &attr.tts;
            let doc_attr = tts.to_string();
            if let Some(tail) = doc_attr.split("= \"").nth(1) {
                if let Some(docs) = tail.rsplit("\"").nth(1) {
                    return Some(docs.to_string())
                }
            }
        }
    }
    None
}

#[proc_macro_attribute]
pub fn hello(_: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = syn::parse_macro_input!(item);
    let name = &input.ident;
    let docs = extract_documentation(&input.attrs)
        .unwrap_or_else(|| " Undocumented.".to_string());
    let result = quote! {
        #[doc = #docs]
        fn #name() {
            println!("Hello {}! {:?}", stringify!(#name), #docs);
        }
    };
    result.into()
}

fn stringify(item: &quote::ToTokens) -> String {
    let mut token_stream = proc_macro2::TokenStream::new();
    item.to_tokens(&mut token_stream);
    token_stream.to_string()
}

fn is_trivial_type(ident: &syn::Ident) -> bool {
    ["bool", "u32", "i32", "String", "RawValue"]
        .iter()
        .find(|x| ident == x)
        .is_some()
}

fn parse_named_fields(named_fields: syn::FieldsNamed)
    -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>)
{
    let mut parsed_fields = vec![];
    let mut parsed_impls = vec![];
    for field in named_fields.named.iter() {
        let ident = field.ident.as_ref().unwrap();
        let _docs = extract_documentation(&field.attrs).unwrap_or_else(|| "?".to_string());
        let type_path = match &field.ty {
            syn::Type::Path(type_path) => type_path,
            _ => panic!("cannot handle this type"),
        };
        let segment = type_path.path.segments.iter().last().unwrap();
        if segment.ident == "Index" {
            let _type_argument = match &segment.arguments {
                syn::PathArguments::AngleBracketed(path_args) => {
                    match path_args.args.iter().next().unwrap() {
                        syn::GenericArgument::Type(ty) => ty,
                        _ => panic!("malformed Index type argument"),
                    }
                },
                _ => panic!("malformed Index type"),
            };
            parsed_fields.push(
                quote! {
                    pub #ident : #type_path,
                }
            );
            parsed_impls.push(
                quote! {
                    pub fn #ident (&mut self, index: u32) -> &mut Self {
                        self.#ident = Index::new(index);
                        self
                    }
                }
            );
        } else if is_trivial_type(&segment.ident) {
            parsed_fields.push(
                quote! {
                    pub #ident : #type_path,
                }
            );
            parsed_impls.push(
                quote! {
                    pub fn #ident (&mut self, #ident: #type_path ) -> &mut Self {
                        self.#ident = #ident;
                        self
                    }
                }
            );
        } else {
            panic!("unknown type `{}`", stringify(&field.ty));
        }
    }
    (parsed_fields, parsed_impls)
}

#[proc_macro_derive(Wrapper)]
pub fn derive_wrapper(input: TokenStream) -> TokenStream {
    let item: syn::ItemStruct = syn::parse_macro_input!(input);
    let (fields, impls) = match item.fields {
        syn::Fields::Named(named_fields) => parse_named_fields(named_fields),
        _ => panic!("#[derive(Wrapper)] only works on structs with named fields"),
    };

    let result = quote! {
        #[derive(Debug)]
        struct GeneratedStruct {
            #(
                #fields
            )*
        }

        impl GeneratedStruct {
            #(
                #impls
            )*
        }
    };
    result.into()
}
