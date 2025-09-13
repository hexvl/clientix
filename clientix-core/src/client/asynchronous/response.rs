use bytes::Bytes;
use futures_util::TryStreamExt;
use reqwest::Response;
use serde::de::DeserializeOwned;
use crate::client::asynchronous::stream::ClientixStream;
use crate::client::asynchronous::stream::sse::ClientixSSEStream;
use crate::client::response::{ClientixError, ClientixResponse, ClientixResult};

pub struct AsyncResponseHandler {
    result: ClientixResult<Response>
}

impl AsyncResponseHandler {

    pub fn new(result: ClientixResult<Response>) -> AsyncResponseHandler {
        AsyncResponseHandler { result }
    }

    pub async fn bytes(self) -> ClientixResult<ClientixResponse<Bytes>> {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    response.bytes().await?
                ))
            },
            Err(error) => Err(error),
        }
    }

    pub fn bytes_stream(self) -> ClientixResult<ClientixStream> {
        match self.result {
            Ok(response) => {
                Ok(ClientixStream::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    response.bytes_stream().map_err(ClientixError::from)
                ))
            },
            Err(error) => Err(error)
        }
    }

    pub async fn text(self) -> ClientixResult<ClientixResponse<String>> {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    response.text().await?
                ))
            },
            Err(error) => Err(error),
        }
    }

    pub async fn text_with_encoding(self, encoding: &str) -> ClientixResult<ClientixResponse<String>> {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    response.text_with_charset(encoding).await?
                ))
            },
            Err(error) => Err(error),
        }
    }

    pub fn text_stream(self) -> ClientixResult<ClientixSSEStream<String>> {
        Ok(self.bytes_stream()?.sse())
    }

    pub async fn json<T>(self) -> ClientixResult<ClientixResponse<T>> where T: DeserializeOwned + Clone {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    serde_json::from_str::<T>(response.text().await?.as_str())?
                ))
            },
            Err(error) => Err(error),
        }
    }

    pub fn json_stream<T>(self) -> ClientixResult<ClientixSSEStream<T>> where T: DeserializeOwned + Clone {
        Ok(self.text_stream()?.json_stream())
    }

    pub async fn xml<T>(self) -> ClientixResult<ClientixResponse<T>> where T: DeserializeOwned + Clone {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    serde_xml_rs::from_str::<T>(response.text().await?.as_str())?
                ))
            },
            Err(error) => Err(error),
        }
    }
    
    pub async fn xml_stream<T>(self) -> ClientixResult<ClientixSSEStream<T>> where T: DeserializeOwned + Clone {
        Ok(self.text_stream()?.xml_stream())
    }
    
    pub async fn urlencoded<T>(self) -> ClientixResult<ClientixResponse<T>> where T: DeserializeOwned + Clone {
        match self.result {
            Ok(response) => {
                Ok(ClientixResponse::new(
                    response.version(),
                    response.content_length(),
                    response.status(),
                    response.url().clone(),
                    response.remote_addr(),
                    response.headers().clone(),
                    serde_urlencoded::from_str::<T>(response.text().await?.as_str())?
                ))
            },
            Err(error) => Err(error),
        }
    }
    
}