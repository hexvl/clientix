use quote::quote;
use syn::__private::TokenStream2;
use syn::{LitStr, PatType};
use syn::parse::Parser;
use crate::utils::throw_error;

#[derive(Clone, Default, Debug)]
pub struct SegmentConfig {
    argument: Option<Box<syn::Pat>>,
    name: Option<String>,
    default_value: Option<String>,
    dry_run: bool,
}

impl SegmentConfig {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse_stream(attrs: TokenStream2, dry_run: bool) -> Self {
        let mut segment = Self::new();
        segment.dry_run = dry_run;

        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident("name") => {
                    segment.name = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident("default_value") => {
                    segment.default_value = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected method param: {}", meta.path.get_ident().unwrap())))
            }
        });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), dry_run),
        };

        segment
    }

    pub fn parse_argument(pat_type: &PatType, attrs: TokenStream2, dry_run: bool) -> Self {
        let mut segment = Self::parse_stream(attrs, dry_run);
        segment.argument = Some(pat_type.pat.clone());

        segment
    }

    pub fn compile(&self) -> TokenStream2 {
        let segment_variable = self.argument.clone().expect("missing segment attribute");
        let segment_id = if let Some(name) = &self.name {
            name.clone()
        } else {
            format!("{}", quote! {#segment_variable})
        };

        quote! {
            arguments.insert(#segment_id.to_string(), #segment_variable.to_string());
        }
    }

}