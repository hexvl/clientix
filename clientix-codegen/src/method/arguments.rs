use quote::{quote, ToTokens};
use syn::{Meta, PatType};
use syn::__private::TokenStream2;
use clientix_core::core::headers::content_type::ContentType;
use crate::method::body::BodyConfig;
use crate::method::header::HeaderConfig;
use crate::method::placeholder::PlaceholderConfig;
use crate::method::query::QueryConfig;
use crate::method::segment::SegmentConfig;
use crate::utils::throw_error;

#[derive(Clone, Default, Debug)]
pub struct ArgumentsConfig {
    segments: Vec<SegmentConfig>,
    queries: Vec<QueryConfig>,
    headers: Vec<HeaderConfig>,
    placeholders: Vec<PlaceholderConfig>,
    body: Option<BodyConfig>,
    dry_run: bool,
}

#[allow(dead_code)]
impl ArgumentsConfig {

    pub fn new(dry_run: bool) -> Self {
        Self {
            segments: vec![],
            queries: vec![],
            headers: vec![],
            placeholders: vec![],
            body: None,
            dry_run,
        }
    }
    
    pub fn segments(&self) -> &Vec<SegmentConfig> {
        &self.segments
    }

    pub fn queries(&self) -> &Vec<QueryConfig> {
        &self.queries
    }

    pub fn headers(&self) -> &Vec<HeaderConfig> {
        &self.headers
    }

    pub fn placeholders(&self) -> &Vec<PlaceholderConfig> {
        &self.placeholders
    }

    pub fn body(&self) -> Option<&BodyConfig> {
        self.body.as_ref()
    }

    pub fn add(&mut self, pat_type: &mut PatType) {
        let mut not_processed_attrs = Vec::new();
        
        pat_type.attrs.clone().into_iter().map(|attr_expr| match attr_expr.meta.clone() {
            Meta::Path(value) => (value, TokenStream2::new(), attr_expr),
            Meta::List(value) => (value.path, value.tokens.to_token_stream(), attr_expr),
            Meta::NameValue(value) => (value.path, TokenStream2::new(), attr_expr),
        }).for_each(|(path, attrs, attr_expr)| {
            match path {
                ref path if path.is_ident("segment") => {
                    self.segments.push(SegmentConfig::parse_argument(pat_type, attrs, self.dry_run));
                },
                ref path if path.is_ident("query") => {
                    self.queries.push(QueryConfig::parse_argument(pat_type, attrs, self.dry_run));
                },
                ref path if path.is_ident("header") => {
                    self.headers.push(HeaderConfig::parse_argument(pat_type, attrs, self.dry_run));
                },
                ref path if path.is_ident("placeholder") => {
                    self.placeholders.push(PlaceholderConfig::parse_argument(pat_type, attrs, self.dry_run));
                },
                ref path if path.is_ident("body") => {
                    match self.body {
                        None => self.body = Some(BodyConfig::parse_argument(pat_type, attrs, self.dry_run)),
                        Some(_) => throw_error("multiple body arg", self.dry_run),
                    }
                },
                ref path if path.is_ident("args") => {
                    self.parse_args_struct(pat_type, self.dry_run);
                }
                _ => {
                    not_processed_attrs.push(attr_expr);
                }
            }
        });
        
        pat_type.attrs = not_processed_attrs;
    }

    pub fn compile_segments(&self, path: Option<&String>) -> TokenStream2 {
        if let Some(path) = path {
            if self.segments().is_empty() {
                quote!(.path(#path))
            } else {
                let mut stream = TokenStream2::from(quote! {
                    let mut arguments = std::collections::HashMap::new();
                });

                for segment_variable in self.segments().iter() {
                    stream.extend(segment_variable.compile());
                }

                stream.extend(quote! {
                    clientix::prelude::strfmt::strfmt(#path, &arguments).expect("failed to format header").as_str()
                });

                quote!(.path({#stream}))
            }
        } else {
            quote!()
        }
    }

    pub fn compile_headers(&self) -> TokenStream2 {
        let mut stream = TokenStream2::new();
        if self.headers.is_empty() {
            stream.extend(quote! {});
        } else {
            for header_variable in self.headers.iter() {
                stream.extend(header_variable.compile_with_placeholders(&self.placeholders));
            }
        }

        stream
    }

    pub fn compile_queries(&self) -> TokenStream2 {
        if self.queries.is_empty() {
            quote! {}
        } else {
            let mut stream = TokenStream2::new();
            for query_variable in self.queries.iter() {
                stream.extend(query_variable.compile());
            }

            stream
        }
    }

    pub fn compile_body(&self, consumes: Option<ContentType>) -> TokenStream2 {
        if let Some(body_variable) = &self.body {
            body_variable.compile(consumes)
        } else {
            quote! {}
        }
    }

    
    fn parse_args_struct(&self, _pat_type: &mut PatType, _dry_run: bool) -> TokenStream2 {
        // TODO: реализовать парсинг структуры.
        //  Реализацию можно выполнить через создание соответствующей структуры,
        //  которая сама лично парсит свои атрибуты на полях и имплементирует соответствующие
        //  геттеры по полям. В результате эти геттеры должны быть вызваны в местах задавания
        //  соответствующих переменных. Возможно, потребуется доработка clientix-core,
        //  но лучше постараться избежать этого

        TokenStream2::from(quote! {})
    }

}