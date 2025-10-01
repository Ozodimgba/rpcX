//! Procedural macros for the SDK
//!
//! Future: Derive macros to auto-implement AccountParser

use proc_macro::TokenStream;

#[proc_macro_derive(AccountParser)]
pub fn derive_account_parser(_input: TokenStream) -> TokenStream {
    // TODO: Implement derive macro
    TokenStream::new()
}
