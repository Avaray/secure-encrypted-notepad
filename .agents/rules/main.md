---
trigger: always_on
---

1. Use English exclusively for all outputs, including chat responses, code comments, and documentation (specifically within the /docs directory and todo.md), regardless of the input language.
2. Use "git ls-files" to get a clean, flat list of all tracked files.
    * Instruction: When the tree view is too large or complex, use this command to provide a noise-free list of all source files currently managed by Git, automatically excluding everything in .gitignore.
    * Command: git ls-files
3. Use "lsd" command (LSDeluxe) to visualize the project structure as a tree while filtering out build artifacts.
   * Instruction: Use the tree view with explicit ignores. Disable icons and colors to minimize token usage and keep the output as clean text.
   * Command: lsd --tree -I target -I node_modules -I build -I .git --icon never --color never
4. Use "fd" command for fast file discovery and workspace mapping, ensuring you filter out unnecessary search results and save tokens.
   * Instruction: Use "fd" in two ways:
     a) To understand the workspace structure without flooding the context, use a depth-limited search (-d 2) on the "crates/" folder.
     b) To find specific files, always exclude "target", "node_modules", and specific build directories.
   * Commands:
     - Workspace Map: fd -d 2 . crates/
     - File Discovery: fd "pattern" -E target -E "node_modules" -E "crates/sen-android/android/app/build"
5. Use "rg" command (ripgrep) to search for text or code patterns, making sure to bypass irrelevant directories to increase search speed and accuracy.
   * Instruction: Exclude the "/target", "scripts/node_modules", and Android build folders. Use single quotes for globs to prevent Bash from interpreting the "!" character.
   * Command: rg "text" -g '!target/*' -g '!scripts/node_modules/*' -g '!crates/sen-android/android/app/build/*'
6. Use Windows-native process termination commands such as taskkill or PowerShell Stop-Process to stop applications; on Windows 10 22H2, do not rely on Linux kill commands.
7. Use "bun" (bun.sh runtime) for scripting tasks when JavaScript or TypeScript is sufficient; only use "python" as a last resort if the task cannot be handled well with bun.
8. If your code changes involve translation files (i18n), add and remove translation entries only in en.yml; do not translate or modify any i18n files other than en.yml.
9. To map source code to specific files, refer to the docs/development.md file. Use the 'Project Structure' section, which contains an indexed list of files and descriptions of their functional roles.
10. For all inquiries regarding encryption architecture, refer to docs/encryption_architecture.md. This file serves as the definitive and most current documentation for the project's cryptographic design and security protocols.
11. Use "cargo tree" to check the dependencies and their versions before suggesting new third-party crates.
12. Run "cargo metadata --no-deps --format-version 1 | jq '.packages[] | {name, version, manifest_path}'" to get a JSON summary of all crates in the workspace. Use this to identify crate locations and their internal dependencies.
14. Always format git commit messages strictly according to the Conventional Commits specification. Use the format "<type>[optional scope]: <description>" for the title with standard types (feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert). The title's description must be written in the imperative mood (e.g., "add feature", not "added feature"), start with a lowercase letter, and must not end with a period. Follow the title with a body containing a short bulleted list of all changes. Always provide the final commit message inside a single code block for easy copying.
