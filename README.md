# 🔐 SEN - Secure Encrypted Notepad

**SEN** is a local-first desktop application built in **Rust**, designed for security. It uses **keyfile-only authentication** instead of passwords, ensuring that your documents cannot be decrypted without physical access to your unique keyfile.

![SEN Screenshot](crates/sen-desktop/assets/screenshot.jpg)

🚧 Please note that **SEN** is still in development. While the core functionality is in place, you may encounter bugs or missing features.

---

## 📚 Documentation

For detailed information, check out the following guides in the `/docs` directory:
- [Encryption Architecture](docs/encryption_architecture.md) – Technical details on how your data is protected.
- [Development Guide](docs/development.md) – Instructions on how to set up the environment and build the project.
- [Project TODO](docs/todo.md) – A list of planned features and known issues.

---

## 🖥️ Platform Support

| Platform | Architecture | Target | Keychain Backend |
|----------|-------------|--------|-----------------|
| Windows | x86_64 | `x86_64-pc-windows-msvc` | Windows Credential Manager |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | libsecret / KWallet |
| macOS | x86_64 (Intel) | `x86_64-apple-darwin` | Keychain Access |
| macOS | ARM64 (Apple Silicon) | `aarch64-apple-darwin` | Keychain Access |

> **Portable:** SEN is a single self-contained binary — no installation required. Just download and run.  
> **Linux note:** Requires a running secret service daemon (e.g. `gnome-keyring` or `kwallet`).

---

## ⚠️ Important Disclaimer

- **Losing your keyfile means permanently losing access to your data.** 
- Always back up your keyfiles securely.
- Use at your own risk.
