use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::__private::TokenStream2;
use syn::{Attribute, LitStr, PatType};
use syn::parse::Parser;
use crate::utils::throw_error;

#[derive(Clone, Default)]
pub struct PlaceholderConfig {
    argument: Option<Box<syn::Pat>>,
    dry_run: bool,
}

impl PlaceholderConfig {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse_stream(attrs: TokenStream2, dry_run: bool) -> Self {
        let mut placeholder = Self::new();
        placeholder.dry_run = dry_run;

        let parser = syn::meta::parser(|meta| { Ok(()) });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), dry_run),
        };

        placeholder
    }
    
    pub fn parse_argument(pat_type: &PatType, attribute: &Attribute, dry_run: bool) -> Self {
        let mut placeholder = Self::parse_stream(attribute.to_token_stream(), dry_run);
        placeholder.argument = Some(pat_type.pat.clone());
        
        placeholder
    }

    pub fn compile(&self) -> TokenStream2 {
        let placeholder_variable = self.argument.clone().expect("missing segment attribute");
        let placeholder_id = format!("{}", quote! {#placeholder_variable});
        quote! {
            arguments.insert(#placeholder_id.to_string(), #placeholder_variable.to_string());
        }
    }
    
}