//! Parser builder and runtime
//!
//! The core of the SDK. Allows registration of different account and instruction
//! types and dispatches parsing to the appropriate handler.

use std::collections::HashMap;
use crate::bindings::component::solana_rpcx_bindings::types::*;
use crate::error::*;

/// Type alias for account parser functions
pub type AccountParserFn = Box<dyn Fn(&[u8]) -> Result<ParsedAccount, ParseError> + Send + Sync>;

/// Type alias for instruction parser functions
pub type InstructionParserFn = Box<dyn Fn(&[u8]) -> Result<ParsedInstruction, ParseError> + Send + Sync>;

/// Configuration for a single account parser
pub struct AccountParserConfig {
    pub type_name: String,
    pub discriminator: Option<Vec<u8>>,
    pub parser: AccountParserFn,
}

/// Configuration for a single instruction parser
pub struct InstructionParserConfig {
    pub name: String,
    pub discriminator: Option<Vec<u8>>,
    pub parser: InstructionParserFn,
}

/// Builder for creating parsers
pub struct ParserBuilder {
    program_id: String,
    account_parsers: Vec<AccountParserConfig>,
    instruction_parsers: Vec<InstructionParserConfig>,
    metadata: Option<ProgramMetadata>,
}

impl ParserBuilder {
    /// Create a new parser builder for a specific program
    pub fn new(program_id: impl Into<String>) -> Self {
        Self {
            program_id: program_id.into(),
            account_parsers: Vec::new(),
            instruction_parsers: Vec::new(),
            metadata: None,
        }
    }
    
    /// Register an Anchor account (8-byte discriminator + Borsh)
    #[cfg(feature = "anchor")]
    pub fn register_anchor_account<T, F>(mut self, to_json: F) -> Self
    where
        T: anchor_lang::AccountDeserialize + 'static,
        F: Fn(&T) -> Result<String, String> + Send + Sync + 'static,
    {
        let type_name = extract_type_name::<T>();
        let type_name_clone = type_name.clone(); // Tech debt: Lazy code
        let discriminator = crate::compute_anchor_discriminator("account", &type_name);
        
        let parser: AccountParserFn = Box::new(move |data: &[u8]| {
            if data.len() < 8 {
                return Err(ParseError::InsufficientData("Account data too short".to_string()));
            }
            
            if &data[0..8] != &discriminator {
                return Err(ParseError::UnknownAccountType("Wrong discriminator".to_string()));
            }
            
            let mut data_slice = &data[8..];
            let account = T::try_deserialize(&mut data_slice)
                .map_err(|e| ParseError::DeserializationFailed(e.to_string()))?;
            
            let json = to_json(&account)
                .map_err(|e| ParseError::InvalidData(e))?;
            
            Ok(ParsedAccount {
                account_type: type_name_clone.clone(),
                data: json,
                discriminator: Some(discriminator.to_vec()),
            })
        });
        
        self.account_parsers.push(AccountParserConfig {
            type_name,
            discriminator: Some(discriminator.to_vec()),
            parser,
        });
        
        self
    }
    
    /// Register a native Borsh account (no discriminator)
    pub fn register_borsh_account<T>(mut self, type_name: impl Into<String>) -> Self
    where
        T: borsh::BorshDeserialize + serde::Serialize + 'static,
    {
        let type_name = type_name.into();
        let type_name_clone = type_name.clone();
        
        let parser: AccountParserFn = Box::new(move |data: &[u8]| {
            parse_borsh_account::<T>(data, &type_name_clone)
        });
        
        self.account_parsers.push(AccountParserConfig {
            type_name,
            discriminator: None,
            parser,
        });
        
        self
    }
    
    /// Register account with custom discriminator
    pub fn register_account_with_discriminator<T>(
        mut self,
        type_name: impl Into<String>,
        discriminator: Vec<u8>,
    ) -> Self
    where
        T: borsh::BorshDeserialize + serde::Serialize + 'static,
    {
        let type_name = type_name.into();
        let disc_size = discriminator.len();
        let type_name_clone = type_name.clone();
        let disc_clone = discriminator.clone();
        
        let parser: AccountParserFn = Box::new(move |data: &[u8]| {
            parse_account_with_discriminator::<T>(
                data,
                &type_name_clone,
                &disc_clone,
                disc_size
            )
        });
        
        self.account_parsers.push(AccountParserConfig {
            type_name,
            discriminator: Some(discriminator),
            parser,
        });
        
        self
    }
    
