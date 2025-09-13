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

pub trait ClientixStreamInterface<T>: Stream {

    fn version(&self) -> Version;

    fn content_length(&self) -> Option<u64>;

    fn status(&self) -> StatusCode;

    fn url(&self) -> &Url;

    fn remote_addr(&self) -> Option<SocketAddr>;

    fn headers(&self) -> &HeaderMap;

    #[allow(async_fn_in_trait)]
    async fn execute<F>(self, handle: F) where F: FnMut(ClientixResult<T>);

    #[allow(async_fn_in_trait)]
    async fn collect(self) -> ClientixResult<Vec<T>>;

}

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

}

impl Stream for ClientixStream {
    type Item = ClientixResult<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}

impl ClientixStreamInterface<Bytes> for ClientixStream {

    fn version(&self) -> Version {
        self.version
    }

    fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    fn status(&self) -> StatusCode {
        self.status
    }

    fn url(&self) -> &Url {
        &self.url
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    async fn execute<F>(mut self, mut handle: F) where F: FnMut(ClientixResult<Bytes>) {
        while let Some(result) = self.stream.next().await {
            handle(result);
        }
    }

    async fn collect(self) -> ClientixResult<Vec<Bytes>> {
        self.stream.try_collect().await
    }

}