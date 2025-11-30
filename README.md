# 🔐 SED v3.0 - Secure Encrypted Document Editor with Version Control

**Production-ready** cross-platform edytor tekstu z:

- **Dual-factor authentication** (hasło + keyfile)
- **Built-in version control** (jak git dla zaszyfrowanych plików)
- **End-to-end encryption** dla wszystkich danych i historii

Dla Windows, Linux i macOS.

---

## ✨ Nowe w v3.0: Version Control System

### 🎯 Co to oznacza?

- **Każdy Save tworzy snapshot** - nigdy nie stracisz poprzednich wersji
- **Historia jest zaszyfrowana** - tak samo bezpieczna jak główny plik
- **Przeglądaj stare wersje** - zobacz co zmieniłeś tydzień temu
- **Przywróć dowolną wersję** - cofnij błędne zmiany
- **Cleanup automatyczny** - usuń wersje starsze niż X dni

### 📂 Struktura plików

```
document.sed                          # Główny plik (najnowsza wersja)
document.sed.history/                 # Folder z historią
├── version_2025-11-30_09-58-23.sed  # Snapshot z 9:58
├── version_2025-11-30_10-15-42.sed  # Snapshot z 10:15
└── version_2025-11-30_11-30-00.sed  # Snapshot z 11:30
```

**BEZPIECZEŃSTWO**: Wszystkie snapshoty są zaszyfrowane tym samym hasłem +
keyfile!

---

## 🚀 Quick Start

### 1️⃣ Build application

```bash
cargo new sed --bin
cd sed

# Copy files:
# - Cargo.toml
# - src/main.rs
# - src/app.rs
# - src/crypto.rs
# - src/history.rs
# - src/settings.rs

cargo build --release
```

### 2️⃣ First start

```bash
./target/release/sed
```

### 3️⃣ Setup authentication

1. **Security → Generate New Keyfile** → Zapisz jako `my.key`
2. Wpisz hasło w polu "Password"
3. Status zmieni się na "🔐 Ready"

### 4️⃣ Pracuj z dokumentem

1. **Pisz tekst** w edytorze
2. **File → Save** → Wybierz lokalizację (np. `notes.sed`)
3. **✓ Saved** - główny plik + pierwszy snapshot utworzony!

### 5️⃣ Zobacz historię

1. **History → Toggle Panel** → Panel boczny się otworzy
2. Zobaczysz wszystkie wersje z timestampami
3. Kliknij **👁 View** aby zobaczyć starą wersję
4. Kliknij **↩️ Restore** aby przywrócić
5. Kliknij **🗑️ Delete** aby usunąć snapshot

---

## 📖 Version Control - Szczegółowy Guide

### Jak działa auto-snapshot?

**Domyślnie** (jeśli `auto_snapshot_on_save = true`):

```
Edit document → Save → Główny plik zapisany + Snapshot utworzony
```

**Wyłącz w Settings** jeśli chcesz manualne snapshoty.

### Przykładowy workflow

```bash
# Dzień 1 - 9:00 AM
Piszesz: "TODO: Implement feature X"
Save → version_2025-11-30_09-00-00.sed

# Dzień 1 - 2:00 PM  
Dodajesz: "Feature X implemented successfully"
Save → version_2025-11-30_14-00-00.sed

# Dzień 2 - 10:00 AM
Zmieniasz: "Refactored feature X..."
Save → version_2025-12-01_10-00-00.sed

# O nie! Refactoring zepsuł coś!
History → Version z 2:00 PM → Restore
Boom! Wszystko działa znowu 🎉
```

### View (Read-Only Preview)

```
Click: 👁 View
↓
Nowe okno z zawartością wersji (read-only)
↓
Możesz przeczytać ale nie edytować
```

**Use case**: "Co dokładnie zmieniłem między wczoraj a dziś?"

### Restore (Przywróć wersję)

```
Click: ↩️ Restore
↓
Confirmation dialog: "Restore this version?"
↓
Obecna wersja → Backup snapshot (automatically)
↓
Wybrana wersja → Główny plik
↓
Editor reloaded z przywróconą treścią
```

