use syn::{LitBool, LitStr};
use syn::__private::TokenStream2;
use syn::parse::Parser;
use crate::utils::throw_error;

#[derive(Clone, Debug)]
pub struct HeaderAttributes {
    name: Option<String>,
    value: Option<String>,
    sensitive: bool,
}

impl HeaderAttributes {
    
    pub fn new(name: Option<String>, value: Option<String>, sensitive: bool) -> Self {
        Self { name, value, sensitive }
    }
    
    pub fn parse(attrs: TokenStream2, dry_run: bool) -> Self {
        let mut attributes = Self::new(None, None, false);
        
        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident("name") => {
                    attributes.name = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident("value") => {
                    attributes.value = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident("sensitive") => {
                    attributes.sensitive = meta.value()?.parse::<LitBool>()?.value();

                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected method param: {}", meta.path.get_ident().unwrap())))
            }
        });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), dry_run),
        };
        
        attributes
    }
    
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    
    pub fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }
    
    pub fn sensitive(&self) -> bool {
        self.sensitive
    }
    
}