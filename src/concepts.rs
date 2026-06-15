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
//!
//! Per-language context delimiters (e.g. " in ", " в ", " में ", "中") are
//! loaded from the same file under the `context_delimiter` pattern kind.
//! They let a query like "what is IIR in ML" split into a concept term
//! (`iir`) and a context term (`ml`); the ranker then prefers a record whose
//! `contexts` list contains the parsed context.

use std::sync::OnceLock;

use crate::seed;

pub use crate::seed::{ConceptRecord, ContextRecord};

fn concepts() -> &'static [ConceptRecord] {
    static CELL: OnceLock<Vec<ConceptRecord>> = OnceLock::new();
    CELL.get_or_init(seed::concepts).as_slice()
}

fn concept_contexts() -> &'static [ContextRecord] {
    static CELL: OnceLock<Vec<ContextRecord>> = OnceLock::new();
    CELL.get_or_init(seed::concept_contexts).as_slice()
}

/// Resolve a free-form context phrase (e.g. "ml", "машинное обучение") to a
/// registered [`ContextRecord`] via alias or localized-label match. Returns
/// `None` when no registry record claims the phrase.
#[must_use]
pub fn resolve_context_label(raw_context: &str) -> Option<&'static ContextRecord> {
    let normalized = normalize_concept_term(raw_context);
    if normalized.is_empty() {
        return None;
    }
    concept_contexts()
        .iter()
        .find(|record| record.matches(&normalized))
}

fn concept_prefixes() -> &'static [(String, String)] {
    static CELL: OnceLock<Vec<(String, String)>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut prefixes = seed::prompt_patterns()
            .into_iter()
            .filter(|p| p.intent == "concept_lookup" && p.kind == "prefix")
            .map(|p| (p.text.to_lowercase(), p.language))
            .collect::<Vec<_>>();
        prefixes.sort_by_key(|p| std::cmp::Reverse(p.0.len()));
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
        suffixes.sort_by_key(|s| std::cmp::Reverse(s.len()));
        suffixes
    })
    .as_slice()
}

fn concept_context_delimiters() -> &'static [String] {
    static CELL: OnceLock<Vec<String>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut delimiters = seed::prompt_patterns()
            .into_iter()
            .filter(|p| p.intent == "concept_lookup" && p.kind == "context_delimiter")
            .map(|p| p.text)
            .collect::<Vec<_>>();
        delimiters.sort_by_key(|d| std::cmp::Reverse(d.len()));
        delimiters
    })
    .as_slice()
}

/// Outcome of parsing a "what is X" style prompt.
///
/// `term` is the concept candidate; `context`, when present, is the
/// disambiguating context phrase the user appended via a language-specific
/// delimiter (" in ", " в ", " में ", "中", ...). Both strings are
/// lowercased and trimmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptQuery {
    pub term: String,
    pub context: Option<String>,
}

/// Extract a `(concept, optional context)` pair from a "what is X" style
/// prompt. Returns `None` when the prompt does not look like a definition
/// request, which lets the solver fall through to other handlers (greeting,
/// arithmetic, etc.).
///
/// Patterns come from `data/seed/prompt-patterns.lino` (English, Russian,
/// Hindi, Chinese prefixes, suffixes, and context delimiters).
pub fn extract_concept_query(prompt: &str) -> Option<ConceptQuery> {
    let trimmed = prompt.trim();
    let trimmed = trimmed
        .trim_end_matches(['?', '。', '.', '!', '!', ',', ',', ';', ':'])
        .trim();
    if trimmed.is_empty() {
        return None;
    }
    let trimmed = strip_leading_request(trimmed);

    if let Some(body) = strip_suffix_pattern(trimmed) {
        return finalize_concept_query(&body);
    }

    let lower = trimmed.to_lowercase();
    if let Some(body) = strip_meaning_question_body(trimmed, &lower) {
        return finalize_concept_query(body);
    }

    if let Some(body) = strip_inverted_who_is(trimmed, &lower) {
        return finalize_concept_query(body);
    }

    let mut body: Option<&str> = None;
    for (prefix, _language) in concept_prefixes() {
        if let Some(rest) = lower.strip_prefix(prefix.as_str()) {
            let start = trimmed.len() - rest.len();
            body = Some(trimmed[start..].trim());
            break;
        }
    }
    let body = body?;
    finalize_concept_query(body)
}

