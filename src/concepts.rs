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

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::seed::response_for;
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::humanize_url;

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

fn concept_response_language_markers() -> &'static [(String, &'static str)] {
    static CELL: OnceLock<Vec<(String, &'static str)>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut markers = seed::lexicon()
            .meanings_with_role(seed::ROLE_RESPONSE_LANGUAGE_MARKER)
            .filter_map(|meaning| {
                let language = meaning_defined_language_code(meaning)?;
                Some(
                    meaning
                        .words()
                        .map(move |word| (word.to_lowercase(), language)),
                )
            })
            .flatten()
            .filter(|(marker, _language)| !marker.is_empty())
            .collect::<Vec<_>>();
        markers.sort_by_key(|(marker, _language)| std::cmp::Reverse(marker.len()));
        markers
    })
    .as_slice()
}

fn meaning_defined_language_code(meaning: &seed::Meaning) -> Option<&'static str> {
    meaning
        .defined_by
        .iter()
        .find_map(|slug| language_code(slug))
}

fn language_code(slug: &str) -> Option<&'static str> {
    // Issue #706: resolved through the registry, so `language_spanish` maps to
    // `es` the moment Spanish is a seed record — no arm to add here.
    crate::language::language_for_concept_slug(slug).map(crate::Language::slug)
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
    pub response_language: Option<String>,
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
    let lower_for_response_marker = trimmed.to_lowercase();
    let (stripped_trimmed, outer_response_language) =
        strip_trailing_response_language_marker(&lower_for_response_marker);
    let stripped_trimmed_owned;
    let trimmed = if outer_response_language.is_some() {
        stripped_trimmed_owned = stripped_trimmed.to_owned();
        stripped_trimmed_owned.as_str()
    } else {
        trimmed
    };
    let trimmed = strip_leading_request(trimmed);

    if let Some(body) = strip_suffix_pattern(trimmed) {
        return finalize_concept_query_with_response_language(&body, outer_response_language);
    }

    let lower = trimmed.to_lowercase();
    if let Some(body) = strip_meaning_question_body(trimmed, &lower) {
        return finalize_concept_query_with_response_language(body, outer_response_language);
    }

    if let Some(body) = strip_inverted_who_is(trimmed, &lower) {
        return finalize_concept_query_with_response_language(body, outer_response_language);
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
    finalize_concept_query_with_response_language(body, outer_response_language)
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

/// The subject a meaning interrogation asks about — "what does X mean",
/// "meaning of X" — the one definition shape the cue-lexicon prefix set
/// cannot see: its lead ("what does") is not itself a concept cue, and the
/// interrogated surface is followed by a tail ("mean") instead of ending the
/// prompt. The concept handler routes the same shape through
/// [`extract_concept_query`], so this stays the one authority for the subject.
#[must_use]
pub fn meaning_question_subject(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim().trim_end_matches(['?', '.']).trim();
    let lower = trimmed.to_lowercase();
    strip_meaning_question_body(trimmed, &lower).map(str::to_owned)
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

fn finalize_concept_query_with_response_language(
    body: &str,
    inherited_response_language: Option<&str>,
) -> Option<ConceptQuery> {
    let body = body
        .trim()
        .trim_end_matches(['?', '。', '.', '!', '!', ',', ',', ';', ':'])
        .trim()
        .to_lowercase();
    if body.is_empty() {
        return None;
    }
    let mut response_language = inherited_response_language;
    let (trimmed_body, language) = strip_trailing_response_language_marker(&body);
    let mut trimmed_body = trimmed_body;
    response_language = response_language.or(language);
    trimmed_body = strip_concept_idiom_suffix(trimmed_body);
    let (trimmed_body_after_suffix, language) =
        strip_trailing_response_language_marker(trimmed_body);
    trimmed_body = trimmed_body_after_suffix;
    response_language = response_language.or(language);
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
        response_language: response_language.map(str::to_owned),
    })
}

fn strip_concept_idiom_suffix(body: &str) -> &str {
    body.strip_suffix(" mean")
        .or_else(|| body.strip_suffix(" stand for"))
        .unwrap_or(body)
        .trim()
}

