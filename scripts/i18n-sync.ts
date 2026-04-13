import path from "node:path";
import { Glob } from "bun";

// Terminal colors using ANSI escape sequences
const colors = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  cyan: "\x1b[36m",
  blue: "\x1b[34m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  red: "\x1b[31m",
  gray: "\x1b[90m",
};

const LOCALES_DIR = path.resolve(import.meta.dir, "../crates/sen-i18n/locales");
const SOURCE_FILE = path.join(LOCALES_DIR, "en.yml");

const LANGUAGE_MAP: Record<string, string> = {
  ar: "Arabic",
  cz: "Czech",
  de: "German",
  es: "Spanish",
  fr: "French",
  it: "Italian",
  ja: "Japanese",
  nl: "Dutch",
  pl: "Polish",
  "pt-BR": "Portuguese (Brazil)",
  ru: "Russian",
  sk: "Slovak",
  uk: "Ukrainian",
  "zh-CN": "Simplified Chinese",
};

async function translate(texts: Record<string, string>, targetLang: string, apiKey: string): Promise<Record<string, string>> {
  if (Object.keys(texts).length === 0) return {};

  console.log(`${colors.cyan}Translating ${Object.keys(texts).length} keys to ${targetLang}...${colors.reset}`);

  const prompt = `
You are a professional translator for a software application called "SEN (Secure Encrypted Notepad)".
Translate the following English strings into ${targetLang}.
Keep the tone professional and concise.
CRITICAL: Maintain any placeholders exactly as they are (e.g., %{count}, %{file}, %{limit}, %s, %d).
Return the result ONLY as a JSON object where the keys match the input keys.

Input JSON:
${JSON.stringify(texts, null, 2)}
`;

  const url = `https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-latest:generateContent?key=${apiKey}`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      contents: [{ parts: [{ text: prompt }] }],
      generationConfig: {
        responseMimeType: "application/json",
      },
    }),
  });

  if (!response.ok) {
    const err = await response.text();
    throw new Error(`Gemini API error: ${response.status} ${err}`);
  }

  const data = await response.json();
  const textResponse = data.candidates?.[0]?.content?.parts?.[0]?.text;

  if (!textResponse) {
    throw new Error("Empty response from Gemini");
  }

  return JSON.parse(textResponse);
}

async function syncLocale(targetFile: string, dryRun: boolean, apiKey: string) {
  const langCode = path.basename(targetFile, ".yml");
  const langName = LANGUAGE_MAP[langCode] || langCode;

  console.log(`\n${colors.blue}Processing ${colors.bold}${langName}${colors.reset}${colors.blue} (${langCode}.yml)...${colors.reset}`);

  // Load source as text to preserve structure and comments
  const sourceRaw = await Bun.file(SOURCE_FILE).text();
  const sourceLines = sourceRaw.split(/\r?\n/);

  // Load target file as object to check for existing keys
  let targetData: Record<string, string> = {};
  const targetBunFile = Bun.file(targetFile);

  if (await targetBunFile.exists()) {
    const targetRaw = await targetBunFile.text();
    // UŻYCIE WBUDOWANEGO PARSERA YAML Z BUNA:
    targetData = (Bun.YAML.parse(targetRaw) as Record<string, string>) || {};
  }

  const missingKeys: Record<string, string> = {};

  // First pass: identify missing keys
  // Simple regex to extract keys from "key: "value"" or key: 'value' or key: value
  const keyRegex = /^([\w.]+):\s*(?:"(.*)"|'(.*)'|(.*))$/;

  for (const line of sourceLines) {
    const match = line.match(keyRegex);
    if (match) {
      const key = match[1];
      const sourceValue = match[2] || match[3] || match[4] || "";

      if (targetData[key] === undefined) {
        missingKeys[key] = sourceValue;
      }
    }
  }

  let translations: Record<string, string> = {};
  if (Object.keys(missingKeys).length > 0) {
    try {
      translations = await translate(missingKeys, langName, apiKey);

      // Sprawdzenie czy model AI zwrócił tłumaczenia dla wszystkich wymaganych kluczy
      const requestedKeys = Object.keys(missingKeys);
      const returnedKeys = Object.keys(translations);
      const reallyMissingKeys = requestedKeys.filter(k => !returnedKeys.includes(k));

      if (reallyMissingKeys.length > 0) {
        throw new Error(`Model AI pominął niektóre klucze! Brakuje: ${reallyMissingKeys.join(", ")}`);
      }
    } catch (error: any) {
      console.error(`${colors.red}  [X] AI Translation failed: ${error.message}${colors.reset}`);
      throw error; // Rzucamy błąd wyżej, aby zablokować zapis niepełnego pliku
    }
  } else {
    console.log(`${colors.gray}  All keys up to date.${colors.reset}`);
  }

  // Second pass: construct results by mirroring en.yml structure
  const resultLines: string[] = [];
  for (const line of sourceLines) {
    const match = line.match(keyRegex);
    if (match) {
      const key = match[1];
      const value = targetData[key] ?? translations[key] ?? match[2] ?? match[3] ?? match[4] ?? "";

      if (translations[key]) {
        console.log(`${colors.green}  [+] ${key}${colors.reset}`);
      }

      // Safely wrap value in quotes if it's not already
      resultLines.push(`${key}: ${JSON.stringify(value)}`);
    } else {
      // Keep comments and empty lines
      resultLines.push(line);
    }
  }

  if (!dryRun) {
    await Bun.write(targetFile, resultLines.join("\n"));
    console.log(`${colors.green}  Successfully updated ${langCode}.yml${colors.reset}`);
  } else if (Object.keys(translations).length > 0) {
    console.log(`${colors.yellow}  [Dry Run] Would have updated ${langCode}.yml${colors.reset}`);
  }
}

async function main() {
  const GEMINI_API_KEY = Bun.env.GEMINI_API_KEY;

  if (!GEMINI_API_KEY) {
    console.error(`${colors.red}Error: GEMINI_API_KEY is not set in environment variables (or .env file)${colors.reset}`);
    process.exit(1);
  }

  const args = Bun.argv.slice(2);
  const specificFile = args.find(arg => !arg.startsWith("-"));
  const dryRun = args.includes("--dry-run");

  if (dryRun) {
    console.log(`${colors.yellow}DRY RUN: No files will be changed.${colors.reset}`);
  }

  const files: string[] = [];

  if (specificFile) {
    const fileName = specificFile.endsWith(".yml") ? specificFile : `${specificFile}.yml`;
    files.push(path.join(LOCALES_DIR, fileName));
  } else {
    const glob = new Glob("*.yml");
    for await (const file of glob.scan({ cwd: LOCALES_DIR })) {
      if (file !== "en.yml") {
        files.push(path.join(LOCALES_DIR, file));
      }
    }
  }

  let hasErrors = false; // Śledzenie błędów dla całego procesu

  for (const file of files) {
    try {
      await syncLocale(file, dryRun, GEMINI_API_KEY);
    } catch (e: any) {
      console.error(`${colors.red}Failed to sync ${file}: ${e.message}${colors.reset}`);
      hasErrors = true; // Flaga błędu ustawiona na true
    }
  }

  // Wymuszamy zatrzymanie procesu z kodem błędu, jeśli cokolwiek poszło nie tak
  if (hasErrors) {
    console.error(`\n${colors.red}Skrypt zakończony z błędem. Nie wszystkie pliki zostały w pełni przetłumaczone.${colors.reset}`);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error(`${colors.red}Zakończono krytycznym błędem: ${error.message}${colors.reset}`);
  process.exit(1);
});
