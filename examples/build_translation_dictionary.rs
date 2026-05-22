//! Regenerate `data/seed/translations.lino` — the 128-entry common-noun
//! dictionary shared between the Rust pipeline and the browser worker.
//!
//! Issue #221 narrowed the offline translation strategy: instead of
//! shipping a 3,500-file URL-hash cache or an 826-line JSON dictionary,
//! we keep a single reviewable Links Notation file with the top common
//! nouns and a short list of greeting phrases. The browser worker reads
//! the same file via `fetch()` (no JSON glue layer) so both surfaces
//! agree on contents.
//!
//! Run with:
//!
//! ```bash
//! # Optional: populate the cache first so the run is fully offline.
//! FORMAL_AI_LIVE_API=1 cargo run --release --example refresh_translation_cache
//!
//! # Rebuild the dictionary (no live API needed if the cache is warm).
//! cargo run --release --example build_translation_dictionary
//! ```
//!
//! The seed list mirrors `refresh_translation_cache.rs` so every word the
//! demo and tests rely on lands in the dictionary.
//!
//! Output format (one record per lemma, indented Links Notation):
//!
//! ```text
//! translation_en_apple
//!   language "en"
//!   lemma "apple"
//!   aliases "apple|apples"
//!   target "ru"
//!     surface "яблоко"
//!   target "hi"
//!     surface "सेब"
//!   target "zh"
//!     surface "蘋果"
//! ```
//!
//! The 128-entry cap (R221.cap, enforced by
//! `src/translation/dictionary.rs::tests::embedded_dictionary_parses_with_entries`)
//! keeps the bundle small enough for mobile devices; richer translations
//! fall back to the live Wiktionary/Wikidata APIs at runtime.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use formal_ai::translation::{CachedHttpClient, CurlClient, TranslationPipeline};

// Common-noun seed list — keep in sync with refresh_translation_cache.rs.
// The cap is 128 entries (R221.cap); en + ru combined must not exceed it.
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
];

const GREETING_PHRASES_EN: &[&str] = &["hello", "thank you", "yes", "no"];

const TARGET_LANGUAGES: &[&str] = &["en", "ru", "hi", "zh"];

const DICTIONARY_CAP: usize = 128;

#[derive(Debug, Default, Clone)]
struct Entry {
    language: String,
    lemma: String,
    aliases: Vec<String>,
    targets: BTreeMap<String, String>,
}

