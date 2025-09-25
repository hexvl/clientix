mod client;
mod method;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
use clientix_core::prelude::reqwest::Method;
use crate::client::parse_client;
use crate::method::parse_header;
use crate::method::parse_method;

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
- #[segment] - maps method arguments to path segments (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])
- #[placeholder] - maps method arguments to request header placeholders

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
- #[segment] - maps method arguments to path segments (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])
- #[placeholder] - maps method arguments to request header placeholders

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
- #[segment] - maps method arguments to path segments (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])
- #[placeholder] - maps method arguments to request header placeholders

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
- #[segment] - maps method arguments to path segments (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])
- #[placeholder] - maps method arguments to request header placeholders

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
- #[segment] - maps method arguments to path segments (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])
- #[placeholder] - maps method arguments to request header placeholders

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
- #[segment] - maps method arguments to path segments (simple types, String)
- #[query] - maps method arguments to query parameters (simple types, String)
- #[header] - maps method arguments to request headers (simple types, String)
- #[body] - maps method arguments to request body (object implemented #[data_transfer])
- #[placeholder] - maps method arguments to request header placeholders

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

It also supports filling #[placeholder] into header values.

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

    // TODO: научить data_transfer использовать все возможности serde на максимум, сделать так,
    //  чтобы подобный подход не вредил блоку derive и другим макросам и дал возможность пользователю самому решать о содержимом derive также
    TokenStream::from(quote! {
        #[derive(clientix::prelude::serde::Serialize, clientix::prelude::serde::Deserialize, Debug, Clone)]
        #[serde(crate = "clientix::prelude::serde")]
        #vis struct #ident #fields
    })
}

#[proc_macro_attribute]
pub fn request_args(_attrs: TokenStream, _item: TokenStream) -> TokenStream {
    // TODO: реализовать соответствующую логику для парсинга структуры аргументов запроса,
    //  включая #[segment], #[placeholder], #[query], #[header], #[body]. Необходимо для поддержки
    //  аргументов запроса с макросом #[args]. Необходима имплементация геттеров для получения
    //  соответствующих значений полей
    TokenStream::from(quote! {})
}

#[proc_macro_derive(RequestArgs, attributes(segment, placeholder, query, header, body))]
pub fn request_args_derive(_item: TokenStream) -> TokenStream {
    TokenStream::from(quote! {})
}