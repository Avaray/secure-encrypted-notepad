use orion::aead;
use orion::kdf;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use zeroize::{ZeroizeOnDrop, Zeroizing};

/// Magic number for file format verification: "SEN1"
/// SEN format: keyfile hash is inside encrypted payload (not appended in plaintext)
const MAGIC_NUMBER: &[u8; 4] = b"SEN1";

/// Component sizes in file
const MAGIC_SIZE: usize = 4;
const SALT_SIZE: usize = 32;
const KEYFILE_HASH_SIZE: usize = 32;

/// Structure holding keyfile hash with automatic zeroing
#[derive(ZeroizeOnDrop)]
struct KeyfileHash {
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
            CryptoError::InvalidFormat => write!(f, "Invalid file format or corrupted data"),
            CryptoError::InvalidMagicNumber => write!(f, "Invalid magic number (not a SEN file)"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed - check your keyfile"),
            CryptoError::KeyfileError(msg) => write!(f, "Keyfile Error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

/// SECURITY: Hash keyfile content using SHA-256
fn hash_keyfile(keyfile_path: &Path) -> Result<KeyfileHash, CryptoError> {
    let keyfile_content = fs::read(keyfile_path)
        .map_err(|e| CryptoError::KeyfileError(format!("Cannot read keyfile: {}", e)))?;

    if keyfile_content.is_empty() {
        return Err(CryptoError::KeyfileError("Keyfile is empty".to_string()));
    }

    let mut hasher = Sha256::new();
    hasher.update(&keyfile_content);
    let hash_result = hasher.finalize();

    Ok(KeyfileHash::from_slice(&hash_result))
}

/// SECURITY: Derive encryption key from keyfile_hash + salt
fn derive_key_from_keyfile(
    keyfile_hash: &KeyfileHash,
    salt: &[u8],
) -> Result<aead::SecretKey, CryptoError> {
    let kdf_input = Zeroizing::new(keyfile_hash.as_bytes().to_vec());
    let kdf_password = kdf::Password::from_slice(&kdf_input)?;
    let kdf_salt = kdf::Salt::from_slice(salt)?;

    // Derive 32-byte key (256 bits) for XChaCha20-Poly1305
    let derived_key = kdf::derive_key(&kdf_password, &kdf_salt, 3, 19456, 32)?;
    let secret_key = aead::SecretKey::from_slice(derived_key.unprotected_as_bytes())?;

    Ok(secret_key)
}

/// GENERATE RANDOM KEYFILE
pub fn generate_keyfile(output_path: &Path) -> Result<(), CryptoError> {
    let mut keyfile_data = Zeroizing::new(vec![0u8; 256]);
    rand::thread_rng().fill_bytes(&mut keyfile_data);

    fs::write(output_path, &*keyfile_data)
        .map_err(|e| CryptoError::KeyfileError(format!("Cannot write keyfile: {}", e)))?;

    Ok(())
}

/// FILE ENCRYPTION
/// SEN1 Structure: [MAGIC 4B] [SALT 32B] [ENCRYPTED DATA]
/// Encrypted payload: [KEYFILE HASH 32B] [CONTENT BYTES]
/// The keyfile hash is inside the encrypted payload for security
pub fn encrypt_file(
    content: &str,
    keyfile_path: &Path,
    output_path: &Path,
) -> Result<(), CryptoError> {
    // 1. Generate Salt
    let mut salt = [0u8; SALT_SIZE];
    rand::thread_rng().fill_bytes(&mut salt);

    // 2. Hash Keyfile & Derive Key
    let keyfile_hash = hash_keyfile(keyfile_path)?;
    let secret_key = derive_key_from_keyfile(&keyfile_hash, &salt)?;

    // 3. Build plaintext: [keyfile_hash 32B] + [content]
    // Keyfile hash is embedded inside encrypted data for verification
    let content_bytes = content.as_bytes();
    let mut payload = Zeroizing::new(Vec::with_capacity(KEYFILE_HASH_SIZE + content_bytes.len()));
    payload.extend_from_slice(keyfile_hash.as_bytes());
    payload.extend_from_slice(content_bytes);

    // 4. Encrypt
    let ciphertext = aead::seal(&secret_key, &payload)?;

    // 5. Assemble File: [MAGIC] [SALT] [ENCRYPTED DATA]
    let mut file_data = Vec::with_capacity(MAGIC_SIZE + SALT_SIZE + ciphertext.len());
    file_data.extend_from_slice(MAGIC_NUMBER);
    file_data.extend_from_slice(&salt);
    file_data.extend_from_slice(&ciphertext);

    fs::write(output_path, file_data)?;
    Ok(())
}

/// FILE DECRYPTION
/// SEN: keyfile hash is verified from inside the encrypted payload
pub fn decrypt_file(
    keyfile_path: &Path,
    encrypted_file_path: &Path,
) -> Result<String, CryptoError> {
    let file_data = fs::read(encrypted_file_path)?;

    // Basic size check: magic + salt + at least some encrypted data
    if file_data.len() < MAGIC_SIZE + SALT_SIZE + 1 {
        return Err(CryptoError::InvalidFormat);
    }

    // 1. Validate Magic
    if &file_data[0..MAGIC_SIZE] != MAGIC_NUMBER {
        return Err(CryptoError::InvalidMagicNumber);
    }

    // 2. Split components: [MAGIC 4B] [SALT 32B] [ENCRYPTED DATA]
    let salt_end = MAGIC_SIZE + SALT_SIZE;
    let salt = &file_data[MAGIC_SIZE..salt_end];
    let encrypted_data = &file_data[salt_end..];

    // 3. Hash keyfile & derive key
    let keyfile_hash = hash_keyfile(keyfile_path)?;
    let secret_key = derive_key_from_keyfile(&keyfile_hash, salt)?;

    // 4. Decrypt
    let plaintext_bytes =
        aead::open(&secret_key, encrypted_data).map_err(|_| CryptoError::DecryptionFailed)?;
    let plaintext = Zeroizing::new(plaintext_bytes);

    // 5. Verify keyfile hash from decrypted payload: [HASH 32B] [CONTENT]
    if plaintext.len() < KEYFILE_HASH_SIZE {
        return Err(CryptoError::InvalidFormat);
    }

    let stored_hash = &plaintext[..KEYFILE_HASH_SIZE];
    if keyfile_hash.as_bytes() != stored_hash {
        return Err(CryptoError::KeyfileError(
            "Keyfile mismatch - wrong keyfile for this file".to_string(),
        ));
    }

    // 6. Extract content (after the hash)
    let content_bytes = &plaintext[KEYFILE_HASH_SIZE..];
    let content =
        String::from_utf8(content_bytes.to_vec()).map_err(|_| CryptoError::InvalidFormat)?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    /// Helper: create a temp keyfile with given content
    fn create_temp_keyfile(name: &str, content: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join("sen_tests");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    /// Helper: create a random 256-byte keyfile
    fn create_random_keyfile(name: &str) -> PathBuf {
        let mut data = vec![0u8; 256];
        rand::thread_rng().fill_bytes(&mut data);
        create_temp_keyfile(name, &data)
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let keyfile = create_random_keyfile("test_roundtrip.key");
        let dir = std::env::temp_dir().join("sen_tests");
        fs::create_dir_all(&dir).unwrap();
        let output = dir.join("roundtrip.sen");
        let content = "Hello, SEN! This is a test document.\nLine 2.\n";

        encrypt_file(content, &keyfile, &output).expect("Encryption should succeed");

        let decrypted = decrypt_file(&keyfile, &output).expect("Decryption should succeed");
        assert_eq!(decrypted, content);

        // Verify SEN1 magic
        let raw = fs::read(&output).unwrap();
        assert_eq!(&raw[0..4], MAGIC_NUMBER);

        // Verify keyfile hash is NOT at the end of the file (legacy SED2 vulnerability)
        let keyfile_hash = hash_keyfile(&keyfile).unwrap();
        let file_end = &raw[raw.len().saturating_sub(32)..];
        assert_ne!(file_end, keyfile_hash.as_bytes(), "Keyfile hash must NOT be at end of file in plaintext");

        // Cleanup
        let _ = fs::remove_file(&output);
        let _ = fs::remove_file(&keyfile);
    }

    #[test]
    fn test_wrong_keyfile_fails() {
        let keyfile1 = create_random_keyfile("test_wrong_key1.key");
        let keyfile2 = create_random_keyfile("test_wrong_key2.key");
        let dir = std::env::temp_dir().join("sen_tests");
        fs::create_dir_all(&dir).unwrap();
        let output = dir.join("wrong_key.sen");

        encrypt_file("Secret data", &keyfile1, &output).unwrap();

        let result = decrypt_file(&keyfile2, &output);
        assert!(result.is_err(), "Decryption with wrong keyfile should fail");

        // Cleanup
        let _ = fs::remove_file(&output);
        let _ = fs::remove_file(&keyfile1);
        let _ = fs::remove_file(&keyfile2);
    }

    #[test]
    fn test_invalid_magic_number() {
        let keyfile = create_random_keyfile("test_magic.key");
        let dir = std::env::temp_dir().join("sen_tests");
        fs::create_dir_all(&dir).unwrap();
        let output = dir.join("bad_magic.sen");

        // Write a file with wrong magic number
        let mut data = vec![0u8; 100];
        data[0..4].copy_from_slice(b"FAKE");
        fs::write(&output, &data).unwrap();

        let result = decrypt_file(&keyfile, &output);
        assert!(matches!(result, Err(CryptoError::InvalidMagicNumber)));

        // Cleanup
        let _ = fs::remove_file(&output);
        let _ = fs::remove_file(&keyfile);
    }

    #[test]
    fn test_corrupted_file_too_short() {
        let keyfile = create_random_keyfile("test_corrupt.key");
        let dir = std::env::temp_dir().join("sen_tests");
        fs::create_dir_all(&dir).unwrap();
        let output = dir.join("corrupt.sen");

        // Write a file that's too short (just magic + partial salt)
        let mut data = vec![0u8; 10];
        data[0..4].copy_from_slice(MAGIC_NUMBER);
        fs::write(&output, &data).unwrap();

        let result = decrypt_file(&keyfile, &output);
        assert!(result.is_err(), "Truncated file should fail");

        // Cleanup
        let _ = fs::remove_file(&output);
        let _ = fs::remove_file(&keyfile);
    }

    #[test]
    fn test_empty_keyfile_rejected() {
        let keyfile = create_temp_keyfile("test_empty.key", b"");
        let dir = std::env::temp_dir().join("sen_tests");
        fs::create_dir_all(&dir).unwrap();
        let output = dir.join("empty_key.sen");

        let result = encrypt_file("test", &keyfile, &output);
        assert!(matches!(result, Err(CryptoError::KeyfileError(_))));

        // Cleanup
        let _ = fs::remove_file(&keyfile);
    }

    #[test]
    fn test_empty_content_roundtrip() {
        let keyfile = create_random_keyfile("test_empty_content.key");
        let dir = std::env::temp_dir().join("sen_tests");
        fs::create_dir_all(&dir).unwrap();
        let output = dir.join("empty_content.sen");

        encrypt_file("", &keyfile, &output).expect("Encrypting empty content should succeed");
        let decrypted = decrypt_file(&keyfile, &output).expect("Decrypting should succeed");
        assert_eq!(decrypted, "");

        // Cleanup
        let _ = fs::remove_file(&output);
        let _ = fs::remove_file(&keyfile);
    }

    #[test]
    fn test_large_content_roundtrip() {
        let keyfile = create_random_keyfile("test_large.key");
        let dir = std::env::temp_dir().join("sen_tests");
        fs::create_dir_all(&dir).unwrap();
        let output = dir.join("large.sen");
        let content = "A".repeat(100_000); // 100KB content

        encrypt_file(&content, &keyfile, &output).unwrap();
        let decrypted = decrypt_file(&keyfile, &output).unwrap();
        assert_eq!(decrypted, content);

        // Cleanup
        let _ = fs::remove_file(&output);
        let _ = fs::remove_file(&keyfile);
    }
}
