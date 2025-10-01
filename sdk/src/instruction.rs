//! Instruction parsing traits and helpers

use crate::bindings::component::solana_rpcx_bindings::types::*;

/// Trait for instruction parsing
pub trait InstructionParser: Sized {
    /// Instruction name
    fn instruction_name() -> &'static str;
    
    /// Try to parse from instruction data
    fn try_parse(data: &[u8]) -> Result<Self, ParseError>;
    
    /// Check if this parser can handle the instruction
    fn can_parse(data: &[u8]) -> bool {
        Self::try_parse(data).is_ok()
    }
    
    /// Convert to JSON string
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