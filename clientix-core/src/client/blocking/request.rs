use http::Method;
use strfmt::strfmt;
use crate::client::blocking::client::BlockingClient;
use crate::client::blocking::response::BlockingResponseHandler;
use crate::client::request::{ClientixRequestBuilder, RequestConfig};
use crate::client::response::{ClientixError, ClientixErrorData, ClientixResult};

pub struct BlockingRequest {
    client: BlockingClient,
    method: Method,
    config: RequestConfig,
    result: ClientixResult<()>,
}

impl ClientixRequestBuilder for BlockingRequest {

    fn config(&mut self) -> &mut RequestConfig {
        &mut self.config
    }

    fn result(&mut self) -> &mut ClientixResult<()> {
        &mut self.result
    }
    
}

impl BlockingRequest {

    pub fn new(client: BlockingClient, method: Method) -> Self {
        BlockingRequest {
            client,
            method,
            config: Default::default(),
            result: Ok(())
        }
    }

    pub fn builder(client: BlockingClient, method: Method) -> Self {
        BlockingRequest::new(client, method)
    }

    pub fn send(self) -> BlockingResponseHandler {
        if let Err(error) = self.result {
            return BlockingResponseHandler::new(Err(error));
        }

        let method_path = strfmt(self.config.get_path(), self.config.get_path_segments()).expect("failed to format path");
        let full_path = format!("{}{}", self.client.path, method_path);
        let url = format!("{}{}", self.client.url, full_path);

        match self.client.client.lock() {
            Ok(client) => {
                let mut request_builder = match self.method {
                    Method::GET => client.get(url.clone()),
                    Method::POST => client.post(url.clone()),
                    Method::PUT => client.put(url.clone()),
                    Method::DELETE => client.delete(url.clone()),
                    Method::HEAD => client.head(url.clone()),
                    Method::PATCH => client.patch(url.clone()),
                    _ => {
                        let error_data = ClientixErrorData::builder().message(format!("invalid method: {:?}", self.method).as_str()).build();
                        return BlockingResponseHandler::new(Err(ClientixError::InvalidRequest(error_data, None)));
                    },
                };

                request_builder = request_builder
                    .headers(self.config.get_headers().clone())
                    .query(self.config.get_queries());

                request_builder = match self.config.get_body() {
                    Some(body) => request_builder.body::<String>(body.into()),
                    None => request_builder,
                };

                request_builder = match self.config.get_timeout() {
                    Some(timeout) => request_builder.timeout(timeout),
                    None => request_builder,
                };

                match request_builder.send() {
                    Ok(response) => BlockingResponseHandler::new(Ok(response)),
                    Err(error) => BlockingResponseHandler::new(Err(ClientixError::Http(ClientixErrorData::new(), Some(error.into()))))
                }
            },
            Err(error) => {
                let error_data = ClientixErrorData::builder().message(format!("client locked: {:?}", error).as_str()).build();
                BlockingResponseHandler::new(Err(ClientixError::Other(error_data, None)))
            }
        }
    }

}