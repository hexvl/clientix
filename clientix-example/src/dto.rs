use std::collections::HashMap;
use clientix::{data_transfer, request_args};

#[request_args]
pub struct RequestArgs {
    #[segment]
    pub segment_1: String,
    #[segment]
    pub segment_2: String,
    #[query]
    pub query_1: String,
    #[query]
    pub query_2: String,
    #[body]
    pub body: String,
}

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