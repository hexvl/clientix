use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemTrait, TraitItem, Visibility};
use syn::__private::{Span, TokenStream2};
use crate::attributes::client::ClientAttributes;
use crate::method::MethodCompiler;

#[derive(Clone, Debug)]
pub struct ClientCompiler {
    ident: Ident,
    visibility: Visibility,
    attributes: ClientAttributes,
    methods: Vec<MethodCompiler>
}

impl ClientCompiler {

    pub fn parse(item: TokenStream2, attrs: TokenStream2) -> Self {
        let attributes = ClientAttributes::parse(attrs);

        let item: ItemTrait = match syn::parse2(item) {
            Ok(input) => input,
            Err(err) => panic!("{}", err)
        };

        let ident = item.ident;
        let visibility = item.vis;

        let methods = item.items.into_iter()
            .filter_map(|item| match item {
                TraitItem::Fn(fn_item) => Some(fn_item),
                _ => None
            })
            .map(|item| MethodCompiler::new(item.sig, item.attrs, attributes.async_supported()))
            .collect::<Vec<_>>();

        Self { ident, visibility, attributes, methods }
    }

    pub fn compile(&self) -> TokenStream2 {
        let compiled_interface = self.compile_interface();
        let compiled_builder = self.compile_builder();
        let compiled_client = self.compile_client();

        TokenStream2::from(quote! {
            #compiled_interface
            #compiled_builder
            #compiled_client
        })
    }

    fn compile_interface(&self) -> TokenStream2 {
        let client_interface_name = Ident::new(&format!("{}{}", &self.ident, "Interface"), Span::call_site());
        let client_interface_declarations_fn = self.methods.iter()
            .map(|method| method.compile_declaration())
            .collect::<Vec<_>>();

        TokenStream2::from(quote! {
            trait #client_interface_name {
                #(#client_interface_declarations_fn)*
            }
        })
    }

    fn compile_builder(&self) -> TokenStream2 {
        let client_struct_name = &self.ident;
        let client_visibility = &self.visibility;
        let client_builder_name = Ident::new(&format!("{}{}", &self.ident, "Builder"), Span::call_site());
        let client_url = if let Some(url) = self.attributes.url() { quote!(.url(#url)) } else { quote!() };
        let client_path = if let Some(path) = self.attributes.path() { quote!(.path(#path)) } else { quote!() };
        let client_type_method = if self.attributes.async_supported() { quote!(asynchronous()) } else { quote!(blocking()) };

        TokenStream2::from(quote! {
            #client_visibility struct #client_builder_name {
                clientix_builder: clientix::client::ClientixBuilder
            }

            impl #client_builder_name {
                pub fn new() -> Self {
                    let clientix_builder = clientix::client::Clientix::builder()
                        #client_url
                        #client_path;

                    Self { clientix_builder }
                }

                pub fn url(mut self, url: &str) -> Self {
                    self.clientix_builder = self.clientix_builder.url(url);
                    self
                }

                pub fn path(mut self, path: &str) -> Self {
                    self.clientix_builder = self.clientix_builder.path(path);
                    self
                }

                pub fn user_agent(mut self, user_agent: &str) -> Self {
                    self.clientix_builder = self.clientix_builder.user_agent(user_agent);
                    self
                }

                pub fn header(mut self, key: &str, value: &str) -> Self {
                    self.clientix_builder = self.clientix_builder.header(key, value, false);
                    self
                }

                pub fn basic_auth(mut self, username: &str, password: &str) -> Self {
                    self.clientix_builder = self.clientix_builder.basic_auth(username, password);
                    self
                }
                
                pub fn bearer_auth(mut self, token: &str) -> Self {
                    self.clientix_builder = self.clientix_builder.bearer_auth(token);
                    self
                }
                
                pub fn headers(mut self, headers: std::collections::HashMap<String, String>) -> Self {
                    self.clientix_builder = self.clientix_builder.headers(headers);
                    self
                }

                pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
                    self.clientix_builder = self.clientix_builder.timeout(timeout);
                    self
                }

                pub fn read_timeout(mut self, read_timeout: std::time::Duration) -> Self {
                    self.clientix_builder = self.clientix_builder.read_timeout(read_timeout);
                    self
                }

                pub fn connect_timeout(mut self, connect_timeout: std::time::Duration) -> Self {
                    self.clientix_builder = self.clientix_builder.connect_timeout(connect_timeout);
                    self
                }

                pub fn connection_verbose(mut self, connection_verbose: bool) -> Self {
                    self.clientix_builder = self.clientix_builder.connection_verbose(connection_verbose);
                    self
                }

                pub fn setup(self) -> #client_struct_name {
                    let clientix = self.clientix_builder.build();

                    #client_struct_name {
                        client: clientix.#client_type_method,
                        config: clientix.config().clone()
                    }
                }
            }
        })
    }

    fn compile_client(&self) -> TokenStream2 {
        let client_struct_name = &self.ident;
        let client_visibility = &self.visibility;
        let client_builder_name = Ident::new(&format!("{}{}", &self.ident, "Builder"), Span::call_site());

        let client_type = TokenStream2::from(if self.attributes.async_supported() {
            quote! {clientix::client::asynchronous::AsyncClient}
        } else {
            quote! {clientix::client::blocking::BlockingClient}
        });

        let client_definitions = self.methods.iter()
            .map(|method| method.compile_definition())
            .collect::<Vec<_>>();

        TokenStream2::from(quote! {
            #client_visibility struct #client_struct_name {
                client: #client_type,
                config: clientix::client::ClientConfig
            }

            impl #client_struct_name {
                pub fn config() -> #client_builder_name {
                    #client_builder_name::new()
                }

                pub fn new() -> Self {
                    #client_struct_name::config().setup()
                }
            }

            impl #client_struct_name {
                #(#client_definitions)*
            }
        })
    }

}

pub fn parse_client(item: TokenStream, attrs: TokenStream) -> TokenStream {
    let client_compiler = ClientCompiler::parse(TokenStream2::from(item), TokenStream2::from(attrs));
    let compiled_client = client_compiler.compile();

    TokenStream::from(quote!(#compiled_client))
}