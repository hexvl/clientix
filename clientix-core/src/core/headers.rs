pub mod content_type {
    use std::fmt::Display;
    use reqwest::header::HeaderValue;

    #[derive(Debug, Clone, Copy)]
    pub enum ContentType {
        ApplicationJson,
        ApplicationXml,
        ApplicationXWwwFormUrlEncoded,
        TextHtml,
        TextEventStream
    }

    impl TryFrom<String> for ContentType {
        type Error = ();

        fn try_from(value: String) -> Result<Self, Self::Error> {
            match value.as_str() {
                "application/json" => Ok(ContentType::ApplicationJson),
                "application/xml" => Ok(ContentType::ApplicationXml),
                "application/x-www-form-urlencoded" => Ok(ContentType::ApplicationXWwwFormUrlEncoded),
                "text/html" => Ok(ContentType::TextHtml),
                "text/event-bytes" => Ok(ContentType::TextEventStream),
                _ => Err(())
            }
        }
    }

    impl Display for ContentType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let str = match self {
                ContentType::ApplicationJson => "application/json",
                ContentType::ApplicationXml => "application/xml",
                ContentType::ApplicationXWwwFormUrlEncoded => "application/x-www-form-urlencoded",
                ContentType::TextHtml => "text/html",
                ContentType::TextEventStream => "text/event-bytes"
            };
            
            write!(f, "{}", str)
        }
    }

    impl TryFrom<ContentType> for HeaderValue {

        type Error = http::header::InvalidHeaderValue;

        fn try_from(value: ContentType) -> Result<Self, Self::Error> {
            let string: String = value.to_string();
            HeaderValue::from_str(string.as_str())
        }
    }
    
}