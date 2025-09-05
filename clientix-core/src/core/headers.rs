pub mod content_type {
    use reqwest::header::HeaderValue;

    #[derive(Debug, Clone, Copy)]
    pub enum ContentType {
        ApplicationJson,
        ApplicationXml,
        ApplicationXWwwFormUrlEncoded,
        TextHtml
    }

    impl TryFrom<String> for ContentType {
        type Error = ();

        fn try_from(value: String) -> Result<Self, Self::Error> {
            match value.as_str() {
                "application/json" => Ok(ContentType::ApplicationJson),
                "application/xml" => Ok(ContentType::ApplicationXml),
                "application/x-www-form-urlencoded" => Ok(ContentType::ApplicationXWwwFormUrlEncoded),
                "text/html" => Ok(ContentType::TextHtml),
                _ => Err(())
            }
        }
    }

    impl From<ContentType> for String {
        fn from(value: ContentType) -> String {
            match value {
                ContentType::ApplicationJson => "application/json".to_string(),
                ContentType::ApplicationXml => "application/xml".to_string(),
                ContentType::ApplicationXWwwFormUrlEncoded => "application/x-www-form-urlencoded".to_string(),
                ContentType::TextHtml => "text/html".to_string(),
            }
        }
    }

    impl TryFrom<ContentType> for HeaderValue {

        type Error = http::header::InvalidHeaderValue;

        fn try_from(value: ContentType) -> Result<Self, Self::Error> {
            let string: String = value.into();
            HeaderValue::from_str(string.as_str())
        }
    }
    
}