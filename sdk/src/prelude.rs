//! Prelude module with common imports
//!
//! ```
//! use solana_rpcx_sdk::prelude::*;
//! ```

// Re-export bindings types
pub use crate::bindings::component::solana_rpcx_bindings::types::*;

// Re-export Guest traits
pub use crate::bindings::exports::component::solana_rpcx_bindings::{
    program_parser::Guest as ProgramParserGuest,
    accounts_transformer::Guest as AccountsTransformerGuest,
    accounts_transformer_setup::{
        Guest as AccountsTransformerSetupGuest,
        TransformerRequest,
        SeedComponent,
    },
    transaction_transformer::Guest as TransactionTransformerGuest,
    view_function::Guest as ViewFunctionGuest,
};

// Re-export SDK types
pub use crate::{
    Parser, ParserBuilder,
    AccountParser, InstructionParser,
    AccountParserConfig, InstructionParserConfig,
    compute_anchor_discriminator,
};

// Re-export common external types
pub use borsh::{BorshDeserialize, BorshSerialize};
pub use serde::{Deserialize, Serialize};