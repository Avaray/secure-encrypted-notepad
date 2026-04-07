# SEN Encryption Architecture

## 1. Document Encryption (.sen files)
The primary purpose of SEN is to keep your documents secure using keyfile-only authentication.

### Methods & Algorithms
- **Encryption Algorithm**: `XChaCha20-Poly1305` (via `orion` crate). This is a modern, fast, and extremely secure "Authenticated Encryption with Associated Data" (AEAD) algorithm.
- **Key Derivation (KDF)**: `Argon2id`.
  - **Input**: The `SHA-256` hash of your chosen keyfile + a random 32-byte salt generated per file.
  - **Parameters**: 3 iterations, 19,456 KB (19MB) of memory.
- **Verification**: The `SHA-256` hash of the keyfile is prepended to the plaintext **inside** the encrypted payload. During decryption, after the AEAD tag natively verifies data integrity, the app explicitly checks this internal 32-byte hash against the provided keyfile to guarantee a match before displaying content.

### File Structure
```text
[4-byte Magic: "SEN1"]
[32-byte Random Salt]
[Encrypted Payload (Nonce + Ciphertext + MAC Tag)]
```
Inside the `Encrypted Payload`, the decrypted plaintext is structured sequentially as:
```text
[32-byte Keyfile Hash] + [Composite Document String]
```

#### Composite Document String
The Document String is a UTF-8 string combining the visible text with internal metadata, separated by the text delimiter `\n<>\n`:
```text
[Current Text Content]
\n<>\n
[JSON Serialized HistoryData]
```
The JSON `HistoryData` object contains the document's history state, including structural history (snapshot array), the `max_history_length` configuration, and an optional hidden `autosave` slot.

---

## 2. Configuration Encryption (config.toml)
To improve usability, SEN remembers your global keyfile path and starting directory without exposing them in plaintext on the file system.

### Methods & Algorithms
- **Path Encryption**: Target values in `config.toml` are encrypted with `AES-256-GCM`.
- **Master Key Generation**: A unique 256-bit (32-byte) random AES master key is generated on the first run.
- **Machine Binding & Key Wrapping (HKDF)**: 
  - To prevent attackers from copying your configuration to another machine to decrypt it, the master key is **wrapped (encrypted)** before being saved to disk.
  - A wrapping key is dynamically derived via `HKDF-SHA256` using **machine-specific hardware entropy** (e.g., Windows `MachineGuid`, Linux `/etc/machine-id`, macOS `IOPlatformUUID`), the OS username, and a 16-byte random salt.
  - The master key is encrypted with this wrapping key using `AES-256-GCM` and saved to a discrete file: `<config_dir>/sen/.keyfile_key`.
- **Key File Format**: The contents of `.keyfile_key` on disk are formatted as: `[Version Byte (1)][Salt (16)][Nonce (12)][Wrapped Key + Tag (48)]`.
- **Per-Value Encryption**: Every time a path is updated in `config.toml`, a fresh random 12-byte nonce is generated. The path is then encrypted with the unwrapped master key and stored as `base64_nonce:base64_ciphertext`.

### Encrypted Fields
- `keyfile_path_encrypted`: The absolute path to your active global keyfile.
- `file_tree_dir_encrypted`: The absolute path to the directory currently opened in your file tree.

---

## 3. Memory Security
- **Wiping Secrets**: SEN leverages the `zeroize` crate to ensure cryptographic secrets (keyfile hashes, derived wrapping keys, KDF inputs) are immediately and securely overwritten in RAM as soon as they go out of scope.
- **Zeroizing Payloads**: Sensitive buffers, such as raw file plaintext or intermediate encryption stages, are processed inside `Zeroizing<Vec<u8>>` containers to prevent fragments of decrypted work from lingering in memory sectors or page-files.
