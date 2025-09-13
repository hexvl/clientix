use quote::quote;
use syn::{GenericArgument, PathArguments, PathSegment, ReturnType, Type};
use syn::__private::TokenStream2;
use clientix_core::core::headers::content_type::ContentType;
use crate::utils::throw_error;

const CLIENTIX_RESULT_TYPE: &str = "ClientixResult";
const CLIENTIX_RESPONSE_TYPE: &str = "ClientixResponse";
const CLIENTIX_STREAM_TYPE: &str = "ClientixStream";
const CLIENTIX_SSE_STREAM_TYPE: &str = "ClientixSSEStream";
const OPTION_TYPE: &str = "Option";
const STRING_TYPE: &str = "String";

#[derive(Clone, Default)]
pub enum ReturnKind {
    #[default]
    Unit,
    ClientixResultOfResponseOfString,
    ClientixResultOfResponse,
    ClientixResultOfSSEStreamOfString,
    ClientixResultOfSSEStream,
    ClientixResultOfStream,
    ClientixResultOfString,
    ClientixResult,
    OptionOfResponseOfString,
    OptionOfResponse,
    OptionOfSSEStreamOfString,
    OptionOfSSEStream,
    OptionOfStream,
    OptionOfString,
    Option,
    ClientixSSEStreamOfString,
    ClientixSSEStream,
    ClientixStream,
    ClientixResponseOfString,
    ClientixResponse,
    String,
    Other
}

#[derive(Clone, Default)]
pub struct OutputConfig {
    kind: ReturnKind,
    async_supported: bool,
    produces: Option<ContentType>,
    dry_run: bool
}

impl From<ReturnType> for ReturnKind {
    fn from(value: ReturnType) -> Self {
        match value {
            ReturnType::Default => ReturnKind::Unit,
            ReturnType::Type(_, ty) => {
                let first_segment = extract_last_path_segment(&ty);
                let second_segment = first_segment.and_then(|value| extract_inner_first_path_segment(&value));
                let third_segment = second_segment.and_then(|value| extract_inner_first_path_segment(&value));

                let first_segment_ident = first_segment.map(|value| value.ident.to_string()).unwrap_or(String::default());
                let second_segment_ident = second_segment.map(|value| value.ident.to_string()).unwrap_or(String::default());
                let third_segment_ident = third_segment.map(|value| value.ident.to_string()).unwrap_or(String::default());

                match (first_segment_ident.as_str(), second_segment_ident.as_str(), third_segment_ident.as_str()) {
                    (CLIENTIX_RESULT_TYPE, CLIENTIX_RESPONSE_TYPE, STRING_TYPE) => ReturnKind::ClientixResultOfResponseOfString,
                    (CLIENTIX_RESULT_TYPE, CLIENTIX_RESPONSE_TYPE, _) => ReturnKind::ClientixResultOfResponse,
                    (CLIENTIX_RESULT_TYPE, CLIENTIX_SSE_STREAM_TYPE, STRING_TYPE) => ReturnKind::ClientixResultOfSSEStreamOfString,
                    (CLIENTIX_RESULT_TYPE, CLIENTIX_SSE_STREAM_TYPE, _) => ReturnKind::ClientixResultOfSSEStream,
                    (CLIENTIX_RESULT_TYPE, CLIENTIX_STREAM_TYPE, _) => ReturnKind::ClientixResultOfStream,
                    (CLIENTIX_RESULT_TYPE, STRING_TYPE, _) => ReturnKind::ClientixResultOfString,
                    (CLIENTIX_RESULT_TYPE, _, _) => ReturnKind::ClientixResult,
                    (OPTION_TYPE, CLIENTIX_RESPONSE_TYPE, STRING_TYPE) => ReturnKind::OptionOfResponseOfString,
                    (OPTION_TYPE, CLIENTIX_RESPONSE_TYPE, _) => ReturnKind::OptionOfResponse,
                    (OPTION_TYPE, CLIENTIX_SSE_STREAM_TYPE, STRING_TYPE) => ReturnKind::OptionOfSSEStreamOfString,
                    (OPTION_TYPE, CLIENTIX_SSE_STREAM_TYPE, _) => ReturnKind::OptionOfSSEStream,
                    (OPTION_TYPE, CLIENTIX_STREAM_TYPE, _) => ReturnKind::OptionOfStream,
                    (OPTION_TYPE, STRING_TYPE, _) => ReturnKind::OptionOfString,
                    (OPTION_TYPE, _, _) => ReturnKind::Option,
                    (CLIENTIX_SSE_STREAM_TYPE, STRING_TYPE, _) => ReturnKind::ClientixSSEStreamOfString,
                    (CLIENTIX_SSE_STREAM_TYPE, _, _) => ReturnKind::ClientixSSEStream,
                    (CLIENTIX_STREAM_TYPE, _, _) => ReturnKind::ClientixStream,
                    (CLIENTIX_RESPONSE_TYPE, STRING_TYPE, _) => ReturnKind::ClientixResponseOfString,
                    (CLIENTIX_RESPONSE_TYPE, _, _) => ReturnKind::ClientixResponse,
                    (STRING_TYPE, _, _) => ReturnKind::String,
                    _ => ReturnKind::Other
                }
            }
        }
    }

}

