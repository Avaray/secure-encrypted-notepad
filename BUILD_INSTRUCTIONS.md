# 📝 Build Instructions - HOW TO SETUP

## ⚠️ IMPORTANT: Files are NOT created automatically!

The code I provided is in **artifacts** (code blocks above). You need to
**manually copy** each file to your project directory.

---

## Step-by-Step Setup

### 1. Create project structure

```bash
cargo new sed --bin
cd sed
```

This creates:

```
sed/
├── Cargo.toml
└── src/
    └── main.rs
```

### 2. Copy ALL files from artifacts

You need to **manually copy** the content of each artifact to the corresponding
file:

#### A) Replace `Cargo.toml`

- Open the `Cargo.toml` file in your `sed/` directory
- **DELETE all existing content**
- Copy content from artifact "Cargo.toml" (above)
- Paste into your `Cargo.toml`
- Save

#### B) Replace `src/main.rs`

- Open `src/main.rs`
- **DELETE all existing content**
- Copy content from artifact "src/main.rs" (above)
- Paste into your `src/main.rs`
- Save

#### C) Create `src/crypto.rs`

- Create NEW file: `src/crypto.rs`
- Copy content from artifact "src/crypto.rs" (above)
- Paste into your `src/crypto.rs`
- Save

#### D) Create `src/history.rs`

- Create NEW file: `src/history.rs`
- Copy content from artifact "src/history.rs" (above)
- Paste into your `src/history.rs`
- Save

#### E) Create `src/settings.rs`

- Create NEW file: `src/settings.rs`
- Copy content from artifact "src/settings.rs" (above)
- Paste into your `src/settings.rs`
- Save

#### F) Create `src/app.rs`

- Create NEW file: `src/app.rs`
- Copy content from artifact "src/app.rs" (above)
- Paste into your `src/app.rs`
- Save

### 3. Verify file structure

Your project should look like this:

```
sed/
├── Cargo.toml          ✅ Updated with all dependencies
└── src/
    ├── main.rs         ✅ Entry point
    ├── app.rs          ✅ GUI logic (NEW FILE)
    ├── crypto.rs       ✅ Encryption (NEW FILE)
    ├── history.rs      ✅ Version control (NEW FILE)
    └── settings.rs     ✅ User preferences (NEW FILE)
```

### 4. Build the project

```bash
# From sed/ directory:
cargo build --release
```

If you see compilation errors, check:

- ✅ All 5 files created (main.rs, app.rs, crypto.rs, history.rs, settings.rs)
- ✅ Cargo.toml updated with correct dependencies
- ✅ No typos when copying code

### 5. Run the application

```bash
# Linux/macOS:
./target/release/sed

# Windows:
.\target\release\sed.exe
```

---

## ❌ Fixed Issues

### Issue 1: `unresolved module crypto`

**Cause**: File `src/crypto.rs` doesn't exist

**Fix**: Create the file and copy content from artifact

### Issue 2: `use of unresolved module serde_json`

**Cause**: Missing dependency in Cargo.toml

**Fix**: Already fixed - `serde_json = "1.0"` added to Cargo.toml

### Issue 3: `unused mut` warning

**Cause**: Variable declared as `mut` but never modified

**Fix**: Already fixed - changed `let mut style` to proper usage

### Issue 4: `unexpected argument --strip`

**Cause**: `--strip` is not a cargo build argument, it's a Cargo.toml
configuration

**Fix**: Already configured in `Cargo.toml`:

```toml
[profile.release]
strip = true  # This handles stripping
```

Use: `cargo build --release` (without --strip)

---

## 🎉 Success!

If everything compiled successfully, you should see:

```
Compiling sed v3.0.0 (/path/to/sed)
 Finished release [optimized] target(s) in X.XXs
```

Binary location:

- **Linux/macOS**: `target/release/sed`
- **Windows**: `target\release\sed.exe`

---

## 🔧 Troubleshooting

### "cannot find module X"

→ File `src/X.rs` is missing. Create it and copy content from artifact.

### "unresolved import"

→ Check Cargo.toml has all dependencies. Re-copy the entire Cargo.toml content.

### "expected X, found Y"

→ Syntax error during copy. Re-copy the entire file content carefully.

### Still not working?

1. Delete `sed/` folder completely
2. Start again from step 1
3. Copy files **one by one** and verify each

---

## 📚 Next Steps

Once compiled successfully:

1. Read the README.md for usage instructions
2. Generate a keyfile: Security → Generate New Keyfile
3. Set a password
4. Start writing encrypted documents!

**Happy coding! 🔐**
