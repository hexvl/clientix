use quote::quote;
use syn::__private::TokenStream2;
use crate::attributes::param::ParamAttributes;

#[derive(Clone, Debug)]
pub struct SegmentArgumentCompiler {
    tokens: TokenStream2,
    attributes: ParamAttributes
}

impl SegmentArgumentCompiler {

    pub fn parse(item: TokenStream2, attrs: TokenStream2, dry_run: bool) -> Self {
        Self {
            tokens: item,
            attributes: ParamAttributes::parse(attrs, dry_run)
        }
    }

    pub fn compile(&self) -> TokenStream2 {
        let segment_variable = &self.tokens;
        let segment_id = if let Some(name) = self.attributes.name() {
            name
        } else {
            &format!("{}", quote!(#segment_variable))
        };

        quote!(arguments.insert(#segment_id.to_string(), #segment_variable.to_string());)
    }

}