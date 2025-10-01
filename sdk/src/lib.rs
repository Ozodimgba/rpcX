//! # Solana RPCX SDK
//!
//! Framework-agnostic SDK for building Solana account and instruction parsers
//! that compile to WASM.
//!
//! ## Features
//! - Works with Anchor, native Solana, Pinocchio, and custom programs
//! - Supports multiple serialization formats (Borsh, Bincode, custom)
//! - Flexible discriminator strategies
//! - Type-safe parser builder
//!
//! ## Example
//! ```ignore
//! use solana_rpcx_sdk::prelude::*;
//!
//! let parser = ParserBuilder::new("YourProgramID")
//!     .register_anchor_account::<MyAccount>()
//!     .build();
//! ```

// Re-export bindings
pub mod bindings {
    pub use solana_rpcx_bindings::*;
}

// Core modules
mod error;
mod account;
mod instruction;
mod parser;
mod serialization;
mod discriminator;
mod transformer;
mod transaction;
mod view;
mod utils;

// Convenience module
pub mod prelude;

// Public re-exports
pub use error::*;
pub use account::*;
pub use instruction::*;
pub use parser::*;
pub use serialization::*;
pub use discriminator::*;
pub use transformer::*;
pub use transaction::*;
pub use view::*;
pub use utils::*;

// Macros (if feature enabled)
#[cfg(feature = "macros")]
pub use solana_rpcx_macros::*;