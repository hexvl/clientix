use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_core::Stream;
use futures_util::{StreamExt, TryStreamExt};
use http::{HeaderMap, StatusCode, Version};
use reqwest::Url;
use crate::client::result::ClientixResult;

pub struct ClientixStream<T> {
    version: Version,
    content_length: Option<u64>,
    status: StatusCode,
    url: Url,
    remote_addr: Option<SocketAddr>,
    headers: HeaderMap,
    stream: Pin<Box<dyn Stream<Item = ClientixResult<T>>>>,
}

impl<T> Stream for ClientixStream<T> {
    type Item = ClientixResult<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}

impl<T> ClientixStream<T> {

    pub fn new(
        version: Version,
        content_length: Option<u64>,
        status: StatusCode,
        url: Url,
        remote_addr: Option<SocketAddr>,
        headers: HeaderMap,
        stream: impl Stream<Item = ClientixResult<T>> + 'static
    ) -> Self {
        Self {
            version,
            content_length,
            status,
            url,
            remote_addr,
            headers,
            stream: Box::pin(stream)
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
    
    pub fn url(&self) -> &Url {
        &self.url
    }
    
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }
    
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub async fn execute<F>(mut self, mut handle: F) where F: FnMut(ClientixResult<T>) {
        while let Some(result) = self.stream.next().await {
            handle(result);
        }
    }

    pub async fn collect(self) -> ClientixResult<Vec<T>> {
        self.stream.try_collect().await
    }

}