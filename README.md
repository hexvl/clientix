# Clientix

Clientix is a Rust library for building HTTP clients and requests declaratively with procedural macros - no need to write complex imperative or functional logic.

## Description

With a simple procedural macro placed above a trait and its methods, Clientix lets you implement HTTP clients easily and efficiently - supporting both async and blocking modes. This makes it flexible enough to fit a wide range of scenarios, depending on your needs.

Currently, Clientix is built on top of reqwest with tokio as the async runtime. Future plans include adding support for other HTTP backends and giving you the ability to customize the underlying logic with minimal changes.

## Usage

To get started, you only need to add a single dependency:
```
cargo add clientix
```
And if you prefer to declare it explicitly in your Cargo.toml, just add it under the [dependencies] section:
```
clientix = "0.0.4"
```

## Examples

This section shows a few examples. For more details, please refer to the documentation.

Writing a synchronous blocking client:
```rust
use clientix::client::result::{ClientixResponse, ClientixResult};
use clientix::{clientix, get};

#[clientix(url = "https://api.restful-api.dev")]
trait ExampleClient {

    #[get(path = "/objects", consumes = "application/json", produces = "application/json")]
    fn get(&self) -> ClientixResult<ClientixResponse<String>>;
    
}

fn main() {
    let client = ExampleClient::new();

    let result = client.get();

    match result {
        Ok(value) => println!("{:?}", value),
        Err(error) => eprintln!("error occurred: {error}")
    }
}
```

However, writing a synchronous client comes with its trade-offs, so we also offer the option to implement an asynchronous client as a preferred alternative:
```rust
use clientix::client::result::{ClientixResponse, ClientixResult};
use clientix::{clientix, get};

#[clientix(url = "https://api.restful-api.dev", async = true)]
trait ExampleClient {

    #[get(path = "/objects", consumes = "application/json", produces = "application/json")]
    async fn get(&self) -> ClientixResult<ClientixResponse<String>>;

}

#[tokio::main]
async fn main() {
    let client = ExampleClient::new();

    let result = client.get().await;

    match result {
        Ok(value) => println!("{:?}", value),
        Err(error) => eprintln!("error occurred: {error}")
    }
}
```

The examples above demonstrate very simple clients. If you want to, for instance, receive an object as a result or send one via POST, you'll need to define the corresponding DTOs using the #[data_transfer] procedural macro:
```rust
use std::collections::HashMap;
use clientix::client::result::{ClientixResponse, ClientixResult};
use clientix::{clientix, data_transfer, get, post};

#[data_transfer]
struct CreateObjectRequest {
    name: String,
    data: HashMap<String, String>,
}

#[data_transfer]
struct CreatedObjectResponse {
    id: String,
    name: String,
    data: HashMap<String, String>
}

#[clientix(url = "https://api.restful-api.dev", async = true)]
trait ExampleClient {

    #[post(path = "/objects", consumes = "application/json", produces = "application/json")]
    async fn post(&self, #[body] request: CreateObjectRequest) -> ClientixResult<ClientixResponse<CreatedObjectResponse>>;

}

#[tokio::main]
async fn main() {
    let client = ExampleClient::new();

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
```

Note the #[body] macro on the post method argument - it’s required to map an object to the request body. You also have the following argument macros available:
- #[segment] - maps method arguments to path segments
- #[query] - maps method arguments to query parameters
- #[header] - maps method arguments to request headers

Future plans include expanding the argument macros to provide more flexible client configuration options.

The following example demonstrates how to implement a client with SSE response support using the OpenAI API as an example:
```rust
use std::io;
use std::io::Write;
use clientix::{clientix, data_transfer, post};
use clientix::client::asynchronous::ClientixStream;

#[data_transfer]
pub struct CompletionRequest {
    pub model: String,
    pub stream: bool,
    pub messages: Vec<CompletionMessage>
}

#[data_transfer]
pub struct CompletionMessage {
    pub role: String,
    pub content: String
}

#[data_transfer]
pub struct CompletionResponse {
    pub model: String,
    pub choices: Vec<CompletionChoice>
}

#[data_transfer]
pub struct CompletionChoice {
    pub message: Option<CompletionMessage>,
    pub delta: Option<CompletionDelta>
}

#[data_transfer]
pub struct CompletionDelta {
    pub content: Option<String>
}

#[clientix(url = "https://api.openai.com", path = "/v1/chat", async = true)]
trait ChatClient {
    
    #[post(path = "/completions", consumes = "application/json", produces = "application/json")]
    async fn stream_chat_completions(&self, #[body] request: CompletionRequest) -> Option<ClientixStream<CompletionResponse>>;

}

#[tokio::main]
async fn main() {
    let chat_client = ChatClient::config()
        .url("https://api.openai.com")
        .bearer_auth("<Open AI API token>")
        .setup();

    let request = CompletionRequest {
        model: "gpt-4o".to_string(),
        stream: true,
        messages: vec![
            CompletionMessage {
                role: "user".to_string(),
                content: "Hello! Write story, please.".to_string(),
            }
        ],
    };

    match chat_client.stream_chat_completions(request).await {
        Some(stream) => {
            println!("status: {}", stream.status());

            stream.execute(|value| {
                if let Ok(response) = value {
                    print!("{}", response.choices[0].delta.clone().map(|delta| delta.content.unwrap_or(String::new())).unwrap_or(String::new()));
                    io::stdout().flush().unwrap();
                }
            }).await
        },
        None => println!("error")
    }
}
```

More examples can be found in the [examples](https://github.com/hexvl/clientix/blob/main/clientix-example/src/lib.rs) directory.

## Support & Contribution

We’d be thrilled if you joined us in supporting and contributing to this project! Whether it’s reporting issues, suggesting improvements, or submitting pull requests, your help is always welcome. Together, we can make Clientix even better.