impl OutputConfig {

    pub fn new(return_type: ReturnType, async_supported: bool, produces: Option<ContentType>, dry_run: bool) -> Self {
        let kind = return_type.into();
        Self { kind, async_supported, produces, dry_run }
    }

    pub fn compile(&self) -> TokenStream2 {
        match self.kind {
            ReturnKind::Unit => self.compile_unit(),
            ReturnKind::ClientixResultOfResponseOfString => self.compile_text_response_result(),
            ReturnKind::ClientixResultOfResponse => self.compile_object_response_result(),
            ReturnKind::ClientixResultOfSSEStreamOfString => self.compile_text_stream_result(),
            ReturnKind::ClientixResultOfSSEStream => self.compile_object_stream_result(),
            ReturnKind::ClientixResultOfStream => self.compile_bytes_stream_result(),
            ReturnKind::ClientixResultOfString => self.compile_text_result(),
            ReturnKind::ClientixResult => self.compile_object_result(),
            ReturnKind::OptionOfResponseOfString => self.compile_text_response_option(),
            ReturnKind::OptionOfResponse => self.compile_object_response_option(),
            ReturnKind::OptionOfSSEStreamOfString => self.compile_text_stream_option(),
            ReturnKind::OptionOfSSEStream => self.compile_object_stream_option(),
            ReturnKind::OptionOfStream => self.compile_bytes_stream_option(),
            ReturnKind::OptionOfString => self.compile_text_option(),
            ReturnKind::Option => self.compile_object_option(),
            ReturnKind::ClientixSSEStreamOfString => self.compile_text_stream(),
            ReturnKind::ClientixSSEStream => self.compile_object_stream(),
            ReturnKind::ClientixStream => self.compile_bytes_stream(),
            ReturnKind::ClientixResponseOfString => self.compile_text_response(),
            ReturnKind::ClientixResponse => self.compile_object_response(),
            ReturnKind::String => self.compile_text(),
            ReturnKind::Other => self.compile_object()
        }
    }

    fn compile_unit(&self) -> TokenStream2 {
        quote! {;}
    }

    fn compile_text_response_result(&self) -> TokenStream2 {
        let compiled_async_directive = self.compile_async();
        quote! {
            #compiled_async_directive
            .text()
            #compiled_async_directive
        }
    }

