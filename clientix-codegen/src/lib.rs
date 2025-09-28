mod client;
mod method;
mod utils;
mod attributes;
mod derive;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, ItemStruct};
use clientix_core::prelude::reqwest::Method;
use crate::client::parse_client;
use crate::method::parse_header;
use crate::method::parse_method;
use syn::parse::Parser;
use crate::utils::throw_error;

/**
A procedural macro for building an HTTP client. It includes the following attributes:
- url - the base part of the client’s URL, e.g. http://localhost:8080
- path - an additional part of the URL path that precedes method paths
- async - if true, the client is asynchronous; otherwise, it is blocking
*/
#[proc_macro_attribute]
pub fn clientix(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: научить clientix работать со структурными объектам, а не только с trait
    parse_client(item, attrs)
}

/**
A procedural macro for building an HTTP GET method of trait. It includes the following attributes:
- path - a part of the URL path (String)
- consumes - content type for request, support: application/json, application/xml, application/x-www-form-urlencoded (String)
- produces - accept type for response, support: application/json, application/xml, application/x-www-form-urlencoded (String)

GET method supports argument macros:
- #[segment] - maps method arguments to path segments and header placeholders (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])

*/
#[proc_macro_attribute]
pub fn get(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: научить get работать с независимыми функциями и со структурными методами (блок impl)
    parse_method(Method::GET, item, attrs)
}

/**
A procedural macro for building an HTTP POST method of trait. It includes the following attributes:
- path - a part of the URL path (String)
- consumes - content type for request, support: application/json, application/xml, application/x-www-form-urlencoded (String)
- produces - accept type for response, support: application/json, application/xml, application/x-www-form-urlencoded (String)

POST method supports argument macros:
- #[segment] - maps method arguments to path segments and header placeholders (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])

RequestBody must implement the #[data_transfer] macro.
*/
#[proc_macro_attribute]
pub fn post(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: научить post работать с независимыми функциями и со структурными методами (блок impl)
    parse_method(Method::POST, item, attrs)
}

/**
A procedural macro for building an HTTP PUT method of trait. It includes the following attributes:
- path - a part of the URL path (String)
- consumes - content type for request, support: application/json, application/xml, application/x-www-form-urlencoded (String)
- produces - accept type for response, support: application/json, application/xml, application/x-www-form-urlencoded (String)

PUT method supports argument macros:
- #[segment] - maps method arguments to path segments and header placeholders (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])

RequestBody must implement the #[data_transfer] macro.
*/
#[proc_macro_attribute]
pub fn put(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: научить put работать с независимыми функциями и со структурными методами (блок impl)
    parse_method(Method::PUT, item, attrs)
}

/**
A procedural macro for building an HTTP DELETE method of trait. It includes the following attributes:
- path - a part of the URL path (String)
- consumes - content type for request, support: application/json, application/xml, application/x-www-form-urlencoded (String)
- produces - accept type for response, support: application/json, application/xml, application/x-www-form-urlencoded (String)

DELETE method supports argument macros:
- #[segment] - maps method arguments to path segments and header placeholders (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])

*/
#[proc_macro_attribute]
pub fn delete(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: научить delete работать с независимыми функциями и со структурными методами (блок impl)
    parse_method(Method::DELETE, item, attrs)
}

/**
A procedural macro for building an HTTP HEAD method of trait. It includes the following attributes:
- path - a part of the URL path (String)
- consumes - content type for request, support: application/json, application/xml, application/x-www-form-urlencoded (String)
- produces - accept type for response, support: application/json, application/xml, application/x-www-form-urlencoded (String)

HEAD method supports argument macros:
- #[segment] - maps method arguments to path segments and header placeholders (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])

*/
#[proc_macro_attribute]
pub fn head(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: научить head работать с независимыми функциями и со структурными методами (блок impl)
    parse_method(Method::HEAD, item, attrs)
}

