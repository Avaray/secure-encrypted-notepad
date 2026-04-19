import path from "node:path";
import { Glob } from "bun";

const CRATES_DIR = path.resolve(import.meta.dir, "../crates");
const LOCALES_DIR = path.join(CRATES_DIR, "sen-i18n/locales");
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

  console.log(`Translating ${Object.keys(texts).length} keys to ${targetLang}...`);

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

  // FIX: Rzutowanie na 'any', ponieważ domyślnie w nowym TS 'json()' może zwracać 'unknown'
  const data = (await response.json()) as any;
  const textResponse = data.candidates?.[0]?.content?.parts?.[0]?.text;

  if (!textResponse) {
    throw new Error("Empty response from Gemini");
  }

  return JSON.parse(textResponse);
}

async function translateBatch(langTexts: Record<string, Record<string, string>>, apiKey: string): Promise<Record<string, Record<string, string>>> {
  console.log(`Translating a batch of keys across multiple languages...`);

  const prompt = `
You are a professional translator for a software application called "SEN (Secure Encrypted Notepad)".
You are given a JSON object where the keys are target languages, and the values are objects containing string keys and English strings to translate.
Translate the English strings into the respective target languages.
Keep the tone professional and concise.
CRITICAL: Maintain any placeholders exactly as they are (e.g., %{count}, %{file}, %{limit}, %s, %d).
Return the result ONLY as a JSON object with the exact same structure (Language -> key -> translated string) where the keys match the input entirely.

Input JSON:
${JSON.stringify(langTexts, null, 2)}
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

  const data = (await response.json()) as any;
  const textResponse = data.candidates?.[0]?.content?.parts?.[0]?.text;

  if (!textResponse) {
    throw new Error("Empty response from Gemini");
  }

  return JSON.parse(textResponse);
}

async function runCleanup(dryRun: boolean) {
  console.log(`Scanning Rust files in ${CRATES_DIR} for unused i18n keys...`);

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

  const sourceBunFile = Bun.file(SOURCE_FILE);
  if (!(await sourceBunFile.exists())) {
    throw new Error(`Source file not found at ${SOURCE_FILE}`);
  }

  const sourceRaw = await sourceBunFile.text();
  const sourceLines = sourceRaw.split(/\r?\n/);
  const keyRegex = /^([\w.]+):\s*(?:"(.*)"|'(.*)'|(.*))$/;

  const resultLines: string[] = [];
  let removedCount = 0;

  for (const line of sourceLines) {
    const match = line.match(keyRegex);
    if (match && match[1]) {
      const key = match[1];
      const isUsed = allRustCode.includes(`"${key}"`);

      if (isUsed) {
        resultLines.push(line);
      } else {
        removedCount++;
        console.log(`  [-] Unused key removed: ${key}`);
      }
    } else {
      resultLines.push(line);
    }
  }

  if (removedCount === 0) {
    console.log("No unused keys found. en.yml is perfectly clean.");
    return;
  }

  console.log(`\nFound ${removedCount} unused keys.`);

  if (!dryRun) {
    await Bun.write(SOURCE_FILE, resultLines.join("\n"));
    console.log(`Successfully updated en.yml.`);
  } else {
    console.log("[Dry Run] Would have updated en.yml.");
  }
}

interface LangTask {
  file: string;
  langCode: string;
  langName: string;
  targetData: Record<string, string>;
  syncKeys: Record<string, string>;
  translations: Record<string, string>;
  hasFailed: boolean;
}

async function prepareTask(targetFile: string, forceKeys?: string[]): Promise<LangTask> {
  const langCode = path.basename(targetFile, ".yml");
  const langName = LANGUAGE_MAP[langCode] || langCode;

  const sourceRaw = await Bun.file(SOURCE_FILE).text();
  const sourceLines = sourceRaw.split(/\r?\n/);

  let targetData: Record<string, string> = {};
  const targetBunFile = Bun.file(targetFile);

  if (await targetBunFile.exists()) {
    const targetRaw = await targetBunFile.text();
    targetData = (Bun.YAML.parse(targetRaw) as Record<string, string>) || {};
  }

  const syncKeys: Record<string, string> = {};
  const keyRegex = /^([\w.]+):\s*(?:"(.*)"|'(.*)'|(.*))$/;

  for (const line of sourceLines) {
    const match = line.match(keyRegex);
    if (match && match[1]) {
      const key = match[1];
      const sourceValue = match[2] || match[3] || match[4] || "";

      if (forceKeys) {
        if (forceKeys.includes(key)) {
          syncKeys[key] = sourceValue;
        }
      } else if (targetData[key] === undefined) {
        syncKeys[key] = sourceValue;
      }
    }
  }

  return { file: targetFile, langCode, langName, targetData, syncKeys, translations: {}, hasFailed: false };
}

async function finalizeTask(task: LangTask, dryRun: boolean) {
  const sourceRaw = await Bun.file(SOURCE_FILE).text();
  const sourceLines = sourceRaw.split(/\r?\n/);
  const keyRegex = /^([\w.]+):\s*(?:"(.*)"|'(.*)'|(.*))$/;

  const resultLines: string[] = [];
  for (const line of sourceLines) {
    const match = line.match(keyRegex);
    if (match && match[1]) {
      const key = match[1];
      // Updated priority: new translations (for forced keys or newly missing) take precedence
      const value = task.translations[key] ?? task.targetData[key] ?? match[2] ?? match[3] ?? match[4] ?? "";

      if (task.translations[key]) {
        console.log(`  [+] ${key}`);
      }

      resultLines.push(`${key}: ${JSON.stringify(value)}`);
    } else {
      resultLines.push(line);
    }
  }

  if (!dryRun) {
    await Bun.write(task.file, resultLines.join("\n"));
    console.log(`  Successfully updated ${task.langCode}.yml`);
  } else if (Object.keys(task.translations).length > 0) {
    console.log(`  [Dry Run] Would have updated ${task.langCode}.yml`);
  }
}

async function main() {
  const GEMINI_API_KEY = Bun.env.GEMINI_API_KEY;

  const args = Bun.argv.slice(2);
  const dryRun = args.includes("--dry-run");
  const cleanup = args.includes("--cleanup");
  const rawArgs = args.filter(arg => !arg.startsWith("-"));

  let targetFiles: string[] = [];
  let userKeys: string[] = [];

  for (const arg of rawArgs) {
    if (arg.endsWith(".yml") || LANGUAGE_MAP[arg]) {
      const fileName = arg.endsWith(".yml") ? arg : `${arg}.yml`;
      targetFiles.push(path.join(LOCALES_DIR, fileName));
    } else if (arg.startsWith("[") && arg.endsWith("]")) {
      try {
        const parsed = JSON.parse(arg);
        if (Array.isArray(parsed)) {
          userKeys.push(...parsed);
        }
      } catch (e) {
        userKeys.push(arg);
      }
    } else {
      userKeys.push(arg);
    }
  }

  if (!GEMINI_API_KEY && !dryRun) {
    console.error(`Error: GEMINI_API_KEY is not set in environment variables`);
    process.exit(1);
  }

  if (dryRun) {
    console.log(`DRY RUN: No files will be changed.`);
  }

  if (cleanup) {
    try {
      await runCleanup(dryRun);
    } catch (e: any) {
      console.error(`Error during cleanup: ${e.message}`);
      process.exit(1);
    }
  }

  // 1. Determine target files
  const files: string[] = [];
  if (targetFiles.length > 0) {
    files.push(...targetFiles);
  } else {
    const glob = new Glob("*.yml");
    for await (const file of glob.scan({ cwd: LOCALES_DIR })) {
      if (file !== "en.yml") {
        files.push(path.join(LOCALES_DIR, file));
      }
    }
  }

  // 2. Expand wildcards if needed
  let finalKeys: string[] | undefined = undefined;
  if (userKeys.length > 0) {
    const sourceRaw = await Bun.file(SOURCE_FILE).text();
    const sourceLines = sourceRaw.split(/\r?\n/);
    const keyRegex = /^([\w.]+):\s*(?:"(.*)"|'(.*)'|(.*))$/;
    const allKeysInSource: string[] = [];
    
    for (const line of sourceLines) {
      const match = line.match(keyRegex);
      if (match && match[1]) allKeysInSource.push(match[1]);
    }

    finalKeys = [];
    for (const uKey of userKeys) {
      if (uKey.endsWith(".*")) {
        const prefix = uKey.slice(0, -1); // e.g. "status."
        const matched = allKeysInSource.filter(k => k.startsWith(prefix));
        finalKeys.push(...matched);
      } else {
        if (allKeysInSource.includes(uKey)) {
          finalKeys.push(uKey);
        } else {
          console.warn(`Warning: Key "${uKey}" not found in en.yml, skipping.`);
        }
      }
    }
    
    if (finalKeys.length === 0) {
      console.error("Error: No valid keys found to synchronize.");
      process.exit(1);
    }
    
    console.log(`Synchronizing ${finalKeys.length} specific keys...`);
  }

  let hasErrors = false;
  const tasks: LangTask[] = [];

  for (const file of files) {
    tasks.push(await prepareTask(file, finalKeys));
  }

  let totalSync = 0;
  let maxSyncPerLang = 0;
  for (const task of tasks) {
    const count = Object.keys(task.syncKeys).length;
    totalSync += count;
    if (count > maxSyncPerLang) {
      maxSyncPerLang = count;
    }
    if (count > 0 && dryRun && !GEMINI_API_KEY) {
      console.error(`  [X] Would update ${count} keys in ${task.langCode}.yml: ${Object.keys(task.syncKeys).join(", ")}`);
      hasErrors = true;
    }
  }

  if (dryRun && !GEMINI_API_KEY) {
    if (hasErrors) {
      process.exit(1);
    }
    return;
  }

  if (totalSync > 0) {
    let batchInput: Record<string, Record<string, string>> = {};

    if (maxSyncPerLang < 20) {
      for (const task of tasks) {
        if (Object.keys(task.syncKeys).length > 0) {
          batchInput[task.langName] = task.syncKeys;
        }
      }

      try {
        const batchResult = await translateBatch(batchInput, GEMINI_API_KEY as string);
        for (const task of tasks) {
          if (Object.keys(task.syncKeys).length > 0) {
            const returnedTranslations = batchResult[task.langName];
            if (!returnedTranslations) {
              console.error(`  [X] AI Batch Translation skipped language: ${task.langName}`);
              task.hasFailed = true;
              hasErrors = true;
              continue;
            }
            
            const requestedKeys = Object.keys(task.syncKeys);
            const returnedKeys = Object.keys(returnedTranslations);
            const reallyMissingKeys = requestedKeys.filter(k => !returnedKeys.includes(k));

            if (reallyMissingKeys.length > 0) {
              console.error(`  [X] AI Batch Translation skipped some keys for ${task.langName}! Missing: ${reallyMissingKeys.join(", ")}`);
              task.hasFailed = true;
              hasErrors = true;
            } else {
              task.translations = returnedTranslations;
            }
          }
        }
      } catch (error: any) {
        console.error(`  [X] Batch AI Translation failed: ${error.message}`);
        hasErrors = true;
        for (const task of tasks) {
          task.hasFailed = true;
        }
      }
    } else {
      for (const task of tasks) {
        if (Object.keys(task.syncKeys).length > 0) {
          try {
            console.log(`\nProcessing ${task.langName} (${task.langCode}.yml)...`);
            const translations = await translate(task.syncKeys, task.langName, GEMINI_API_KEY as string);
            
            const requestedKeys = Object.keys(task.syncKeys);
            const returnedKeys = Object.keys(translations);
            const reallyMissingKeys = requestedKeys.filter(k => !returnedKeys.includes(k));

            if (reallyMissingKeys.length > 0) {
              throw new Error(`The AI model skipped some keys! Missing: ${reallyMissingKeys.join(", ")}`);
            }
            
            task.translations = translations;
          } catch (error: any) {
            console.error(`  [X] AI Translation failed for ${task.langName}: ${error.message}`);
            task.hasFailed = true;
            hasErrors = true;
          }
        }
      }
    }

    for (const task of tasks) {
      if (Object.keys(task.syncKeys).length > 0) {
        if (task.hasFailed) {
          console.log(`\nSkipping ${task.langName} (${task.langCode}.yml) due to translation errors.`);
          continue;
        }
        if (!totalSync || maxSyncPerLang < 20) {
          console.log(`\nProcessing ${task.langName} (${task.langCode}.yml)...`);
        }
        await finalizeTask(task, dryRun);
      } else {
         console.log(`\nProcessing ${task.langName} (${task.langCode}.yml)...`);
         console.log(`  All keys up to date.`);
      }
    }
  } else {
     for (const task of tasks) {
         console.log(`\nProcessing ${task.langName} (${task.langCode}.yml)...`);
         console.log(`  All keys up to date.`);
     }
  }

  if (hasErrors) {
    console.error(`\nScript finished with errors. Not all files were fully translated.`);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error(`Finished with a critical error: ${error.message}`);
  process.exit(1);
});

