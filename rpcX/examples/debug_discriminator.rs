use anyhow::Result;

fn main() -> Result<()> {
    use base64::{Engine as _, engine::general_purpose};
    use sha2::{Digest, Sha256};
    
    let base64_data = "1V3mvCjdz0TLk1URUR0AW6lT34eRmA4qTlC2nyoBwDQ0cpc6RzyG06WcuYagBgC7ccpYRLu0gOfDE24pspc0pwtbyZ0NiE2oFAAAAFNjYWxleC1XYWxsZXQtSXNzdWVyxvp6877brTo9ZfNqq8l0MbG75MLS9uDkfKYCA0UvXWHIAAAAAAAAAAMAAAAAAAAAAHAVpwAAAAAAr6LVFgKSaLB5e6BlHo+/1NjGdiyILqvFx97m1kb0reAAAAAAAAAAAPwAAAAAAAAAABcAAAAAAAAAAwAAAAr3soMTKY36twUyWuZDRbuav7jxegx2s2J/KR0Ave4sAQAAAAAAAAAUAAAAAAAAAL3ZkGtHDX/qNkLAD2g/vSqWO0u5gxQZNTNXUciQOZX0WCVqSYyFXpPxs9IqFyCQwfPyXVMICbqvp0D98SF4iYgBAAAAAAAAABQAAAAAAAAAaiYuGtwQav7t6+nKMLRgLUXMwZQyU9va3yJMWkiyCVbSLRPCkObw05aBCQU3vrbTGuTEACp7/mWtnpxEeCQVpsYAAAAAAAAAEwAAAAAAAAA9769YrqLSGssxsDCDK3Bgz563MDY4eioK+rcRRHccKwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    
    let data = general_purpose::STANDARD.decode(base64_data)?;
    println!("Decoded {} bytes", data.len());

    if data.len() >= 8 {
        let actual_disc = &data[0..8];
        println!("Bytes: {:?}", actual_disc);
        println!("Hex:   {}", hex_string(actual_disc));
    } else {
        println!("Account data too short!");
    }
    println!();
    
    // Try different possible names
    let candidates = vec![
        ("account", "SplitWallet"),
        ("account", "split_wallet"),
        ("account", "splitwallet"),
        ("global", "SplitWallet"),
        ("state", "SplitWallet"),
    ];
    
    for (namespace, name) in candidates {
        let computed = compute_discriminator(namespace, name);
        let preimage = format!("{}:{}", namespace, name);
        
        println!("\nPreimage: \"{}\"", preimage);
        println!("  Bytes: {:?}", computed);
        println!("  Hex:   {}", hex_string(&computed));
        
        // Check if it matches
        if data.len() >= 8 && &data[0..8] == &computed[..] {
            println!("match found");
        }
    }
  
    Ok(())
}

fn compute_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    use sha2::{Digest, Sha256};
    let preimage = format!("{}:{}", namespace, name);
    let hash = Sha256::digest(preimage.as_bytes());
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash[..8]);
    discriminator
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}