fn strip_leading_request(input: &str) -> &str {
    const REQUEST_PREFIXES: &[&str] = &["please tell me,", "please tell me", "tell me,", "tell me"];
    const QUESTION_STARTS: &[&str] = &["who ", "what ", "what's ", "who's "];
    let lower = input.to_lowercase();
    for prefix in REQUEST_PREFIXES {
        let Some(rest_lower) = lower.strip_prefix(prefix) else {
            continue;
        };
        let rest_start = input.len() - rest_lower.len();
        let rest = input[rest_start..].trim_start();
        let rest_lower = rest.to_lowercase();
        if QUESTION_STARTS
            .iter()
            .any(|question_start| rest_lower.starts_with(question_start))
        {
            return rest;
        }
    }
    input
}

fn strip_inverted_who_is<'a>(input: &'a str, lower: &str) -> Option<&'a str> {
    let rest_lower = lower.strip_prefix("who ")?;
    let body_lower = rest_lower.strip_suffix(" is")?;
    let body_start = input.len() - rest_lower.len();
    let body_end = body_start + body_lower.len();
    let body = input[body_start..body_end].trim();
    if body.is_empty() || matches!(body.to_lowercase().as_str(), "is" | "was" | "are") {
        return None;
    }
    Some(body)
}

fn strip_meaning_question_body<'a>(input: &'a str, lower: &str) -> Option<&'a str> {
    for prefix in [
        "what is the meaning of ",
        "what's the meaning of ",
        "what is meaning of ",
        "meaning of ",
    ] {
        if lower.starts_with(prefix) {
            return clean_meaning_candidate(&input[prefix.len()..]);
        }
    }

    for suffix in [" mean", " means", " meaning"] {
        if !lower.ends_with(suffix) {
            continue;
        }
        let stem = input[..input.len() - suffix.len()].trim();
        let stem_lower = stem.to_lowercase();
        for prefix in [
            "what does the word ",
            "what does ",
            "what do ",
            "what did ",
            "what is the word ",
            "what is ",
            "what's ",
            "what i ",
        ] {
            if stem_lower.starts_with(prefix) {
                return clean_meaning_candidate(&stem[prefix.len()..]);
            }
        }
    }

    None
}

fn clean_meaning_candidate(value: &str) -> Option<&str> {
    let body = value
        .trim()
        .trim_matches(['"', '\'', '`', '“', '”', '‘', '’'])
        .trim();
    if body.is_empty() {
        return None;
    }
    let lower = body.to_lowercase();
    if matches!(
        lower.as_str(),
        "it" | "that" | "this" | "word" | "the word" | "mean" | "means" | "meaning" | "i"
    ) {
        return None;
    }
    Some(body)
}

fn finalize_concept_query(body: &str) -> Option<ConceptQuery> {
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
    let (term, context) = split_term_and_context(trimmed_body);
    if term.is_empty() {
        return None;
    }
    Some(ConceptQuery {
        term,
        context: context.filter(|c| !c.is_empty()),
    })
}

/// Split a question body on the first matching context delimiter. The
/// delimiters come from `data/seed/prompt-patterns.lino` so adding a new
/// language requires no Rust changes.
fn split_term_and_context(body: &str) -> (String, Option<String>) {
    for delimiter in concept_context_delimiters() {
        if let Some(idx) = body.find(delimiter.as_str()) {
            let term = body[..idx].trim().to_owned();
            let context = body[idx + delimiter.len()..].trim().to_owned();
            if !term.is_empty() && !context.is_empty() {
                return (term, Some(context));
            }
        }
    }
    (body.to_owned(), None)
}

fn strip_suffix_pattern(input: &str) -> Option<String> {
    for suffix in concept_suffixes() {
        if let Some(rest) = input.strip_suffix(suffix.as_str()) {
            return Some(rest.trim().to_owned());
        }
    }
    None
}

/// Result of a concept-lookup ranking pass.
///
/// `context_match` is `true` when the user supplied a context phrase and a
/// record in the seed listed it under `contexts`. Callers (the solver handler)
/// use this flag to choose between the plain and the in-context response
/// template.
#[derive(Debug, Clone)]
pub struct ConceptLookup {
    pub record: &'static ConceptRecord,
    pub context_match: bool,
    pub context: Option<String>,
}

