use std::collections::HashMap;
use std::time::Duration;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Body;
use serde::Serialize;
use strfmt::strfmt;
use crate::client::asynchronous::client::AsyncClient;
use crate::client::asynchronous::response::AsyncResponseHandler;
use crate::client::RequestPath;
use crate::client::result::{ClientixError, ClientixErrorData, ClientixResult};
use crate::core::headers::content_type::ContentType;

pub struct AsyncRequestConfig {
    path: RequestPath,
    headers: HeaderMap,
    queries: Vec<(String, String)>,
    body: Option<Body>,
    timeout: Option<Duration>
}

pub struct AsyncRequestBuilder {
    client: AsyncClient,
    method: Method,
    config: AsyncRequestConfig,
    result: ClientixResult<()>
}

impl AsyncRequestBuilder {

    pub fn builder(client: AsyncClient, method: Method) -> AsyncRequestBuilder {
        AsyncRequestBuilder {
            client,
            method,
            config: AsyncRequestConfig {
                path: RequestPath {
                    path_str: Default::default(),
                    segments: Default::default()
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
            ContentType::ApplicationJson => self.body_json(body),
            ContentType::ApplicationXWwwFormUrlEncoded => self.body_form(body),
            ContentType::ApplicationXml => self.body_xml(body),
            _ => {
                let error_data = ClientixErrorData::builder().message(format!("invalid content type: {:?}", content_type).as_str()).build();
                self.result = Err(ClientixError::InvalidRequest(error_data, None));
            }
        };

        self
    }

    fn body_json<T: Serialize>(&mut self, body: T) {
        match serde_json::to_string(&body) {
            Ok(body) => self.config.body = Some(body.into()),
            Err(err) => self.result = Err(ClientixError::InvalidRequest(ClientixErrorData::new(), Some(err.into())))
        };

        self.config.headers.insert(CONTENT_TYPE, ContentType::ApplicationJson.try_into().unwrap());
    }

    fn body_xml<T: Serialize>(&mut self, body: T) {
        match serde_xml_rs::to_string(&body) {
            Ok(body) => self.config.body = Some(body.into()),
            Err(err) => self.result = Err(ClientixError::InvalidRequest(ClientixErrorData::new(), Some(err.into())))
        };

        self.config.headers.insert(CONTENT_TYPE, ContentType::ApplicationXml.try_into().unwrap());
    }

    fn body_form<T: Serialize>(&mut self, body: T) {
        match serde_urlencoded::to_string(&body) {
            Ok(body) => self.config.body = Some(body.into()),
            Err(err) => self.result = Err(ClientixError::InvalidRequest(ClientixErrorData::new(), Some(err.into())))
        };

        self.config.headers.insert(CONTENT_TYPE, ContentType::ApplicationXWwwFormUrlEncoded.try_into().unwrap());
    }

    pub async fn send(self) -> AsyncResponseHandler {
        if let Err(error) = self.result {
            return AsyncResponseHandler::new(Err(error));
        }

        let method_path = strfmt(&self.config.path.path_str, &self.config.path.segments).expect("failed to format path");
        let full_path = format!("{}{}", self.client.path, method_path);
        let url = format!("{}{}", self.client.url, full_path);

        match self.client.client.lock() {
            Ok(client) => {
                let mut request_builder = match self.method {
                    Method::GET => client.get(url),
                    Method::POST => client.post(url),
                    Method::PUT => client.put(url),
                    Method::DELETE => client.delete(url),
                    Method::HEAD => client.head(url),
                    Method::PATCH => client.patch(url),
                    _ => {
                        let error_data = ClientixErrorData::builder().message(format!("invalid method: {:?}", self.method).as_str()).build();
                        return AsyncResponseHandler::new(Err(ClientixError::InvalidRequest(error_data, None)));
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

                match request_builder.send().await {
                    Ok(response) => AsyncResponseHandler::new(Ok(response)),
                    Err(error) => AsyncResponseHandler::new(Err(ClientixError::Http(ClientixErrorData::new(), Some(error.into()))))
                }
            },
            Err(err) => {
                let error_data = ClientixErrorData::builder().message(format!("client locked: {:?}", err).as_str()).build();
                AsyncResponseHandler::new(Err(ClientixError::Other(error_data, None)))
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