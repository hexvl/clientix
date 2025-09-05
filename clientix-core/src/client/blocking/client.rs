use std::sync::{Arc, Mutex};
use http::Method;
use reqwest::blocking::Client as ReqwestClient;
use crate::client::blocking::request::BlockingRequestBuilder;
use crate::client::ClientConfig;

#[derive(Clone)]
pub struct BlockingClient {
    pub client: Arc<Mutex<ReqwestClient>>,
    pub url: String,
    pub path: String
}

impl BlockingClient {

    pub fn get(&self) -> BlockingRequestBuilder {
        BlockingRequestBuilder::builder(self.clone(), Method::GET)
    }

    pub fn post(&self) -> BlockingRequestBuilder {
        BlockingRequestBuilder::builder(self.clone(), Method::POST)
    }

    pub fn put(&self) -> BlockingRequestBuilder {
        BlockingRequestBuilder::builder(self.clone(), Method::PUT)
    }

    pub fn delete(&self) -> BlockingRequestBuilder {
        BlockingRequestBuilder::builder(self.clone(), Method::DELETE)
    }

    pub fn head(&self) -> BlockingRequestBuilder {
        BlockingRequestBuilder::builder(self.clone(), Method::HEAD)
    }
    
    pub fn patch(&self) -> BlockingRequestBuilder {
        BlockingRequestBuilder::builder(self.clone(), Method::PATCH)
    }
    
}

impl From<ClientConfig> for BlockingClient {

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
            client = client.connect_timeout(connect_timeout);
        }

        client = client.connection_verbose(config.connection_verbose);

        let url = config.url.expect("missing url");
        let path = config.path.unwrap_or(String::new());
        let client = Arc::new(Mutex::new(client.build().expect("failed to build blocking client")));

        BlockingClient { client, url, path }
    }

}