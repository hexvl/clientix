use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, Meta, Signature, TraitItem};
use syn::__private::TokenStream2;
use clientix_core::prelude::reqwest::header::{ACCEPT, CONTENT_TYPE};
use clientix_core::prelude::reqwest::Method;
use crate::attributes::method::MethodAttributes;
use crate::method::header::HeaderCompiler;
use crate::method::signature::SignatureCompiler;

const GET_METHOD_MACRO: &str = "get";
const POST_METHOD_MACRO: &str = "post";
const PUT_METHOD_MACRO: &str = "put";
const DELETE_METHOD_MACRO: &str = "delete";
const HEAD_METHOD_MACRO: &str = "head";
const PATCH_METHOD_MACRO: &str = "patch";
const HEADER_METHOD_MACRO: &str = "header";


#[derive(Clone, Debug)]
pub struct MethodCompiler {
    macros: Vec<Attribute>,
    headers: Vec<HeaderCompiler>,
    signature: SignatureCompiler,
    attributes: MethodAttributes,
    dry_run: bool
}

impl MethodCompiler {

    pub fn new(signature: Signature, attributes: Vec<Attribute>, async_supported: bool) -> Self {
        let mut macros = Vec::new();
        let mut headers = Vec::new();
        let mut method_attributes = None;

        attributes.iter()
            .map(|attr_expr| match &attr_expr.meta {
                Meta::Path(value) => (value, TokenStream2::new(), attr_expr),
                Meta::List(value) => (&value.path, value.tokens.to_token_stream(), attr_expr),
                Meta::NameValue(value) => (&value.path, TokenStream2::new(), attr_expr),
            })
            .for_each(|(path, attrs, attr_expr)| {
                match path {
                    ref path if path.is_ident(HEADER_METHOD_MACRO) => {
                        headers.push(HeaderCompiler::parse(attrs, false));
                        macros.insert(0, attr_expr.clone());
                    },
                    ref path if path.is_ident(GET_METHOD_MACRO) => {
                        method_attributes = Some(MethodAttributes::parse(Method::GET, attrs, false));
                        macros.push(attr_expr.clone());
                    }
                    ref path if path.is_ident(POST_METHOD_MACRO) => {
                        method_attributes = Some(MethodAttributes::parse(Method::POST, attrs, false));
                        macros.push(attr_expr.clone());
                    }
                    ref path if path.is_ident(PUT_METHOD_MACRO) => {
                        method_attributes = Some(MethodAttributes::parse(Method::PUT, attrs, false));
                        macros.push(attr_expr.clone());
                    },
                    ref path if path.is_ident(DELETE_METHOD_MACRO) => {
                        method_attributes = Some(MethodAttributes::parse(Method::DELETE, attrs, false));
                        macros.push(attr_expr.clone());
                    },
                    ref path if path.is_ident(HEAD_METHOD_MACRO) => {
                        method_attributes = Some(MethodAttributes::parse(Method::HEAD, attrs, false));
                        macros.push(attr_expr.clone());
                    },
                    ref path if path.is_ident(PATCH_METHOD_MACRO) => {
                        method_attributes = Some(MethodAttributes::parse(Method::PATCH, attrs, false));
                        macros.push(attr_expr.clone());
                    },
                    _ => {},
                };
            });

        if let Some(attributes) = method_attributes {
            let signature = SignatureCompiler::parse(signature, async_supported, attributes.produces().cloned(), false);

            Self {
                macros,
                headers,
                signature,
                attributes,
                dry_run: false
            }
        } else {
            panic!("not valid macro");
        }
    }

    pub fn parse(method: Method, item: TokenStream, attrs: TokenStream) -> Self {
        match syn::parse2(TokenStream2::from(item)) {
            Ok(TraitItem::Fn(item)) => {
                let attributes = MethodAttributes::parse(method, TokenStream2::from(attrs), true);
                let signature = SignatureCompiler::parse(item.sig, false, attributes.produces().cloned(), true);

                Self {
                    macros: vec![],
                    headers: vec![],
                    signature,
                    attributes,
                    dry_run: false,
                }
            }
            Ok(_) => panic!("unsupported item"),
            Err(err) => panic!("{}", err.to_string().as_str()),
        }
    }

    pub fn compile_declaration(&self) -> TokenStream2 {
        let attributes = &self.macros;
        let signature =  self.signature.signature();

        if self.dry_run {
            quote!(#signature;)
        } else {
            quote! {
                #(#attributes)*
                #[allow(async_fn_in_trait)]
                #signature;
            }
        }
    }

    pub fn compile_definition(&self) -> TokenStream2 {
        let sig = self.signature.signature();

        let compiled_method = self.compile_method();
        let compiled_path = self.compile_path();
        let compiled_headers = self.compile_headers();
        let compiled_queries = self.compile_queries();
        let compiled_body = self.compile_body();
        let compiled_result = self.compile_output();

        quote! {
            pub #sig {
                use clientix::client::request::ClientixRequestBuilder;

                self.client
                    #compiled_method
                    #compiled_path
                    #compiled_headers
                    #compiled_queries
                    #compiled_body
                    .send()
                    #compiled_result
            }
        }
    }

    fn compile_method(&self) -> TokenStream2 {
        TokenStream2::from(match *self.attributes.method() {
            Method::GET => quote! {.get()},
            Method::POST => quote! {.post()},
            Method::PUT => quote! {.put()},
            Method::DELETE => quote! {.delete()},
            Method::HEAD => quote! {.head()},
            Method::PATCH => quote! {.patch()},
            _ => panic!("missing method type")
        })
    }

    fn compile_path(&self) -> TokenStream2 {
        self.signature.compile_segments(self.attributes.path())
    }

    fn compile_headers(&self) -> TokenStream2 {
        let mut stream = self.signature.compile_headers();

        for header in &self.headers {
            stream.extend(header.compile(self.signature.segments()))
        }

        if let Some(content_type) = self.attributes.consumes() {
            stream.extend(HeaderCompiler::new(Some(CONTENT_TYPE.to_string()), Some(content_type.to_string()), false).compile(self.signature.segments()));
        }

        if let Some(accept_type) = self.attributes.produces() {
            stream.extend(HeaderCompiler::new(Some(ACCEPT.to_string()), Some(accept_type.to_string()), false).compile(self.signature.segments()));
        }

        stream
    }

    fn compile_queries(&self) -> TokenStream2 {
        self.signature.compile_queries()
    }

    fn compile_body(&self) -> TokenStream2 {
        self.signature.compile_body(self.attributes.consumes().cloned())
    }

    fn compile_output(&self) -> TokenStream2 {
        self.signature.compile_output()
    }

}