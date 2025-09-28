pub mod segment;
pub mod query;
pub mod body;
pub mod header;
pub mod args;
pub mod output;

use quote::{quote, ToTokens};
use syn::{FnArg, Meta, PatType, Signature};
use syn::__private::TokenStream2;
use clientix_core::core::headers::content_type::ContentType;
use crate::method::signature::args::ArgsArgumentCompiler;
use crate::method::signature::body::BodyArgumentCompiler;
use crate::method::signature::header::HeaderArgumentCompiler;
use crate::method::signature::output::OutputCompiler;
use crate::method::signature::query::QueryArgumentCompiler;
use crate::method::signature::segment::SegmentArgumentCompiler;
use crate::utils::throw_error;

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

        pat_type.attrs.clone().into_iter().map(|attr_expr| match attr_expr.meta.clone() {
            Meta::Path(value) => (value, TokenStream2::new(), attr_expr),
            Meta::List(value) => (value.path, value.tokens.to_token_stream(), attr_expr),
            Meta::NameValue(value) => (value.path, TokenStream2::new(), attr_expr),
        }).for_each(|(path, attrs, attr_expr)| {
            match path {
                ref path if path.is_ident("segment") => {
                    self.segments.push(SegmentArgumentCompiler::parse(pat_type.pat.to_token_stream(), attrs, self.dry_run));
                },
                ref path if path.is_ident("query") => {
                    self.queries.push(QueryArgumentCompiler::parse(pat_type.pat.to_token_stream(), attrs, self.dry_run));
                },
                ref path if path.is_ident("header") => {
                    self.headers.push(HeaderArgumentCompiler::parse(pat_type.pat.to_token_stream(), attrs, self.dry_run));
                },
                ref path if path.is_ident("args") => {
                    self.args.push(ArgsArgumentCompiler::parse(pat_type.pat.to_token_stream()));
                }
                ref path if path.is_ident("body") => {
                    match self.body {
                        None => self.body = Some(BodyArgumentCompiler::parse(pat_type.pat.to_token_stream())),
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
                quote!(.path(#path))
            } else {
                let mut stream = TokenStream2::from(quote! {
                    let mut arguments = std::collections::HashMap::new();
                });

                for args_variable in self.args.iter() {
                    let args_segments = args_variable.compile_segments();
                    stream.extend(quote!(arguments.extend(#args_segments);));
                }

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
                stream.extend(header_variable.compile());
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

    pub fn compile_output(&self) -> TokenStream2 {
        if let Some(output) = &self.output {
            output.compile()
        } else {
            quote!()
        }
    }

}