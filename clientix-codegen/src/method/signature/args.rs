use quote::quote;
use syn::__private::TokenStream2;

#[derive(Clone, Debug)]
pub struct ArgsArgumentCompiler {
    tokens: TokenStream2
}

impl ArgsArgumentCompiler {

    pub fn parse(item: TokenStream2) -> Self {
        Self { tokens: item }
    }

    pub fn compile_segments(&self) -> TokenStream2 {
        let tokens = &self.tokens;
        quote!(#tokens.segments())
    }

    pub fn compile_queries(&self) -> TokenStream2 {
        let tokens = &self.tokens;
        quote!(#tokens.queries())
    }

    pub fn compile_headers(&self) -> TokenStream2 {
        let tokens = &self.tokens;
        quote!(#tokens.headers())
    }

}