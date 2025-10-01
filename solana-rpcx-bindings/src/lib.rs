// This crate ONLY re-exports the generated bindings
// It does NOT implement Guest traits - 

#[allow(warnings)]
pub mod bindings;

// Re-export everything so SDK and users can access it
pub use bindings::*;
