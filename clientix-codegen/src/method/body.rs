use quote::quote;
use syn::__private::TokenStream2;
use syn::{ PatType};
use syn::parse::Parser;
use clientix_core::core::headers::content_type::ContentType;
use crate::utils::throw_error;

#[derive(Clone, Default, Debug)]
pub struct BodyConfig {
    argument: Option<Box<syn::Pat>>,
    dry_run: bool,
}

impl BodyConfig {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse_stream(attrs: TokenStream2, dry_run: bool) -> Self {
        let mut body = Self::new();
        body.dry_run = dry_run;

        let parser = syn::meta::parser(|_| { Ok(()) });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), dry_run),
        };

        body
    }

    pub fn parse_argument(pat_type: &PatType, attrs: TokenStream2, dry_run: bool) -> Self {
        let mut body = Self::parse_stream(attrs, dry_run);
        body.argument = Some(pat_type.pat.clone());
        
        body
    }
    
    pub fn compile(&self, consumes: Option<ContentType>) -> TokenStream2 {
        let content_type: String = match consumes {
            Some(value) => value.to_string(),
            None => ContentType::ApplicationJson.to_string()
        };

        let body_variable = self.argument.clone().expect("missing segment attribute");
        quote! {
            .body(#body_variable, #content_type.to_string().try_into().unwrap())
        }
    }

}