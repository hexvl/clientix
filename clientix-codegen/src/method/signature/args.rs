use quote::{quote, ToTokens};
use syn::__private::{Span, TokenStream2};
use syn::{Ident, PatType};

#[derive(Clone, Debug)]
pub struct ArgsArgumentCompiler {
    ident: Ident
}

impl ArgsArgumentCompiler {

    pub fn parse(pat_type: PatType) -> Self {
        let ident = Ident::new(&format!("{}", &pat_type.pat.to_token_stream()), Span::call_site());

        Self {
            ident
        }
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