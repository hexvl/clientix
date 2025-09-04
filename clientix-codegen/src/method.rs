use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, FnArg, LitStr, Meta, Signature, TraitItemFn};
use syn::__private::TokenStream2;
use syn::parse::Parser;
use clientix_core::core::headers::content_type::ContentType;
use clientix_core::prelude::reqwest::Method;
use crate::return_kind::ReturnKind;

#[derive(Clone)]
pub struct MethodArgumentsConfig {
    request_segment_args: Vec<Box<syn::Pat>>,
    request_query_args: Vec<Box<syn::Pat>>,
    request_header_args: Vec<Box<syn::Pat>>,
    request_body_arg: Option<Box<syn::Pat>>,
}

#[derive(Clone)]
pub struct MethodConfig {
    item: Option<TraitItemFn>,
    attributes: Vec<Attribute>,
    signature: Option<Signature>,
    method: Option<Method>,
    path: Option<String>,
    consumes: ContentType,
    produces: ContentType,
    async_supported: bool,
    postprocessing: bool,
    arguments_config: MethodArgumentsConfig,
}

impl From<TraitItemFn> for MethodConfig {

    fn from(item: TraitItemFn) -> Self {
        let mut method_attrs = MethodConfig {
            item: Some(item.clone()),
            attributes: item.clone().attrs,
            signature: Some(item.clone().sig),
            method: None,
            path: None,
            consumes: ContentType::ApplicationJson,
            produces: ContentType::ApplicationJson,
            async_supported: false,
            postprocessing: false,
            arguments_config: MethodArgumentsConfig {
                request_segment_args: vec![],
                request_query_args: vec![],
                request_header_args: vec![],
                request_body_arg: None
            }
        };

        for attr_expr in method_attrs.attributes.clone().iter() {
            match &attr_expr.meta {
                Meta::Path(value) => {
                    let method: Method = value.get_ident()
                        .map(|value| value.to_string().to_uppercase())
                        .map(|value| value.as_str().try_into().expect(format!("invalid method: {value}").as_str()))
                        .expect("unexpected ident");

                    method_attrs.parse_attrs(method.into(), TokenStream2::new());
                    method_attrs.parse_args(item.clone());
                }
                Meta::List(value) => {
                    let method: Method = value.path.get_ident()
                        .map(|value| value.to_string().to_uppercase())
                        .map(|value| value.as_str().try_into().expect(format!("invalid method: {value}").as_str()))
                        .expect("unexpected ident");
                    
                    let attrs: TokenStream2 = value.tokens.to_token_stream();

                    method_attrs.parse_attrs(method.into(), attrs);
                    method_attrs.parse_args(item.clone());
                }
                _ => {
                    panic!("unexpected attribute");
                }
            }
        }

        method_attrs
    }

}

impl MethodConfig {

    pub fn create(method: Method, item: TokenStream, attrs: TokenStream) -> Self {
        let mut method_config = MethodConfig {
            item: None,
            attributes: vec![],
            signature: None,
            method: None,
            path: None,
            consumes: ContentType::ApplicationJson,
            produces: ContentType::ApplicationJson,
            async_supported: false,
            postprocessing: true,
            arguments_config: MethodArgumentsConfig {
                request_segment_args: vec![],
                request_query_args: vec![],
                request_header_args: vec![],
                request_body_arg: None
            }
        };

        method_config.parse(method, TokenStream2::from(item), TokenStream2::from(attrs));

        method_config
    }

    pub fn set_async_supported(&mut self, async_supported: bool) {
        self.async_supported = async_supported;
    }

    pub fn compile_declaration(&self) -> TokenStream2 {
        let attrs = self.get_attributes();
        let signature = self.get_signature();

        if self.postprocessing {
            quote! {
                #[allow(async_fn_in_trait)]
                #(#attrs)*
                #signature;
            }
        } else {
            let item = self.item.clone().expect("unexpected method");
            quote! {#item}
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
            Some(Method::OPTIONS) => quote! {.options()},
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
        if self.arguments_config.request_header_args.is_empty() {
            quote! {}
        } else {
            let mut stream = TokenStream2::new();
            for header_variable in self.arguments_config.request_header_args.iter() {
                let header_id = format!("{}", quote! {#header_variable});
                stream.extend(quote! {
                    .header(#header_id, #header_variable)
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
            quote! {
                .body(#body_variable, clientix::core::headers::content_type::ContentType::ApplicationJson)
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
        let compiled_json_response = quote! {
            #compiled_async_directive
            .json()
            #compiled_async_directive
        };
        
        match ReturnKind::from(self.get_signature()) {
            ReturnKind::Unit => quote! {;},
            ReturnKind::ClientixResultOfResponseOfString => compiled_text_response,
            ReturnKind::ClientixResultOfResponse => compiled_json_response,
            ReturnKind::ClientixResultOfStreamOfString => {
                if self.async_supported {
                    quote! {
                        .await
                        .text_stream()
                    }
                } else {
                    self.throw_error("Streams not supported for not async clients");
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
                    self.throw_error("Streams not supported for not async clients");
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
                    #compiled_json_response
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
                    #compiled_json_response
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
                    self.throw_error("Streams not supported for not async clients");
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
                    self.throw_error("Streams not supported for not async clients");
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
                    #compiled_json_response
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
                    self.throw_error("Streams not supported for not async clients");
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
                    self.throw_error("Streams not supported for not async clients");
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
                    #compiled_json_response
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
                    #compiled_json_response
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

    fn parse(&mut self, method: Method, item: TokenStream2, attrs: TokenStream2) {
        self.parse_item(item);
        self.parse_attrs(method, attrs);
    }

    fn parse_item(&mut self, item: TokenStream2) {

        let item: TraitItemFn = match syn::parse2(item) {
            Ok(item) => item,
            Err(err) => panic!("{}", err.to_string())
        };

        self.parse_args(item);
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
                        ref attr if attr.is_ident("body") => {
                            match self.arguments_config.request_body_arg {
                                None => self.arguments_config.request_body_arg = Some(arg_type.pat.clone()),
                                Some(_) => self.throw_error("multiple body arg")
                            }
                        }
                        _ => attrs.push(attr.clone()),
                    }
                });

                arg_type.attrs = attrs;
            });

        self.signature = Some(item.sig);
    }

    fn parse_attrs(&mut self, method: Method, attrs: TokenStream2) {
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
                            self.throw_error("invalid content-type for consumes");
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
                            self.throw_error("invalid content-type for produces");
                        }
                    }

                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected method param: {}", meta.path.get_ident().unwrap())))
            }
        });

        match parser.parse2(attrs) {
            Ok(_) => (),
            Err(error) => self.throw_error(error.to_string().as_str()),
        }

        self.method = Some(method);
    }

    fn get_attributes(&self) -> Vec<Attribute> {
        self.attributes.clone()
    }

    fn get_signature(&self) -> Signature {
        self.signature.clone().expect("missing method signature")
    }

    fn throw_error(&self, message: &str) {
        if self.postprocessing {
            panic!("{}", message);
        } else {
            eprintln!("{}", message);
        }
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