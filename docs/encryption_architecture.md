# SEN Encryption Architecture

## 1. Document Encryption (.sen files)
The primary purpose of SEN is to keep your documents secure using keyfile-only authentication.

### Methods & Algorithms
- **Encryption Algorithm**: `XChaCha20-Poly1305` (via `orion` crate). This is a modern, fast, and extremely secure "Authenticated Encryption with Associated Data" (AEAD) algorithm.
- **Key Derivation (KDF)**: `Argon2id`.
  - **Input**: `SHA-256` hash of your chosen keyfile + a random 32-byte salt.
  - **Parameters**: 3 iterations, 19,456 KB (19MB) of memory.
- **Verification**: The `SHA-256` hash of the keyfile is embedded **inside** the encrypted payload. This allows the app to verify if the correct keyfile was provided during decryption without exposing the hash in plaintext.

### File Structure
```text
[4-byte Magic: "SEN1"]
[32-byte Random Salt]
[Encrypted Payload (Nonce + Ciphertext + Tag)]
  - Inside Encrypted Payload: [32-byte Keyfile Hash] + [Actual Text Content]
```

---

## 2. Configuration Encryption (config.toml)
To improve usability, SEN can remember your global keyfile path and starting directory without storing them in plaintext.

### Methods & Algorithms
- **Encryption Algorithm**: `AES-256-GCM`.
- **Key Management**:
  - A unique 256-bit (32-byte) random key is generated on the first run.
  - This key is stored in a hidden file: `<config_dir>/sen/.keyfile_key`.
- **Encryption Procedure**:
  - Every time a path is saved, a new random 12-byte nonce is generated.
  - The path is encrypted and stored in `config.toml` as `base64_nonce:base64_ciphertext`.

### Encrypted Fields
- `keyfile_path_encrypted`: The full path to your global keyfile.
- `file_tree_dir_encrypted`: The full path to the directory where the file tree starts.

---

## 3. Memory Security
- **Wiping Secrets**: SEN uses the `zeroize` crate to ensure that sensitive data (like keyfile hashes and derived keys) is overwritten in RAM as soon as it is no longer needed.
- **Zeroizing Payloads**: Encrypted payloads are handled using `Zeroizing<Vec<u8>>` to prevent traces from lingering in memory.
