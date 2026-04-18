//! Config encryption utilities for protecting sensitive paths in config.toml.
//!
//! Uses AES-256-GCM with a key stored in a dedicated file in the config
//! directory.  The key is wrapped (encrypted) using a wrapping key derived
//! from machine-specific entropy via HKDF-SHA256, so copying the key file
//! to a different machine or user account renders it useless.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hkdf::Hkdf;
use sha2::Sha256;

/// Name of the key file stored alongside config.toml
const KEY_FILENAME: &str = ".keyfile_key";

/// Version byte for the wrapped key file format
const KEY_FILE_VERSION: u8 = 0x01;

/// Length of the random salt stored alongside the wrapped key
const SALT_LEN: usize = 16;

/// HKDF info string used during key derivation
const HKDF_INFO: &[u8] = b"sen-config-key-wrap";

// ---------------------------------------------------------------------------
// Machine entropy collection (platform-specific)
// ---------------------------------------------------------------------------

/// Retrieve a machine-unique identifier.
///
/// - **Windows**: `MachineGuid` from the registry
/// - **Linux**: `/etc/machine-id` (fallback `/var/lib/dbus/machine-id`)
/// - **macOS**: `IOPlatformUUID` via `ioreg`
#[cfg(target_os = "windows")]
fn get_machine_id() -> Result<String, Box<dyn std::error::Error>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey("SOFTWARE\\Microsoft\\Cryptography")?;
    let guid: String = key.get_value("MachineGuid")?;
    Ok(guid)
}

#[cfg(target_os = "linux")]
fn get_machine_id() -> Result<String, Box<dyn std::error::Error>> {
    std::fs::read_to_string("/etc/machine-id")
        .or_else(|_| std::fs::read_to_string("/var/lib/dbus/machine-id"))
        .map(|s| s.trim().to_string())
        .map_err(|e| e.into())
}

#[cfg(target_os = "macos")]
fn get_machine_id() -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            if let Some(uuid) = line.split('"').nth(3) {
                return Ok(uuid.to_string());
            }
        }
    }
    Err("Failed to read IOPlatformUUID from ioreg".into())
}

// Fallback for other/unsupported platforms — allows compilation but
// provides no machine binding (equivalent to old behavior).
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn get_machine_id() -> Result<String, Box<dyn std::error::Error>> {
    sen_debug!("config_crypto: WARNING — no machine-ID source for this platform");
    Ok("unknown-platform".to_string())
}

/// Return the current OS username (best-effort).
fn get_username() -> String {
    std::env::var("USERNAME") // Windows
        .or_else(|_| std::env::var("USER")) // Linux / macOS
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Concatenate machine ID and username into a single entropy blob.
fn get_machine_entropy() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let machine_id = get_machine_id()?;
    let username = get_username();

    let mut entropy = Vec::with_capacity(machine_id.len() + 1 + username.len());
    entropy.extend_from_slice(machine_id.as_bytes());
    entropy.push(b':');
    entropy.extend_from_slice(username.as_bytes());
    Ok(entropy)
}

// ---------------------------------------------------------------------------
// Key wrapping / unwrapping helpers
// ---------------------------------------------------------------------------

/// Derive a 32-byte wrapping key from machine entropy + a random salt.
fn derive_wrapping_key(
    entropy: &[u8],
    salt: &[u8],
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), entropy);
    let mut wrapping_key = [0u8; 32];
    hkdf.expand(HKDF_INFO, &mut wrapping_key)
        .map_err(|_| "HKDF expansion failed")?;
    Ok(wrapping_key)
}

/// Wrap (encrypt) a raw 32-byte config key for disk storage.
///
/// On-disk layout: `[VERSION(1)][SALT(16)][NONCE(12)][WRAPPED(48)]` = 77 bytes.
fn wrap_config_key(
    raw_key: &[u8; 32],
    entropy: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let salt: [u8; SALT_LEN] = {
        let mut s = [0u8; SALT_LEN];
        use rand::RngCore;
        rand::rng().fill_bytes(&mut s);
        s
    };

    let wrapping_key = derive_wrapping_key(entropy, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&wrapping_key)?;
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let wrapped = cipher
        .encrypt(&nonce, raw_key.as_slice())
        .map_err(|e| format!("Key wrapping failed: {}", e))?;

    let mut output = Vec::with_capacity(1 + SALT_LEN + 12 + wrapped.len());
    output.push(KEY_FILE_VERSION);
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&wrapped);
    Ok(output)
}

/// Unwrap (decrypt) a config key from the on-disk format.
fn unwrap_config_key(
    file_contents: &[u8],
    entropy: &[u8],
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    // Minimum: 1 (version) + 16 (salt) + 12 (nonce) + 48 (wrapped key + tag)
    if file_contents.len() < 1 + SALT_LEN + 12 + 48 {
        return Err(format!(
            "Key file too short ({} bytes, expected at least 77)",
            file_contents.len()
        )
        .into());
    }
    if file_contents[0] != KEY_FILE_VERSION {
        return Err(format!("Unknown key file version: {}", file_contents[0]).into());
    }

    let salt = &file_contents[1..1 + SALT_LEN];
    let nonce_bytes = &file_contents[1 + SALT_LEN..1 + SALT_LEN + 12];
    let wrapped = &file_contents[1 + SALT_LEN + 12..];

    let wrapping_key = derive_wrapping_key(entropy, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&wrapping_key)?;
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
    let raw_key_vec = cipher
        .decrypt(nonce, wrapped)
        .map_err(|_| "Key unwrapping failed (different machine/user or corrupted key file)")?;

    if raw_key_vec.len() != 32 {
        return Err(format!(
            "Unwrapped key has wrong length: {} (expected 32)",
            raw_key_vec.len()
        )
        .into());
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&raw_key_vec);
    Ok(key)
}

