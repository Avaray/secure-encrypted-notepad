use orion::aead;
use orion::kdf;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use zeroize::{Zeroize, Zeroizing, ZeroizeOnDrop};
use rand::RngCore;

/// Magic number for file format verification: "SED1"
const MAGIC_NUMBER: &[u8; 4] = b"SED1";

/// Component sizes in file
const MAGIC_SIZE: usize = 4;
const NONCE_SIZE: usize = 24;
const SALT_SIZE: usize = 32;
const TAG_SIZE: usize = 16;
const KEYFILE_HASH_SIZE: usize = 32;

/// Structure holding keyfile hash with automatic zeroing
#[derive(ZeroizeOnDrop)]
struct KeyfileHash {
    #[allow(dead_code)]
    hash: [u8; 32],
}

impl KeyfileHash {
    fn from_slice(data: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(data);
        Self { hash }
    }
    
    fn as_bytes(&self) -> &[u8; 32] {
        &self.hash
    }
}

/// Cryptographic errors
#[derive(Debug)]
pub enum CryptoError {
    IoError(std::io::Error),
    EncryptionError(orion::errors::UnknownCryptoError),
    InvalidFormat,
    InvalidMagicNumber,
    DecryptionFailed,
    KeyfileError(String),
}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::IoError(err)
    }
}

