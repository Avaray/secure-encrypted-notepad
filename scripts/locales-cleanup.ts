import path from "node:path";
import { Glob } from "bun";

// Paths based on your project structure
const CRATES_DIR = path.resolve(import.meta.dir, "../crates");
const LOCALES_DIR = path.join(CRATES_DIR, "sen-i18n/locales");
const SOURCE_FILE = path.join(LOCALES_DIR, "en.yml");

async function main() {
    const args = Bun.argv.slice(2);
    const dryRun = args.includes("--dry-run");

    console.log(`Scanning Rust files in ${CRATES_DIR} for unused i18n keys...`);

    // 1. Gather all Rust files content into one large string for fast searching
    const glob = new Glob("**/*.rs");
    let allRustCode = "";
    let filesScanned = 0;

    for await (const file of glob.scan({ cwd: CRATES_DIR })) {
        const filePath = path.join(CRATES_DIR, file);
        const content = await Bun.file(filePath).text();
        allRustCode += content + "\n";
        filesScanned++;
    }

    console.log(`Successfully scanned ${filesScanned} Rust files.`);

    // 2. Load en.yml
    const sourceBunFile = Bun.file(SOURCE_FILE);
    if (!(await sourceBunFile.exists())) {
        console.error(`Error: Source file not found at ${SOURCE_FILE}`);
        process.exit(1);
    }

    const sourceRaw = await sourceBunFile.text();
    const sourceLines = sourceRaw.split(/\r?\n/);

    // Regex to match "key: value" keeping the original structure from your sync script
    const keyRegex = /^([\w.]+):\s*(?:"(.*)"|'(.*)'|(.*))$/;

    const resultLines: string[] = [];
    let removedCount = 0;
    const removedKeys: string[] = [];

    // 3. Process en.yml line by line
    for (const line of sourceLines) {
        const match = line.match(keyRegex);
        if (match && match[1]) {
            const key = match[1];

            // We look for the exact string literal in Rust code, e.g., "my.translation.key"
            // Wrapping it in quotes prevents partial matches (e.g. matching 'key' in 'key_name')
            const isUsed = allRustCode.includes(`"${key}"`);

            if (isUsed) {
                // Keep the line if the key is found in Rust code
                resultLines.push(line);
            } else {
                // Key is not found, we drop this line
                removedCount++;
                removedKeys.push(key);
                console.log(`  [-] Unused key removed: ${key}`);
            }
        } else {
            // Keep comments and empty lines
            resultLines.push(line);
        }
    }

    if (removedCount === 0) {
        console.log("No unused keys found. en.yml is perfectly clean.");
        return;
    }

    console.log(`\nFound ${removedCount} unused keys.`);

    // 4. Save results
    if (dryRun) {
        console.log("DRY RUN: No files were changed.");
    } else {
        await Bun.write(SOURCE_FILE, resultLines.join("\n"));
        console.log(`Successfully updated en.yml.`);
    }
}

main().catch((error) => {
    console.error(`Finished with a critical error: ${error.message}`);
    process.exit(1);
});