    /// Register a fully custom account parser
    pub fn register_custom_account<F>(
        mut self,
        type_name: impl Into<String>,
        discriminator: Option<Vec<u8>>,
        parser: F,
    ) -> Self
    where
        F: Fn(&[u8]) -> Result<ParsedAccount, ParseError> + Send + Sync + 'static,
    {
        self.account_parsers.push(AccountParserConfig {
            type_name: type_name.into(),
            discriminator,
            parser: Box::new(parser),
        });
        
        self
    }
    
    /// Register an Anchor instruction
    pub fn register_anchor_instruction<T>(
        mut self,
        name: impl Into<String>,
    ) -> Self
    where
        T: borsh::BorshDeserialize + serde::Serialize + 'static,
    {
        let name = name.into();
        let discriminator = crate::compute_anchor_discriminator("global", &name);
        let name_clone = name.clone();
        
        let parser: InstructionParserFn = Box::new(move |data: &[u8]| {
            parse_anchor_instruction::<T>(data, &name_clone, &discriminator)
        });
        
        self.instruction_parsers.push(InstructionParserConfig {
            name,
            discriminator: Some(discriminator.to_vec()),
            parser,
        });
        
        self
    }
    
    /// Register a native Borsh instruction
    pub fn register_borsh_instruction<T>(
        mut self,
        name: impl Into<String>,
    ) -> Self
    where
        T: borsh::BorshDeserialize + serde::Serialize + 'static,
    {
        let name = name.into();
        let name_clone = name.clone();
        
        let parser: InstructionParserFn = Box::new(move |data: &[u8]| {
            parse_borsh_instruction::<T>(data, &name_clone)
        });
        
        self.instruction_parsers.push(InstructionParserConfig {
            name,
            discriminator: None,
            parser,
        });
        
        self
    }
    
    /// Register a custom instruction parser
    pub fn register_custom_instruction<F>(
        mut self,
        name: impl Into<String>,
        discriminator: Option<Vec<u8>>,
        parser: F,
    ) -> Self
    where
        F: Fn(&[u8]) -> Result<ParsedInstruction, ParseError> + Send + Sync + 'static,
    {
        self.instruction_parsers.push(InstructionParserConfig {
            name: name.into(),
            discriminator,
            parser: Box::new(parser),
        });
        
        self
    }
    
    /// Set program metadata
    pub fn with_metadata(mut self, metadata: ProgramMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Build the final parser
    pub fn build(self) -> Parser {
        Parser {
            program_id: self.program_id,
            account_parsers: self.account_parsers,
            instruction_parsers: self.instruction_parsers,
            metadata: self.metadata,
        }
    }
}

/// Runtime parser that dispatches to registered handlers
pub struct Parser {
    program_id: String,
    account_parsers: Vec<AccountParserConfig>,
    instruction_parsers: Vec<InstructionParserConfig>,
    metadata: Option<ProgramMetadata>,
}

impl Parser {
    /// Parse an account using registered parsers
    pub fn parse_account(&self, account: &SolanaAccount) -> Result<ParsedAccount, ParseError> {
        // Check owner matches
        if account.owner != self.program_id {
            return Err(ParseError::UnknownAccountType(
                format!("Wrong owner: expected {}, got {}", self.program_id, account.owner)
            ));
        }
        
        // Try each parser in order
        let mut last_error = None;
        for config in &self.account_parsers {
            match (config.parser)(&account.data) {
                Ok(result) => return Ok(result),
                Err(e) => last_error = Some(e),
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            ParseError::UnknownAccountType("No parser could handle this account".to_string())
        }))
    }
    
    /// Parse an instruction using registered parsers
    pub fn parse_instruction(&self, instruction: &InstructionData) -> Result<ParsedInstruction, ParseError> {
        let mut last_error = None;
        for config in &self.instruction_parsers {
            match (config.parser)(&instruction.data) {
                Ok(result) => return Ok(result),
                Err(e) => last_error = Some(e),
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            ParseError::UnknownAccountType("No instruction parser matched".to_string())
        }))
    }
    
    /// Check if this parser can handle the given owner/data
    pub fn can_parse(&self, owner: &str, _data: &[u8]) -> bool {
        owner == self.program_id
    }
    
    /// Get list of supported account types
    pub fn get_supported_types(&self) -> Vec<String> {
        self.account_parsers
            .iter()
            .map(|c| c.type_name.clone())
            .collect()
    }
    
    /// Get program metadata
    pub fn get_metadata(&self) -> Option<ProgramMetadata> {
        self.metadata.clone()
    }
}