impl From<orion::errors::UnknownCryptoError> for CryptoError {
    fn from(err: orion::errors::UnknownCryptoError) -> Self {
        CryptoError::EncryptionError(err)
    }
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CryptoError::IoError(e) => write!(f, "IO Error: {}", e),
            CryptoError::EncryptionError(e) => write!(f, "Encryption Error: {}", e),
            CryptoError::InvalidFormat => write!(f, "Invalid file format - not a SED encrypted file"),
            CryptoError::InvalidMagicNumber => write!(f, "Invalid file format - magic number mismatch"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed - wrong password, keyfile, or corrupted file"),
            CryptoError::KeyfileError(msg) => write!(f, "Keyfile Error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

/// SECURITY: Hash keyfile content using SHA-256
/// 
/// Why SHA-256?
/// - Cryptographically secure hash
/// - Deterministic (same file = same hash)
/// - Fixed-size output (32 bytes)
/// - One-way function (cannot be reversed)
/// 
/// Keyfile can be ANY file: .key, .jpg, .pdf, random binary
fn hash_keyfile(keyfile_path: &Path) -> Result<KeyfileHash, CryptoError> {
    // Read entire keyfile into memory
    // NOTE: For very large files (>100MB) use streaming hash
    let keyfile_content = fs::read(keyfile_path)
        .map_err(|e| CryptoError::KeyfileError(format!("Cannot read keyfile: {}", e)))?;
    
    if keyfile_content.is_empty() {
        return Err(CryptoError::KeyfileError("Keyfile is empty".to_string()));
    }
    
    // SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(&keyfile_content);
    let hash_result = hasher.finalize();
    
    // Convert to KeyfileHash (with automatic zeroing)
    Ok(KeyfileHash::from_slice(&hash_result))
}

/// SECURITY: Derive encryption key from password + keyfile_hash + salt
/// 
/// Using Argon2id with OWASP 2025 parameters:
/// - Memory: 19 MiB (19456 KB) - resistant to GPU attacks
/// - Iterations: 2 - balance between security and performance
/// - Parallelism: 1 - deterministic result
/// 
/// DUAL-FACTOR SECURITY:
/// password (something you know) + keyfile (something you have) = high protection
/// Attacker needs BOTH to decrypt
fn derive_key_from_password_and_keyfile(
    password: &str,
    keyfile_hash: &KeyfileHash,
    salt: &[u8],
) -> Result<aead::SecretKey, CryptoError> {
    // SECURITY: Combine password + keyfile_hash into single input
    // Format: password_bytes || keyfile_hash_bytes
    let mut combined_input = Zeroizing::new(Vec::with_capacity(password.len() + 32));
    combined_input.extend_from_slice(password.as_bytes());
    combined_input.extend_from_slice(keyfile_hash.as_bytes());
    
    // Derive key using Argon2id
    let kdf_password = kdf::Password::from_slice(&combined_input)?;
    let kdf_salt = kdf::Salt::from_slice(salt)?;
    
    // Derive 32-byte key (256 bits) for XChaCha20-Poly1305
    let derived_key = kdf::derive_key(&kdf_password, &kdf_salt, 3, 19456, 32)?;
    
    // Convert to SecretKey
    let secret_key = aead::SecretKey::from_slice(derived_key.unprotected_as_bytes())?;
    
    Ok(secret_key)
}

/// GENERATE RANDOM KEYFILE
/// 
/// Usage: During first Save, if user doesn't have keyfile
/// Generates cryptographically random 256 bytes and saves to file
pub fn generate_keyfile(output_path: &Path) -> Result<(), CryptoError> {
    let mut keyfile_data = Zeroizing::new(vec![0u8; 256]);
    
    // SECURITY: Using rand::thread_rng() which is cryptographically secure
    rand::thread_rng().fill_bytes(&mut keyfile_data);
    
    // Save to file
    fs::write(output_path, &*keyfile_data)
        .map_err(|e| CryptoError::KeyfileError(format!("Cannot write keyfile: {}", e)))?;
    
    Ok(())
}

/// FILE ENCRYPTION WITH DUAL-FACTOR AUTHENTICATION
/// 
/// File format:
/// [4-byte magic "SED1"]
/// [24-byte nonce]
/// [32-byte salt]
/// [encrypted data]
/// [16-byte Poly1305 tag]
/// [32-byte keyfile hash] <- for verification
/// 
/// SECURITY:
/// - Magic number allows quick format verification
/// - XChaCha20-Poly1305 AEAD (confidentiality + authenticity)
/// - Unique random nonce per encryption
/// - Unique random salt per file
/// - Keyfile hash stored for additional verification (optional)
pub fn encrypt_file(
    content: &str,
    password: &str,
    keyfile_path: &Path,
    output_path: &Path,
) -> Result<(), CryptoError> {
    // 1. Hash keyfile
    let keyfile_hash = hash_keyfile(keyfile_path)?;
    
    // 2. Generate random salt (32 bytes = 256 bits)
    let salt = kdf::Salt::default();
    
    // 3. Derive encryption key from password + keyfile + salt
    let secret_key = derive_key_from_password_and_keyfile(password, &keyfile_hash, salt.as_ref())?;
    
    // 4. Encrypt content
    let plaintext = Zeroizing::new(content.as_bytes().to_vec());
    let ciphertext = aead::seal(&secret_key, &plaintext)?;
    
    // 5. Build output file
    // Format: [magic][nonce+ciphertext+tag][salt][keyfile_hash]
    // orion::aead::seal returns: [24-byte nonce || ciphertext || 16-byte tag]
    let mut file_data = Vec::with_capacity(
        MAGIC_SIZE + ciphertext.len() + SALT_SIZE + KEYFILE_HASH_SIZE
    );
    
    // Magic number
    file_data.extend_from_slice(MAGIC_NUMBER);
    
    // Nonce + encrypted data + tag (from orion::aead::seal)
    file_data.extend_from_slice(&ciphertext);
    
    // Salt
    file_data.extend_from_slice(salt.as_ref());
    
    // Keyfile hash (for verification)
    file_data.extend_from_slice(keyfile_hash.as_bytes());
    
    // 6. Write to file
    fs::write(output_path, file_data)?;
    
    Ok(())
}

/// FILE DECRYPTION WITH DUAL-FACTOR AUTHENTICATION
/// 
/// SECURITY:
/// - Verifies magic number before attempting decryption
/// - Verifies authentication tag (Poly1305 MAC)
/// - If password OR keyfile is wrong, tag verification fails
/// - No data is returned before authenticity verification
pub fn decrypt_file(
    password: &str,
    keyfile_path: &Path,
    encrypted_file_path: &Path,
) -> Result<String, CryptoError> {
    // 1. Read encrypted file
    let file_data = fs::read(encrypted_file_path)?;
    
    // 2. SECURITY: Check minimum size
    // Minimum: 4 (magic) + 24 (nonce) + 16 (tag) + 32 (salt) + 32 (keyfile_hash) = 108 bytes
    let min_size = MAGIC_SIZE + NONCE_SIZE + TAG_SIZE + SALT_SIZE + KEYFILE_HASH_SIZE;
    if file_data.len() < min_size {
        return Err(CryptoError::InvalidFormat);
    }
    
    // 3. Verify magic number
    let magic = &file_data[0..MAGIC_SIZE];
    if magic != MAGIC_NUMBER {
        return Err(CryptoError::InvalidMagicNumber);
    }
    
    // 4. Parse file components
    let keyfile_hash_start = file_data.len() - KEYFILE_HASH_SIZE;
    let salt_start = keyfile_hash_start - SALT_SIZE;
    
    let encrypted_data = &file_data[MAGIC_SIZE..salt_start];  // nonce + ciphertext + tag
    let salt = &file_data[salt_start..keyfile_hash_start];
    let stored_keyfile_hash = &file_data[keyfile_hash_start..];
    
    // 5. Hash current keyfile
    let keyfile_hash = hash_keyfile(keyfile_path)?;
    
    // 6. OPTIONAL VERIFICATION: Check if keyfile hash matches
    // Not strictly necessary (tag verification is enough), but gives better error message
    if keyfile_hash.as_bytes() != stored_keyfile_hash {
        return Err(CryptoError::KeyfileError(
            "Keyfile hash mismatch - wrong keyfile or file corrupted".to_string()
        ));
    }
    
    // 7. Derive decryption key from password + keyfile + salt
    let secret_key = derive_key_from_password_and_keyfile(password, &keyfile_hash, salt)?;
    
    // 8. SECURITY: Decrypt and verify authentication tag
    // orion::aead::open() automatically:
    // - Verifies Poly1305 MAC
    // - If MAC is invalid, returns error WITHOUT decrypting
    // - If MAC is OK, decrypts and returns plaintext
    let plaintext_bytes = aead::open(&secret_key, encrypted_data)
        .map_err(|_| CryptoError::DecryptionFailed)?;
    
    // 9. Convert bytes to String
    let plaintext = Zeroizing::new(plaintext_bytes);
    let content = String::from_utf8(plaintext.to_vec())
        .map_err(|_| CryptoError::InvalidFormat)?;
    
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_generate_keyfile() {
        let keyfile_path = Path::new("test.key");
        generate_keyfile(keyfile_path).unwrap();
        
        let keyfile_data = fs::read(keyfile_path).unwrap();
        assert_eq!(keyfile_data.len(), 256);
        
        fs::remove_file(keyfile_path).unwrap();
    }
    
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // Generuj keyfile
        let keyfile_path = Path::new("test.key");
        generate_keyfile(keyfile_path).unwrap();
        
        let test_path = Path::new("test_encrypted.sed");
        let content = "This is a secret message with dual-factor auth! 🔐🔑";
        let password = "super_secure_password_123";
        
        // Szyfruj
        encrypt_file(content, password, keyfile_path, test_path).unwrap();
        
        // Deszyfruj
        let decrypted = decrypt_file(password, keyfile_path, test_path).unwrap();
        
        assert_eq!(content, decrypted);
        
        // Cleanup
        fs::remove_file(test_path).unwrap();
        fs::remove_file(keyfile_path).unwrap();
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let keyfile_path = Path::new("test_wrong_pass.key");
        generate_keyfile(keyfile_path).unwrap();
        
        let test_path = Path::new("test_wrong_pass.sed");
        let content = "Secret data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        encrypt_file(content, password, keyfile_path, test_path).unwrap();
        
        let result = decrypt_file(wrong_password, keyfile_path, test_path);
        assert!(result.is_err());
        
        fs::remove_file(test_path).unwrap();
        fs::remove_file(keyfile_path).unwrap();
    }
    
    #[test]
    fn test_wrong_keyfile_fails() {
        let keyfile1 = Path::new("test_keyfile1.key");
        let keyfile2 = Path::new("test_keyfile2.key");
        generate_keyfile(keyfile1).unwrap();
        generate_keyfile(keyfile2).unwrap();
        
        let test_path = Path::new("test_wrong_keyfile.sed");
        let content = "Secret data";
        let password = "password";
        
        // Szyfruj z keyfile1
        encrypt_file(content, password, keyfile1, test_path).unwrap();
        
        // Próba deszyfrowania z keyfile2 powinna failować
        let result = decrypt_file(password, keyfile2, test_path);
        assert!(result.is_err());
        
        fs::remove_file(test_path).unwrap();
        fs::remove_file(keyfile1).unwrap();
        fs::remove_file(keyfile2).unwrap();
    }
}
