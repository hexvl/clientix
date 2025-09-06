use std::collections::HashMap;
use std::time::Duration;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use http::{HeaderMap, HeaderName, HeaderValue};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use crate::client::response::{ClientixError, ClientixErrorData, ClientixResult};
use crate::core::headers::content_type::ContentType;

#[derive(Clone, Debug, Default)]
pub struct RequestConfig {
    path: String,
    headers: HeaderMap,
    queries: Vec<(String, String)>,
    body: Option<String>,
    timeout: Option<Duration>
}

pub trait ClientixRequestBuilder {
    
    fn config(&mut self) -> &mut RequestConfig;
    
    fn result(&mut self) -> &mut ClientixResult<()>;
    
    fn path(mut self, path: &str) -> Self where Self: Sized {
        self.config().set_path(path);
        self
    }

    fn query(mut self, key: &str, value: &str) -> Self where Self: Sized {
        self.config().add_query(key, value);
        self
    }

    fn queries(mut self, queries: HashMap<String, String>) -> Self where Self: Sized {
        for (key, value) in queries {
            self = self.query(key.as_str(), value.as_str());
        }

        self
    }

    fn header(mut self, key: &str, value: &str) -> Self where Self: Sized {
        self.config().set_header(key, value, false);
        self
    }

    fn headers(mut self, headers: HashMap<String, String>) -> Self where Self: Sized {
        for (key, value) in headers {
            self = self.header(key.as_str(), value.as_str());
        }

        self
    }

    fn basic_auth(mut self, username: &str, password: &str) -> Self where Self: Sized {
        self.config().basic_auth(username, password);
        self
    }

    fn bearer_auth(mut self, token: &str) -> Self where Self: Sized {
        self.config().bearer_auth(token);
        self
    }

    fn body<T: Serialize>(mut self, body: T, content_type: ContentType) -> Self where Self: Sized {
        *self.result() = self.config().set_body(body, content_type);
        self
    }
    
}

impl RequestConfig {
    
    pub fn new() -> Self {
        RequestConfig {
            path: Default::default(),
            headers: Default::default(),
            queries: Default::default(),
            body: None,
            timeout: None,
        }
    }
    
    pub fn get_path(&self) -> &String { 
        &self.path 
    }
    
    pub fn set_path(&mut self, path: &str) {
        self.path = path.to_string();
    }
    
    pub fn get_queries(&self) -> &Vec<(String, String)> {
        &self.queries
    }
    
    pub fn add_query(&mut self, key: &str, value: &str) {
        self.queries.push((key.to_string(), value.to_string()));
    }

    pub fn add_queries(&mut self, queries: HashMap<String, String>) {
        for (key, value) in queries {
            self.queries.push((key, value));
        }
    }

    pub fn set_queries(&mut self, queries: HashMap<String, String>) {
        self.queries.clear();

        for (key, value) in queries {
            self.queries.push((key, value));
        }
    }
    
    pub fn get_headers(&self) -> &HeaderMap {
        &self.headers
    }
    
    pub fn set_header(&mut self, key: &str, value: &str, sensitive: bool) {
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

        self.headers.insert(header_name, header_value);
    }

    pub fn set_headers(&mut self, headers: HashMap<String, String>) {
        for (key, value) in headers {
            self.set_header(key.as_str(), value.as_str(), false);
        }
    }

    pub fn basic_auth(&mut self, username: &str, password: &str) {
        let basic_token = format!("Basic {}", BASE64_STANDARD.encode(format!("{username}:{password}")));
        self.set_header(AUTHORIZATION.as_str(), basic_token.as_str(), true);
    }

    pub fn bearer_auth(&mut self, token: &str) {
        self.set_header(AUTHORIZATION.as_str(), format!("Bearer {}", token).as_str(), true);
    }

    pub fn get_body(&self) -> &Option<String> {
        &self.body
    }
    
    pub fn set_body<T: Serialize>(&mut self, body: T, content_type: ContentType) -> ClientixResult<()> {
        match content_type {
            ContentType::ApplicationJson => self.set_json_body(body),
            ContentType::ApplicationXWwwFormUrlEncoded => self.set_form_body(body),
            ContentType::ApplicationXml => self.set_xml_body(body),
            _ => Err(ClientixError::InvalidRequest(
                ClientixErrorData::builder()
                    .message(format!("invalid content type: {:?}", content_type).as_str())
                    .build(), 
                None
            ))
        }
    }

    fn set_json_body<T: Serialize>(&mut self, body: T) -> ClientixResult<()> {
        match serde_json::to_string(&body) {
            Ok(body) => {
                self.body = Some(body);
                self.headers.insert(CONTENT_TYPE, ContentType::ApplicationJson.try_into().unwrap());
                Ok(())
            },
            Err(err) => Err(ClientixError::InvalidRequest(Default::default(), Some(err.into())))
        }
    }

    fn set_xml_body<T: Serialize>(&mut self, body: T) -> ClientixResult<()> {
        match serde_xml_rs::to_string(&body) {
            Ok(body) => {
                self.body = Some(body);
                self.headers.insert(CONTENT_TYPE, ContentType::ApplicationXml.try_into().unwrap());
                Ok(())
            },
            Err(err) => Err(ClientixError::InvalidRequest(Default::default(), Some(err.into())))
        }
    }

    fn set_form_body<T: Serialize>(&mut self, body: T) -> ClientixResult<()> {
        match serde_urlencoded::to_string(&body) {
            Ok(body) => {
                self.body = Some(body);
                self.headers.insert(CONTENT_TYPE, ContentType::ApplicationXWwwFormUrlEncoded.try_into().unwrap());
                Ok(())
            },
            Err(err) => Err(ClientixError::InvalidRequest(Default::default(), Some(err.into())))
        }
    }
    
    pub fn get_timeout(&self) -> Option<Duration> {
        self.timeout
    }
    
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = Some(timeout);
    }
    
}