//! Compile a static translation dictionary the browser worker can load
//! at start-up. This solves issue #221 for the demo: the worker cannot
//! reach Wiktionary directly for every word the user types, but it can
//! load a small JSON dictionary that the Rust pipeline has already
//! resolved offline.
//!
//! Run with:
//!
//! ```bash
//! # Optional: populate the cache first so the run is fully offline.
//! FORMAL_AI_LIVE_API=1 cargo run --release --example refresh_translation_cache
//!
//! # Build the dictionary (no live API needed if the cache is warm).
//! cargo run --release --example build_translation_dictionary
//! ```
//!
//! Writes `src/web/translation-dictionary.json`. The schema is:
//!
//! ```json
//! {
//!   "version": 1,
//!   "generated_at": "...",
//!   "by_lemma": {
//!     "en": { "tomato": { "ru": "помидор", "hi": "टमाटर", "zh": "番茄" } },
//!     "ru": { "помидор": { "en": "tomato" } }
//!   },
//!   "aliases": { "ru": { "помидора": "помидор", "помидору": "помидор", ... } }
//! }
//! ```
//!
//! The seed list mirrors `refresh_translation_cache.rs` so every word the
//! demo and tests rely on lands in the dictionary.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use formal_ai::translation::{CachedHttpClient, CurlClient, TranslationPipeline};

// Common-noun seed list — keep in sync with refresh_translation_cache.rs.
const COMMON_ENGLISH_NOUNS: &[&str] = &[
    "apple",
    "tomato",
    "cucumber",
    "potato",
    "carrot",
    "onion",
    "garlic",
    "bread",
    "milk",
    "water",
    "tea",
    "coffee",
    "sugar",
    "salt",
    "cheese",
    "butter",
    "egg",
    "rice",
    "pasta",
    "fish",
    "meat",
    "chicken",
    "beef",
    "pork",
    "lemon",
    "orange",
    "banana",
    "grape",
    "strawberry",
    "peach",
    "pear",
    "watermelon",
    "cabbage",
    "pepper",
    "mushroom",
    "house",
    "car",
    "book",
    "dog",
    "cat",
    "horse",
    "bird",
    "tree",
    "flower",
    "river",
    "mountain",
    "sun",
    "moon",
    "star",
    "sea",
    "city",
    "table",
    "chair",
    "door",
    "window",
    "school",
    "hospital",
    "doctor",
    "teacher",
    "student",
    "child",
    "friend",
    "family",
    "mother",
    "father",
    "brother",
    "sister",
    "computer",
    "phone",
    "music",
    "language",
    "country",
];

const COMMON_RUSSIAN_NOUNS: &[&str] = &[
    "яблоко",
    "помидор",
    "огурец",
    "картофель",
    "морковь",
    "лук",
    "чеснок",
    "хлеб",
    "молоко",
    "вода",
    "чай",
    "кофе",
    "сахар",
    "соль",
    "сыр",
    "масло",
    "яйцо",
    "рис",
    "макароны",
    "рыба",
    "мясо",
    "курица",
    "говядина",
    "свинина",
    "лимон",
    "апельсин",
    "банан",
    "виноград",
    "клубника",
    "персик",
    "груша",
    "арбуз",
    "капуста",
    "перец",
    "гриб",
    "дом",
    "машина",
    "книга",
    "собака",
    "кошка",
    "лошадь",
    "птица",
    "дерево",
    "цветок",
    "река",
    "гора",
    "солнце",
    "луна",
    "звезда",
    "море",
    "город",
    "стол",
    "стул",
    "дверь",
    "окно",
    "школа",
    "больница",
    "врач",
    "учитель",
    "студент",
    "ребёнок",
    "друг",
    "семья",
    "мать",
    "отец",
    "брат",
    "сестра",
    "компьютер",
    "телефон",
    "музыка",
    "язык",
    "страна",
];

const GREETING_PHRASES_RU: &[&str] = &[
    "как у тебя дела",
    "как дела",
    "спасибо",
    "привет",
    "да",
    "нет",
];

const GREETING_PHRASES_EN: &[&str] = &["hello", "thank you", "yes", "no"];

const TARGET_LANGUAGES: &[&str] = &["en", "ru", "hi", "zh"];

type LangMap = BTreeMap<String, String>;
type LemmaMap = BTreeMap<String, LangMap>;
type ByLang = BTreeMap<String, LemmaMap>;
type AliasIndex = BTreeMap<String, BTreeMap<String, String>>;

