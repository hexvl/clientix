use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, FnArg, LitStr, Meta, Signature, TraitItemFn};
use syn::__private::TokenStream2;
use syn::parse::Parser;
use clientix_core::core::headers::content_type::ContentType;
use clientix_core::prelude::reqwest::Method;
use crate::header::HeaderConfig;
use crate::return_kind::ReturnKind;
use crate::utils::throw_error;

const GET_METHOD_MACRO: &str = "get";
const POST_METHOD_MACRO: &str = "post";
const PUT_METHOD_MACRO: &str = "put";
const DELETE_METHOD_MACRO: &str = "delete";
const HEAD_METHOD_MACRO: &str = "head";
const PATCH_METHOD_MACRO: &str = "patch";
const HEADER_METHOD_MACRO: &str = "header";

#[derive(Clone)]
pub struct MethodArgumentsConfig {
    request_segment_args: Vec<Box<syn::Pat>>,
    request_query_args: Vec<Box<syn::Pat>>,
    request_header_args: Vec<Box<syn::Pat>>,
    request_placeholder_arg: Vec<Box<syn::Pat>>,
    request_body_arg: Option<Box<syn::Pat>>,
}

#[derive(Clone)]
pub struct MethodConfig {
    attributes: Vec<Attribute>,
    signature: Option<Signature>,
    method: Option<Method>,
    path: Option<String>,
    consumes: ContentType,
    produces: ContentType,
    headers: Vec<HeaderConfig>,
    async_supported: bool,
    dry_run: bool,
    arguments_config: MethodArgumentsConfig,
}

impl From<TraitItemFn> for MethodConfig {

    fn from(item: TraitItemFn) -> Self {
        let mut method_attrs = MethodConfig {
            attributes: Vec::new(),
            signature: None,
            method: None,
            path: None,
            consumes: ContentType::ApplicationJson,
            produces: ContentType::ApplicationJson,
            headers: Default::default(),
            async_supported: false,
            dry_run: false,
            arguments_config: MethodArgumentsConfig {
                request_segment_args: vec![],
                request_query_args: vec![],
                request_header_args: vec![],
                request_placeholder_arg: vec![],
                request_body_arg: None
            }
        };

        let attributes = item.attrs.clone();
        method_attrs.parse_macros(HEADER_METHOD_MACRO, &attributes);
        method_attrs.parse_macros(GET_METHOD_MACRO, &attributes);
        method_attrs.parse_macros(POST_METHOD_MACRO, &attributes);
        method_attrs.parse_macros(PUT_METHOD_MACRO, &attributes);
        method_attrs.parse_macros(DELETE_METHOD_MACRO, &attributes);
        method_attrs.parse_macros(HEAD_METHOD_MACRO, &attributes);
        method_attrs.parse_macros(PATCH_METHOD_MACRO, &attributes);

        method_attrs.parse_args(item);
        method_attrs
    }

}

impl MethodConfig {

    pub fn create(method: Method, item: TokenStream, attrs: TokenStream) -> Self {
        let mut method_config = MethodConfig {
            attributes: Vec::new(),
            signature: None,
            method: None,
            path: None,
            consumes: ContentType::ApplicationJson,
            produces: ContentType::ApplicationJson,
            headers: Default::default(),
            async_supported: false,
            dry_run: true,
            arguments_config: MethodArgumentsConfig {
                request_segment_args: vec![],
                request_query_args: vec![],
                request_header_args: vec![],
                request_placeholder_arg: vec![],
                request_body_arg: None
            }
        };

        method_config.parse_stream(method, TokenStream2::from(item), TokenStream2::from(attrs));

        method_config
    }

    pub fn set_async_supported(&mut self, async_supported: bool) {
        self.async_supported = async_supported;
    }

    pub fn compile_declaration(&self) -> TokenStream2 {
        let attributes = self.get_attributes();
        let signature = self.get_signature();

        if self.dry_run {
            quote! {
                #signature;
            }
        } else {
            quote! {
                #(#attributes)*
                #[allow(async_fn_in_trait)]
                #signature;
            }
        }
    }