fn russian_inflections(lemma: &str) -> Vec<String> {
    let lemma_lower = lemma.to_lowercase();
    let mut forms: Vec<String> = Vec::new();
    let stem_after_strip = |word: &str, suffix: char| -> Option<String> {
        if word.ends_with(suffix) {
            Some(word.chars().take(word.chars().count() - 1).collect())
        } else {
            None
        }
    };
    if let Some(stem) = stem_after_strip(&lemma_lower, 'о') {
        for suffix in ["а", "у", "ом", "е", "и", "ам", "ами", "ах"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
    if let Some(stem) = stem_after_strip(&lemma_lower, 'а') {
        for suffix in ["ы", "е", "у", "ой", "ою", "и", "ам", "ами", "ах"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
    if let Some(stem) = stem_after_strip(&lemma_lower, 'я') {
        for suffix in ["и", "е", "ю", "ёй", "ёю", "ям", "ями", "ях"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
    if lemma_lower.ends_with('ь') {
        let stem: String = lemma_lower
            .chars()
            .take(lemma_lower.chars().count() - 1)
            .collect();
        for suffix in ["и", "ью", "ей", "ям", "ями", "ях"] {
            forms.push(format!("{stem}{suffix}"));
        }
    }
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

fn slug_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == ' ' || ch == '-' || ch == '_' {
            out.push('_');
        }
    }
    out
}

fn record_alias(entry: &mut Entry, alias: &str) {
    let normalized = alias.to_lowercase();
    if normalized.is_empty() {
        return;
    }
    if !entry.aliases.iter().any(|a| a == &normalized) {
        entry.aliases.push(normalized);
    }
}

fn main() {
    let cache_dir = std::env::var("FORMAL_AI_TRANSLATION_CACHE_DIR")
        .unwrap_or_else(|_| formal_ai::translation::cache::DEFAULT_CACHE_DIR.to_owned());
    let output_path = std::env::var("FORMAL_AI_TRANSLATION_DICTIONARY")
        .unwrap_or_else(|_| "data/seed/translations.lino".to_owned());
    println!("cache_dir   = {cache_dir}");
    println!("output_path = {output_path}");

    let http = CachedHttpClient::new(&cache_dir, CurlClient::default());
    let pipeline = TranslationPipeline::new(&http);

    let mut entries: BTreeMap<(String, String), Entry> = BTreeMap::new();
    let mut gaps: Vec<String> = Vec::new();

    let upsert = |entries: &mut BTreeMap<(String, String), Entry>,
                  gaps: &mut Vec<String>,
                  surface: &str,
                  source: &str,
                  aliases: Vec<String>| {
        let entry = entries
            .entry((source.to_owned(), surface.to_lowercase()))
            .or_insert_with(|| Entry {
                language: source.to_owned(),
                lemma: surface.to_owned(),
                aliases: Vec::new(),
                targets: BTreeMap::new(),
            });
        record_alias(entry, surface);
        for alias in aliases {
            record_alias(entry, &alias);
        }
        for target in TARGET_LANGUAGES {
            if source == *target {
                continue;
            }
            match pipeline.translate(surface, source, target) {
                Ok(translation) => {
                    if let Some(candidate) = translation.primary_surface() {
                        entry
                            .targets
                            .insert((*target).to_owned(), candidate.to_owned());
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
        let aliases = english_inflections(noun);
        upsert(&mut entries, &mut gaps, noun, "en", aliases);
    }
    for noun in COMMON_RUSSIAN_NOUNS {
        let aliases = russian_inflections(noun);
        upsert(&mut entries, &mut gaps, noun, "ru", aliases);
    }
    for phrase in GREETING_PHRASES_EN {
        upsert(&mut entries, &mut gaps, phrase, "en", Vec::new());
    }

    assert!(
        entries.len() <= DICTIONARY_CAP,
        "dictionary must stay under {DICTIONARY_CAP} entries (R221.cap); got {}",
        entries.len()
    );

    let mut lino = String::new();
    let mut first = true;
    for entry in entries.values() {
        if entry.targets.is_empty() {
            continue;
        }
        if !first {
            lino.push('\n');
        }
        first = false;
        writeln!(
            lino,
            "translation_{}_{}",
            entry.language,
            slug_segment(&entry.lemma)
        )
        .expect("write lino");
        writeln!(lino, "  language \"{}\"", entry.language).expect("write lino");
        writeln!(lino, "  lemma \"{}\"", entry.lemma).expect("write lino");
        if !entry.aliases.is_empty() {
            writeln!(lino, "  aliases \"{}\"", entry.aliases.join("|")).expect("write lino");
        }
        for (target, surface) in &entry.targets {
            writeln!(lino, "  target \"{target}\"").expect("write lino");
            writeln!(lino, "    surface \"{surface}\"").expect("write lino");
        }
    }

    let line_count = lino.lines().count();
    assert!(
        line_count <= 1500,
        "dictionary file must stay under 1500 lines; got {line_count}"
    );

    let path = Path::new(&output_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(path, lino).expect("write dictionary");
    println!(
        "wrote {} entries ({} lines) -> {output_path}",
        entries.len(),
        line_count,
    );
    if !gaps.is_empty() {
        eprintln!("\n{} gaps:", gaps.len());
        for gap in &gaps {
            eprintln!("  - {gap}");
        }
    }
}
