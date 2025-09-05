use std::net::SocketAddr;
use reqwest::{StatusCode, Url, Version};
use reqwest::header::HeaderMap;
use thiserror::Error;

pub type ClientixResult<T> = Result<T, ClientixError>;

#[derive(Debug, Default)]
pub struct ClientixErrorData {
    message: Option<String>
}

pub struct ClientixErrorBuilder {
    message: Option<String>
}

#[derive(Error, Debug)]
pub enum ClientixError {
    #[error("Network error")]
    Http(ClientixErrorData, #[source] Option<Box<dyn std::error::Error + Send + Sync>>),

    #[error("IO error")]
    IO(ClientixErrorData, #[source] Option<Box<dyn std::error::Error + Send + Sync>>),

    #[error("Invalid request")]
    InvalidRequest(ClientixErrorData, #[source] Option<Box<dyn std::error::Error + Send + Sync>>),

    #[error("invalid response")]
    InvalidResponse(ClientixErrorData, #[source] Option<Box<dyn std::error::Error + Send + Sync>>),

    #[error("Other error")]
    Other(ClientixErrorData, #[source] Option<Box<dyn std::error::Error + Send + Sync>>),
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

impl From<reqwest::Error> for ClientixError {
    fn from(err: reqwest::Error) -> ClientixError {
        ClientixError::Http(ClientixErrorData::new(), Some(Box::new(err)))
    }
}

impl From<serde_json::Error> for ClientixError {
    fn from(err: serde_json::Error) -> ClientixError {
        ClientixError::IO(ClientixErrorData::new(), Some(Box::new(err)))
    }
}

impl From<serde_xml_rs::Error> for ClientixError {
    fn from(err: serde_xml_rs::Error) -> ClientixError {
        ClientixError::IO(ClientixErrorData::new(), Some(Box::new(err)))
    }
}

impl From<serde_urlencoded::de::Error> for ClientixError {
    fn from(err: serde_urlencoded::de::Error) -> ClientixError {
        ClientixError::IO(ClientixErrorData::new(), Some(Box::new(err)))
    }
}

impl ClientixErrorData {
    pub fn new() -> ClientixErrorData {
        ClientixErrorData {
            message: None
        }
    }

    pub fn message(&self) -> &Option<String> {
        &self.message
    }

    pub fn builder() -> ClientixErrorBuilder {
        ClientixErrorBuilder::new()
    }

}

impl ClientixErrorBuilder {

    fn new() -> Self {
        ClientixErrorBuilder {
            message: None
        }
    }

    pub fn message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn build(self) -> ClientixErrorData {
        ClientixErrorData {
            message: self.message
        }
    }

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