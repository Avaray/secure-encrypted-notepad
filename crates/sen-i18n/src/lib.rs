use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Mutex;

static CURRENT_LOCALE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("en".to_string()));

static TRANSLATIONS: Lazy<HashMap<&'static str, HashMap<&'static str, &'static str>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    macro_rules! load_lang {
        ($code:expr, $file:expr) => {
            map.insert($code, parse_yaml(include_str!($file)));
        };
    }

    load_lang!("ar", "../locales/ar.yml");
    load_lang!("cz", "../locales/cz.yml");
    load_lang!("de", "../locales/de.yml");
    load_lang!("en", "../locales/en.yml");
    load_lang!("es", "../locales/es.yml");
    load_lang!("fr", "../locales/fr.yml");
    load_lang!("it", "../locales/it.yml");
    load_lang!("ja", "../locales/ja.yml");
    load_lang!("nl", "../locales/nl.yml");
    load_lang!("pl", "../locales/pl.yml");
    load_lang!("pt-BR", "../locales/pt-BR.yml");
    load_lang!("ru", "../locales/ru.yml");
    load_lang!("sk", "../locales/sk.yml");
    load_lang!("uk", "../locales/uk.yml");
    load_lang!("zh-CN", "../locales/zh-CN.yml");

    map
});

fn parse_yaml(content: &'static str) -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim();
            let mut val = v.trim();
            if val.starts_with('"') && val.ends_with('"') {
                val = &val[1..val.len() - 1];
            }
            map.insert(key, val);
        }
    }
    map
}

pub fn locale() -> String {
    let lock = CURRENT_LOCALE.lock().unwrap();
    if lock.is_empty() {
        "en".to_string()
    } else {
        lock.clone()
    }
}

pub fn set_locale(loc: &str) {
    if let Ok(mut lock) = CURRENT_LOCALE.lock() {
        *lock = loc.to_string();
    }
}

pub fn _rust_i18n_translate<'r>(locale: &str, key: &'r str) -> Cow<'r, str> {
    let t = &*TRANSLATIONS;
    
    // Try current locale
    if let Some(target_map) = t.get(locale) {
        if let Some(&val) = target_map.get(key) {
            return Cow::Borrowed(val);
        }
    }
    
    // Try base locale (e.g. "pt" if "pt-BR" is requested)
    if let Some(base) = locale.split('-').next() {
        if base != locale {
            if let Some(base_map) = t.get(base) {
                if let Some(&val) = base_map.get(key) {
                    return Cow::Borrowed(val);
                }
            }
        }
    }

    // Fallback to "en"
    if locale != "en" {
        if let Some(en_map) = t.get("en") {
            if let Some(&val) = en_map.get(key) {
                return Cow::Borrowed(val);
            }
        }
    }
    
    // If not found, return key
    Cow::Borrowed(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_all_languages_have_all_keys() {
        let t = &*TRANSLATIONS;
        let en_map = t.get("en").expect("Missing English translations");
        let en_keys: HashSet<_> = en_map.keys().cloned().collect();
        
        let mut missing_keys = Vec::new();

        for (lang, map) in t.iter() {
            if *lang == "en" {
                continue;
            }
            for key in en_keys.iter() {
                if !map.contains_key(key) {
                    missing_keys.push(format!("Language '{}' is missing key '{}'", lang, key));
                }
            }
        }

        if !missing_keys.is_empty() {
            for msg in &missing_keys {
                eprintln!("{}", msg);
            }
            panic!("Missing translations found in {} locations. Run 'bun scripts/i18n-sync.ts' to fix.", missing_keys.len());
        }
    }
}
