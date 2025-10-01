use crate::bindings::component::solana_rpcx_bindings::types::ParseError;

/// SDK-specific errors that convert to ParseError
#[derive(Debug)]
pub enum SdkError {
    DeserializationFailed(String),
    InvalidData(String),
    InsufficientData(String),
    UnknownType(String),
}

impl From<SdkError> for ParseError {
    fn from(err: SdkError) -> ParseError {
        match err {
            SdkError::DeserializationFailed(msg) => ParseError::DeserializationFailed(msg),
            SdkError::InvalidData(msg) => ParseError::InvalidData(msg),
            SdkError::InsufficientData(msg) => ParseError::InsufficientData(msg),
            SdkError::UnknownType(msg) => ParseError::UnknownAccountType(msg),
        }
    }
}