fn strip_trailing_response_language_marker(body: &str) -> (&str, Option<&'static str>) {
    let body = body.trim();
    for (marker, language) in concept_response_language_markers() {
        let Some(rest) = strip_trailing_marker(body, marker) else {
            continue;
        };
        return (rest, Some(*language));
    }
    (body, None)
}

fn strip_trailing_marker<'a>(body: &'a str, marker: &str) -> Option<&'a str> {
    let start = body.len().checked_sub(marker.len())?;
    if !body.ends_with(marker) || start == 0 {
        return None;
    }
    let before = body[..start].chars().next_back()?;
    if !is_response_language_marker_boundary(before, marker) {
        return None;
    }
    let stem = body[..start]
        .trim()
        .trim_end_matches([',', '，', ';', '；', ':', '：', '、'])
        .trim();
    (!stem.is_empty()).then_some(stem)
}

fn is_response_language_marker_boundary(before: char, marker: &str) -> bool {
    before.is_whitespace()
        || matches!(before, ',' | '，' | ';' | '；' | ':' | '：' | '、')
        || marker.chars().next().is_some_and(is_cjk)
}

const fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0x2CEB0..=0x2EBEF
    )
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
    if let Some(context) = query.context.as_deref()
        && let Some(reversed) = rank_for_pair(context, Some(&query.term))
        && direct
            .as_ref()
            .is_none_or(|lookup| !lookup.context_match && reversed.context_match)
    {
        return Some(reversed);
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

    if let Some(ctx) = context_normalized.as_deref()
        && let Some(record) = term_matches
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

/// The offline concept-lookup answer, rendered from the seed registry.
///
/// Plan 09 leaf 18 moved this orchestration out of the handler dispatcher's
/// module: extraction, ranking and context resolution are the machinery above,
/// the response wording is seed (`data/seed/multilingual-responses.lino` for
/// the in-context variants, the plain template below), and what remains is the
/// seed-record rendering every reader of this registry shares.
pub fn try_concept_lookup(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    try_concept_lookup_with_response_language(prompt, log, None)
}

/// Concept lookup that can be forced to render in a specific response language.
///
/// Issue #556: a response-language follow-up replays the previous request with
/// a forced target language so the whole answerable class re-renders in the
/// requested language. When `forced_response_language` is `Some`, it is used
/// unless the prompt already carries its own explicit response-language marker
/// (e.g. "what is X in Russian"), which stays authoritative.
pub fn try_concept_lookup_with_response_language(
    prompt: &str,
    log: &mut EventLog,
    forced_response_language: Option<&str>,
) -> Option<SymbolicAnswer> {
    let mut query = extract_concept_query(prompt)?;
    if query.response_language.is_none()
        && let Some(forced) = forced_response_language
    {
        query.response_language = Some(forced.to_owned());
    }
    log.append("concept_lookup:request", query.term.clone());
    if let Some(context) = query.context.as_deref() {
        log.append("concept_lookup:context", context.to_owned());
    }
    if let Some(response_language) = query.response_language.as_deref() {
        log.append(
            "concept_lookup:response-language",
            response_language.to_owned(),
        );
        log.append("language_to", response_language.to_owned());
    }
    let Some(lookup) = lookup_concept_query(&query) else {
        log.append("concept_lookup:miss", query.term);
        return None;
    };
    let record: &'static ConceptRecord = lookup.record;
    log.append("concept_lookup:hit", record.slug.clone());
    let detected_language = detect_language(prompt);
    let language = query
        .response_language
        .as_deref()
        .unwrap_or_else(|| detected_language.slug());
    let localized = record.localized_for(language);
    let source_for_log = localized
        .map(|loc| loc.source.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source.as_str());
    // Issue #21: log the percent-decoded IRI form so diagnostic chips stay
    // readable. Rendering uses humanize_url too (see render_concept_*).
    log.append("source", humanize_url(source_for_log));
    if !record.wikidata.is_empty() {
        log.append("wikidata", record.wikidata.clone());
    }
    if lookup.context_match {
        if let Some(context) = lookup.context.as_deref() {
            log.append("concept_lookup:context-match", context.to_owned());
            let body = render_concept_in_context(language, context, record);
            return Some(finalize_simple(
                prompt,
                log,
                "concept_lookup_in_context",
                "response:concept_lookup_in_context",
                &body,
                0.9,
            ));
        }
    } else if let Some(context) = lookup.context.as_deref() {
        log.append("concept_lookup:context-mismatch", context.to_owned());
    }
    let body = render_concept_plain(language, record);
    Some(finalize_simple(
        prompt,
        log,
        "concept_lookup",
        "response:concept_lookup",
        &body,
        0.9,
    ))
}

/// Render a plain `concept_lookup` body using the localized variant when
/// available (so `что такое IIR` in Russian returns the ru.wikipedia.org
/// summary, not the English one).
fn render_concept_plain(language: &str, record: &ConceptRecord) -> String {
    let localized = record.localized_for(language);
    let term = localized
        .map(|loc| loc.term.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.term.as_str());
    let summary = localized
        .map(|loc| loc.summary.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.summary.as_str());
    let source = localized
        .map(|loc| loc.source.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source.as_str());
    let source_kind = localized
        .map(|loc| loc.source_kind.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source_kind.as_str());
    let source_markup = render_source_link(source);
    format!(
        "{term} ({category}): {summary}\n\nSource: {source_markup} ({source_kind}).",
        category = record.category,
    )
}

/// Issue #21: render a URL as a readable IRI while keeping the canonical
/// percent-encoded form as the link target. Returns the bare URL when the
/// humanized and encoded forms match (no link wrapping needed).
pub fn render_source_link(source: &str) -> String {
    let human = humanize_url(source);
    if human == source {
        source.to_owned()
    } else {
        format!("[{human}]({source})")
    }
}

/// Render a `concept_lookup_in_context` body, preferring the language-specific
/// template loaded from `data/seed/multilingual-responses.lino`. Falls back
/// to the English template (and, if that is missing, a hardcoded one) so the
/// solver still works when seed loading fails.
///
/// Maintainer requirement R8 (issue #20): use the full disambiguated context
/// name, e.g. `В контексте «ml» (Машинное обучение)`. The raw user-typed
/// context is shown verbatim and the resolved registry label is appended in
/// parentheses; when the two collide (user already typed the localized
/// label) the `no_alias` template is used to avoid `«ml» (ml)`.
///
/// Maintainer requirement R9: the term and summary use the localized variant
/// (e.g. `Фильтр с бесконечной импульсной характеристикой… или IIR-фильтр`)
/// when the user's prevailing language has a `localized` block.
#[allow(clippy::literal_string_with_formatting_args)]
fn render_concept_in_context(language: &str, context: &str, record: &ConceptRecord) -> String {
    let context_record = resolve_context_label(context);
    let context_label =
        context_record.map_or_else(|| context.to_owned(), |c| c.label_for(language).to_owned());
    let use_no_alias = context_label.trim().to_lowercase() == context.trim().to_lowercase();
    let intent_variant = if use_no_alias {
        "concept_lookup_in_context_no_alias"
    } else {
        "concept_lookup_in_context"
    };
    let template = response_for(intent_variant, language)
        .or_else(|| response_for(intent_variant, "en"))
        .or_else(|| response_for("concept_lookup_in_context", language))
        .or_else(|| response_for("concept_lookup_in_context", "en"))
        .unwrap_or_else(|| {
            String::from(
                "In the context of {context} ({context_label}), {term} ({category}) means: \
                 {summary}\n\nSource: {source} ({source_kind}).",
            )
        });
    let localized = record.localized_for(language);
    let term = localized
        .map(|loc| loc.term.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.term.as_str());
    let summary = localized
        .map(|loc| loc.summary.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.summary.as_str());
    let source = localized
        .map(|loc| loc.source.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source.as_str());
    let source_kind = localized
        .map(|loc| loc.source_kind.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source_kind.as_str());
    let source_markup = render_source_link(source);
    template
        .replace("{context_label}", &context_label)
        .replace("{context}", context)
        .replace("{term}", term)
        .replace("{category}", &record.category)
        .replace("{summary}", summary)
        .replace("{source}", &source_markup)
        .replace("{source_kind}", source_kind)
}
