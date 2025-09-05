use std::collections::HashMap;
use std::time::Duration;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::blocking::Body;
use serde::Serialize;
use strfmt::strfmt;
use crate::client::blocking::client::BlockingClient;
use crate::client::blocking::response::BlockingResponseHandler;
use crate::client::RequestPath;
use crate::client::result::{ClientixError, ClientixErrorData, ClientixResult};
use crate::core::headers::content_type::ContentType;

struct BlockingRequestConfig {
    path: RequestPath,
    headers: HeaderMap,
    queries: Vec<(String, String)>,
    body: Option<Body>,
    timeout: Option<Duration>
}

pub struct BlockingRequestBuilder {
    client: BlockingClient,
    method: Method,
    config: BlockingRequestConfig,
    result: ClientixResult<()>,
}

impl BlockingRequestBuilder {

    pub fn builder(client: BlockingClient, method: Method) -> BlockingRequestBuilder {
        BlockingRequestBuilder {
            client,
            method,
            config: BlockingRequestConfig {
                path: RequestPath {
                    path_str: Default::default(),
                    segments: Default::default(),
                },
                headers: Default::default(),
                queries: Default::default(),
                body: Default::default(),
                timeout: None
            },
            result: Ok(())
        }
    }

    pub fn path(mut self, path: &str) -> Self {
        self.config.path.path_str = path.to_string();

        self
    }

    pub fn path_segment(mut self, id: &str, value: &str) -> Self {
        self.config.path.segments.insert(id.to_string(), value.to_string());

        self
    }

    pub fn query(mut self, key: &str, value: &str) -> Self {
        self.config.queries.push((key.to_string(), value.to_string()));

        self
    }

    pub fn queries(mut self, queries: HashMap<String, String>) -> Self {
        for (key, value) in queries {
            self.config.queries.push((key, value));
        }

        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.insert_header(key, value, false);

        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        for (key, value) in headers {
            self.insert_header(key.as_str(), value.as_str(), false);
        }

        self
    }

    pub fn basic_auth(mut self, username: &str, password: &str) -> Self {
        let basic_token = format!("Basic {}", BASE64_STANDARD.encode(format!("{username}:{password}")));
        self.insert_header(AUTHORIZATION.as_str(), basic_token.as_str(), true);

        self
    }

    pub fn bearer_auth(mut self, token: &str) -> Self {
        self.insert_header(AUTHORIZATION.as_str(), format!("Bearer {}", token).as_str(), true);

        self
    }

    pub fn body<T: Serialize>(mut self, body: T, content_type: ContentType) -> Self {
        match content_type {
            ContentType::ApplicationJson => {
                match serde_json::to_string(&body) {
                    Ok(body) => self.config.body = Some(body.into()),
                    Err(err) => self.result = Err(ClientixError::IO(ClientixErrorData::new(), Some(err.into())))
                }
            }
            ContentType::ApplicationXWwwFormUrlEncoded => {
                match serde_urlencoded::to_string(&body) {
                    Ok(body) => self.config.body = Some(body.into()),
                    Err(err) => self.result = Err(ClientixError::IO(ClientixErrorData::new(), Some(err.into())))
                }
            },
            _ => {
                let error_data = ClientixErrorData::builder().message(format!("invalid content type: {:?}", content_type).as_str()).build();
                self.result = Err(ClientixError::InvalidRequest(error_data, None));
            }
        };
        
        match content_type.try_into() {
            Ok(content_type) => {
                self.config.headers.insert(CONTENT_TYPE, content_type);
            },
            Err(err) => {
                let error_data = ClientixErrorData::builder().message(format!("invalid content type: {:?}. {:?}", content_type, err).as_str()).build();
                self.result = Err(ClientixError::InvalidRequest(error_data, None));
            }
        }

        self
    }

    pub fn send(self) -> BlockingResponseHandler {
        if let Err(error) = self.result {
            return BlockingResponseHandler::new(Err(error));
        }

        let method_path = strfmt(&self.config.path.path_str, &self.config.path.segments).expect("failed to format path");
        let full_path = format!("{}{}", self.client.path, method_path);
        let url = format!("{}{}", self.client.url, full_path);

        match self.client.client.lock() {
            Ok(client) => {
                let mut request_builder = match self.method {
                    Method::GET => client.get(url.clone()),
                    Method::POST => client.post(url.clone()),
                    Method::PUT => client.put(url.clone()),
                    Method::DELETE => client.delete(url.clone()),
                    Method::HEAD => client.head(url.clone()),
                    Method::PATCH => client.patch(url.clone()),
                    _ => {
                        let error_data = ClientixErrorData::builder().message(format!("invalid method: {:?}", self.method).as_str()).build();
                        return BlockingResponseHandler::new(Err(ClientixError::InvalidRequest(error_data, None)));
                    },
                };

                request_builder = request_builder
                    .headers(self.config.headers)
                    .query(&self.config.queries);

                request_builder = match self.config.body {
                    Some(body) => request_builder.body(body),
                    None => request_builder,
                };

                request_builder = match self.config.timeout {
                    Some(timeout) => request_builder.timeout(timeout),
                    None => request_builder,
                };

                match request_builder.send() {
                    Ok(response) => BlockingResponseHandler::new(Ok(response)),
                    Err(error) => BlockingResponseHandler::new(Err(ClientixError::Http(ClientixErrorData::new(), Some(error.into()))))
                }
            },
            Err(error) => {
                let error_data = ClientixErrorData::builder().message(format!("client locked: {:?}", error).as_str()).build();
                BlockingResponseHandler::new(Err(ClientixError::Other(error_data, None)))
            }
        }
    }

    fn insert_header(&mut self, key: &str, value: &str, sensitive: bool) {
        let header_name = if let Ok(name) = HeaderName::from_bytes(key.as_bytes()) {
            name
        } else {
            return;
        };

        let mut header_value = if let Ok(value) = HeaderValue::from_str(&value) {
            value
        } else {
            return;
        };

        header_value.set_sensitive(sensitive);

        self.config.headers.insert(header_name, header_value);
    }

}