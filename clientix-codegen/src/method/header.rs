use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::parse::Parser;
use syn::{LitBool, LitStr, PatType};
use crate::method::placeholder::PlaceholderConfig;
use crate::utils::throw_error;

#[derive(Clone, Default)]
pub struct HeaderConfig {
    argument: Option<Box<syn::Pat>>,
    name: Option<String>,
    value: Option<String>,
    sensitive: bool,
    dry_run: bool,
}

impl HeaderConfig {

    pub fn new(name: Option<String>, value: Option<String>) -> Self {
        Self {
            argument: None,
            name,
            value,
            sensitive: false,
            dry_run: false
        }
    }

    pub fn parse_stream(attrs: TokenStream2, dry_run: bool) -> Self {
        let mut header = Self::default();
        header.dry_run = dry_run;

        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident("name") => {
                    header.name = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident("value") => {
                    header.value = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident("sensitive") => {
                    header.sensitive = meta.value()?.parse::<LitBool>()?.value();

                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected method param: {}", meta.path.get_ident().unwrap())))
            }
        });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), dry_run),
        };

        header
    }

    pub fn parse_argument(pat_type: &PatType, attrs: TokenStream2, dry_run: bool) -> Self {
        let mut header = Self::parse_stream(attrs, dry_run);
        header.argument = Some(pat_type.pat.clone());

        header
    }

    pub fn compile(&self) -> TokenStream2 {
        self.compile_with_placeholders(&Vec::new())
    }

    pub fn compile_with_placeholders(&self, placeholders: &Vec<PlaceholderConfig>) -> TokenStream2 {
        if let Some(header_argument) = &self.argument {
            let header_id = format!("{}", quote! {#header_argument});
            quote!(.header(#header_id, #header_argument.to_string().as_str()))
        } else {
            if self.name.is_none() && self.value.is_none() {
                return quote! {}
            }

            let name = self.name.clone().unwrap();
            let value = self.value.clone().unwrap();

            let mut stream = TokenStream2::new();
            if !placeholders.is_empty() {
                stream.extend(quote! {
                    let mut arguments = std::collections::HashMap::new();
                });

                for placeholder_variable in placeholders.iter() {
                    stream.extend(placeholder_variable.compile())
                }

                stream.extend(quote! {
                    clientix::prelude::strfmt::strfmt(#value, &arguments).expect("failed to format header").as_str()
                });

                TokenStream2::from(quote! {
                    .header(#name, {#stream})
                })
            } else {
                TokenStream2::from(quote! {
                    .header(#name, #value)
                })
            }
        }
    }

}

pub fn parse_header(item: TokenStream, attrs: TokenStream) -> TokenStream {
    HeaderConfig::parse_stream(TokenStream2::from(attrs), true);
    TokenStream::from(item)
}