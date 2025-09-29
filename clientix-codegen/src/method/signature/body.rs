use quote::quote;
use syn::__private::TokenStream2;
use syn::{Ident, Type};

const OPTION_TYPE: &str = "Option";

#[derive(Clone, Debug)]
pub struct BodyArgumentCompiler {
    ident: Ident,
    ty: Type,
}

impl BodyArgumentCompiler {

    pub fn parse(ident: Ident, ty: Type) -> Self {
        Self { ident, ty }
    }

    pub fn name(&self) -> String {
        let body_ident = &self.ident;
        let body_name = format!("{}", quote!(#body_ident));

        body_name
    }
    
    pub fn ty(&self) -> Type {
        self.ty.clone()
    }
    
    pub fn value(&self) -> TokenStream2 {
        let body_ident = &self.ident;
        if self.is_option() {
            quote!(#body_ident)
        } else {
            quote!(Some(#body_ident))
        }
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