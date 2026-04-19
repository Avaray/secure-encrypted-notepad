# Project Scripts

This directory contains scripts for managing localization and other utility tasks. All scripts are powered by [Bun](https://bun.sh/).

## Requirements
- **Gemini API Key**: Synchronization scripts require the `GEMINI_API_KEY` environment variable to be set. You can obtain your key at [Google AI Studio](https://aistudio.google.com/app/api-keys).

## Scripts List

### `locales-sync.ts`
The primary tool for managing translation files. It can synchronize, re-translate specific keys, and clean up unused keys.

**Capabilities:**
- **AI Translation**: Automatically detects missing keys in target languages and translates them.
- **Forced Sync**: Re-translates specific keys or groups (prefixes like `status.*`) across all languages.
- **Cleanup**: Removes unused translation keys from `en.yml` by scanning the Rust codebase.
- **Targeting**: Can target all languages or a specific one (e.g., `pl.yml`).

---

## Usage Examples

### Synchronization & Translation

**Sync all missing keys:**
```bash
bun run scripts/locales-sync.ts
```

**Force sync specific keys across all languages:**
```bash
bun run scripts/locales-sync.ts status.zen_enabled status.auth_failed
```

**Force sync all keys starting with a prefix (wildcard):**
```bash
bun run scripts/locales-sync.ts status.*
```

**Sync specific language only:**
```bash
bun run scripts/locales-sync.ts pl.yml
```

### Cleanup

**Remove unused keys from en.yml (and other languages):**
```bash
bun run scripts/locales-sync.ts --cleanup
```

**Cleanup and Sync together:**
```bash
bun run scripts/locales-sync.ts --cleanup
```

### Miscellaneous

**Dry run (see what would change without applying):**
```bash
bun run scripts/locales-sync.ts --dry-run
```

**Combine options (e.g., Cleanup + Forced Sync + Dry Run):**
```bash
bun run scripts/locales-sync.ts --cleanup status.* --dry-run
```
