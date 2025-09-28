use quote::quote;
use syn::__private::TokenStream2;
use crate::attributes::param::ParamAttributes;

#[derive(Clone, Debug)]
pub struct QueryArgumentCompiler {
    tokens: TokenStream2,
    attributes: ParamAttributes,
}

impl QueryArgumentCompiler {
    
    pub fn parse(item: TokenStream2, attrs: TokenStream2, dry_run: bool) -> Self {
        Self {
            tokens: item,
            attributes: ParamAttributes::parse(attrs, dry_run)
        }
    }

    pub fn compile(&self) -> TokenStream2 {
        let query_variable = &self.tokens;
        let query_id = if let Some(name) = self.attributes.name() {
            name
        } else {
            &format!("{}", quote!(#query_variable))
        };
        
        quote!(.query(#query_id, #query_variable.to_string().as_str()))
    }

}