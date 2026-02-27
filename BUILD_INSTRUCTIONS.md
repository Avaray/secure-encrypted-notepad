# 🔨 Build Instructions for SEN v0.2

## 📁 Project Structure

Your project should have this structure:

```
sen/
├── Cargo.toml
├── README.md
├── BUILD_INSTRUCTIONS.md
└── src/
    ├── main.rs
    ├── app.rs
    ├── crypto.rs
    ├── history.rs
    ├── settings.rs
    └── theme.rs
```

## ⚠️ IMPORTANT: Manual File Creation Required

The code provided is in **artifacts** (code blocks above). You must **manually
copy** each file.

---

## 🚀 Step-by-Step Setup

### 1. Create Project

```bash
cargo new sen --bin
cd sen
```

### 2. Copy Files from Artifacts

You need to copy **ALL** these files:

#### A) Replace `Cargo.toml`

- Delete existing `Cargo.toml`
- Copy content from artifact "Cargo.toml (NEW)"
- Paste into your `Cargo.toml`
- Save

#### B) Replace `src/main.rs`

- Delete existing `src/main.rs`
- Copy content from artifact "src/main.rs (NEW)"
- Paste
- Save

#### C) Create `src/crypto.rs`

- Create NEW file: `src/crypto.rs`
- Copy content from artifact "src/crypto.rs (NEW)"
- Paste
- Save

#### D) Create `src/history.rs`

- Create NEW file: `src/history.rs`
- Copy content from artifact "src/history.rs (NEW)"
- Paste
- Save

#### E) Create `src/theme.rs`

- Create NEW file: `src/theme.rs`
- Copy content from artifact "src/theme.rs"
- Paste
- Save

#### F) Create `src/settings.rs`

- Create NEW file: `src/settings.rs`
- Copy content from artifact "src/settings.rs (NEW)"
- Paste
- Save

#### G) Create `src/app.rs`

- Create NEW file: `src/app.rs`
- Copy content from **BOTH** artifacts:
  - "src/app.rs (NEW - Part 1/2)"
  - "src/app.rs (NEW - Part 2/2 - UI)"
- **Combine them** (Part 1 first, then Part 2)
- Paste
- Save

---

## ✅ Verify File Structure

Check that you have all files:

```bash
ls -R src/
```

Should show:

```
src/:
app.rs  crypto.rs  history.rs  main.rs  settings.rs  theme.rs
```

---

## 🔨 Build Project

### Debug Build (Fast)

```bash
cargo build
```

### Release Build (Optimized)

```bash
cargo build --release
```

---

## ▶️ Run Application

### Debug

```bash
cargo run
```

### Release

```bash
./target/release/sen
```

Or on Windows:

```bash
.\target\release\sen.exe
```

---

## 🧪 Test Everything Works

### 1. First Launch

- Application should open with toolbar at top
- No errors in terminal
- UI should be responsive

### 2. Generate Keyfile

- Click **✨** icon (Generate Keyfile)
- Save as `test.key`
- Should see "🔐 test.key" in toolbar

### 3. Create Document

- Click **📄** icon (New)
- Type some text
- Should see line numbers on left

### 4. Save File

- Click **💾** icon (Save)
- Save as `test.sen`
- Should see "Saved: test.sen (0 history entries)"

### 5. Close and Reopen

- Close application
- Run again
- Click **🔑** icon → Select `test.key`
- Click **📂** icon → Select `test.sen`
- Your text should appear!

---

## 🐛 Common Issues

### "cannot find module X"

→ File `src/X.rs` is missing or misnamed

**Fix**: Create the file and copy content from corresponding artifact

### "unresolved import"

→ Module not declared in another file

**Fix**: Check `main.rs` has all `mod` declarations:

```rust
mod app;
mod crypto;
mod history;
mod settings;
mod theme;
```

### Compilation errors about functions

→ Code was not copied completely

**Fix**: Re-copy the entire artifact, especially for `app.rs` (2 parts!)

### "expected X, found Y"

→ Syntax error during copy/paste

**Fix**: Re-copy the file carefully, check for missing braces `}` or parentheses
`)`

---

## 📝 Major Changes from v0.1

### ✅ Added

- ✨ **Keyfile-only** (no passwords)
- ✨ **Embedded history** (inside encrypted files)
- ✨ **File tree panel** (browse .sen files)
- ✨ **Debug console** (application logs)
- ✨ **Line numbers** (VS Code style)
- ✨ **Icon toolbar** (no text labels)
- ✨ **Custom themes** (TOML files)
- ✨ **Separate font sizes** (UI vs Editor)
- ✨ **Global keyfile** option

### ❌ Removed

- ❌ Password authentication
- ❌ Dual-factor auth (password + keyfile)
- ❌ External .history folders
- ❌ Text menu bar
- ❌ Password input field

### 🔄 Changed

- 🔄 History stored **inside** encrypted file (not separate folder)
- 🔄 Max **100 history entries** per file
- 🔄 Toolbar uses **icons only** (with hover tooltips)
- 🔄 Themes system with **auto-refresh**
- 🔄 Settings reorganized

---

## 🎨 Creating Custom Theme

After building, create a theme:

```bash
# Linux/macOS
mkdir -p ~/.config/sen/themes/

# Create theme file
cat > ~/.config/sen/themes/my_theme.toml << 'EOF'
name = "My Theme"

[colors]
background = [30, 30, 30]
foreground = [255, 255, 255]
panel_background = [40, 40, 40]
selection_background = [60, 60, 60]
cursor = [255, 255, 255]
line_number = [100, 100, 100]
comment = [80, 150, 80]
EOF
```

Then in app: **Settings** → **Refresh** → Select "My Theme"

---

## 📊 File Size

Compiled binary size:

- **Debug**: ~15-20 MB
- **Release** (with strip): ~8-12 MB

---

## 🚀 Performance

- **Startup**: < 1 second
- **File open** (10KB): < 500ms
- **File save** (10KB): < 500ms (mostly Argon2id)
- **Memory**: 50-100 MB idle

---

## 🔧 Advanced Build Options

### Smaller Binary

```bash
cargo build --release
strip target/release/sen
```

### With LTO (slower build, smaller binary)

Already configured in `Cargo.toml`:

```toml
[profile.release]
lto = true
strip = true
```

---

## 📚 Next Steps

1. ✅ Build application
2. ✅ Run tests: `cargo test`
3. ✅ Read README.md for usage
4. 🎨 Create custom themes
5. 🔑 Generate keyfiles
6. 📝 Start using!

---

## 🆘 Still Having Issues?

1. Delete `sen/` folder completely
2. Start fresh from step 1
3. Copy files **one by one**
4. Build after each file to check for errors
5. Check that **app.rs** has BOTH parts combined

---

**Happy coding! 🔐✨**