**BEZPIECZEŃSTWO**: Obecna wersja jest automatycznie backupowana przed restore!

### Delete (Usuń snapshot)

```
Click: 🗑️ Delete
↓
Confirmation: "Delete permanently?"
↓
Snapshot usunięty z dysku
```

**⚠️ UWAGA**: To jest trwałe! Nie da się cofnąć!

### Cleanup Old Versions

**Manual**:

```
History → Cleanup Old
↓
Usuwa wersje starsze niż {snapshot_retention_days}
```

**Automatic**: TODO w przyszłej wersji

**Settings**:

```
snapshot_retention_days = 30  # Usuń wersje >30 dni
snapshot_retention_days = 0   # Zachowaj wszystko
```

---

## 🔐 Bezpieczeństwo Version Control

### Czy historia jest zaszyfrowana?

**TAK!** Każdy snapshot używa:

- ✅ Tego samego hasła co główny plik
- ✅ Tego samego keyfile
- ✅ Unikalnego salt per snapshot
- ✅ Unikalnego nonce per snapshot
- ✅ XChaCha20-Poly1305 AEAD

### Co jeśli ktoś ukradnie folder .history/?

**Bezpieczne!** Bez hasła + keyfile:

- ❌ Nie może odszyfrować żadnego snapshotu
- ❌ Nie może zobaczyć treści
- ❌ Nie wie nawet co jest w środku

### Czy mogę mieć różne hasła dla snapshotów?

**NIE.** Wszystkie snapshoty używają tego samego hasła + keyfile co główny plik.

**Dlaczego?**

- Uproszczenie UX (jeden password do wszystkiego)
- Backup/restore działa seamlessly
- Jeśli zmienisz hasło, musisz re-encrypt wszystko

### Czy snapshoty mają separate authentication tags?

**TAK!** Każdy snapshot ma własny 16-byte Poly1305 MAC.

Jeśli snapshot zostanie skorumpowany:

- ✅ Inne snapshoty nadal działają
- ✅ Authentication failuje tylko dla skorumpowanego
- ✅ Nie ma risk cascade corruption

---

## 🎨 Funkcje GUI

### Menu Bar

**File**:

- 📄 New - Nowy dokument
- 📂 Open - Otwórz zaszyfrowany plik
- 💾 Save - Zapisz (+ auto-snapshot jeśli enabled)
- 💾 Save As - Zapisz jako nowy plik
- ❌ Exit - Wyjdź

**Security**:

- 🔑 Select Keyfile - Wybierz istniejący keyfile
- ✨ Generate Keyfile - Wygeneruj nowy (256 random bytes)

**History**:

- 📜 Toggle Panel - Pokaż/ukryj history panel
- 🗑️ Cleanup Old - Usuń stare wersje

**Settings**:

- ⚙️ Preferences - Font, theme, version control options

**Help**:

- ℹ️ About - Info o aplikacji

### History Panel (Togglowany)

**Stats** (góra):

```
Versions: 15    Total: 2.3 MB
```

**Actions**:

- 🗑️ Cleanup Old - Usuń według retention policy
- 🔄 Refresh - Odśwież listę wersji

**Version List** (scrollable):

```
┌─────────────────────────────┐
│ 📅 2025-11-30 14:23:15     │
│ 💾 125.3 KB                 │
│ 💬 "Refactored chapter 3"   │ (optional comment)
│ [👁 View] [↩️ Restore] [🗑️ Delete] │
└─────────────────────────────┘

┌─────────────────────────────┐
│ 📅 2025-11-30 09:15:42     │
│ 💾 118.7 KB                 │
│ [👁 View] [↩️ Restore] [🗑️ Delete] │
└─────────────────────────────┘

... (more versions)
```

### Settings Panel

**Appearance**:

- Theme: 🌙 Dark / ☀️ Light
- Font Size: 8-32 px (slider)
- Font Family: Monospace / Proportional

**Version Control**:

- ✅ Auto-snapshot on save (checkbox)
- Retention: 0-365 days (slider)

**Security**:

- ✅ Remember keyfile path (checkbox)

---

