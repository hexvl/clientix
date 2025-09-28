use quote::quote;
use syn::__private::TokenStream2;
use crate::attributes::header::HeaderAttributes;
use crate::method::signature::segment::SegmentArgumentCompiler;

#[derive(Clone, Debug)]
pub struct HeaderCompiler {
    attributes: HeaderAttributes,
}

impl HeaderCompiler {

    pub fn new(name: Option<String>, value: Option<String>, sensitive: bool) -> Self {
        Self { attributes: HeaderAttributes::new(name, value, sensitive) }
    }

    pub fn parse(attrs: TokenStream2, dry_run: bool) -> Self {
        Self { attributes: HeaderAttributes::parse(attrs, dry_run) }
    }

    pub fn compile(&self, segments: &Vec<SegmentArgumentCompiler>) -> TokenStream2 {
        if self.attributes.name().is_none() && self.attributes.value().is_none() {
            return quote! {}
        }

        let name = self.attributes.name().clone().unwrap();
        let value = self.attributes.value().clone().unwrap();
        let sensitive = self.attributes.sensitive();

        let mut stream = TokenStream2::new();
        if !segments.is_empty() {
            stream.extend(quote!(let mut arguments = std::collections::HashMap::new();));

            for segment_variable in segments.iter() {
                stream.extend(segment_variable.compile())
            }

            stream.extend(quote!(clientix::prelude::strfmt::strfmt(#value, &arguments).expect("failed to format header").as_str()));

            TokenStream2::from(quote!(.header(#name, {#stream}, #sensitive)))
        } else {
            TokenStream2::from(quote!(.header(#name, #value, #sensitive)))
        }
    }

}