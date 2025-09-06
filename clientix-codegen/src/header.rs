use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::parse::Parser;
use syn::{LitBool, LitStr};
use crate::utils::throw_error;

#[derive(Clone)]
pub struct HeaderConfig {
    name: Option<String>,
    value: Option<String>,
    sensitive: bool,
    dry_run: bool
}

impl HeaderConfig {

    pub fn new() -> Self {
        HeaderConfig {
            name: Default::default(),
            value: Default::default(),
            sensitive: false,
            dry_run: false
        }
    }

    pub fn parse(attrs: TokenStream2, dry_run: bool) -> Self {
        let mut header = HeaderConfig::new();
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

    pub fn compile_header(&self, placeholders: Vec<Box<syn::Pat>>) -> TokenStream2 {
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
                let placeholder_id = format!("{}", quote! {#placeholder_variable});
                stream.extend(quote! {
                    arguments.insert(#placeholder_id.to_string(), #placeholder_variable.to_string());
                });
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

pub fn parse_header(item: TokenStream, attrs: TokenStream) -> TokenStream {
    HeaderConfig::parse(TokenStream2::from(attrs), true);
    TokenStream::from(item)
}