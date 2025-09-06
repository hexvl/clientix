use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, FnArg, LitStr, Meta, Signature, TraitItemFn};
use syn::__private::TokenStream2;
use syn::parse::Parser;
use clientix_core::core::headers::content_type::ContentType;
use clientix_core::prelude::reqwest::header::{ACCEPT, CONTENT_TYPE};
use clientix_core::prelude::reqwest::Method;
use crate::method::arguments::ArgumentsConfig;
use crate::method::header::HeaderConfig;
use crate::method::output::OutputConfig;
use crate::utils::throw_error;

const GET_METHOD_MACRO: &str = "get";
const POST_METHOD_MACRO: &str = "post";
const PUT_METHOD_MACRO: &str = "put";
const DELETE_METHOD_MACRO: &str = "delete";
const HEAD_METHOD_MACRO: &str = "head";
const PATCH_METHOD_MACRO: &str = "patch";
const HEADER_METHOD_MACRO: &str = "header";

#[derive(Clone, Default)]
pub struct MethodConfig {
    attributes: Vec<Attribute>,
    signature: Option<Signature>,
    method: Option<Method>,
    path: Option<String>,
    consumes: Option<ContentType>,
    produces: Option<ContentType>,
    headers: Vec<HeaderConfig>,
    async_supported: bool,
    dry_run: bool,
    arguments_config: ArgumentsConfig,
    output_config: OutputConfig
}

impl MethodConfig {

    pub fn create_by_item(item: TraitItemFn, async_supported: bool) -> Self {
        let mut method_attrs: MethodConfig = Default::default();
        method_attrs.async_supported = async_supported;

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

    pub fn create(method: Method, item: TokenStream, attrs: TokenStream) -> Self {
        let mut method_config = MethodConfig::default();
        method_config.dry_run = true;

        method_config.parse_stream(method, TokenStream2::from(item), TokenStream2::from(attrs));

        method_config
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

        let compiled_path = self.compile_path();
        let compiled_headers = self.compile_headers();
        let compiled_queries = self.compile_queries();
        let compiled_body = self.compile_body();
        let compiled_result = self.compile_output();
        let compiled_method = self.compile_method();

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

    fn compile_path(&self) -> TokenStream2 {
        self.arguments_config.compile_segments(self.path.as_ref())
    }

    fn compile_headers(&self) -> TokenStream2 {
        let mut stream = self.arguments_config.compile_headers();

        if let Some(content_type) = self.consumes {
            stream.extend(HeaderConfig::new(Some(CONTENT_TYPE.to_string()), Some(content_type.to_string())).compile());
        }

        if let Some(accept_type) = self.produces {
            stream.extend(HeaderConfig::new(Some(ACCEPT.to_string()), Some(accept_type.to_string())).compile());
        }

        stream
    }

    fn compile_queries(&self) -> TokenStream2 {
        self.arguments_config.compile_queries()
    }

    fn compile_body(&self) -> TokenStream2 {
        self.arguments_config.compile_body(self.consumes)
    }

    fn compile_output(&self) -> TokenStream2 {
        self.output_config.compile()
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
                Meta::NameValue(value) => (&value.path, TokenStream2::new(), attr_expr),
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
        self.arguments_config = ArgumentsConfig::new(self.dry_run);

        item.sig.inputs
            .iter_mut()
            .filter_map(|arg| match arg {
                FnArg::Receiver(_) => None,
                FnArg::Typed(arg_type) => Some(arg_type),
            })
            .for_each(|arg_type| self.arguments_config.add(arg_type));

        self.signature = Some(item.sig.clone());
        self.output_config = OutputConfig::new(item.sig.output, self.async_supported, self.produces, self.dry_run);
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
                            self.consumes = Some(consumes);
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
                            self.produces = Some(produces);
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
        let header_config = HeaderConfig::parse_stream(attrs, false);
        self.headers.push(header_config);
    }

    fn get_attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    fn get_signature(&self) -> Signature {
        self.signature.clone().expect("missing method signature")
    }

}