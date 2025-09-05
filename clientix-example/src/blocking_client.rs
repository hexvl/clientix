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