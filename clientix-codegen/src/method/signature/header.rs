use quote::{quote, ToTokens};
use syn::__private::{Span, TokenStream2};
use syn::{Field, Ident, PatType, Type};
use crate::attributes::header::HeaderAttributes;

const OPTION_TYPE: &str = "Option";

#[derive(Clone, Debug)]
pub struct HeaderArgumentCompiler {
    ident: Ident,
    ty: Type,
    is_field: bool,
    attributes: HeaderAttributes,
}

impl HeaderArgumentCompiler {
    
    pub fn parse_argument(pat_type: PatType, attrs: TokenStream2, dry_run: bool) -> Self {
        Self {
            ident: Ident::new(&format!("{}", &pat_type.pat.to_token_stream()), Span::call_site()),
            ty: *pat_type.ty,
            is_field: false,
            attributes: HeaderAttributes::parse(attrs, dry_run),
        }
    }
            
    pub fn parse_field(field: Field, attrs: TokenStream2, dry_run: bool) -> Self {
        Self {
            ident: field.ident.clone().unwrap(),
            ty: field.ty,
            is_field: true,
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
        let compiled_self = if self.is_field { quote!(&self.) } else { quote!() };
        if self.is_option() {
            if let Some(value) = self.attributes.value() {
                quote! { 
                    match #compiled_self #header_ident {
                        Some(value) => Some(value),
                        None => Some(#value)
                    }
                }
            } else {
                quote!(#compiled_self #header_ident)
            }
        } else {
            quote!(Some(#compiled_self #header_ident))
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