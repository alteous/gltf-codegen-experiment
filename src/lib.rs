extern crate proc_macro;

use quote::quote;
use quote::ToTokens;
use proc_macro::TokenStream;

fn find_and_extract_documentation(attributes: &[syn::Attribute]) -> Option<String> {
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
    let docs = find_and_extract_documentation(&input.attrs)
        .unwrap_or_else(|| " Undocumented.".to_string());
    let result = quote! {
        #[doc = #docs]
        fn #name() {
            println!("Hello {}! {:?}", stringify!(#name), #docs);
        }
    };
    result.into()
}
