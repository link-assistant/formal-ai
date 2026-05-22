//! Offline translation dictionary.
//!
//! Issue #221 narrowed the offline translation strategy to a single
//! reviewable artefact: a Links Notation file (`data/seed/translations.lino`)
//! that lists the 128 most common nouns we want to translate without
//! hitting the live Wiktionary or Wikidata APIs.
//!
//! The file is embedded via [`include_str!`] so a built binary needs no
//! filesystem access at runtime; the browser worker reads the same file
//! over `fetch()` so the two surfaces share a single source of truth.
//!
//! The format is intentionally narrow — one record per lemma, with a
//! `target "<code>"` block per supported target language and an optional
//! `aliases "<form1>|<form2>|..."` line that enumerates inflected forms
//! (plurals, declensions). See `data/seed/translations.lino` for an
//! example.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The full embedded dictionary text. Shared between the Rust pipeline
/// and the browser worker (which fetches the same file at runtime).
pub const SOURCE_TEXT: &str = include_str!("../../data/seed/translations.lino");

/// One dictionary entry: a lemma in a source language with its
/// translations into one or more target languages.
#[derive(Debug, Clone)]
pub struct DictionaryEntry {
    pub language: String,
    pub lemma: String,
    pub aliases: Vec<String>,
    pub targets: HashMap<String, String>,
}

/// Parsed dictionary, indexed by `(language, surface_lower)` →
/// canonical entry. The same entry is registered under the canonical
/// lemma and every alias so callers can look up inflected forms
/// directly.
///
/// `reverse` mirrors every `(target_lang, target_surface_lower)` back to
/// the owning source entry so that, given an en→ru entry for
/// `thank you → спасибо`, `lookup("спасибо", "ru", "en")` succeeds
/// without a second dictionary entry. This lets the 128-noun cap cover
/// both directions for every lemma instead of just one.
#[derive(Debug, Default)]
pub struct Dictionary {
    entries: HashMap<(String, String), DictionaryEntry>,
    reverse: HashMap<(String, String), DictionaryEntry>,
}

impl Dictionary {
    /// Parse a dictionary from the indented Links Notation text.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut entries: HashMap<(String, String), DictionaryEntry> = HashMap::new();
        let mut reverse: HashMap<(String, String), DictionaryEntry> = HashMap::new();
        let mut current: Option<DictionaryEntry> = None;
        let mut current_target: Option<String> = None;

        for raw_line in text.lines() {
            if raw_line.trim().is_empty() {
                continue;
            }
            let indent = raw_line.bytes().take_while(|b| *b == b' ').count();
            let content = &raw_line[indent..];
            if indent == 0 {
                // Flush previous entry then start a new record header.
                if let Some(entry) = current.take() {
                    register(&mut entries, &mut reverse, &entry);
                }
                if content.starts_with("translation_") {
                    current = Some(DictionaryEntry {
                        language: String::new(),
                        lemma: String::new(),
                        aliases: Vec::new(),
                        targets: HashMap::new(),
                    });
                    current_target = None;
                }
                continue;
            }
            let Some(entry) = current.as_mut() else {
                continue;
            };
            if indent == 2 {
                current_target = None;
                if let Some((name, value)) = parse_kv(content) {
                    match name {
                        "language" => value.clone_into(&mut entry.language),
                        "lemma" => value.clone_into(&mut entry.lemma),
                        "aliases" => {
                            entry.aliases = value
                                .split('|')
                                .map(str::trim)
                                .filter(|alias| !alias.is_empty())
                                .map(str::to_owned)
                                .collect();
                        }
                        "target" => current_target = Some(value.to_owned()),
                        _ => {}
                    }
                }
            } else if indent >= 4 {
                if let (Some(target), Some((name, value))) =
                    (current_target.as_ref(), parse_kv(content))
                {
                    if name == "surface" {
                        entry.targets.insert(target.clone(), value.to_owned());
                    }
                }
            }
        }
        if let Some(entry) = current.take() {
            register(&mut entries, &mut reverse, &entry);
        }
        Self { entries, reverse }
    }

    /// Look up a translation by `(surface, source_lang, target_lang)`.
    ///
    /// Match is case-insensitive on the surface; both canonical lemmas
    /// and the aliases list are consulted. If the source surface is the
    /// target side of an entry registered under a different language
    /// (e.g. the Russian translation of an English lemma), the entry's
    /// source-language lemma is returned in reverse.
    #[must_use]
    pub fn lookup(&self, surface: &str, source: &str, target: &str) -> Option<&str> {
        let key = (source.to_owned(), surface.trim().to_lowercase());
        if let Some(entry) = self.entries.get(&key) {
            if entry.language == source {
                if let Some(value) = entry.targets.get(target) {
                    return Some(value.as_str());
                }
            }
        }
        if let Some(entry) = self.reverse.get(&key) {
            if entry.language == target {
                return Some(entry.lemma.as_str());
            }
            if let Some(value) = entry.targets.get(target) {
                return Some(value.as_str());
            }
        }
        None
    }

    /// Return the canonical lemma for an inflected form, if known. When
    /// the surface is a target-side translation of an entry written in
    /// the other direction, the target lemma is returned so callers can
    /// still cite a canonical form.
    #[must_use]
    pub fn lemma(&self, surface: &str, source: &str) -> Option<&str> {
        let key = (source.to_owned(), surface.trim().to_lowercase());
        if let Some(entry) = self.entries.get(&key) {
            if entry.language == source {
                return Some(entry.lemma.as_str());
            }
        }
        self.reverse.get(&key).and_then(|entry| {
            entry
                .targets
                .get(source)
                .map(String::as_str)
                .or(Some(entry.lemma.as_str()))
        })
    }

    /// Number of distinct entries (counting one per (language, lemma) pair).
    #[must_use]
    pub fn len(&self) -> usize {
        // Each entry is registered once per alias; count unique by the
        // (language, lemma) pair.
        let mut seen = std::collections::HashSet::new();
        for entry in self.entries.values() {
            seen.insert((entry.language.clone(), entry.lemma.clone()));
        }
        seen.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Process-wide dictionary loaded from the embedded `.lino` text.
#[must_use]
pub fn shared() -> &'static Dictionary {
    static DICTIONARY: OnceLock<Dictionary> = OnceLock::new();
    DICTIONARY.get_or_init(|| Dictionary::parse(SOURCE_TEXT))
}