fn record(by_lang: &mut ByLang, source: &str, surface: &str, target: &str, candidate: &str) {
    let key = surface.to_lowercase();
    by_lang
        .entry(source.to_owned())
        .or_default()
        .entry(key)
        .or_default()
        .insert(target.to_owned(), candidate.to_owned());
}

fn record_alias(aliases: &mut AliasIndex, language: &str, alias: &str, lemma: &str) {
    let alias_normalized = alias.to_lowercase();
    let lemma_normalized = lemma.to_lowercase();
    if alias_normalized == lemma_normalized {
        return;
    }
    aliases
        .entry(language.to_owned())
        .or_default()
        .insert(alias_normalized, lemma_normalized);
}

/// Generate common Russian noun inflections via a deterministic suffix
/// table. Real Wiktionary parsing of inflection templates is huge, so
/// for issue #221 we ship the common 2nd-declension / 3rd-declension
/// case suffixes — enough to cover `помидора`, `огурцом`, `яблоку` etc.
fn russian_inflections(lemma: &str) -> Vec<String> {
    let lemma_lower = lemma.to_lowercase();
    let mut forms: Vec<String> = Vec::new();
    // Strip final vowel to get the stem when applicable.
    let stem_after_strip = |word: &str, suffix: char| -> Option<String> {
        if word.ends_with(suffix) {
            let stem: String = word.chars().take(word.chars().count() - 1).collect();
            Some(stem)
        } else {
            None
        }
    };
    // Words ending in -о (neuter, e.g. яблоко): а, у, ом, е, и, ам, ами, ах
    if let Some(stem) = stem_after_strip(&lemma_lower, 'о') {
        for suffix in ["а", "у", "ом", "е", "и", "ам", "ами", "ах"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
    // Words ending in -а (1st declension feminine, e.g. книга): ы, е, у, ой, ою, и, ам, ами, ах
    if let Some(stem) = stem_after_strip(&lemma_lower, 'а') {
        for suffix in ["ы", "е", "у", "ой", "ою", "и", "ам", "ами", "ах"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
    // Words ending in -я (e.g. семья): и, е, ю, ёй, ёю, ям, ями, ях
    if let Some(stem) = stem_after_strip(&lemma_lower, 'я') {
        for suffix in ["и", "е", "ю", "ёй", "ёю", "ям", "ями", "ях"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
    // Words ending in -ь (3rd declension): и, ью, ей, ям, ями, ях
    if lemma_lower.ends_with('ь') {
        let stem: String = lemma_lower
            .chars()
            .take(lemma_lower.chars().count() - 1)
            .collect();
        for suffix in ["и", "ью", "ей", "ям", "ями", "ях"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
    // Words ending in a consonant (2nd declension masculine, e.g. помидор):
    // а, у, ом, е, ы, ов, ам, ами, ах
    let last_char = lemma_lower.chars().last();
    let ends_in_vowel_or_sign = matches!(
        last_char,
        Some('а' | 'я' | 'о' | 'е' | 'ё' | 'и' | 'ы' | 'у' | 'ю' | 'ь' | 'ъ')
    );
    if !ends_in_vowel_or_sign && last_char.is_some() {
        let stem = &lemma_lower;
        for suffix in ["а", "у", "ом", "е", "ы", "ов", "ам", "ами", "ах"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
    forms
}

/// English inflections: plural -s / -es so `apples`, `tomatoes` resolve.
fn english_inflections(lemma: &str) -> Vec<String> {
    let lemma_lower = lemma.to_lowercase();
    let mut forms: Vec<String> = Vec::new();
    if lemma_lower.ends_with('s')
        || lemma_lower.ends_with('x')
        || lemma_lower.ends_with("sh")
        || lemma_lower.ends_with("ch")
    {
        forms.push(format!("{lemma_lower}es"));
    } else if lemma_lower.ends_with('y') && lemma_lower.chars().count() > 1 {
        let stem: String = lemma_lower
            .chars()
            .take(lemma_lower.chars().count() - 1)
            .collect();
        forms.push(format!("{stem}ies"));
    } else if !lemma_lower.ends_with('s') {
        forms.push(format!("{lemma_lower}s"));
    }
    forms
}

fn main() {
    let cache_dir = std::env::var("FORMAL_AI_TRANSLATION_CACHE_DIR")
        .unwrap_or_else(|_| "data/translation-cache".to_owned());
    let output_path = std::env::var("FORMAL_AI_TRANSLATION_DICTIONARY")
        .unwrap_or_else(|_| "src/web/translation-dictionary.json".to_owned());
    println!("cache_dir   = {cache_dir}");
    println!("output_path = {output_path}");

    let http = CachedHttpClient::new(&cache_dir, CurlClient::default());
    let pipeline = TranslationPipeline::new(&http);

    let mut by_lemma: ByLang = BTreeMap::new();
    let mut aliases: AliasIndex = BTreeMap::new();
    let mut gaps: Vec<String> = Vec::new();

    let mut run_pair = |surface: &str, source: &str| {
        for target in TARGET_LANGUAGES {
            if source == *target {
                continue;
            }
            match pipeline.translate(surface, source, target) {
                Ok(translation) => {
                    if let Some(candidate) = translation.primary_surface() {
                        record(&mut by_lemma, source, surface, target, candidate);
                    } else {
                        gaps.push(format!("{source}->{target} \"{surface}\""));
                    }
                }
                Err(error) => {
                    gaps.push(format!("{source}->{target} \"{surface}\" error={error}"));
                }
            }
        }
    };

    for noun in COMMON_ENGLISH_NOUNS {
        run_pair(noun, "en");
        for form in english_inflections(noun) {
            record_alias(&mut aliases, "en", &form, noun);
        }
    }
    for noun in COMMON_RUSSIAN_NOUNS {
        run_pair(noun, "ru");
        for form in russian_inflections(noun) {
            record_alias(&mut aliases, "ru", &form, noun);
        }
    }
    for phrase in GREETING_PHRASES_RU {
        run_pair(phrase, "ru");
    }
    for phrase in GREETING_PHRASES_EN {
        run_pair(phrase, "en");
    }

    let mut entries_count = 0_usize;
    for lemmas in by_lemma.values() {
        for langs in lemmas.values() {
            entries_count += langs.len();
        }
    }
    let alias_count: usize = aliases.values().map(BTreeMap::len).sum();

    // Hand-render JSON so we don't add serde_json to the runtime deps;
    // the schema is small and stable.
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str("  \"version\": 1,\n");
    writeln!(json, "  \"entries\": {entries_count},").expect("write json");
    writeln!(json, "  \"alias_count\": {alias_count},").expect("write json");
    json.push_str("  \"by_lemma\": {\n");
    let langs: Vec<&String> = by_lemma.keys().collect();
    for (lang_idx, lang) in langs.iter().enumerate() {
        writeln!(json, "    {}: {{", json_string(lang)).expect("write json");
        let lemmas = &by_lemma[*lang];
        let lemma_keys: Vec<&String> = lemmas.keys().collect();
        for (lemma_idx, lemma) in lemma_keys.iter().enumerate() {
            write!(json, "      {}: {{", json_string(lemma)).expect("write json");
            let targets = &lemmas[*lemma];
            let target_keys: Vec<&String> = targets.keys().collect();
            for (target_idx, target_lang) in target_keys.iter().enumerate() {
                write!(
                    json,
                    "{}: {}",
                    json_string(target_lang),
                    json_string(&targets[*target_lang]),
                )
                .expect("write json");
                if target_idx + 1 < target_keys.len() {
                    json.push_str(", ");
                }
            }
            json.push('}');
            if lemma_idx + 1 < lemma_keys.len() {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("    }");
        if lang_idx + 1 < langs.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  },\n");
    json.push_str("  \"aliases\": {\n");
    let alias_langs: Vec<&String> = aliases.keys().collect();
    for (lang_idx, lang) in alias_langs.iter().enumerate() {
        writeln!(json, "    {}: {{", json_string(lang)).expect("write json");
        let pairs = &aliases[*lang];
        let alias_keys: Vec<&String> = pairs.keys().collect();
        for (alias_idx, alias) in alias_keys.iter().enumerate() {
            write!(
                json,
                "      {}: {}",
                json_string(alias),
                json_string(&pairs[*alias]),
            )
            .expect("write json");
            if alias_idx + 1 < alias_keys.len() {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("    }");
        if lang_idx + 1 < alias_langs.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  }\n");
    json.push_str("}\n");

    let path = Path::new(&output_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(path, json).expect("write dictionary");
    println!("wrote {entries_count} entries, {alias_count} aliases -> {output_path}");
    if !gaps.is_empty() {
        eprintln!("\n{} gaps:", gaps.len());
        for gap in &gaps {
            eprintln!("  - {gap}");
        }
        // Note: we deliberately do NOT exit non-zero here. The seed list
        // is intentionally aggressive; a few unresolved entries are
        // acceptable and easier to spot in the dictionary itself.
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                write!(out, "\\u{:04x}", c as u32).expect("write json escape");
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