// ---------------------------------------------------------------------------
// Public API (signatures unchanged)
// ---------------------------------------------------------------------------

/// Retrieve or generate a 256-bit AES key stored in a file.
///
/// The key is wrapped with machine-specific entropy so the `.keyfile_key`
/// file is useless on a different machine or user account.
///
/// On first call the key is generated from `OsRng`, wrapped, and written to
/// `<config_dir>/sen/.keyfile_key`.  Subsequent calls read and unwrap the
/// existing key.
///
/// **Legacy migration**: if a raw 32-byte key file is detected (pre-wrapping
/// format), it is automatically re-saved in the new wrapped format.
pub fn get_or_create_config_key() -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir()
        .ok_or("Cannot determine config directory")?
        .join("sen");
    std::fs::create_dir_all(&config_dir)?;

    let key_path = config_dir.join(KEY_FILENAME);
    let entropy = get_machine_entropy()?;

    // Try to load existing key
    if key_path.exists() {
        let contents = std::fs::read(&key_path)?;

        // ── LEGACY FORMAT: raw 32-byte key (no wrapping) ──
        if contents.len() == 32 {
            sen_debug!("config_crypto: detected LEGACY raw key format, migrating...");
            let mut key = [0u8; 32];
            key.copy_from_slice(&contents);

            // Re-save in new wrapped format
            match wrap_config_key(&key, &entropy) {
                Ok(wrapped) => {
                    crate::fs::atomic_write(&key_path, &wrapped)?;
                    sen_debug!(
                        "config_crypto: migrated to wrapped format ({} bytes, fingerprint: {:02x}{:02x}{:02x}{:02x})",
                        wrapped.len(),
                        key[0], key[1], key[2], key[3]
                    );
                }
                Err(e) => {
                    sen_debug!(
                        "config_crypto: WARNING — migration failed ({}), keeping legacy format",
                        e
                    );
                }
            }
            return Ok(key);
        }

        // ── NEW FORMAT: version + salt + nonce + wrapped key ──
        match unwrap_config_key(&contents, &entropy) {
            Ok(key) => {
                sen_debug!(
                    "config_crypto: LOADED wrapped key (fingerprint: {:02x}{:02x}{:02x}{:02x})",
                    key[0],
                    key[1],
                    key[2],
                    key[3]
                );
                return Ok(key);
            }
            Err(e) => {
                sen_debug!("config_crypto: failed to unwrap key ({}), regenerating", e);
                // Fall through to generate a new key
            }
        }
    }

    // Generate a fresh random key
    let key_obj = Aes256Gcm::generate_key(OsRng);
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_obj);

    // Write in wrapped format
    let wrapped = wrap_config_key(&key, &entropy)?;
    crate::fs::atomic_write(&key_path, &wrapped)?;

    sen_debug!(
        "config_crypto: GENERATED NEW wrapped key ({} bytes, fingerprint: {:02x}{:02x}{:02x}{:02x})",
        wrapped.len(),
        key[0], key[1], key[2], key[3]
    );

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_unwrap_roundtrip() {
        let key = [42u8; 32];
        let entropy = b"test-machine-id:test-user";
        let wrapped = wrap_config_key(&key, entropy).unwrap();
        let unwrapped = unwrap_config_key(&wrapped, entropy).unwrap();
        assert_eq!(key, unwrapped);
    }

    #[test]
    fn test_wrong_entropy_fails() {
        let key = [42u8; 32];
        let entropy_a = b"machine-A:user-A";
        let entropy_b = b"machine-B:user-B";
        let wrapped = wrap_config_key(&key, entropy_a).unwrap();
        assert!(
            unwrap_config_key(&wrapped, entropy_b).is_err(),
            "Unwrapping with different entropy should fail"
        );
    }

    #[test]
    fn test_wrapped_file_length() {
        let key = [0u8; 32];
        let entropy = b"test";
        let wrapped = wrap_config_key(&key, entropy).unwrap();
        // 1 (version) + 16 (salt) + 12 (nonce) + 48 (32 key + 16 GCM tag) = 77
        assert_eq!(wrapped.len(), 77);
    }

    #[test]
    fn test_version_byte() {
        let key = [0u8; 32];
        let entropy = b"test";
        let wrapped = wrap_config_key(&key, entropy).unwrap();
        assert_eq!(wrapped[0], KEY_FILE_VERSION);
    }

    #[test]
    fn test_truncated_file_rejected() {
        let entropy = b"test";
        // Too short to be valid
        let short = vec![KEY_FILE_VERSION; 30];
        assert!(unwrap_config_key(&short, entropy).is_err());
    }

    #[test]
    fn test_wrong_version_rejected() {
        let key = [0u8; 32];
        let entropy = b"test";
        let mut wrapped = wrap_config_key(&key, entropy).unwrap();
        wrapped[0] = 0xFF; // corrupt version byte
        assert!(unwrap_config_key(&wrapped, entropy).is_err());
    }

    #[test]
    fn test_encrypt_decrypt_path_roundtrip() {
        let key = [99u8; 32];
        let path = "C:\\Users\\Test\\.keyfile";
        let encrypted = encrypt_keyfile_path(&key, path).unwrap();
        let decrypted = decrypt_keyfile_path(&key, &encrypted).unwrap();
        assert_eq!(path, decrypted);
    }
}
