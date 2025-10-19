use syn::__private::TokenStream2;
use syn::LitStr;
use syn::parse::Parser;
use crate::utils::throw_error;

const NAME_ATTR: &str = "name";
const VALUE_ATTR: &str = "value";

#[derive(Clone, Debug)]
pub struct ParamAttributes {
    name: Option<String>,
    value: Option<String>,
}

impl ParamAttributes {

    pub fn new(name: Option<String>, value: Option<String>) -> Self {
        Self { name, value }
    }

    pub fn parse(attrs: TokenStream2, dry_run: bool) -> Self {
        let mut attributes = Self::new(None, None);

        let parser = syn::meta::parser(|meta| {
            match meta.path {
                ref path if path.is_ident(NAME_ATTR) => {
                    attributes.name = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                ref path if path.is_ident(VALUE_ATTR) => {
                    attributes.value = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                }
                _ => Err(meta.error(format!("unexpected method param: {}", meta.path.get_ident().unwrap())))
            }
        });

        match parser.parse2(attrs.clone().into()) {
            Ok(_) => (),
            Err(error) => throw_error(error.to_string().as_str(), dry_run),
        };

        attributes
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }

}