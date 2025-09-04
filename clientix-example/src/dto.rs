use std::collections::HashMap;
use clientix::data_transfer;

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