use crate::client::asynchronous::stream::{ClientixStream, ClientixStreamInterface};
use crate::client::response::{ClientixError, ClientixResult};
use futures_core::Stream;
use futures_util::{StreamExt, TryStreamExt};
use http::{HeaderMap, StatusCode, Version};
use reqwest::Url;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};
use encoding_rs::UTF_8;
use serde::de::DeserializeOwned;

const ID_PROPERTY: &str = "id:";
const EVENT_PROPERTY: &str = "event:";
const COMMENT_PROPERTY: &str = ":";
const RETRY_PROPERTY: &str = "retry:";
const DATA_PROPERTY: &str = "data:";

#[derive(Debug, Clone)]
pub struct SSE<T> {
    id: Option<String>,
    event: Option<String>,
    comment: Option<String>,
    retry: Option<u64>,
    data: Option<T>
}

impl<T> SSE<T> {

    fn new() -> Self {
        Self {
            id: None,
            event: None,
            comment: None,
            retry: None,
            data: None,
        }
    }

    pub fn id(&self) -> &Option<String> {
        &self.id
    }

    pub fn event(&self) -> &Option<String> {
        &self.event
    }

    pub fn comment(&self) -> &Option<String> {
        &self.comment
    }

    pub fn retry(&self) -> &Option<u64> {
        &self.retry
    }

    pub fn data(&self) -> &Option<T> {
        &self.data
    }

}

pub struct ClientixSSEStream<T> {
    version: Version,
    content_length: Option<u64>,
    status: StatusCode,
    url: Url,
    remote_addr: Option<SocketAddr>,
    headers: HeaderMap,
    stream: Pin<Box<dyn Stream<Item = ClientixResult<SSE<T>>>>>,
}

impl<T> ClientixSSEStream<T> {

    pub fn new(
        version: Version,
        content_length: Option<u64>,
        status: StatusCode,
        url: Url,
        remote_addr: Option<SocketAddr>,
        headers: HeaderMap,
        stream: impl Stream<Item = ClientixResult<SSE<T>>> + 'static
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

}

impl ClientixSSEStream<String> {

    pub fn object_stream<T, F>(self, mut convert: F) -> ClientixSSEStream<T> 
    where T: DeserializeOwned + Clone, F: FnMut(&str) -> ClientixResult<T> + 'static {
        let version = self.version();
        let content_length = self.content_length();
        let status = self.status();
        let url = self.url().clone();
        let remote_addr = self.remote_addr();
        let headers = self.headers().clone();
        let stream = self
            .filter(|line| match line {
                Ok(line) if !line.data.clone().unwrap_or(String::new()).contains("[DONE]") => futures_util::future::ready(true),
                _ => futures_util::future::ready(false)
            })
            .map(move |line| match line {
                Ok(line) => {
                    let mut sse = SSE::new();
                    sse.id = line.id.clone();
                    sse.event = line.event.clone();
                    sse.comment = line.comment.clone();
                    sse.retry = line.retry;
                    sse.data =  Some(convert(line.data.clone().unwrap_or(String::new()).as_str())?);

                    Ok(sse)
                },
                Err(err) => Err(err),
            });
        
        ClientixSSEStream::new(version, content_length, status, url, remote_addr, headers, stream)
    }

    pub fn json_stream<T>(self) -> ClientixSSEStream<T> where T: DeserializeOwned + Clone {
        self.object_stream(|string| {
            serde_json::from_str::<T>(string).map_err(ClientixError::from)
        })
    }
    
    pub fn xml_stream<T>(self) -> ClientixSSEStream<T> where T: DeserializeOwned + Clone {
        self.object_stream(|string| serde_xml_rs::from_str::<T>(string).map_err(ClientixError::from))
    }
    
}

impl<T> Stream for ClientixSSEStream<T> {
    type Item = ClientixResult<SSE<T>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}

impl<T> ClientixStreamInterface<SSE<T>> for ClientixSSEStream<T> {

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

    async fn execute<F>(mut self, mut handle: F) where F: FnMut(ClientixResult<SSE<T>>) {
        while let Some(result) = self.stream.next().await {
            handle(result);
        }
    }

    async fn collect(self) -> ClientixResult<Vec<SSE<T>>> {
        self.stream.try_collect().await
    }

}

impl From<ClientixStream> for ClientixSSEStream<String> {
    fn from(stream: ClientixStream) -> Self {
        let version = stream.version();
        let content_length = stream.content_length();
        let status = stream.status();
        let url = stream.url().clone();
        let remote_addr = stream.remote_addr();
        let headers = stream.headers().clone();

        let mut buffer = String::new();
        let stream = stream
            .map(|chunk| match chunk {
                Ok(chunk) => {
                    let (text, _, _) = UTF_8.decode(&chunk);
                    Ok(text.to_string())
                },
                Err(error) => Err(error),
            })
            .flat_map(move |text| match text {
                Ok(text) => {
                    let mut events = Vec::new();
                    for line in text.lines() {
                        let mut sse = SSE::new();
                        match line {
                            line if line.starts_with(ID_PROPERTY) => {
                                sse.id = line.strip_prefix(ID_PROPERTY)
                                    .map(str::trim)
                                    .map(str::to_string);
                            }
                            line if line.starts_with(EVENT_PROPERTY) => {
                                sse.event = line.strip_prefix(EVENT_PROPERTY)
                                    .map(str::trim)
                                    .map(str::to_string);
                            }
                            line if line.starts_with(COMMENT_PROPERTY) => {
                                sse.comment = line.strip_prefix(COMMENT_PROPERTY)
                                    .map(str::trim)
                                    .map(str::to_string);
                            }
                            line if line.starts_with(RETRY_PROPERTY) => {
                                sse.retry = line.strip_prefix(RETRY_PROPERTY)
                                    .map(str::trim)
                                    .map(u64::from_str)
                                    .map(|result| match result {
                                        Ok(value) => Some(value),
                                        Err(_) => None
                                    })
                                    .unwrap_or(None);
                            }
                            line if line.starts_with(DATA_PROPERTY) => {
                                buffer.push_str(line.trim_start_matches(DATA_PROPERTY).trim());
                            }
                            _ => {
                                if !buffer.is_empty() {
                                    sse.data = Some(buffer.to_string());
                                    buffer.clear();

                                    events.push(Ok(sse))
                                }
                            }
                        }
                    }

                    futures_util::stream::iter(events)
                }
                Err(error) => futures_util::stream::iter(vec![Err(error)])
            });

        ClientixSSEStream::new(
            version,
            content_length,
            status,
            url,
            remote_addr,
            headers,
            stream
        )
    }
}