    pub fn compile_definition(&self) -> TokenStream2 {
        let sig = self.get_signature();

        let method_path = self.path.clone().unwrap_or(String::new());
        let compiled_segments = self.compile_segments();
        let compiled_headers = self.compile_headers();
        let compiled_queries = self.compile_queries();
        let compiled_body = self.compile_body();
        let compiled_result = self.compile_result();
        let compiled_method = self.compile_method();

        quote! {
            pub #sig {
                use clientix::client::request::ClientixRequestBuilder;
                
                self.client
                    #compiled_method
                    .path(#method_path)
                    #compiled_segments
                    #compiled_headers
                    #compiled_queries
                    #compiled_body
                    .send()
                    #compiled_result
            }
        }
    }

    fn compile_method(&self) -> TokenStream2 {
        TokenStream2::from(match self.method {
            Some(Method::GET) => quote! {.get()},
            Some(Method::POST) => quote! {.post()},
            Some(Method::PUT) => quote! {.put()},
            Some(Method::DELETE) => quote! {.delete()},
            Some(Method::HEAD) => quote! {.head()},
            Some(Method::PATCH) => quote! {.patch()},
            _ => panic!("missing method type")
        })
    }

    fn compile_segments(&self) -> TokenStream2 {
        if self.arguments_config.request_segment_args.is_empty() {
            quote! {}
        } else {
            let mut stream = TokenStream2::new();
            for segment_variable in self.arguments_config.request_segment_args.iter() {
                let segment_id = format!("{}", quote! {#segment_variable});
                stream.extend(quote! {
                    .path_segment(#segment_id, #segment_variable)
                });
            }

            stream
        }
    }

    fn compile_headers(&self) -> TokenStream2 {
        if self.arguments_config.request_header_args.is_empty() && self.headers.is_empty() {
            quote! {}
        } else {
            let mut stream = TokenStream2::new();
            for header_variable in self.arguments_config.request_header_args.iter() {
                let header_id = format!("{}", quote! {#header_variable});
                stream.extend(quote! {
                    .header(#header_id, #header_variable)
                })
            }

            for header in self.headers.iter() {
                let compiled_header = header.compile_header(self.arguments_config.request_placeholder_arg.clone());
                stream.extend(quote! {
                    #compiled_header
                })
            }

            stream
        }
    }

    fn compile_queries(&self) -> TokenStream2 {
        if self.arguments_config.request_query_args.is_empty() {
            quote! {}
        } else {
            let mut stream = TokenStream2::new();
            for query_variable in self.arguments_config.request_query_args.iter() {
                let query_id = format!("{}", quote! {#query_variable});
                stream.extend(quote! {
                    .query(#query_id, #query_variable)
                })
            }

            stream
        }
    }

    fn compile_body(&self) -> TokenStream2 {
        if let Some(body_variable) = &self.arguments_config.request_body_arg {
            let content_type: String = self.consumes.into();
            quote! {
                .body(#body_variable, #content_type.to_string().try_into().unwrap())
            }
        } else {
            quote! {}
        }
    }

    fn compile_result(&self) -> TokenStream2 {
        let compiled_async_directive = self.compile_async();
        let compiled_text_response = quote! {
            #compiled_async_directive
            .text()
            #compiled_async_directive
        };
        let compiled_object_method = match self.produces {
            ContentType::ApplicationJson => quote!{.json()},
            ContentType::ApplicationXml => quote!{.xml()},
            ContentType::ApplicationXWwwFormUrlEncoded => quote!{.urlencoded()},
            ContentType::TextHtml => quote!{.text()}
        };

        let compiled_object_response = quote! {
            #compiled_async_directive
            #compiled_object_method
            #compiled_async_directive
        };
        
        match ReturnKind::from(self.get_signature()) {
            ReturnKind::Unit => quote! {;},
            ReturnKind::ClientixResultOfResponseOfString => compiled_text_response,
            ReturnKind::ClientixResultOfResponse => compiled_object_response,
            ReturnKind::ClientixResultOfStreamOfString => {
                if self.async_supported {
                    quote! {
                        .await
                        .text_stream()
                    }
                } else {
                    throw_error("Streams not supported for not async clients", self.dry_run);
                    quote!()
                }
            },
            ReturnKind::ClientixResultOfStream => {
                if self.async_supported {
                    quote! {
                        .await
                        .json_stream()
                    }
                } else {
                    throw_error("Streams not supported for not async clients", self.dry_run);
                    quote!()
                }
            },
            ReturnKind::ClientixResultOfString => {
                quote! {
                    #compiled_text_response
                    .map(|response| response.body())
                }
            }
            ReturnKind::ClientixResult => {
                quote! {
                    #compiled_object_response
                    .map(|response| response.body())
                }
            }
            ReturnKind::OptionOfResponseOfString => {
                quote! {
                    #compiled_text_response
                    .ok()
                }
            }
            ReturnKind::OptionOfResponse => {
                quote! {
                    #compiled_object_response
                    .ok()
                }
            }
            ReturnKind::OptionOfStreamOfString => {
                if self.async_supported {
                    quote! {
                        .await
                        .text_stream()
                        .ok()
                    }
                } else {
                    throw_error("Streams not supported for not async clients", self.dry_run);
                    quote!()
                }
            }
            ReturnKind::OptionOfStream => {
                if self.async_supported {
                    quote! {
                        .await
                        .json_stream()
                        .ok()
                    }
                } else {
                    throw_error("Streams not supported for not async clients", self.dry_run);
                    quote!()
                }
            }
            ReturnKind::OptionOfString => {
                quote! {
                    #compiled_text_response
                    .map(|response| response.body())
                    .ok()
                }
            }
            ReturnKind::Option => {
                quote! {
                    #compiled_object_response
                    .map(|response| response.body())
                    .ok()
                }
            }
            ReturnKind::ClientixStreamOfString => {
                if self.async_supported {
                    quote! {
                        .await
                        .text_stream()
                        .unwrap()
                    }
                } else {
                    throw_error("Streams not supported for not async clients", self.dry_run);
                    quote!()
                }
            }
            ReturnKind::ClientixStream => {
                if self.async_supported {
                    quote! {
                        .await
                        .json_stream()
                        .unwrap()
                    }
                } else {
                    throw_error("Streams not supported for not async clients", self.dry_run);
                    quote!()
                }
            }
            ReturnKind::ClientixResponseOfString => {
                quote! {
                    #compiled_text_response
                    .unwrap()
                }
            }
            ReturnKind::ClientixResponse => {
                quote! {
                    #compiled_object_response
                    .unwrap()
                }
            }
            ReturnKind::String => {
                quote! {
                    #compiled_text_response
                    .map(|response| response.body())
                    .unwrap()
                }
            }
            ReturnKind::Other => {
                quote! {
                    #compiled_object_response
                    .map(|response| response.body())
                    .unwrap()
                }
            }
        }
    }

    fn compile_async(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {.await}
        } else {
            quote! {}
        }
    }

    fn parse_stream(&mut self, method: Method, item: TokenStream2, attrs: TokenStream2) {
        self.parse_item(method, item, attrs);
    }

    fn parse_item(&mut self, method: Method, item: TokenStream2, attrs: TokenStream2) {
        match syn::parse2(item) {
            Ok(item) => {
                self.parse_attrs(method.to_string().to_lowercase(), attrs);
                self.parse_args(item);
            },
            Err(err) => throw_error(format!("{}", err.to_string()).as_str(), self.dry_run)
        };
    }

    fn parse_macros(&mut self, macro_name: &str, attributes: &Vec<Attribute>) {
        attributes.iter()
            .map(|attr_expr| match &attr_expr.meta {
                Meta::Path(value) => (value, TokenStream2::new(), attr_expr),
                Meta::List(value) => (&value.path, value.tokens.to_token_stream(), attr_expr),
                Meta::NameValue(_) => panic!("unexpected attribute syntax"),
            })
            .filter(|(path, _, _)| path.is_ident(macro_name))
            .for_each(|(_, tokens, attr_expr)| {
                self.parse_attrs(macro_name.to_string(), tokens);
                self.attributes.push(attr_expr.clone());
            });
    }

    fn parse_attrs(&mut self, ident: String, attrs: TokenStream2) {
        match ident.as_str() {
            HEADER_METHOD_MACRO => self.parse_header_attrs(attrs),
            GET_METHOD_MACRO => self.parse_method_attrs(Method::GET, attrs),
            POST_METHOD_MACRO => self.parse_method_attrs(Method::POST, attrs),
            PUT_METHOD_MACRO => self.parse_method_attrs(Method::PUT, attrs),
            DELETE_METHOD_MACRO => self.parse_method_attrs(Method::DELETE, attrs),
            HEAD_METHOD_MACRO => self.parse_method_attrs(Method::HEAD, attrs),
            PATCH_METHOD_MACRO => self.parse_method_attrs(Method::PATCH, attrs),
            _ => throw_error("not valid macro", self.dry_run),
        };
    }

    fn parse_args(&mut self, mut item: TraitItemFn) {
        item.sig.inputs
            .iter_mut()
            .filter_map(|arg| match arg {
                FnArg::Receiver(_) => None,
                FnArg::Typed(arg_type) => Some(arg_type),
            })
            .for_each(|arg_type| {
                let mut attrs = Vec::new();
                arg_type.attrs.iter().for_each(|attr| {
                    match attr.path() {
                        ref attr if attr.is_ident("segment") => {
                            // TODO: очень не гибко, добавить возможность задавать алиас для сегмента пути
                            self.arguments_config.request_segment_args.push(arg_type.pat.clone());
                        },
                        ref attr if attr.is_ident("query") => {
                            // TODO: очень не гибко, добавить возможность задавать алиас для параметра запроса
                            self.arguments_config.request_query_args.push(arg_type.pat.clone());
                        }
                        ref attr if attr.is_ident("header") => {
                            // TODO: сделать отдельное определение для авторизационных заголовков по типу
                            // TODO: очень не гибко, добавить возможность задавать алиас для заголовка
                            self.arguments_config.request_header_args.push(arg_type.pat.clone());
                        }
                        ref attr if attr.is_ident("placeholder") => {
                            self.arguments_config.request_placeholder_arg.push(arg_type.pat.clone());
                        }
                        ref attr if attr.is_ident("body") => {
                            match self.arguments_config.request_body_arg {
                                None => self.arguments_config.request_body_arg = Some(arg_type.pat.clone()),
                                Some(_) => throw_error("multiple body arg", self.dry_run)
                            }
                        }
                        _ => attrs.push(attr.clone()),
                    }
                });

                arg_type.attrs = attrs;
            });

        self.signature = Some(item.sig.clone());
    }

    fn parse_method_attrs(&mut self, method: Method, attrs: TokenStream2) {
        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident("path") => {
                    self.path = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident("consumes") => {
                    match meta.value()?.parse::<LitStr>()?.value().try_into() {
                        Ok(consumes) => {
                            self.consumes = consumes;
                        }
                        Err(_) => {
                            throw_error("invalid content-type for consumes", self.dry_run);
                        }
                    };

                    Ok(())
                }
                ref path if path.is_ident("produces") => {
                    match meta.value()?.parse::<LitStr>()?.value().try_into() {
                        Ok(produces) => {
                            self.produces = produces;
                        }
                        Err(_) => {
                            throw_error("invalid content-type for produces", self.dry_run);
                        }
                    }

                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected method param: {}", meta.path.get_ident().unwrap())))
            }
        });

        match parser.parse2(attrs) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), self.dry_run),
        }

        self.method = Some(method);
    }

    fn parse_header_attrs(&mut self, attrs: TokenStream2) {
        let header_config = HeaderConfig::parse(attrs, false);
        self.headers.push(header_config);
    }

    fn get_attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    fn get_signature(&self) -> Signature {
        self.signature.clone().expect("missing method signature")
    }

}

pub fn parse_method(method: Method, item: TokenStream, attrs: TokenStream) -> TokenStream {
    let method_config = MethodConfig::create(method, item, attrs);

    let compiled_declaration = method_config.compile_declaration();

    let expanded = quote! {
        #compiled_declaration
    };

    TokenStream::from(expanded)
}