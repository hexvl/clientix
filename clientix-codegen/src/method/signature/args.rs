use quote::quote;
use syn::__private::TokenStream2;
use syn::Ident;

#[derive(Clone, Debug)]
pub struct ArgsArgumentCompiler {
    ident: Ident
}

impl ArgsArgumentCompiler {

    pub fn parse(ident: Ident) -> Self {
        Self { ident }
    }

    pub fn compile_segments(&self) -> TokenStream2 {
        let args_ident = &self.ident;
        quote!(#args_ident.segments())
    }

    pub fn compile_queries(&self) -> TokenStream2 {
        let args_ident = &self.ident;
        quote!(#args_ident.queries())
    }

    pub fn compile_headers(&self) -> TokenStream2 {
        let args_ident = &self.ident;
        quote!(#args_ident.headers())
    }

    pub fn compile_body(&self) -> TokenStream2 {
        let args_ident = &self.ident;
        quote!(#args_ident.body())
    }
    
}