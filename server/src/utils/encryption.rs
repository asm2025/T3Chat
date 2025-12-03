use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use once_cell::sync::Lazy;
use std::env;

// Encryption key from environment (should be 32 bytes base64 encoded)
static ENCRYPTION_KEY: Lazy<Key<Aes256Gcm>> = Lazy::new(|| {
    let key_str = env::var("ENCRYPTION_KEY").unwrap_or_else(|_| {
        // For development only - use a default key
        // In production, this should fail if not set
        tracing::warn!("ENCRYPTION_KEY not set, using default (INSECURE for production!)");
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string()
    });

    let key_bytes = general_purpose::STANDARD
        .decode(&key_str)
        .expect("ENCRYPTION_KEY must be valid base64");

    if key_bytes.len() != 32 {
        panic!("ENCRYPTION_KEY must be exactly 32 bytes when decoded");
    }

    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);
    Key::<Aes256Gcm>::from(key_array)
});

/// Encrypt a plaintext string using AES-256-GCM
///
/// Returns a base64-encoded string containing both the nonce and ciphertext
/// Format: base64(nonce || ciphertext)
pub fn encrypt(plaintext: &str) -> Result<String> {
    let cipher = Aes256Gcm::new(&ENCRYPTION_KEY);

    // Generate a random 96-bit nonce
    let nonce_bytes = Aes256Gcm::generate_nonce(&mut OsRng);

    // Encrypt the plaintext
    let ciphertext = cipher
        .encrypt(&nonce_bytes, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Combine nonce and ciphertext: nonce (12 bytes) + ciphertext
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    // Encode as base64
    Ok(general_purpose::STANDARD.encode(&combined))
}

/// Decrypt a ciphertext string using AES-256-GCM
///
/// Expects a base64-encoded string containing both the nonce and ciphertext
/// Format: base64(nonce || ciphertext)
pub fn decrypt(encrypted: &str) -> Result<String> {
    let cipher = Aes256Gcm::new(&ENCRYPTION_KEY);

    // Decode from base64
    let combined = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| anyhow::anyhow!("Invalid base64 encoding: {}", e))?;

    // Split into nonce (first 12 bytes) and ciphertext
    if combined.len() < 12 {
        anyhow::bail!("Encrypted data too short");
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed - invalid key or corrupted data: {}", e))?;

    // Convert to string
    String::from_utf8(plaintext_bytes)
        .map_err(|e| anyhow::anyhow!("Decrypted data is not valid UTF-8: {}", e))
}

/// Generate a random encryption key (for setup/configuration)
///
/// Returns a base64-encoded 32-byte key suitable for use as ENCRYPTION_KEY
pub fn generate_encryption_key() -> String {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    general_purpose::STANDARD.encode(key.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = "sk-test123456789";
        let encrypted = encrypt(plaintext).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let plaintext = "sk-test123456789";
        let encrypted1 = encrypt(plaintext).unwrap();
        let encrypted2 = encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertext (different nonces)
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        assert_eq!(decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_invalid_ciphertext() {
        let result = decrypt("invalid_base64!");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_key() {
        let key = generate_encryption_key();
        assert!(!key.is_empty());

        // Should be valid base64
        let decoded = general_purpose::STANDARD.decode(&key).unwrap();

        // Should be 32 bytes
        assert_eq!(decoded.len(), 32);
    }
}
