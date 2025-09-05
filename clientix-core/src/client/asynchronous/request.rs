use http::Method;
use strfmt::strfmt;
use crate::client::asynchronous::client::AsyncClient;
use crate::client::asynchronous::response::AsyncResponseHandler;
use crate::client::request::{ClientixRequestBuilder, RequestConfig};
use crate::client::response::{ClientixError, ClientixErrorData, ClientixResult};

pub struct AsyncRequest {
    client: AsyncClient,
    method: Method,
    config: RequestConfig,
    result: ClientixResult<()>
}

impl ClientixRequestBuilder for AsyncRequest {

    fn config(&mut self) -> &mut RequestConfig {
        &mut self.config
    }

    fn result(&mut self) -> &mut ClientixResult<()> {
        &mut self.result
    }
    
}

impl AsyncRequest {

    pub fn new(client: AsyncClient, method: Method) -> Self {
        AsyncRequest {
            client,
            method,
            config: Default::default(),
            result: Ok(())
        }
    }

    pub fn builder(client: AsyncClient, method: Method) -> Self {
        AsyncRequest::new(client, method)
    }
    
    pub async fn send(self) -> AsyncResponseHandler {
        if let Err(error) = self.result {
            return AsyncResponseHandler::new(Err(error));
        }

        let method_path = strfmt(&self.config.get_path(), &self.config.get_path_segments()).expect("failed to format path");
        let full_path = format!("{}{}", self.client.path, method_path);
        let url = format!("{}{}", self.client.url, full_path);

        match self.client.client.lock() {
            Ok(client) => {
                let mut request_builder = match self.method {
                    Method::GET => client.get(url),
                    Method::POST => client.post(url),
                    Method::PUT => client.put(url),
                    Method::DELETE => client.delete(url),
                    Method::HEAD => client.head(url),
                    Method::PATCH => client.patch(url),
                    _ => {
                        let error_data = ClientixErrorData::builder().message(format!("invalid method: {:?}", self.method).as_str()).build();
                        return AsyncResponseHandler::new(Err(ClientixError::InvalidRequest(error_data, None)));
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

                match request_builder.send().await {
                    Ok(response) => AsyncResponseHandler::new(Ok(response)),
                    Err(error) => AsyncResponseHandler::new(Err(ClientixError::Http(ClientixErrorData::new(), Some(error.into()))))
                }
            },
            Err(err) => {
                let error_data = ClientixErrorData::builder().message(format!("client locked: {:?}", err).as_str()).build();
                AsyncResponseHandler::new(Err(ClientixError::Other(error_data, None)))
            }
        }
    }

}