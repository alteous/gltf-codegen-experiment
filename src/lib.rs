extern crate proc_macro;

use quote::quote;
use proc_macro::TokenStream;

fn get_docs(attr: &syn::Attribute) -> String {
    let tts = &attr.tts;
    let doc_attr = tts.to_string();
    let tail = doc_attr.split("= \"").nth(1).unwrap();
    tail.rsplit("\"").nth(1).unwrap().to_string()
}

#[proc_macro_attribute]
pub fn hello(_: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = syn::parse_macro_input!(item);
    let name = &input.ident;
    let docs = get_docs(&input.attrs[0]);
    let result = quote! {
        fn #name() {
            println!("Hello {}! {:?}", stringify!(#name), #docs);
        }
    };
    result.into()
}
