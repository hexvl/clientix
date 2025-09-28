pub mod sse;

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::Bytes;
use futures_core::Stream;
use futures_util::{StreamExt, TryStreamExt};
use http::{HeaderMap, StatusCode, Version};
use reqwest::Url;
use crate::client::asynchronous::stream::sse::ClientixSSEStream;
use crate::client::response::ClientixResult;

pub struct ClientixStream {
    version: Version,
    content_length: Option<u64>,
    status: StatusCode,
    url: Url,
    remote_addr: Option<SocketAddr>,
    headers: HeaderMap,
    stream: Pin<Box<dyn Stream<Item = ClientixResult<Bytes>>>>,
}

impl ClientixStream {

    pub fn new(
        version: Version,
        content_length: Option<u64>,
        status: StatusCode,
        url: Url,
        remote_addr: Option<SocketAddr>,
        headers: HeaderMap,
        stream: impl Stream<Item = ClientixResult<Bytes>> + 'static
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
    
    pub fn sse(self) -> ClientixSSEStream<String> {
        self.into()
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

    pub async fn execute<F>(mut self, mut handle: F) where F: FnMut(ClientixResult<Bytes>) {
        while let Some(result) = self.stream.next().await {
            handle(result);
        }
    }

    pub async fn collect(self) -> ClientixResult<Vec<Bytes>> {
        self.stream.try_collect().await
    }
    
}

impl Stream for ClientixStream {
    type Item = ClientixResult<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}