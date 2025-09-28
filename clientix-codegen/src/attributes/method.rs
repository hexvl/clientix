use syn::__private::TokenStream2;
use syn::LitStr;
use syn::parse::Parser;
use clientix_core::core::headers::content_type::ContentType;
use clientix_core::prelude::reqwest::Method;
use crate::utils::throw_error;

#[derive(Clone, Debug)]
pub struct MethodAttributes {
    method: Method,
    path: Option<String>,
    consumes: Option<ContentType>,
    produces: Option<ContentType>,
}

impl MethodAttributes {

    fn new(method: Method, path: Option<String>, consumes: Option<ContentType>, produces: Option<ContentType>) -> Self {
        Self { method, path, consumes, produces }
    }

    pub fn parse(method: Method, attrs: TokenStream2, dry_run: bool) -> Self {
        let mut attributes = Self::new(method, None, None, None);

        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident("path") => {
                    attributes.path = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident("consumes") => {
                    match meta.value()?.parse::<LitStr>()?.value().try_into() {
                        Ok(consumes) => attributes.consumes = Some(consumes),
                        Err(_) => throw_error("invalid content-type for consumes", dry_run)
                    };

                    Ok(())
                }
                ref path if path.is_ident("produces") => {
                    match meta.value()?.parse::<LitStr>()?.value().try_into() {
                        Ok(produces) => attributes.produces = Some(produces),
                        Err(_) => throw_error("invalid content-type for produces", dry_run)
                    }

                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected method param: {}", meta.path.get_ident().unwrap())))
            }
        });

        match parser.parse2(attrs) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), dry_run),
        }

        attributes
    }

    pub fn method(&self) -> &Method {
        &self.method
    }
    
    pub fn path(&self) -> Option<&String> {
        self.path.as_ref()
    }
    
    pub fn consumes(&self) -> Option<&ContentType> {
        self.consumes.as_ref()
    }
    
    pub fn produces(&self) -> Option<&ContentType> {
        self.produces.as_ref()
    }
    
}