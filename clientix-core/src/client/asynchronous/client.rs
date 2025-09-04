use std::sync::{Arc, Mutex};
use http::Method;
use reqwest::Client as ReqwestClient;
use crate::client::asynchronous::request::AsyncRequestBuilder;
use crate::client::ClientConfig;

#[derive(Clone)]
pub struct AsyncClient {
    pub client: Arc<Mutex<ReqwestClient>>,
    pub url: String,
    pub path: String
}

impl AsyncClient {

    pub fn get(&self) -> AsyncRequestBuilder {
        AsyncRequestBuilder::builder(self.clone(), Method::GET)
    }

    pub fn post(&self) -> AsyncRequestBuilder {
        AsyncRequestBuilder::builder(self.clone(), Method::POST)
    }

    pub fn put(&self) -> AsyncRequestBuilder {
        AsyncRequestBuilder::builder(self.clone(), Method::PUT)
    }

    pub fn delete(&self) -> AsyncRequestBuilder {
        AsyncRequestBuilder::builder(self.clone(), Method::DELETE)
    }

}

impl From<ClientConfig> for AsyncClient {

    fn from(config: ClientConfig) -> Self {
        let mut client = ReqwestClient::builder();

        if let Some(user_agent) = config.user_agent {
            client = client.user_agent(user_agent.clone());
        }

        if !config.headers.is_empty() {
            client = client.default_headers(config.headers.clone());
        }

        if let Some(timeout) = config.timeout {
            client = client.timeout(timeout);
        }

        if let Some(connect_timeout) = config.connect_timeout {
            client = client.connect_timeout(connect_timeout);
        }

        client = client.connection_verbose(config.connection_verbose);

        let url = config.url.expect("missing url");
        let path = config.path.unwrap_or(String::new());
        let client = Arc::new(Mutex::new(client.build().expect("failed to build async client")));

        AsyncClient { client, url, path }
    }

}
