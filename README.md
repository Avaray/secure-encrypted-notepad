# 🔐 SEN - Secure Encrypted Notepad

**SEN** is a local-first text editor designed for security. It uses **keyfile-only authentication** instead of passwords, ensuring that your documents cannot be decrypted without physical access to your unique keyfile.

---

## ✨ Core Features

### 🔒 Security
- **Keyfile-Only Auth:** No passwords to remember or crack. Any file (image, binary, text) can act as your keyfile.
- **XChaCha20-Poly1305:** State-of-the-art authenticated encryption.
- **Argon2id KDF & SHA-256:** Robust key derivation and hashing.
- **Encrypted Global Configuration:** Your settings and global keyfile paths are securely encrypted via OS keychain integration.
- **Automatic Clipboard Clearing:** Sensitive copied text is wiped automatically after a timeout.
- **Zero Memory Leaks:** Cryptographic operations use `zeroize` to wipe secrets from RAM.

### 📝 Seamless Editing
- **Modern Interface:** Distraction-free text editing with line numbers, custom font sizes, and word wrap.
- **Auto-Save & Embedded History:** The editor automatically saves your progress. Every `.sen` file contains its own embedded version history (up to 100 snapshots), allowing you to restore or review older versions of the text.
- **Search & Replace:** Built-in powerful text search and replace capabilities.
- **Batch Converter:** Easily encrypt or decrypt multiple files at once.

### 🎨 Fully Customizable
- **Theme Editor:** Built-in GUI tool to create, edit, and apply custom color themes on the fly.
- **File Tree & Logs:** Integrated file browser and debug log console for power users.

---

## 🚀 How It Works

SEN encrypts everything into a single `.sen` file. The file format securely bundles your text and your version history:

```text
[4-byte magic "SEN"] + [32-byte salt] + [Encrypted Content & History] + [32-byte keyfile hash]
```

To read or write a document, you must provide the exact same keyfile used to create it. You can set a **Global Keyfile** in the settings so you don't have to manually load it every time.

---

## Development

```bash
cargo check
cargo run --release
```

---

## ⚠️ Important Disclaimer

- **Losing your keyfile means permanently losing access to your data.** 
- Always back up your keyfiles securely.
- Use at your own risk.
