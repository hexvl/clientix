use quote::quote;
use syn::__private::TokenStream2;
use crate::attributes::header::HeaderAttributes;

#[derive(Clone, Debug)]
pub struct HeaderArgumentCompiler {
    tokens: TokenStream2,
    attributes: HeaderAttributes,
}

impl HeaderArgumentCompiler {
    
    pub fn parse(item: TokenStream2, attrs: TokenStream2, dry_run: bool) -> Self {
        Self {
            tokens: item,
            attributes: HeaderAttributes::parse(attrs, dry_run),
        }
    }

    pub fn compile(&self) -> TokenStream2 {
        let header_variable = &self.tokens;
        let header_id = if let Some(name) = self.attributes.name() {
            name
        } else {
            &format!("{}", quote!(#header_variable))
        };
        let sensitive = self.attributes.sensitive();
        
        quote!(.header(#header_id, #header_variable.to_string().as_str(), #sensitive))
    }

}