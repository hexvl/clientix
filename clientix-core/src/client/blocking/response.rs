use bytes::Bytes;
use reqwest::blocking::Response;
use serde::de::DeserializeOwned;
use crate::client::response::{ClientixResponse, ClientixResult};

pub struct BlockingResponseHandler {
    result: ClientixResult<Response>
}

impl BlockingResponseHandler {

    pub fn new(result: ClientixResult<Response>) -> BlockingResponseHandler {
        BlockingResponseHandler { result }
    }

    pub fn text(self) -> ClientixResult<ClientixResponse<String>> {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    response.text()?
                ))
            },
            Err(error) => Err(error),
        }
    }

    pub fn text_with_encoding(self, encoding: &str) -> ClientixResult<ClientixResponse<String>> {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    response.text_with_charset(encoding)?
                ))
            },
            Err(error) => Err(error),
        }
    }

    pub fn bytes(self) -> ClientixResult<ClientixResponse<Bytes>> {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    response.bytes()?
                ))
            },
            Err(error) => Err(error),
        }
    }

    pub fn json<T>(self) -> ClientixResult<ClientixResponse<T>> where T: DeserializeOwned + Clone {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    serde_json::from_str::<T>(response.text()?.as_str())?
                ))
            },
            Err(error) => Err(error),
        }
    }

}