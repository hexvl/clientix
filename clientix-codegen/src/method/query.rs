use quote::quote;
use syn::__private::TokenStream2;
use syn::{LitStr, PatType};
use syn::parse::Parser;
use crate::utils::throw_error;

#[derive(Clone, Default, Debug)]
pub struct QueryConfig {
    argument: Option<Box<syn::Pat>>,
    name: Option<String>,
    default_value: Option<String>,
    dry_run: bool,
}

impl QueryConfig {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse_stream(attrs: TokenStream2, dry_run: bool) -> Self {
        let mut query = Self::new();
        query.dry_run = dry_run;

        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident("name") => {
                    query.name = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident("default_value") => {
                    query.default_value = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected method param: {}", meta.path.get_ident().unwrap())))
            }
        });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), dry_run),
        };

        query
    }

    pub fn parse_argument(pat_type: &PatType, attrs: TokenStream2, dry_run: bool) -> Self {
        let mut query = Self::parse_stream(attrs, dry_run);
        query.argument = Some(pat_type.pat.clone());

        query
    }

    pub fn compile(&self) -> TokenStream2 {
        let query_variable = self.argument.clone().expect("missing segment attribute");
        let query_id = if let Some(name) = &self.name {
            name.clone()
        } else {
            format!("{}", quote! {#query_variable})
        };
        
        quote!(.query(#query_id, #query_variable.to_string().as_str()))
    }

}