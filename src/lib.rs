extern crate proc_macro;

use quote::quote;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn hello(_: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = syn::parse_macro_input!(item);
    let name = &input.ident;
    let result =  quote! {
        fn #name() {
            println!("Hello {}!", stringify!(#name));
        }
    };
    result.into()
}
