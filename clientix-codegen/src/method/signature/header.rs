use quote::quote;
use syn::__private::TokenStream2;
use syn::{Ident, Type};
use crate::attributes::header::HeaderAttributes;

const OPTION_TYPE: &str = "Option";

#[derive(Clone, Debug)]
pub struct HeaderArgumentCompiler {
    ident: Ident,
    ty: Type,
    attributes: HeaderAttributes,
}

impl HeaderArgumentCompiler {
    
    pub fn parse(ident: Ident, ty: Type, attrs: TokenStream2, dry_run: bool) -> Self {
        Self {
            ident,
            ty,
            attributes: HeaderAttributes::parse(attrs, dry_run),
        }
    }

    pub fn name(&self) -> String {
        let header_ident = &self.ident;
        let header_name = if let Some(name) = self.attributes.name() {
            name.clone()
        } else {
            format!("{}", quote!(#header_ident))
        };
        
        header_name
    }
    
    pub fn ty(&self) -> Type {
        self.ty.clone()
    }
    
    pub fn value(&self) -> TokenStream2 {
        let header_ident = &self.ident;
        if self.is_option() {
            if let Some(value) = self.attributes.value() {
                quote!(#header_ident.unwrap_or(#value))
            } else {
                quote!(#header_ident)
            }
        } else {
            quote!(#header_ident)
        }
    }
    
    pub fn sensitive(&self) -> TokenStream2 {
        let sensitive = self.attributes.sensitive();
        quote!(#sensitive)
    }

    fn is_option(&self) -> bool {
        if let Type::Path(type_path) = self.ty() {
            type_path.path.segments.last()
                .map(|value| value.ident.to_string())
                .map(|value| value == OPTION_TYPE)
                .unwrap_or(false)
        } else {
            false
        }
    }

}