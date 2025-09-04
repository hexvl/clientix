mod method;
mod client;
mod return_kind;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
use crate::client::parse_client;
use crate::method::{parse_method, HttpMethod};

#[proc_macro_attribute]
pub fn clientix(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_client(item, attrs)
}

#[proc_macro_attribute]
pub fn get(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(HttpMethod::Get, item, attrs)
}

#[proc_macro_attribute]
pub fn post(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(HttpMethod::Post, item, attrs)
}

#[proc_macro_attribute]
pub fn put(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(HttpMethod::Put, item, attrs)
}

#[proc_macro_attribute]
pub fn delete(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(HttpMethod::Delete, item, attrs)
}

// TODO: implemented other methods: PATCH, OPTION and etc

#[proc_macro_attribute]
pub fn data_transfer(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    TokenStream::from(quote! {
        #[derive(clientix::prelude::serde::Serialize, clientix::prelude::serde::Deserialize, Debug, Clone)]
        #[serde(crate = "clientix::prelude::serde")]
        #item
    })
}

// TODO: implemented HTTP-methods based independent functions
// TODO: implemented building client based struct params