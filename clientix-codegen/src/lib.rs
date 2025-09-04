mod method;
mod client;
mod return_kind;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
use clientix_core::prelude::reqwest::Method;
use crate::client::parse_client;
use crate::method::parse_method;

#[proc_macro_attribute]
pub fn clientix(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_client(item, attrs)
}

#[proc_macro_attribute]
pub fn get(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::GET, item, attrs)
}

#[proc_macro_attribute]
pub fn post(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::POST, item, attrs)
}

#[proc_macro_attribute]
pub fn put(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::PUT, item, attrs)
}

#[proc_macro_attribute]
pub fn delete(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::DELETE, item, attrs)
}

#[proc_macro_attribute]
pub fn head(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::HEAD, item, attrs)
}

#[proc_macro_attribute]
pub fn options(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::OPTIONS, item, attrs)
}

#[proc_macro_attribute]
pub fn patch(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::PATCH, item, attrs)
}

#[proc_macro_attribute]
pub fn header(attrs: TokenStream, item: TokenStream) -> TokenStream {
    TokenStream::default() // TODO:
}

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