    fn compile_object_response_result(&self) -> TokenStream2 {
        let compiled_async_directive = self.compile_async();
        let compiled_object_method = match self.produces {
            Some(ContentType::ApplicationXml) => quote!{.xml()},
            Some(ContentType::ApplicationXWwwFormUrlEncoded) => quote!{.urlencoded()},
            Some(ContentType::TextHtml) => quote!{.text()},
            _ => quote!{.json()},
        };

        quote! {
            #compiled_async_directive
            #compiled_object_method
            #compiled_async_directive
        }
    }

    fn compile_text_stream_result(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .text_stream()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }

    fn compile_object_stream_result(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .json_stream()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }
    
    fn compile_bytes_stream_result(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .bytes_stream()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }
    
    fn compile_text_result(&self) -> TokenStream2 {
        let compiled_text_response_result = self.compile_text_response_result();
        quote! {
            #compiled_text_response_result
            .map(|response| response.body())
        }
    }

    fn compile_object_result(&self) -> TokenStream2 {
        let compiled_object_response_result = self.compile_object_response_result();
        quote! {
            #compiled_object_response_result
            .map(|response| response.body())
        }
    }

    fn compile_text_response_option(&self) -> TokenStream2 {
        let compiled_text_response_result = self.compile_text_response_result();
        quote! {
            #compiled_text_response_result
            .ok()
        }
    }

    fn compile_object_response_option(&self) -> TokenStream2 {
        let compiled_object_response_result = self.compile_object_response_result();
        quote! {
            #compiled_object_response_result
            .ok()
        }
    }

    fn compile_text_stream_option(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .text_stream()
                .ok()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }

    fn compile_object_stream_option(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .json_stream()
                .ok()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }

    fn compile_bytes_stream_option(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .bytes_stream()
                .ok()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }
    
    fn compile_text_option(&self) -> TokenStream2 {
        let compiled_text_response_result = self.compile_text_response_result();
        quote! {
            #compiled_text_response_result
            .map(|response| response.body())
            .ok()
        }
    }

    fn compile_object_option(&self) -> TokenStream2 {
        let compiled_object_response_result = self.compile_object_response_result();
        quote! {
            #compiled_object_response_result
            .map(|response| response.body())
            .ok()
        }
    }

    fn compile_text_stream(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .text_stream()
                .unwrap()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }

    fn compile_object_stream(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .json_stream()
                .unwrap()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }
    
    fn compile_bytes_stream(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {
                .await
                .bytes_stream()
                .unwrap()
            }
        } else {
            throw_error("Streams not supported for not async clients", self.dry_run);
            quote!()
        }
    }

    fn compile_text_response(&self) -> TokenStream2 {
        let compiled_text_response_result = self.compile_text_response_result();
        quote! {
            #compiled_text_response_result
            .unwrap()
        }
    }

    fn compile_object_response(&self) -> TokenStream2 {
        let compiled_object_response_result = self.compile_object_response_result();
        quote! {
            #compiled_object_response_result
            .unwrap()
        }
    }

    fn compile_text(&self) -> TokenStream2 {
        let compiled_text_response_result = self.compile_text_response_result();
        quote! {
            #compiled_text_response_result
            .map(|response| response.body())
            .unwrap()
        }
    }

    fn compile_object(&self) -> TokenStream2 {
        let compiled_object_response_result = self.compile_object_response_result();
        quote! {
            #compiled_object_response_result
            .map(|response| response.body())
            .unwrap()
        }
    }

    fn compile_async(&self) -> TokenStream2 {
        if self.async_supported {
            quote! {.await}
        } else {
            quote! {}
        }
    }

}

fn extract_last_path_segment(return_type: &Type) -> Option<&PathSegment> {
    if let Type::Path(type_path) = return_type {
        return type_path.path.segments.last()
    }

    None
}

fn extract_inner_first_path_segment(path_segment: &PathSegment) -> Option<&PathSegment> {
    if let PathArguments::AngleBracketed(arguments) = &path_segment.arguments {
        if let Some(argument) = arguments.args.first() {
            if let GenericArgument::Type(ty) = argument {
                return extract_last_path_segment(ty)
            }
        }
    }

    None
}