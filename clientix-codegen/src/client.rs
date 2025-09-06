use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemTrait, TraitItem, LitStr, LitBool, Visibility};
use syn::__private::{Span, TokenStream2};
use syn::parse::Parser;
use crate::method::MethodConfig;

#[derive(Clone)]
pub struct ClientConfig {
    item: Option<ItemTrait>,
    url: Option<String>,
    path: Option<String>,
    async_supported: bool,
    methods: Vec<MethodConfig>
}

impl ClientConfig {

    pub fn create(item: TokenStream, attrs: TokenStream) -> Self {
        let mut client_attrs = ClientConfig { item: None, url: None, path: None, async_supported: false, methods: vec![] };

        client_attrs.parse(TokenStream2::from(item), TokenStream2::from(attrs));

        client_attrs
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
        let client_interface_name = Ident::new(&format!("{}{}", self.get_ident(), "Interface"), Span::call_site());
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
        let client_url = self.get_url();
        let client_path = self.get_path();
        let client_struct_name = self.get_ident();
        let client_visibility = self.get_vis();
        let client_builder_name = Ident::new(&format!("{}{}", self.get_ident(), "Builder"), Span::call_site());
        let client_type_method = if self.async_supported { quote! {asynchronous()} } else { quote! {blocking()} };

        TokenStream2::from(quote! {
            #client_visibility struct #client_builder_name {
                clientix_builder: clientix::client::ClientixBuilder
            }

            impl #client_builder_name {
                pub fn new() -> Self {
                    let clientix_builder = clientix::client::Clientix::builder()
                        .url(#client_url)
                        .path(#client_path);

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
        let client_struct_name = self.get_ident();
        let client_visibility = self.get_vis();
        let client_builder_name = Ident::new(&format!("{}{}", self.get_ident(), "Builder"), Span::call_site());

        let client_type = TokenStream2::from(if self.async_supported {
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

    fn parse(&mut self, item: TokenStream2, attrs: TokenStream2) {
        self.parse_attrs(attrs);
        self.parse_item(item);
    }

    fn parse_item(&mut self, item: TokenStream2) {
        let input: ItemTrait = match syn::parse2(item) {
            Ok(input) => input,
            Err(err) => panic!("{}", err)
        };

        self.item = Some(input);

        let trait_items = self.item.clone().expect("missing item trait").items;
        let trait_methods = trait_items.iter().filter_map(|item| {
            match item {
                TraitItem::Fn(fn_item) => Some(fn_item),
                _ => None
            }
        }).collect::<Vec<_>>();

        for trait_method in trait_methods {
            self.methods.push(MethodConfig::create_by_item(trait_method.clone(), self.async_supported));
        }
    }

    fn parse_attrs(&mut self, attrs: TokenStream2) {
        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident("url") => {
                    self.url = Some(meta.value()?.parse::<LitStr>()?.value());
                    Ok(())
                },
                ref path if path.is_ident("path") => {
                    self.path = Some(meta.value()?.parse::<LitStr>()?.value());
                    Ok(())
                }
                ref path if path.is_ident("async") => {
                    self.async_supported = meta.value()?.parse::<LitBool>()?.value();
                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected client parameter: {}", meta.path.get_ident().map(Ident::to_string).unwrap_or_default())))
            }
        });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(e) => panic!("{}", e)
        };
    }

    fn get_ident(&self) -> Ident {
        self.item.clone().expect("missing client name").ident
    }
    
    fn get_vis(&self) -> Visibility {
        self.item.clone().expect("missing client name").vis
    }

    fn get_url(&self) -> String {
        self.url.clone().unwrap_or(String::new())
    }

    fn get_path(&self) -> String {
        self.path.clone().unwrap_or(String::new())
    }

}

pub fn parse_client(item: TokenStream, attrs: TokenStream) -> TokenStream {
    let client_config = ClientConfig::create(item, attrs);

    let compiled_client = client_config.compile();

    let expanded = quote! {
        #compiled_client
    };

    TokenStream::from(expanded)
}