/// Look up a concept by term, alias, or slug, with optional context-aware
/// disambiguation. Comparison is case-insensitive and ignores leading
/// articles ("the", "a", "an").
///
/// Some languages place the context phrase *before* the concept (Hindi
/// "ML में IIR क्या है"; Chinese "ML 中的 IIR 是什么"); others place it
/// *after* (English "what is IIR in ML"; Russian "что такое IIR в ML").
/// The ranker tries the supplied `(term, context)` ordering first and, if
/// the term half does not match any record, retries with the halves swapped.
/// This keeps the parser delimiter-driven without committing to a per-language
/// word-order rule, matching schema:disambiguatingDescription semantics.
#[must_use]
pub fn lookup_concept_query(query: &ConceptQuery) -> Option<ConceptLookup> {
    let direct = rank_for_pair(&query.term, query.context.as_deref());
    // Reversed ordering (context-first languages).
    if let Some(context) = query.context.as_deref() {
        if let Some(reversed) = rank_for_pair(context, Some(&query.term)) {
            if direct
                .as_ref()
                .is_none_or(|lookup| !lookup.context_match && reversed.context_match)
            {
                return Some(reversed);
            }
        }
    }
    direct
}

fn rank_for_pair(term: &str, context: Option<&str>) -> Option<ConceptLookup> {
    let normalized = normalize_concept_term(term);
    if normalized.is_empty() {
        return None;
    }
    let context_normalized = context
        .map(normalize_concept_term)
        .filter(|c| !c.is_empty());

    let mut term_matches: Vec<&'static ConceptRecord> = concepts()
        .iter()
        .filter(|record| {
            record_matches_query_term(record, &normalized, context_normalized.as_deref())
        })
        .collect();
    if term_matches.is_empty() {
        return None;
    }

    if let Some(ctx) = context_normalized.as_deref() {
        if let Some(record) = term_matches
            .iter()
            .copied()
            .find(|record| record_has_context(record, ctx))
        {
            return Some(ConceptLookup {
                record,
                context_match: true,
                context: Some(ctx.to_owned()),
            });
        }
    }

    // No context match: prefer a record that declares no contexts (which is
    // the safest fallback when the user did supply a context but it didn't
    // match anything), then any remaining term-match. The ordering here is
    // stable so the lookup is deterministic across runs.
    term_matches.sort_by_key(|record| u8::from(!record.contexts.is_empty()));
    let record = term_matches.into_iter().next()?;
    Some(ConceptLookup {
        record,
        context_match: false,
        context: context_normalized,
    })
}

fn record_matches_query_term(
    record: &ConceptRecord,
    normalized: &str,
    context_normalized: Option<&str>,
) -> bool {
    if record_matches_term(record, normalized) {
        return true;
    }
    let Some(context) = context_normalized else {
        return false;
    };
    let combined = format!("{normalized} {context}");
    record_matches_term(record, &combined)
}

fn record_matches_term(record: &ConceptRecord, normalized: &str) -> bool {
    if normalize_concept_term(&record.term) == normalized
        || normalize_concept_term(&record.slug) == normalized
    {
        return true;
    }
    record
        .aliases
        .iter()
        .any(|alias| normalize_concept_term(alias) == normalized)
        || record.localized.iter().any(|localized| {
            normalize_concept_term(&localized.term) == normalized
                || localized
                    .aliases
                    .iter()
                    .any(|alias| normalize_concept_term(alias) == normalized)
        })
}

fn record_has_context(record: &ConceptRecord, context_normalized: &str) -> bool {
    if record
        .contexts
        .iter()
        .any(|candidate| normalize_concept_term(candidate) == context_normalized)
    {
        return true;
    }
    // Fallback: resolve the user-supplied context through the registry and
    // see whether the resolved record's slug is referenced by the concept's
    // `context_links` list. This lets a concept declare contexts purely via
    // Q-ID-anchored references in concept-contexts.lino without restating
    // every alias inline.
    if let Some(context_record) = concept_contexts()
        .iter()
        .find(|c| c.matches(context_normalized))
    {
        return record
            .context_links
            .iter()
            .any(|slug| slug.trim() == context_record.slug);
    }
    false
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
