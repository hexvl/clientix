#![allow(dead_code)]

use quote::{quote, ToTokens};
use syn::__private::{Span, TokenStream2};
use syn::{Field, Ident, PatType, Type};

const OPTION_TYPE: &str = "Option";

#[derive(Clone, Debug)]
pub struct BodyArgumentCompiler {
    ident: Ident,
    ty: Type,
    is_field: bool
}

impl BodyArgumentCompiler {
    
    pub fn parse_argument(pat_type: PatType) -> Self {
        Self {
            ident: Ident::new(&format!("{}", &pat_type.pat.to_token_stream()), Span::call_site()),
            ty: *pat_type.ty,
            is_field: false
        }
    }

    pub fn parse_field(field: Field) -> Self {
        Self {
            ident: field.ident.clone().unwrap(),
            ty: field.ty,
            is_field: true
        }
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