fn register(
    entries: &mut HashMap<(String, String), DictionaryEntry>,
    reverse: &mut HashMap<(String, String), DictionaryEntry>,
    entry: &DictionaryEntry,
) {
    if entry.language.is_empty() || entry.lemma.is_empty() {
        return;
    }
    let lang = &entry.language;
    let lemma = entry.lemma.to_lowercase();
    let mut keys: Vec<String> = entry
        .aliases
        .iter()
        .map(|alias| alias.to_lowercase())
        .collect();
    if !keys.iter().any(|k| k == &lemma) {
        keys.push(lemma);
    }
    for key in keys {
        entries.insert((lang.clone(), key), entry.clone());
    }
    for (target_lang, target_surface) in &entry.targets {
        let key = (target_lang.clone(), target_surface.trim().to_lowercase());
        reverse.entry(key).or_insert_with(|| entry.clone());
    }
}

fn parse_kv(content: &str) -> Option<(&str, &str)> {
    let first_quote = content.find(" \"")?;
    let last_quote = content.rfind('"')?;
    if last_quote <= first_quote + 2 {
        return None;
    }
    let name = content[..first_quote].trim();
    let value = &content[first_quote + 2..last_quote];
    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_dictionary_parses_with_entries() {
        let dict = shared();
        assert!(!dict.is_empty());
        assert!(
            dict.len() <= 128,
            "dictionary must stay under 128 entries (R221.cap)"
        );
    }

    #[test]
    fn english_apple_translates_to_russian() {
        let dict = shared();
        assert_eq!(dict.lookup("apple", "en", "ru"), Some("яблоко"));
    }

    #[test]
    fn russian_pomidor_translates_to_english() {
        let dict = shared();
        assert_eq!(dict.lookup("помидор", "ru", "en"), Some("tomato"));
    }

    #[test]
    fn inflected_form_resolves_via_alias() {
        let dict = shared();
        assert_eq!(dict.lookup("помидоры", "ru", "en"), Some("tomato"));
        assert_eq!(dict.lookup("apples", "en", "ru"), Some("яблоко"));
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let dict = shared();
        assert_eq!(dict.lookup("Apple", "en", "ru"), Some("яблоко"));
        assert_eq!(dict.lookup("APPLE", "en", "ru"), Some("яблоко"));
    }

    #[test]
    fn unknown_surface_returns_none() {
        let dict = shared();
        assert_eq!(dict.lookup("xyzzy", "en", "ru"), None);
    }

    #[test]
    fn parse_kv_extracts_name_and_value() {
        assert_eq!(parse_kv("lemma \"apple\""), Some(("lemma", "apple")));
        assert_eq!(
            parse_kv("aliases \"apple|apples\""),
            Some(("aliases", "apple|apples"))
        );
    }
}