## 🧪 Testing Version Control

### Test 1: Basic Snapshot Creation

```bash
1. Open app
2. Type "Version 1"
3. Save → notes.sed
4. History panel → Should show 1 version
5. Type "Version 2"
6. Save
7. History panel → Should show 2 versions
```

### Test 2: View Old Version

```bash
1. Create 3 versions (content: "V1", "V2", "V3")
2. History → Click "View" on first version
3. Preview window → Should show "V1"
4. Main editor → Still shows "V3"
```

### Test 3: Restore Old Version

```bash
1. Current: "Latest content"
2. History → Select old version with "Original content"
3. Click "Restore" → Confirm
4. Main editor → Should show "Original content"
5. History → Should have NEW snapshot "Backup before restore"
```

### Test 4: Delete Version

```bash
1. History → Select middle version
2. Click "Delete" → Confirm
3. Version removed from list
4. File removed from .history/ folder
```

### Test 5: Cleanup Old Versions

```bash
1. Settings → Retention: 7 days
2. Create versions over multiple days (manual timestamp edit in filenames)
3. History → Cleanup Old
4. Only versions <7 days remain
```

### Test 6: Wrong Password/Keyfile

```bash
1. Create snapshot with password "A" + keyfile "1.key"
2. Try to view with password "B" → Error
3. Try to view with keyfile "2.key" → Error
4. Use correct password + keyfile → Success
```

---

## 📊 Performance & Storage

### Snapshot Size

Każdy snapshot = (Content size) + 108 bytes overhead:

```
4 bytes  - Magic number
24 bytes - Nonce
32 bytes - Salt
N bytes  - Encrypted content
16 bytes - Auth tag
32 bytes - Keyfile hash
```

### Example:

```
Document: 10 KB plain text
↓
Snapshot: ~10.1 KB encrypted
↓
10 versions: ~101 KB total
```

### Storage Optimization Ideas (Future):

- **Compression**: zstd przed szyfrowaniem → 50-70% mniejsze
- **Delta encoding**: Zapisuj tylko różnice między wersjami
- **Deduplication**: Jeśli content identyczny, reuse snapshot

---

## 🔧 Advanced Usage

### Manual Snapshot with Comment (TODO in future)

```rust
// In future version:
create_snapshot(
    &content, 
    &password, 
    &keyfile, 
    &path,
    Some("Before major refactoring".to_string())
);
```

### Export History (TODO)

```bash
# Future feature:
History → Export → encrypted_backup.tar.sed
```

### Diff View (TODO)

```bash
# Future feature:
Select version A + version B → Show diff side-by-side
```

---

## 🐛 Troubleshooting

### "No versions yet. Save to create first snapshot."

- **Przyczyna**: Folder .history/ nie istnieje lub jest pusty
- **Rozwiązanie**: Zapisz dokument - pierwszy snapshot zostanie utworzony

### "Error loading version: Decryption failed"

- **Przyczyna**: Snapshot skorumpowany lub błędne hasło/keyfile
- **Rozwiązanie**: Spróbuj innych snapshotów, sprawdź hasło

### "Cleanup Old: 0 versions deleted"

- **Przyczyna**: Wszystkie wersje są młodsze niż retention_days
- **Rozwiązanie**: Zmniejsz retention_days w Settings lub poczekaj

### Folder .history/ jest ogromny!

- **Przyczyna**: Dużo wersji dużych plików
- **Rozwiązanie**:
  1. History → Cleanup Old
  2. Settings → Zmniejsz retention_days
  3. Manual: Usuń stare snapshoty

### Czy mogę przenieść plik + historię?

**TAK!** Przenieś OBA:

```bash
mv document.sed /new/location/
mv document.sed.history/ /new/location/
```

Aplikacja automatycznie znajdzie historię.

---

## 📋 Changelog

### v3.0.0 (2025-11-30)

- ✨ **NEW**: Built-in version control system
- ✨ **NEW**: Auto-snapshot on save
- ✨ **NEW**: History panel (View/Restore/Delete)
- ✨ **NEW**: Cleanup old versions
- ✨ **NEW**: History stats display
- ✨ **NEW**: Restore confirmation with auto-backup
- ✨ **NEW**: Settings: auto_snapshot, retention_days