// Helper functions

fn extract_type_name<T>() -> String {
    std::any::type_name::<T>()
        .split("::")
        .last()
        .unwrap_or("Unknown")
        .to_string()
}

fn parse_anchor_account<T>(
    data: &[u8],
    type_name: &str,
    discriminator: &[u8; 8],
) -> Result<ParsedAccount, ParseError>
where
    T: borsh::BorshDeserialize + serde::Serialize,
{
    if data.len() < 8 {
        return Err(ParseError::InsufficientData("Account data too short".to_string()));
    }
    
    if &data[0..8] != discriminator {
        return Err(ParseError::UnknownAccountType("Wrong discriminator".to_string()));
    }
    
    let account = T::try_from_slice(&data[8..])
        .map_err(|e| ParseError::DeserializationFailed(e.to_string()))?;
    
    let json = serde_json::to_string(&account)
        .map_err(|e| ParseError::InvalidData(e.to_string()))?;
    
    Ok(ParsedAccount {
        account_type: type_name.to_string(),
        data: json,
        discriminator: Some(discriminator.to_vec()),
    })
}

fn parse_borsh_account<T>(
    data: &[u8],
    type_name: &str,
) -> Result<ParsedAccount, ParseError>
where
    T: borsh::BorshDeserialize + serde::Serialize,
{
    let account = T::try_from_slice(data)
        .map_err(|e| ParseError::DeserializationFailed(e.to_string()))?;
    
    let json = serde_json::to_string(&account)
        .map_err(|e| ParseError::InvalidData(e.to_string()))?;
    
    Ok(ParsedAccount {
        account_type: type_name.to_string(),
        data: json,
        discriminator: None,
    })
}

fn parse_account_with_discriminator<T>(
    data: &[u8],
    type_name: &str,
    discriminator: &[u8],
    disc_size: usize,
) -> Result<ParsedAccount, ParseError>
where
    T: borsh::BorshDeserialize + serde::Serialize,
{
    if data.len() < disc_size {
        return Err(ParseError::InsufficientData("Data too short for discriminator".to_string()));
    }
    
    if &data[0..disc_size] != discriminator {
        return Err(ParseError::UnknownAccountType("Wrong discriminator".to_string()));
    }
    
    let account = T::try_from_slice(&data[disc_size..])
        .map_err(|e| ParseError::DeserializationFailed(e.to_string()))?;
    
    let json = serde_json::to_string(&account)
        .map_err(|e| ParseError::InvalidData(e.to_string()))?;
    
    Ok(ParsedAccount {
        account_type: type_name.to_string(),
        data: json,
        discriminator: Some(discriminator.to_vec()),
    })
}

fn parse_anchor_instruction<T>(
    data: &[u8],
    name: &str,
    discriminator: &[u8; 8],
) -> Result<ParsedInstruction, ParseError>
where
    T: borsh::BorshDeserialize + serde::Serialize,
{
    if data.len() < 8 {
        return Err(ParseError::InsufficientData("Instruction data too short".to_string()));
    }
    
    if &data[0..8] != discriminator {
        return Err(ParseError::UnknownAccountType("Wrong discriminator".to_string()));
    }
    
    let instruction = T::try_from_slice(&data[8..])
        .map_err(|e| ParseError::DeserializationFailed(e.to_string()))?;
    
    let json = serde_json::to_string(&instruction)
        .map_err(|e| ParseError::InvalidData(e.to_string()))?;
    
    Ok(ParsedInstruction {
        instruction_name: name.to_string(),
        data: json,
    })
}

fn parse_borsh_instruction<T>(
    data: &[u8],
    name: &str,
) -> Result<ParsedInstruction, ParseError>
where
    T: borsh::BorshDeserialize + serde::Serialize,
{
    let instruction = T::try_from_slice(data)
        .map_err(|e| ParseError::DeserializationFailed(e.to_string()))?;
    
    let json = serde_json::to_string(&instruction)
        .map_err(|e| ParseError::InvalidData(e.to_string()))?;
    
    Ok(ParsedInstruction {
        instruction_name: name.to_string(),
        data: json,
    })
}