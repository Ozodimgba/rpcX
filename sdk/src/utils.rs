//! Utility functions

use sha2::{Digest, Sha256};

/// Compute Anchor-style discriminator
/// 
/// For accounts: `anchor_discriminator("account", "MyAccount")`
/// For instructions: `anchor_discriminator("global", "my_instruction")`
pub fn compute_anchor_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let hash = Sha256::digest(preimage.as_bytes());
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash[..8]);
    discriminator
}

/// Convert bytes to base58 string
pub fn bytes_to_base58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

/// Convert base58 string to bytes
pub fn base58_to_bytes(s: &str) -> Result<Vec<u8>, String> {
    bs58::decode(s)
        .into_vec()
        .map_err(|e| format!("Invalid base58: {}", e))
}