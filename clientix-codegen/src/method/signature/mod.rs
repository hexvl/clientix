pub(crate) mod segment;
pub(crate) mod query;
pub(crate) mod body;
pub(crate) mod header;
pub(crate) mod args;
pub(crate) mod output;

use quote::{quote, ToTokens};
use syn::{FnArg, Ident, Meta, PatType, Signature};
use syn::__private::{Span, TokenStream2};
use clientix_core::core::headers::content_type::ContentType;
use crate::method::signature::args::ArgsArgumentCompiler;
use crate::method::signature::body::BodyArgumentCompiler;
use crate::method::signature::header::HeaderArgumentCompiler;
use crate::method::signature::output::OutputCompiler;
use crate::method::signature::query::QueryArgumentCompiler;
use crate::method::signature::segment::SegmentArgumentCompiler;
use crate::utils::throw_error;

const SEGMENT_MACRO: &str = "segment";
const QUERY_MACRO: &str = "query";
const HEADER_MACRO: &str = "header";
const ARGS_MACRO: &str = "args";
const BODY_MACRO: &str = "body";

#[derive(Clone, Debug)]
pub struct SignatureCompiler {
    signature: Option<Signature>,
    segments: Vec<SegmentArgumentCompiler>,
    queries: Vec<QueryArgumentCompiler>,
    headers: Vec<HeaderArgumentCompiler>,
    args: Vec<ArgsArgumentCompiler>,
    body: Option<BodyArgumentCompiler>,
    output: Option<OutputCompiler>,
    dry_run: bool,
}

#[allow(dead_code)]
impl SignatureCompiler {

    pub fn new(dry_run: bool) -> Self {
        Self {
            signature: None,
            segments: vec![],
            queries: vec![],
            headers: vec![],
            args: vec![],
            body: None,
            output: None,
            dry_run,
        }
    }

    pub fn signature(&self) -> Signature {
        self.signature.clone().expect("signature")
    }

    pub fn segments(&self) -> &Vec<SegmentArgumentCompiler> {
        &self.segments
    }

    pub fn queries(&self) -> &Vec<QueryArgumentCompiler> {
        &self.queries
    }

    pub fn headers(&self) -> &Vec<HeaderArgumentCompiler> {
        &self.headers
    }

    pub fn body(&self) -> Option<&BodyArgumentCompiler> {
        self.body.as_ref()
    }

    pub fn parse(mut signature: Signature, async_supported: bool, produces: Option<ContentType>, dry_run: bool) -> Self {
        let mut config = Self::new(dry_run);

        signature.inputs
            .iter_mut()
            .filter_map(|arg| match arg {
                FnArg::Receiver(_) => None,
                FnArg::Typed(arg_type) => Some(arg_type),
            })
            .for_each(|arg_type| config.add(arg_type));

        config.signature = Some(signature.clone());
        config.output = Some(OutputCompiler::new(signature.output, async_supported, produces, dry_run));

        config
    }

    pub fn add(&mut self, pat_type: &mut PatType) {
        let mut not_processed_attrs = Vec::new();

        pat_type.attrs.clone().into_iter()
            .map(|attr_expr| match attr_expr.meta.clone() {
            Meta::Path(value) => (value, TokenStream2::new(), attr_expr),
            Meta::List(value) => (value.path, value.tokens.to_token_stream(), attr_expr),
            Meta::NameValue(value) => (value.path, TokenStream2::new(), attr_expr),
        })
            .for_each(|(path, attrs, attr_expr)| {
                match path {
                    ref path if path.is_ident(SEGMENT_MACRO) => {
                        self.segments.push(SegmentArgumentCompiler::parse_argument(pat_type.clone(), attrs, self.dry_run));
                    },
                    ref path if path.is_ident(QUERY_MACRO) => {
                        self.queries.push(QueryArgumentCompiler::parse_argument(pat_type.clone(), attrs, self.dry_run));
                    },
                    ref path if path.is_ident(HEADER_MACRO) => {
                        self.headers.push(HeaderArgumentCompiler::parse_argument(pat_type.clone(), attrs, self.dry_run));
                    },
                    ref path if path.is_ident(ARGS_MACRO) => {
                        self.args.push(ArgsArgumentCompiler::parse(pat_type.clone()));
                    }
                    ref path if path.is_ident(BODY_MACRO) => {
                        match self.body {
                            None => self.body = Some(BodyArgumentCompiler::parse_argument(pat_type.clone())),
                            Some(_) => throw_error("multiple body arg", self.dry_run),
                        }
                    },
                    _ => {
                        not_processed_attrs.push(attr_expr);
                    }
                }
            });

        pat_type.attrs = not_processed_attrs;
    }

