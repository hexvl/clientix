use quote::{quote, ToTokens};
use syn::{DeriveInput, Field, Ident, Meta};
use syn::__private::TokenStream2;
use crate::method::signature::body::BodyArgumentCompiler;
use crate::method::signature::header::HeaderArgumentCompiler;
use crate::method::signature::query::QueryArgumentCompiler;
use crate::method::signature::segment::SegmentArgumentCompiler;
use crate::utils::throw_error;

const SEGMENT_MACRO: &str = "segment";
const QUERY_MACRO: &str = "query";
const HEADER_MACRO: &str = "header";
const BODY_MACRO: &str = "body";

#[derive(Clone, Debug)]
pub struct RequestArgsCompiler {
    ident: Ident,
    segments: Vec<SegmentArgumentCompiler>,
    queries: Vec<QueryArgumentCompiler>,
    headers: Vec<HeaderArgumentCompiler>,
    body: Option<BodyArgumentCompiler>,
    dry_run: bool,
}

impl RequestArgsCompiler {

    fn new(ident: Ident, dry_run: bool) -> Self {
        Self {
            ident,
            segments: vec![],
            queries: vec![],
            headers: vec![],
            body: None,
            dry_run,
        }
    }

    pub fn parse(derive_input: DeriveInput) -> Self {
        let mut compiler = RequestArgsCompiler::new(derive_input.ident, true);

        match derive_input.data {
            syn::Data::Struct(data) => {
                data.fields.into_iter().for_each(|field| compiler.add(field));
            },
            _ => {}
        }

        compiler
    }

    pub fn compile(&self) -> TokenStream2 {
        let ident = &self.ident;
        let compiled_segments_method = self.compile_segments_method();
        let compiled_queries_method = self.compile_queries_method();
        let compiled_headers_method = self.compile_headers_method();
        let compiled_body_method = self.compile_body_method();

        quote! {
            impl #ident {
                #compiled_segments_method
                #compiled_queries_method
                #compiled_headers_method
                #compiled_body_method
            }
        }
    }

    fn add(&mut self, field: Field) {
        field.attrs.clone().into_iter().map(|attr_expr| match attr_expr.meta.clone() {
            Meta::Path(value) => (value, TokenStream2::new(), attr_expr),
            Meta::List(value) => (value.path, value.tokens.to_token_stream(), attr_expr),
            Meta::NameValue(value) => (value.path, TokenStream2::new(), attr_expr),
        }).for_each(|(path, attrs, _)| {
            let ident = field.ident.clone().unwrap();
            let ty = field.ty.clone();
            
            match path {
                ref path if path.is_ident(SEGMENT_MACRO) => {
                    self.segments.push(SegmentArgumentCompiler::parse(ident, ty, attrs, self.dry_run));
                },
                ref path if path.is_ident(QUERY_MACRO) => {
                    self.queries.push(QueryArgumentCompiler::parse(ident, ty, attrs, self.dry_run));
                },
                ref path if path.is_ident(HEADER_MACRO) => {
                    self.headers.push(HeaderArgumentCompiler::parse(ident, ty, attrs, self.dry_run));
                },
                ref path if path.is_ident(BODY_MACRO) => {
                    match self.body {
                        None => self.body = Some(BodyArgumentCompiler::parse(ident, ty)),
                        Some(_) => throw_error("multiple body arg", self.dry_run),
                    }
                },
                _ => {}
            }
        });
    }

    fn compile_segments_method(&self) -> TokenStream2 {
        let mut segments_stream = quote!(let mut arguments = std::collections::HashMap::new(););
        for segment_arg in &self.segments {
            let name = segment_arg.name();
            let value = segment_arg.value();
            segments_stream.extend(quote!(arguments.insert(#name.to_string(), self.#value.to_string());));
        }
        segments_stream.extend(quote!(arguments));

        quote! {
            pub fn segments(&self) -> std::collections::HashMap<String, String> {
                #segments_stream
            }
        }
    }

    fn compile_queries_method(&self) -> TokenStream2 {
        let mut queries_stream = quote!(let mut arguments = std::collections::HashMap::new(););
        for query_arg in &self.queries {
            let name = query_arg.name();
            let value = query_arg.value();
            queries_stream.extend(quote!(arguments.insert(#name.to_string(), self.#value.to_string());));
        }
        queries_stream.extend(quote!(arguments));

        quote! {
            pub fn queries(&self) -> std::collections::HashMap<String, String> {
                #queries_stream
            }
        }
    }

    fn compile_headers_method(&self) -> TokenStream2 {
        let mut headers_stream = quote!(let mut arguments = std::collections::HashMap::new(););
        for header_arg in &self.headers {
            let name = header_arg.name();
            let value = header_arg.value();
            headers_stream.extend(quote!(arguments.insert(#name.to_string(), self.#value.to_string());));
        }
        headers_stream.extend(quote!(arguments));

        quote! {
            pub fn headers(&self) -> std::collections::HashMap<String, String> {
                #headers_stream
            }
        }
    }

    pub fn compile_body_method(&self) -> TokenStream2 {
        if let Some(body_arg) = &self.body {
            let ty = body_arg.ty();
            let value = body_arg.value();

            quote! {
                pub fn body(self) -> Option<#ty> {
                    Some(self.#value)
                }
            }
        } else {
            quote! {
                pub fn body<T>(self) -> Option<T> {
                    None
                }
            }
        }
    }

}

