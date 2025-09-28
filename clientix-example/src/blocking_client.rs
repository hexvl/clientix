use clientix::client::response::{ClientixResponse, ClientixResult};
use clientix::{clientix, get, post};
use crate::dto::{CreateObjectRequest, CreatedObjectResponse};

#[clientix(url = "https://api.restful-api.dev")]
pub trait BlockingExampleClient {
    
    #[get(path = "/objects")]
    fn get_result_full_response_string(&self) -> ClientixResult<ClientixResponse<String>>;

    #[get(path = "/objects")]
    fn get_result_string(&self) -> ClientixResult<String>;

    #[get(path = "/objects")]
    fn get_option_full_response_string(&self) -> Option<ClientixResponse<String>>;

    #[get(path = "/objects")]
    fn get_option_string(&self) -> Option<String>;

    // danger - panic
    #[get(path = "/objects")]
    fn get_string_without_wrapper(&self) -> String;

    #[post(path = "/objects")]
    fn post(&self, #[body] request: CreateObjectRequest) -> ClientixResult<ClientixResponse<CreatedObjectResponse>>;

}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn blocking_get_result_full_response_string_test() {
        let client = BlockingExampleClient::new();

        let result = client.get_result_full_response_string();

        match result {
            Ok(value) => println!("{:?}", value),
            Err(error) => eprintln!("error occurred: {error}")
        }
    }

    #[test]
    fn blocking_get_result_string_test() {
        let client = BlockingExampleClient::new();

        let result = client.get_result_string();

        match result {
            Ok(value) => println!("{value}"),
            Err(error) => eprintln!("error occurred: {error}")
        }
    }

    #[test]
    fn blocking_get_option_full_response_string_test() {
        let client = BlockingExampleClient::new();

        let result = client.get_option_full_response_string();

        match result {
            Some(value) => println!("{:?}", value),
            None => eprintln!("error occurred")
        }
    }

    #[test]
    fn blocking_get_option_string_test() {
        let client = BlockingExampleClient::new();

        let result = client.get_option_string();

        match result {
            Some(value) => println!("{value}"),
            None => eprintln!("error occurred")
        }
    }

    #[test]
    fn blocking_get_string_without_wrapper_test() {
        let client = BlockingExampleClient::new();

        let result = client.get_string_without_wrapper();

        println!("{result}");
    }

    #[test]
    fn blocking_post() {
        let client = BlockingExampleClient::new();

        let mut data = HashMap::new();
        data.insert("year".to_string(), "2019".to_string());
        data.insert("price".to_string(), "1849.99".to_string());
        data.insert("CPU model".to_string(), "Intel Core I9".to_string());
        data.insert("Hard disk size".to_string(), "1 TB".to_string());

        let request = CreateObjectRequest {
            name: "Test".to_string(),
            data,
        };

        let result = client.post(request);

        match result {
            Ok(value) => println!("{:?}", value),
            Err(error) => eprintln!("error occurred: {error}")
        }
    }
    
}