use futures_util::StreamExt;
use bytes::Bytes;
use encoding_rs::UTF_8;
use futures_util::TryStreamExt;
use reqwest::Response;
use serde::de::DeserializeOwned;
use crate::client::asynchronous::stream::ClientixStream;
use crate::client::response::{ClientixError, ClientixResponse, ClientixResult};

pub struct AsyncResponseHandler {
    result: ClientixResult<Response>
}

impl AsyncResponseHandler {

    pub fn new(result: ClientixResult<Response>) -> AsyncResponseHandler {
        AsyncResponseHandler { result }
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

    pub fn text_stream(self) -> ClientixResult<ClientixStream<String>> {
        match self.bytes_stream() {
            Ok(stream) => {
                let version = stream.version();
                let content_length = stream.content_length();
                let status = stream.status();
                let url = stream.url().clone();
                let remote_addr = stream.remote_addr();
                let headers = stream.headers().clone();
                let stream = stream
                    .map(|chunk| match chunk {
                        Ok(chunk) => {
                            let (text, _, _) = UTF_8.decode(&chunk);
                            Ok(text.to_string())
                        },
                        Err(error) => Err(error),
                    })
                    .flat_map(|text| {
                        match text {
                            Ok(text) => {
                                let lines: Vec<ClientixResult<String>> = text
                                    .split("\n")
                                    .map(str::trim)
                                    .flat_map(|line| line.strip_prefix("data:"))
                                    .map(str::trim)
                                    .map(str::to_string)
                                    .map(|value| Ok(value))
                                    .collect::<>();

                                futures_util::stream::iter(lines)
                            }
                            Err(error) => {
                                let lines: Vec<ClientixResult<String>> = vec![Err(error)];
                                futures_util::stream::iter(lines)
                            }
                        }
                    });

                Ok(ClientixStream::new(version, content_length, status, url, remote_addr, headers, stream))
            },
            Err(error) => Err(error),
        }
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

    pub fn bytes_stream(self) -> ClientixResult<ClientixStream<Bytes>> {
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

    pub fn json_stream<T>(self) -> ClientixResult<ClientixStream<T>> where T: DeserializeOwned + Clone {
        match self.text_stream() {
            Ok(stream) => {
                let version = stream.version();
                let content_length = stream.content_length();
                let status = stream.status();
                let url = stream.url().clone();
                let remote_addr = stream.remote_addr();
                let headers = stream.headers().clone();
                let stream = stream
                    .filter(|line| match line {
                        Ok(line) if !line.contains("[DONE]") => futures_util::future::ready(true),
                        _ => futures_util::future::ready(false)
                    })
                    .map(|line| match line {
                        Ok(line) => Ok(serde_json::from_str::<T>(line.as_str())?),
                        Err(err) => Err(err),
                    });

                Ok(ClientixStream::new(version, content_length, status, url, remote_addr, headers, stream))
            }
            Err(error) => Err(error),
        }
    }

}