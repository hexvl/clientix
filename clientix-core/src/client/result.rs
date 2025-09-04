use std::net::SocketAddr;
use reqwest::{StatusCode, Url, Version};
use reqwest::header::HeaderMap;
use thiserror::Error;

pub type ClientixResult<T> = Result<T, ClientixError>;

// TODO: need to correct global error handling
#[derive(Error, Debug)]
pub enum ClientixError {
    #[error("Network error")]
    Network(#[from] reqwest::Error),

    #[error("Invalid request")]
    InvalidRequest(String),

    #[error("invalid response")]
    InvalidResponse(#[from] serde_json::Error),

    #[error("Other error")]
    Other(String),
}

#[derive(Debug)]
pub struct ClientixResponse<T> {
    version: Version,
    content_length: Option<u64>,
    status: StatusCode,
    url: Url,
    remote_addr: Option<SocketAddr>,
    headers: HeaderMap,
    body: T
}

impl <T> ClientixResponse<T> where T: Clone {

    pub fn new(
        version: Version,
        content_length: Option<u64>,
        status: StatusCode,
        url: Url,
        remote_addr: Option<SocketAddr>,
        headers: HeaderMap,
        body: T) -> ClientixResponse<T> {
       ClientixResponse {
           version,
           content_length,
           status,
           url,
           remote_addr,
           headers,
           body
       }
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn body(&self) -> T {
        self.body.clone()
    }

}