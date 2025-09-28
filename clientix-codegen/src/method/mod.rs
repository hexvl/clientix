mod method;
mod header;
mod signature;

pub use method::*;

use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use clientix_core::prelude::reqwest::Method;
use crate::method::header::HeaderCompiler;

pub fn parse_method(method: Method, item: TokenStream, attrs: TokenStream) -> TokenStream {
    let method_config = MethodCompiler::parse(method, item, attrs);

    let compiled_declaration = method_config.compile_declaration();

    let expanded = quote! {
        #compiled_declaration
    };

    TokenStream::from(expanded)
}

pub fn parse_header(item: TokenStream, attrs: TokenStream) -> TokenStream {
    HeaderCompiler::parse(TokenStream2::from(attrs), true);
    TokenStream::from(item)
}