### v2.0.0

- ✨ Dual-factor authentication (password + keyfile)
- ✨ Theme switching (Dark/Light)
- ✨ Font customization
- ✨ Settings persistence

### v1.0.0

- 🎉 Initial release
- 🔐 XChaCha20-Poly1305 encryption
- 🔑 Argon2id key derivation

---

## 🔮 Roadmap (Future Versions)

### v3.1 (Near Future)

- [ ] **User comments** dla snapshotów
- [ ] **Diff view** (side-by-side comparison)
- [ ] **Search in history** (full-text search starych wersji)
- [ ] **Export history** jako encrypted archive

### v3.2

- [ ] **Compression** (zstd przed szyfrowaniem)
- [ ] **Delta encoding** (tylko różnice między wersjami)
- [ ] **Automatic cleanup** (scheduled/on-save)

### v4.0 (Major)

- [ ] **Multiple documents** (tabs)
- [ ] **Branching** (git-style branches)
- [ ] **Merge conflicts** resolution
- [ ] **Remote sync** (encrypted cloud backup)

---

## 🛡️ Security Audit Checklist

- ✅ **No unsafe code** (100% safe Rust)
- ✅ **Zeroize all secrets** (password, keyfile hash, keys)
- ✅ **AEAD encryption** (XChaCha20-Poly1305)
- ✅ **KDF high memory** (Argon2id, 19 MiB)
- ✅ **Random nonces/salts** per snapshot
- ✅ **Authentication-first** decryption
- ✅ **Dual-factor auth** (password + keyfile)
- ✅ **History encrypted** (same security as main file)
- ✅ **Separate auth tags** per snapshot
- ✅ **No plaintext leaks** in history
- ✅ **Timestamp format** (RFC3339 compatible)
- ✅ **File format validation** (magic number check)

---

## 💾 Disk Usage Examples

### Small documents (1-10 KB)

```
Main file: 10 KB
10 snapshots: ~100 KB
Total: ~110 KB
```

### Medium documents (100 KB)

```
Main file: 100 KB
30 snapshots: ~3 MB
Total: ~3.1 MB
```

### Large documents (1 MB)

```
Main file: 1 MB
50 snapshots: ~50 MB
Total: ~51 MB
```

**Tip**: Use retention_days to limit disk usage automatically!

---

## 📚 FAQ

### Q: Czy mogę wyłączyć version control?

**A**: Tak! Settings → Untick "Auto-snapshot on save". Możesz dalej tworzyć
snapshoty manualnie.

### Q: Ile snapshotów mogę mieć?

**A**: Bez limitu (poza miejscem na dysku). Ale użyj retention_days dla
auto-cleanup.

### Q: Czy snapshoty są incrementalne?

**A**: Nie (jeszcze). Każdy snapshot to pełna kopia. W v3.2 planujemy delta
encoding.

### Q: Co jeśli zapomnę hasła?

**A**: **Nie da się odzyskać!** To jest celowe - bezpieczeństwo > convenience.

### Q: Czy mogę używać SED jako "encrypted git"?

**A**: Częściowo tak! Masz: snapshoty, timestamps, restore. Brakuje: branching,
merging, diffs.

### Q: Czy snapshoty spowalniają Save?

**A**: Minimalnie. Główne opóźnienie to Argon2id (~500ms-2s), który jest obecny
przy każdym Save niezależnie od snapshotów.

---

## 🤝 Contributing

Pull requests mile widziane!

**Priority areas**:

1. **Diff view** implementation
2. **Compression** support
3. **User comments** dla snapshotów
4. **Search in history**

---

## 📜 License

MIT lub Apache-2.0 (do wyboru)

---

## ⚠️ Disclaimer

**Educational/personal use project.**

Dla **produkcji**:

- ✅ Professional security audit
- ✅ Compliance review
- ✅ Disaster recovery plan
- ✅ User training

**USE AT YOUR OWN RISK**

---

**Stay secure with version control! 🔐📜🚀**
