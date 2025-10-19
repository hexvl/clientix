use syn::__private::TokenStream2;
use syn::{Ident, LitBool, LitStr};
use syn::parse::Parser;

const URL_ATTR: &str = "url";
const PATH_ATTR: &str = "path";
const ASYNC_ATTR: &str = "async";

#[derive(Clone, Debug)]
pub struct ClientAttributes {
    url: Option<String>,
    path: Option<String>,
    async_supported: bool
}

impl ClientAttributes {

    pub fn new(url: Option<String>, path: Option<String>, async_supported: bool) -> Self {
        Self { url, path, async_supported }
    }

    pub fn parse(attrs: TokenStream2) -> Self {
        let mut attributes = Self::new(None, None, false);

        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident(URL_ATTR) => {
                    attributes.url = Some(meta.value()?.parse::<LitStr>()?.value());
                    Ok(())
                },
                ref path if path.is_ident(PATH_ATTR) => {
                    attributes.path = Some(meta.value()?.parse::<LitStr>()?.value());
                    Ok(())
                }
                ref path if path.is_ident(ASYNC_ATTR) => {
                    attributes.async_supported = meta.value()?.parse::<LitBool>()?.value();
                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected client parameter: {}", meta.path.get_ident().map(Ident::to_string).unwrap_or_default())))
            }
        });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(e) => panic!("{}", e)
        };

        attributes
    }
    
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    pub fn path(&self) -> Option<&String> {
        self.path.as_ref()
    }
    
    pub fn async_supported(&self) -> bool {
        self.async_supported
    }
    
}