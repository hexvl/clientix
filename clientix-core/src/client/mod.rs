pub mod asynchronous;
pub mod blocking;
pub mod result;

use std::collections::HashMap;
use std::time::Duration;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use http::header::AUTHORIZATION;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::client::asynchronous::client::AsyncClient;
use crate::client::blocking::client::BlockingClient;

pub struct Clientix {
    config: ClientConfig
}

pub struct ClientixBuilder {
    config: ClientConfig
}

#[derive(Clone)]
pub struct ClientConfig {
    url: Option<String>,
    path: Option<String>,
    user_agent: Option<String>,
    headers: HeaderMap,
    timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    connection_verbose: bool
}

#[derive(Clone)]
struct RequestPath {
    path_str: String,
    segments: HashMap<String, String>
}

impl Clientix {

    pub fn builder() -> ClientixBuilder {
        ClientixBuilder::new()
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    pub fn set_url(&mut self, url: &str) {
        self.config.url = Some(url.to_string());
    }

    pub fn set_path(&mut self, path: &str) {
        self.config.path = Some(path.to_string());
    }

    pub fn set_user_agent(&mut self, user_agent: &str) {
        self.config.user_agent = Some(user_agent.to_string());
    }

    pub fn set_headers(&mut self, headers: HeaderMap) {
        self.config.headers = headers;
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.config.timeout = Some(timeout);
    }

    pub fn set_read_timeout(&mut self, read_timeout: Duration) {
        self.config.read_timeout = Some(read_timeout);
    }

    pub fn set_connect_timeout(&mut self, connect_timeout: Duration) {
        self.config.connect_timeout = Some(connect_timeout);
    }

    pub fn set_connection_verbose(&mut self, connection_verbose: bool) {
        self.config.connection_verbose = connection_verbose;
    }

    pub fn blocking(&self) -> BlockingClient {
        BlockingClient::from(self.config.clone())
    }

    pub fn asynchronous(&self) -> AsyncClient {
        AsyncClient::from(self.config.clone())
    }

}

impl ClientixBuilder {

    fn new() -> ClientixBuilder {
        ClientixBuilder {
            config: ClientConfig {
                url: None,
                path: None,
                user_agent: None,
                headers: Default::default(),
                timeout: None,
                read_timeout: None,
                connect_timeout: None,
                connection_verbose: false,
            },
        }
    }

    pub fn url(mut self, url: &str) -> ClientixBuilder {
        self.config.url = Some(url.to_string());

        self
    }

    pub fn path(mut self, path: &str) -> ClientixBuilder {
        self.config.path = Some(path.to_string());

        self
    }

    pub fn user_agent(mut self, user_agent: &str) -> ClientixBuilder {
        self.config.user_agent = Some(user_agent.to_string());
        self
    }

    pub fn header(mut self, key: &str, value: &str, sensitive: bool) -> ClientixBuilder {
        let header_name = if let Ok(name) = HeaderName::from_bytes(key.as_bytes()) {
            name
        } else {
            return self
        };

        let mut header_value = if let Ok(value) = HeaderValue::from_str(&value) {
            value
        } else {
            return self
        };

        header_value.set_sensitive(sensitive);

        self.config.headers.insert(header_name, header_value);

        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> ClientixBuilder {
        for (key, value) in headers {
            let header_name = if let Ok(name) = HeaderName::from_bytes(key.as_bytes()) {
                name
            } else {
                continue
            };

            let header_value = if let Ok(value) = HeaderValue::from_str(&value) {
                value
            } else {
                continue
            };

            self.config.headers.insert(header_name, header_value);
        }

        self
    }

    pub fn basic_auth(self, username: &str, password: &str) -> ClientixBuilder {
        let basic_token = format!("Basic {}", BASE64_STANDARD.encode(format!("{username}:{password}")));
        self.header(AUTHORIZATION.as_str(), basic_token.as_str(), true)
    }

    pub fn bearer_auth(self, token: &str) -> ClientixBuilder {
        self.header(AUTHORIZATION.as_str(), format!("Bearer {}", token).as_str(), true)
    }

    pub fn timeout(mut self, timeout: Duration) -> ClientixBuilder {
        self.config.timeout = Some(timeout);
        self
    }

    pub fn read_timeout(mut self, read_timeout: Duration) -> ClientixBuilder {
        self.config.read_timeout = Some(read_timeout);
        self
    }

    pub fn connect_timeout(mut self, connect_timeout: Duration) -> ClientixBuilder {
        self.config.connect_timeout = Some(connect_timeout);
        self
    }

    pub fn connection_verbose(mut self, connection_verbose: bool) -> ClientixBuilder {
        self.config.connection_verbose = connection_verbose;
        self
    }

    pub fn blocking(&self) -> BlockingClient {
        BlockingClient::from(self.config.clone())
    }

    pub fn asynchronous(&self) -> AsyncClient {
        AsyncClient::from(self.config.clone())
    }

    pub fn build(self) -> Clientix {
        Clientix { config: self.config }
    }

}