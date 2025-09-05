use syn::{GenericArgument, PathArguments, PathSegment, ReturnType, Signature, Type};

const CLIENTIX_RESULT_TYPE: &str = "ClientixResult";
const CLIENTIX_RESPONSE_TYPE: &str = "ClientixResponse";
const CLIENTIX_STREAM_TYPE: &str = "ClientixStream";
const OPTION_TYPE: &str = "Option";
const STRING_TYPE: &str = "String";

pub enum ReturnKind {
    Unit,
    ClientixResultOfResponseOfString,
    ClientixResultOfResponse,
    ClientixResultOfStreamOfString,
    ClientixResultOfStream,
    ClientixResultOfString,
    ClientixResult,
    OptionOfResponseOfString,
    OptionOfResponse,
    OptionOfStreamOfString,
    OptionOfStream,
    OptionOfString,
    Option,
    ClientixStreamOfString,
    ClientixStream,
    ClientixResponseOfString,
    ClientixResponse,
    String,
    Other
}

impl From<Signature> for ReturnKind {
    fn from(value: Signature) -> Self {
        match value.output {
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
                    (CLIENTIX_RESULT_TYPE, CLIENTIX_STREAM_TYPE, STRING_TYPE) => ReturnKind::ClientixResultOfStreamOfString,
                    (CLIENTIX_RESULT_TYPE, CLIENTIX_STREAM_TYPE, _) => ReturnKind::ClientixResultOfStream,
                    (CLIENTIX_RESULT_TYPE, STRING_TYPE, _) => ReturnKind::ClientixResultOfString,
                    (CLIENTIX_RESULT_TYPE, _, _) => ReturnKind::ClientixResult,
                    (OPTION_TYPE, CLIENTIX_RESPONSE_TYPE, STRING_TYPE) => ReturnKind::OptionOfResponseOfString,
                    (OPTION_TYPE, CLIENTIX_RESPONSE_TYPE, _) => ReturnKind::OptionOfResponse,
                    (OPTION_TYPE, CLIENTIX_STREAM_TYPE, STRING_TYPE) => ReturnKind::OptionOfStreamOfString,
                    (OPTION_TYPE, CLIENTIX_STREAM_TYPE, _) => ReturnKind::OptionOfStream,
                    (OPTION_TYPE, STRING_TYPE, _) => ReturnKind::OptionOfString,
                    (OPTION_TYPE, _, _) => ReturnKind::Option,
                    (CLIENTIX_STREAM_TYPE, STRING_TYPE, _) => ReturnKind::ClientixStreamOfString,
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