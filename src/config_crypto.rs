//! Config encryption utilities for protecting sensitive paths in config.toml.
//!
//! Uses AES-256-GCM with a key stored in a dedicated file in the config
//! directory.  The keyring approach was unreliable on Windows (entries did
//! not persist between application launches).

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// Name of the key file stored alongside config.toml
const KEY_FILENAME: &str = ".keyfile_key";

/// Retrieve or generate a 256-bit AES key stored in a file.
///
/// On first call the key is generated from `OsRng` and written to
/// `<config_dir>/sed/.keyfile_key`.  Subsequent calls read the existing key.
pub fn get_or_create_config_key() -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir()
        .ok_or("Cannot determine config directory")?
        .join("sed");
    std::fs::create_dir_all(&config_dir)?;

    let key_path = config_dir.join(KEY_FILENAME);

    // Try to load existing key
    if key_path.exists() {
        let contents = std::fs::read(&key_path)?;
        if contents.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&contents);
            eprintln!("[SED] config_crypto: LOADED key from file (fingerprint: {:02x}{:02x}{:02x}{:02x})",
                key[0], key[1], key[2], key[3]);
            return Ok(key);
        }
        eprintln!("[SED] config_crypto: key file has wrong length ({}), regenerating", contents.len());
    }

    // Generate a fresh random key
    let key_obj = Aes256Gcm::generate_key(OsRng);
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_obj);

    eprintln!("[SED] config_crypto: GENERATED NEW key (fingerprint: {:02x}{:02x}{:02x}{:02x})",
        key[0], key[1], key[2], key[3]);

    // Write raw bytes to the key file
    std::fs::write(&key_path, &key)?;

    Ok(key)
}

/// Encrypt a plaintext string (e.g. a keyfile path) with AES-256-GCM.
///
/// A fresh random 12-byte nonce is generated for every call.
/// Returns `"<base64_nonce>:<base64_ciphertext>"`.
pub fn encrypt_keyfile_path(
    key: &[u8; 32],
    plaintext: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("AES-GCM encryption failed: {}", e))?;

    let nonce_b64 = BASE64.encode(nonce);
    let ct_b64 = BASE64.encode(ciphertext);
    Ok(format!("{}:{}", nonce_b64, ct_b64))
}

/// Decrypt a string previously produced by [`encrypt_keyfile_path`].
///
/// Expects `"<base64_nonce>:<base64_ciphertext>"`.  Returns an error on
/// malformed input, wrong key, or tampered ciphertext — never panics.
pub fn decrypt_keyfile_path(
    key: &[u8; 32],
    encrypted: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (nonce_b64, ct_b64) = encrypted
        .split_once(':')
        .ok_or("Invalid encrypted format: missing ':' separator")?;

    let nonce_bytes = BASE64.decode(nonce_b64)?;
    let ciphertext = BASE64.decode(ct_b64)?;

    if nonce_bytes.len() != 12 {
        return Err(format!("Invalid nonce length: {} (expected 12)", nonce_bytes.len()).into());
    }

    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_slice())
        .map_err(|_| "AES-GCM decryption failed (wrong key or corrupted data)")?;

    Ok(String::from_utf8(plaintext)?)
}
