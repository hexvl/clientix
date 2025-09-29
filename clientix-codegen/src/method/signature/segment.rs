use quote::quote;
use syn::__private::TokenStream2;
use syn::{Ident, Type};
use crate::attributes::param::ParamAttributes;

const OPTION_TYPE: &str = "Option";

#[derive(Clone, Debug)]
pub struct SegmentArgumentCompiler {
    ident: Ident,
    ty: Type,
    attributes: ParamAttributes
}

impl SegmentArgumentCompiler {

    pub fn parse(ident: Ident, ty: Type, attrs: TokenStream2, dry_run: bool) -> Self {
        Self {
            ident,
            ty,
            attributes: ParamAttributes::parse(attrs, dry_run)
        }
    }
    
    pub fn name(&self) -> String {
        let segment_ident = &self.ident;
        let segment_name = if let Some(name) = self.attributes.name() {
            name.clone()
        } else {
            format!("{}", quote!(#segment_ident))
        };
        
        segment_name
    }
    
    pub fn ty(&self) -> Type {
        self.ty.clone()
    }
    
    pub fn value(&self) -> TokenStream2 {
        let segment_ident = &self.ident;
        if self.is_option() {
            if let Some(value) = self.attributes.value() {
                quote!(#segment_ident.unwrap_or(#value))
            } else {
                quote!(#segment_ident.unwrap())
            }
        } else {
            quote!(#segment_ident)
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