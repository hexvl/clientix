mod blocking_client;
mod dto;
mod async_client;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::async_client::AsyncExampleClient;
    use crate::blocking_client::BlockingExampleClient;
    use crate::dto::CreateObjectRequest;

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

    #[tokio::test]
    async fn async_get_result_full_response_string_test() {
        let client = AsyncExampleClient::new();

        let result = client.get_result_full_response_string().await;

        match result {
            Ok(value) => println!("{:?}", value),
            Err(error) => eprintln!("error occurred: {error}")
        }
    }

    #[tokio::test]
    async fn async_get_result_string_test() {
        let client = AsyncExampleClient::new();

        let result = client.get_result_string().await;

        match result {
            Ok(value) => println!("{value}"),
            Err(error) => eprintln!("error occurred: {error}")
        }
    }

    #[tokio::test]
    async fn async_get_option_full_response_string_test() {
        let client = AsyncExampleClient::new();

        let result = client.get_option_full_response_string().await;

        match result {
            Some(value) => println!("{:?}", value),
            None => eprintln!("error occurred")
        }
    }

    #[tokio::test]
    async fn async_get_option_string_test() {
        let client = AsyncExampleClient::new();

        let result = client.get_option_string().await;

        match result {
            Some(value) => println!("{value}"),
            None => eprintln!("error occurred")
        }
    }

    #[tokio::test]
    async fn async_get_string_without_wrapper_test() {
        let client = AsyncExampleClient::new();

        let result = client.get_string_without_wrapper().await;

        println!("{result}");
    }

    #[tokio::test]
    async fn async_post() {
        let client = AsyncExampleClient::new();

        let mut data = HashMap::new();
        data.insert("year".to_string(), "2019".to_string());
        data.insert("price".to_string(), "1849.99".to_string());
        data.insert("CPU model".to_string(), "Intel Core I9".to_string());
        data.insert("Hard disk size".to_string(), "1 TB".to_string());

        let request = CreateObjectRequest {
            name: "Test".to_string(),
            data,
        };

        let result = client.post(request).await;

        match result {
            Ok(value) => println!("{:?}", value),
            Err(error) => eprintln!("error occurred: {error}")
        }
    }

}

