use quote::quote;
use syn::__private::TokenStream2;
use clientix_core::core::headers::content_type::ContentType;

#[derive(Clone, Debug)]
pub struct BodyArgumentCompiler {
    tokens: Option<TokenStream2>
}

impl BodyArgumentCompiler {

    pub fn parse(item: TokenStream2) -> Self {
        Self { tokens: Some(item) }
    }
    
    pub fn compile(&self, consumes: Option<ContentType>) -> TokenStream2 {
        let content_type: String = match consumes {
            Some(value) => value.to_string(),
            None => ContentType::ApplicationJson.to_string()
        };

        let body_variable = self.tokens.clone().expect("missing segment attribute");
        quote!(.body(#body_variable, #content_type.to_string().try_into().unwrap()))
    }

}