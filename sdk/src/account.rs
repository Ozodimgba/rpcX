//! Account parsing traits and helpers

use crate::bindings::component::solana_rpcx_bindings::types::*;

/// Trait for types that can be parsed from Solana accounts
/// 
/// This is an optional convenience trait. You can also use the
/// builder methods directly.
pub trait AccountParser: Sized {
    /// Human-readable type name
    fn type_name() -> &'static str;
    
    /// Try to parse from raw account data
    fn try_parse(data: &[u8]) -> Result<Self, ParseError>;
    
    /// Check if this parser can handle the given data
    fn can_parse(data: &[u8]) -> bool {
        Self::try_parse(data).is_ok()
    }
    
    /// Convert to JSON string for output
    fn to_json(&self) -> Result<String, ParseError>
    where
        Self: serde::Serialize,
    {
        serde_json::to_string(self)
            .map_err(|e| ParseError::InvalidData(e.to_string()))
    }
    
    /// Optional: Get discriminator bytes if applicable
    fn discriminator() -> Option<Vec<u8>> {
        None
    }
}