/**
A procedural macro for building an HTTP PATCH method of trait. It includes the following attributes:
- path - a part of the URL path (String)
- consumes - content type for request, support: application/json, application/xml, application/x-www-form-urlencoded (String)
- produces - accept type for response, support: application/json, application/xml, application/x-www-form-urlencoded (String)

PATCH method supports argument macros:
- #[segment] - maps method arguments to path segments and header placeholders (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])

*/
#[proc_macro_attribute]
pub fn patch(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: научить patch работать с независимыми функциями и со структурными методами (блок impl)
    parse_method(Method::PATCH, item, attrs)
}

/**
A procedural macro for adding HTTP headers to a request. It includes the following attributes:
- name - HTTP header name (String)
- value - HTTP header value (String)
- sensitive - sensitive HTTP header value (true/false)

It also supports filling #[segment] into header values.
*/
#[proc_macro_attribute]
pub fn header(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_header(item, attrs)
}

/**
A procedural macro for generating DTO objects.
*/
#[proc_macro_attribute]
pub fn data_transfer(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let vis = item.vis.clone();
    let ident = item.ident.clone();
    let fields = item.fields.clone();
    let attrs = item.attrs.clone();

    TokenStream::from(quote! {
        #(#attrs)*
        #[derive(clientix::prelude::serde::Serialize, clientix::prelude::serde::Deserialize, Clone, Debug)]
        #[serde(crate = "clientix::prelude::serde")]
        #vis struct #ident #fields
    })
}

#[proc_macro_attribute]
pub fn request_args(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let vis = item.vis.clone();
    let ident = item.ident.clone();
    let fields = item.fields.clone();
    let attrs = item.attrs.clone();

    TokenStream::from(quote! {
        #(#attrs)*
        #[derive(clientix::RequestArgs, Clone, Debug)]
        #vis struct #ident #fields
    })
}

#[proc_macro_derive(RequestArgs, attributes(segment, query, header, body))]
pub fn request_args_derive(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);
    let ident = &derive_input.ident;

    let mut segments_stream = quote!(let mut arguments = std::collections::HashMap::new(););
    let mut queries_stream = quote!(let mut arguments = std::collections::HashMap::new(););
    let mut headers_stream = quote!(let mut arguments = std::collections::HashMap::new(););
    let mut body_stream = quote!();
    match derive_input.data {
        syn::Data::Struct(data) => {
            for field in &data.fields {
                for attr in &field.attrs {
                    match attr.path() {
                        ref path if path.is_ident("segment") => {
                            let segment_variable = field.ident.clone().unwrap();
                            let segment_id = format!("{}", quote! {#segment_variable});

                            segments_stream.extend(quote!(arguments.insert(#segment_id.to_string(), self.#segment_variable.to_string());));
                        },
                        ref path if path.is_ident("query") => {
                            let query_variable = field.ident.clone().unwrap();
                            let query_id = format!("{}", quote! {#query_variable});

                            segments_stream.extend(quote!(arguments.insert(#query_id.to_string(), self.#query_variable.to_string());));
                        }
                        ref path if path.is_ident("header") => {
                            let header_variable = field.ident.clone().unwrap();
                            let header_id = format!("{}", quote! {#header_variable});

                            segments_stream.extend(quote!(arguments.insert(#header_id.to_string(), self.#header_variable.to_string());));
                        }
                        ref path if path.is_ident("body") => {
                            let body_variable = field.ident.clone().unwrap();
                            body_stream.extend(quote!(self.#body_variable.to_string()));
                        }
                        _ => throw_error("unsupported attribute", false),
                    }
                }
            }
        },
        _ => {}
    }
    segments_stream.extend(quote!(arguments));
    queries_stream.extend(quote!(arguments));
    headers_stream.extend(quote!(arguments));

    TokenStream::from(quote! {
        impl #ident {

            pub fn segments(&self) -> std::collections::HashMap<String, String> {
                #segments_stream
            }

            pub fn queries(&self) -> std::collections::HashMap<String, String> {
                #queries_stream
            }

            pub fn headers(&self) -> std::collections::HashMap<String, String> {
                #headers_stream
            }

        }
    })
}