    pub fn compile_segments(&self, path: Option<&String>) -> TokenStream2 {
        if let Some(path) = path {
            if self.segments().is_empty() && self.args.is_empty() {
                quote!(let builder = builder.path(#path);)
            } else {
                let mut stream = TokenStream2::from(quote! {
                    let mut arguments = std::collections::HashMap::new();
                });

                for args_argument in self.args.iter() {
                    let args_segments = args_argument.compile_segments();
                    stream.extend(quote!(arguments.extend(#args_segments);));
                }

                for segment_argument in self.segments().iter() {
                    let name = segment_argument.name();
                    let value = segment_argument.value();
                    stream.extend(quote! {
                        if let Some(segment) = #value {
                            arguments.insert(#name.to_string(), segment.to_string());
                        }
                    });
                }

                stream.extend(quote! {
                    clientix::prelude::strfmt::strfmt(#path, &arguments).expect("failed to format header").as_str()
                });

                quote!(let builder = builder.path({#stream});)
            }
        } else {
            quote!()
        }
    }

    pub fn compile_headers(&self) -> TokenStream2 {
        let mut stream = TokenStream2::new();
        if self.headers.is_empty() {
            stream.extend(quote!());
        } else {
            for args_argument in self.args.iter() {
                let headers = args_argument.compile_headers();
                stream.extend(quote!(let builder = builder.headers(#headers);));
            }

            for header_argument in self.headers.iter() {
                let name = header_argument.name();
                let value = header_argument.value();
                let sensitive = header_argument.sensitive();
                stream.extend(quote! {
                    let builder = if let Some(header) = #value {
                        builder.header(#name, header.to_string().as_str(), #sensitive)
                    } else {
                        builder
                    };
                });
            }
        }

        stream
    }

    pub fn compile_queries(&self) -> TokenStream2 {
        let mut stream = TokenStream2::new();
        if self.queries.is_empty() {
            stream.extend(quote!());
        } else {
            for arg_argument in self.args.iter() {
                let queries = arg_argument.compile_queries();
                stream.extend(quote!(let builder = builder.queries(#queries);));
            }

            for query_argument in self.queries.iter() {
                let name = query_argument.name();
                let value = query_argument.value();
                stream.extend(quote! {
                    let builder = if let Some(query) = #value {
                        builder.query(#name, query.to_string().as_str())
                    } else {
                        builder
                    };
                });
            }
        }

        stream
    }

    pub fn compile_body(&self, consumes: Option<ContentType>) -> TokenStream2 {
        let content_type: String = match consumes {
            Some(value) => value.to_string(),
            None => ContentType::ApplicationJson.to_string()
        };

        for argument in self.args.iter() {
            let body = argument.compile_body();
            return quote! {
                let builder = if let Some(body) = #body {
                    builder.body(body, #content_type.to_string().try_into().unwrap())
                } else {
                    builder
                };
            }
        }

        if let Some(body_argument) = &self.body {
            let body = body_argument.value();
            return quote! {
                let builder = if let Some(body) = #body {
                    builder.body(body, #content_type.to_string().try_into().unwrap())
                } else {
                    builder
                };
            }
        }

        quote! {}
    }

    pub fn compile_output(&self) -> TokenStream2 {
        if let Some(output) = &self.output {
            let output = output.compile();

            quote! {
                builder.send()
                    #output
            }
        } else {
            quote!(builder.send();)
        }
    }

}