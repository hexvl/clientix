mod method;
mod client;
mod return_kind;
mod header;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
use clientix_core::prelude::reqwest::Method;
use crate::client::parse_client;
use crate::header::parse_header;
use crate::method::parse_method;

/**
A procedural macro for building an HTTP client. It includes the following attributes:
- url - the base part of the client’s URL, e.g. http://localhost:8080
- path - an additional part of the URL path that precedes method paths
- async - if true, the client is asynchronous; otherwise, it is blocking

Example:
```
#[clientix(url = "http://localhost:8080")]
trait ExampleClient {

    #[get(path = "/", consumes = "application/json", produces = "application/json")]
    fn get(&self) -> ClientixResult<ClientixResponse<String>>;

}
```

The client also supports configuring parameters imperatively. Example:
```
let client = ExampleClient::config()
    .url("http://localhost:8080")
    .path("/test")
    .setup();
```
*/
#[proc_macro_attribute]
pub fn clientix(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_client(item, attrs)
}

/**
A procedural macro for building an HTTP GET method of trait. It includes the following attributes:
- path - a part of the URL path
- consumes - content-type for request
- produces - content-type for response

GET method supports argument macros:
- #[segment] - maps method arguments to path segments
- #[query] - maps method arguments to query parameters
- #[header] - maps method arguments to request headers
- #[body] - maps method arguments to request body

Example:
```
#[get(path = "/{path_query}", consumes = "application/json", produces = "application/json")]
fn get(&self, #[segment] path_query: &str, #[query] query_param: &str, #[header] authorization: &str) -> ClientixResult<ClientixResponse<String>>;
```
*/
#[proc_macro_attribute]
pub fn get(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::GET, item, attrs)
}

/**
A procedural macro for building an HTTP POST method of trait. It includes the following attributes:
- path - a part of the URL path
- consumes - content-type for request
- produces - content-type for response

POST method supports argument macros:
- #[segment] - maps method arguments to path segments
- #[query] - maps method arguments to query parameters
- #[header] - maps method arguments to request headers
- #[body] - maps method arguments to request body

Example:
```
#[post(path = "/{path_query}", consumes = "application/json", produces = "application/json")]
fn post(&self, #[segment] path_query: &str, #[query] query_param: &str, #[header] authorization: &str, #[body] request: RequestBody) -> ClientixResult<ClientixResponse<String>>;
```

RequestBody must implement the #[data_transfer] macro.
*/
#[proc_macro_attribute]
pub fn post(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::POST, item, attrs)
}

/**
A procedural macro for building an HTTP PUT method of trait. It includes the following attributes:
- path - a part of the URL path
- consumes - content-type for request
- produces - content-type for response

PUT method supports argument macros:
- #[segment] - maps method arguments to path segments
- #[query] - maps method arguments to query parameters
- #[header] - maps method arguments to request headers
- #[body] - maps method arguments to request body

Example:
```
#[put(path = "/{path_query}", consumes = "application/json", produces = "application/json")]
fn put(&self, #[segment] path_query: &str, #[query] query_param: &str, #[header] authorization: &str, #[body] request: RequestBody) -> ClientixResult<ClientixResponse<String>>;
```

RequestBody must implement the #[data_transfer] macro.
*/
#[proc_macro_attribute]
pub fn put(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::PUT, item, attrs)
}

/**
A procedural macro for building an HTTP DELETE method of trait. It includes the following attributes:
- path - a part of the URL path
- consumes - content-type for request
- produces - content-type for response

DELETE method supports argument macros:
- #[segment] - maps method arguments to path segments
- #[query] - maps method arguments to query parameters
- #[header] - maps method arguments to request headers
- #[body] - maps method arguments to request body

Example:
```
#[delete(path = "/{path_query}", consumes = "application/json", produces = "application/json")]
fn delete(&self, #[segment] path_query: &str, #[query] query_param: &str, #[header] authorization: &str) -> ClientixResult<ClientixResponse<String>>;
```
*/
#[proc_macro_attribute]
pub fn delete(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::DELETE, item, attrs)
}

/**
A procedural macro for building an HTTP HEAD method of trait. It includes the following attributes:
- path - a part of the URL path
- consumes - content-type for request
- produces - content-type for response

HEAD method supports argument macros:
- #[segment] - maps method arguments to path segments
- #[query] - maps method arguments to query parameters
- #[header] - maps method arguments to request headers
- #[body] - maps method arguments to request body

Example:
```
#[head(path = "/{path_query}", consumes = "application/json", produces = "application/json")]
fn head(&self, #[segment] path_query: &str, #[query] query_param: &str) -> ClientixResult<ClientixResponse<String>>;
```
*/
#[proc_macro_attribute]
pub fn head(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::HEAD, item, attrs)
}

/**
A procedural macro for building an HTTP PATCH method of trait. It includes the following attributes:
- path - a part of the URL path
- consumes - content-type for request
- produces - content-type for response

PATCH method supports argument macros:
- #[segment] - maps method arguments to path segments
- #[query] - maps method arguments to query parameters
- #[header] - maps method arguments to request headers
- #[body] - maps method arguments to request body

Example:
```
#[patch(path = "/{path_query}", consumes = "application/json", produces = "application/json")]
fn patch(&self, #[segment] path_query: &str, #[query] query_param: &str) -> ClientixResult<ClientixResponse<String>>;
```
*/
#[proc_macro_attribute]
pub fn patch(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_method(Method::PATCH, item, attrs)
}

#[proc_macro_attribute]
pub fn header(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse_header(item, attrs)
}

/**
A procedural macro for generating DTO objects.

Example:
```
#[data_transfer]
pub struct CreateObjectRequest {
    pub name: String,
    pub data: HashMap<String, String>,
}

#[data_transfer]
pub struct CreatedObjectResponse {
    pub id: String,
    pub name: String,
    pub data: HashMap<String, String>
}
```
*/
#[proc_macro_attribute]
pub fn data_transfer(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    // TODO: научить data_transfer использовать все возможности serde на максимум
    TokenStream::from(quote! {
        #[derive(clientix::prelude::serde::Serialize, clientix::prelude::serde::Deserialize, Debug, Clone)]
        #[serde(crate = "clientix::prelude::serde")]
        #item
    })
}

// TODO: implemented HTTP-methods based independent functions
// TODO: implemented building client based struct params