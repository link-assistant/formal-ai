//! Offline concept knowledge base loaded from `data/seed/concepts.lino`.
//!
//! The records are parsed once at first access from the embedded
//! `CONCEPTS_LINO` string and cached in a `OnceLock` so every interface
//! (Rust solver, CLI, HTTP server, Telegram bot) reads the same data
//! the browser fetches from `src/web/seed/concepts.lino`.
//!
//! Prompt-question prefixes/suffixes (e.g. "what is", "что такое",
//! "X क्या है", "X 是什么") come from `data/seed/prompt-patterns.lino`
//! so the routing rules can be edited without touching this code.

use std::sync::OnceLock;

use crate::seed;

pub use crate::seed::ConceptRecord;

fn concepts() -> &'static [ConceptRecord] {
    static CELL: OnceLock<Vec<ConceptRecord>> = OnceLock::new();
    CELL.get_or_init(seed::concepts).as_slice()
}

fn concept_prefixes() -> &'static [(String, String)] {
    static CELL: OnceLock<Vec<(String, String)>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut prefixes = seed::prompt_patterns()
            .into_iter()
            .filter(|p| p.intent == "concept_lookup" && p.kind == "prefix")
            .map(|p| (p.text.to_lowercase(), p.language))
            .collect::<Vec<_>>();
        prefixes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        prefixes
    })
    .as_slice()
}

fn concept_suffixes() -> &'static [String] {
    static CELL: OnceLock<Vec<String>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut suffixes = seed::prompt_patterns()
            .into_iter()
            .filter(|p| p.intent == "concept_lookup" && p.kind == "suffix")
            .map(|p| p.text)
            .collect::<Vec<_>>();
        suffixes.sort_by(|a, b| b.len().cmp(&a.len()));
        suffixes
    })
    .as_slice()
}

/// Extract the concept term from a "what is X" style prompt. Returns `None`
/// when the prompt does not look like a definition request, which lets the
/// solver fall through to other handlers (greeting, arithmetic, etc.).
///
/// Patterns come from `data/seed/prompt-patterns.lino` (English, Russian,
/// Hindi, Chinese prefixes and suffixes).
pub fn extract_concept_term(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim();
    let trimmed = trimmed
        .trim_end_matches(['?', '。', '.', '!', '!', ',', ',', ';', ':'])
        .trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(body) = strip_suffix_pattern(trimmed) {
        return finalize_concept_body(&body);
    }

    let lower = trimmed.to_lowercase();
    let mut body: Option<&str> = None;
    for (prefix, _language) in concept_prefixes() {
        if let Some(rest) = lower.strip_prefix(prefix.as_str()) {
            let start = trimmed.len() - rest.len();
            body = Some(trimmed[start..].trim());
            break;
        }
    }
    let body = body?;
    finalize_concept_body(body)
}

fn finalize_concept_body(body: &str) -> Option<String> {
    let body = body
        .trim()
        .trim_end_matches(['?', '。', '.', '!', '!', ',', ',', ';', ':'])
        .trim()
        .to_lowercase();
    if body.is_empty() {
        return None;
    }
    let trimmed_body = body
        .strip_suffix(" mean")
        .or_else(|| body.strip_suffix(" stand for"))
        .unwrap_or(&body)
        .trim();
    if trimmed_body.is_empty() {
        return None;
    }
    Some(trimmed_body.to_owned())
}

fn strip_suffix_pattern(input: &str) -> Option<String> {
    for suffix in concept_suffixes() {
        if let Some(rest) = input.strip_suffix(suffix.as_str()) {
            return Some(rest.trim().to_owned());
        }
    }
    None
}

/// Look up a concept by term, alias, or slug. Comparison is case-insensitive
/// and ignores leading articles ("the", "a", "an").
#[must_use]
pub fn lookup_concept(term: &str) -> Option<&'static ConceptRecord> {
    let normalized = normalize_concept_term(term);
    if normalized.is_empty() {
        return None;
    }
    concepts().iter().find(|record| {
        if normalize_concept_term(&record.term) == normalized
            || normalize_concept_term(&record.slug) == normalized
        {
            return true;
        }
        record
            .aliases
            .iter()
            .any(|alias| normalize_concept_term(alias) == normalized)
    })
}

fn normalize_concept_term(value: &str) -> String {
    let lower = value.to_lowercase();
    let mut stripped = lower.as_str();
    for prefix in ["the ", "a ", "an "] {
        if let Some(rest) = stripped.strip_prefix(prefix) {
            stripped = rest;
            break;
        }
    }
    stripped
        .trim()
        .trim_end_matches(['?', '.', '!', ',', ';', ':'])
        .trim()
        .to_owned()
}
