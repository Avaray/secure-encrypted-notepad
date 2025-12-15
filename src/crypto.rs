use orion::aead;
use orion::kdf;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use zeroize::{ZeroizeOnDrop, Zeroizing};

/// Magic number for file format verification: "SED2"
const MAGIC_NUMBER: &[u8; 4] = b"SED2";

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
            CryptoError::InvalidFormat => write!(f, "Invalid file format - not a SED file"),
            CryptoError::InvalidMagicNumber => write!(f, "Invalid magic number"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed - wrong keyfile"),
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

/// GENERATE RANDOM KEYFILE (no extension required)
pub fn generate_keyfile(output_path: &Path) -> Result<(), CryptoError> {
    let mut keyfile_data = Zeroizing::new(vec![0u8; 256]);
    rand::thread_rng().fill_bytes(&mut keyfile_data);

    fs::write(output_path, &*keyfile_data)
        .map_err(|e| CryptoError::KeyfileError(format!("Cannot write keyfile: {}", e)))?;

    Ok(())
}

/// FILE ENCRYPTION WITH KEYFILE-ONLY
///
/// File format:
/// [4-byte magic "SED2"]
/// [32-byte salt]
/// [encrypted data with embedded nonce/tag]
/// [32-byte keyfile hash]
pub fn encrypt_file(
    content: &str,
    keyfile_path: &Path,
    output_path: &Path,
) -> Result<(), CryptoError> {
    let keyfile_hash = hash_keyfile(keyfile_path)?;
    let salt = kdf::Salt::default();
    let secret_key = derive_key_from_keyfile(&keyfile_hash, salt.as_ref())?;

    let plaintext = Zeroizing::new(content.as_bytes().to_vec());
    let ciphertext = aead::seal(&secret_key, &plaintext)?;

    let mut file_data =
        Vec::with_capacity(MAGIC_SIZE + SALT_SIZE + ciphertext.len() + KEYFILE_HASH_SIZE);
    file_data.extend_from_slice(MAGIC_NUMBER);
    file_data.extend_from_slice(salt.as_ref());
    file_data.extend_from_slice(&ciphertext);
    file_data.extend_from_slice(keyfile_hash.as_bytes());

    fs::write(output_path, file_data)?;
    Ok(())
}

/// FILE DECRYPTION WITH KEYFILE-ONLY
pub fn decrypt_file(
    keyfile_path: &Path,
    encrypted_file_path: &Path,
) -> Result<String, CryptoError> {
    let file_data = fs::read(encrypted_file_path)?;

    let min_size = MAGIC_SIZE + SALT_SIZE + 40 + KEYFILE_HASH_SIZE; // 40 = min nonce+tag
    if file_data.len() < min_size {
        return Err(CryptoError::InvalidFormat);
    }

    let magic = &file_data[0..MAGIC_SIZE];
    if magic != MAGIC_NUMBER {
        return Err(CryptoError::InvalidMagicNumber);
    }

    let keyfile_hash_start = file_data.len() - KEYFILE_HASH_SIZE;
    let salt_end = MAGIC_SIZE + SALT_SIZE;

    let salt = &file_data[MAGIC_SIZE..salt_end];
    let encrypted_data = &file_data[salt_end..keyfile_hash_start];
    let stored_keyfile_hash = &file_data[keyfile_hash_start..];

    let keyfile_hash = hash_keyfile(keyfile_path)?;
    if keyfile_hash.as_bytes() != stored_keyfile_hash {
        return Err(CryptoError::KeyfileError("Keyfile mismatch".to_string()));
    }

    let secret_key = derive_key_from_keyfile(&keyfile_hash, salt)?;
    let plaintext_bytes =
        aead::open(&secret_key, encrypted_data).map_err(|_| CryptoError::DecryptionFailed)?;

    let plaintext = Zeroizing::new(plaintext_bytes);
    let content = String::from_utf8(plaintext.to_vec()).map_err(|_| CryptoError::InvalidFormat)?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyfile_encrypt_decrypt() {
        let keyfile = Path::new("test.key");
        generate_keyfile(keyfile).unwrap();

        let test_file = Path::new("test.sed");
        let content = "Test content";

        encrypt_file(content, keyfile, test_file).unwrap();
        let decrypted = decrypt_file(keyfile, test_file).unwrap();

        assert_eq!(content, decrypted);

        fs::remove_file(test_file).unwrap();
        fs::remove_file(keyfile).unwrap();
    }
}
