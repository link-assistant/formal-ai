//! The *implementation language* modifier: what `"in <language>"` means (#906).
//!
//! Issue #906 reported that the router took whatever word followed "in" as the
//! target programming language, so "Create a file named hello.txt **in the
//! current directory**…" was answered as a request to write a program "in
//! language `the`". The positional scan was right about *where* an unknown
//! language name can appear and wrong about *what* may fill that position.
//!
//! This module owns that one question. Everything it reasons over is seed data:
//!
//! * [`crate::seed::ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION`] — the marker that
//!   introduces the modifier ("in", "на"),
//! * [`crate::seed::ROLE_IMPLEMENTATION_LANGUAGE_NOUN`] — the optional head noun
//!   ("language", "языке"),
//! * [`crate::seed::ROLE_STATEMENT_FUNCTION_WORD`] — the closed class of
//!   determiners, prepositions, copulas and coordinators, in every supported
//!   language, which is exactly the class of words that can never *name* a
//!   language.
//!
//! An unknown name is still accepted — "hello world in elvish" must keep
//! routing as a request in the unknown language `elvish`, because refusing
//! every name outside the catalogue would make the engine unable to report what
//! it was asked for. What it may not do is accept a closed-class word: a
//! determiner is admitted only when the surrounding wording independently
//! evidences a language (a known language name, or the explicit "language"
//! noun).
//!
//! [`without_modifier`] is the same span analysis read the other way round: it
//! removes the modifier so the rest of the request can be understood on its own
//! terms. "Fix the failing CI job in Rust." is a request about a CI job, not
//! about Rust; keeping the modifier in the topic made the unknown-prompt
//! reasoner answer with an encyclopedia definition of the language.

use std::sync::OnceLock;

use crate::seed;

/// A recognized `"<preposition> [language] <name>"` span inside a request.
struct ModifierSpan {
    /// Index of the preposition token that opens the span.
    start: usize,
    /// Index one past the last token of the span.
    end: usize,
    /// The language name the span carries, verbatim (already lowercased when
    /// the caller normalized the text).
    name: String,
}

fn preposition_surfaces() -> &'static [String] {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(|| head_initial_surfaces(seed::ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION))
}

fn language_noun_surfaces() -> &'static [String] {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(|| head_initial_surfaces(seed::ROLE_IMPLEMENTATION_LANGUAGE_NOUN))
}

/// Surfaces of `role` in the two head-initial languages whose language name
/// *follows* the marker. The head-final Hindi/Chinese forms are carried in the
/// seed for coverage but place the name before the marker, which this scan does
/// not chase; every *known* language is resolved script-independently by
/// [`crate::coding::program_language_by_alias`] before the scan runs.
fn head_initial_surfaces(role: &str) -> Vec<String> {
    seed::lexicon()
        .words_for_role_in_languages(role, &["en", "ru"])
        .into_iter()
        .map(|word| word.to_lowercase())
        .collect()
}

fn is_preposition(token: &str) -> bool {
    preposition_surfaces().iter().any(|word| word == token)
}

fn is_language_noun(token: &str) -> bool {
    language_noun_surfaces().iter().any(|word| word == token)
}

/// Is `token` a closed-class word — an article, preposition, copula or
/// coordinator — in any supported language?
fn is_function_word(token: &str) -> bool {
    crate::summarization::vocabulary::function_words()
        .iter()
        .any(|word| word == token)
}

/// Does the catalogue or the coding oracle know `language` by that slug?
#[must_use]
pub fn is_known(language: &str) -> bool {
    crate::coding::program_language_by_slug(language).is_some()
        || crate::knowledge::CodingOracle::knows_language(language)
}

/// A name can only be a language when it has letters: "in 3 steps" names no
/// implementation language.
fn could_name_a_language(token: &str) -> bool {
    token.chars().any(char::is_alphabetic)
}

/// The implementation language a (normalized) request asks for.
///
/// Known languages are resolved first, in any of the four supported request
/// languages, by the catalogue's alias table. Only then does the positional
/// scan read a bare *unknown* name after the modifier marker.
#[must_use]
pub fn requested(normalized: &str) -> Option<String> {
    if let Some(language) = crate::coding::program_language_by_alias(normalized) {
        return Some(String::from(language.slug));
    }
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    modifier_span(&tokens).map(|span| span.name)
}

/// [`requested`] read from raw request text, which it normalizes first.
///
/// The formalizer already holds a normalized prompt and calls [`requested`]
/// directly; this is the entry point for callers — the regression corpus in
/// `tests/unit/issue_906_language_router.rs` among them — that hold the
/// requester's own spelling.
#[must_use]
pub fn requested_in(text: &str) -> Option<String> {
    requested(&crate::engine::normalize_prompt(text))
}

/// `text` with the implementation-language modifier removed, or `None` when it
/// carries no modifier (or consists of nothing else).
///
/// The remainder keeps the caller's own spelling and word order: only the
/// modifier span is dropped, so "Fix the failing CI job in Rust." becomes "Fix
/// the failing CI job." — a request whose topic no longer mentions a language.
#[must_use]
pub fn without_modifier(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let normalized: Vec<String> = words
        .iter()
        .map(|word| crate::engine::normalize_prompt(word))
        .collect();
    let tokens: Vec<&str> = normalized.iter().map(String::as_str).collect();
    let span = modifier_span(&tokens)?;
    let kept: Vec<&str> = words
        .iter()
        .enumerate()
        .filter(|(index, _)| !(span.start..span.end).contains(index))
        .map(|(_, word)| *word)
        .collect();
    if kept.is_empty() {
        return None;
    }
    Some(kept.join(" "))
}

/// `text` with a *trailing* `"in <known language>"` modifier removed, or `None`
/// when it ends in no such modifier.
///
/// This is the conservative reading [`without_modifier`] cannot offer: it fires
/// only at the end of the text and only for a language the catalogue or the
/// oracle already knows, so a payload that merely says "Meet me in Paris" keeps
/// every word. Issue #906 needs it for recovered file content — "…containing
/// Hello World, in JavaScript." names the bytes *and* the language, and only
/// the bytes belong in the file.
#[must_use]
pub fn without_trailing_known_modifier(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let normalized: Vec<String> = words
        .iter()
        .map(|word| crate::engine::normalize_prompt(word))
        .collect();
    let tokens: Vec<&str> = normalized.iter().map(String::as_str).collect();
    let span = modifier_span(&tokens)?;
    if span.end != tokens.len() || !is_known(&span.name) {
        return None;
    }
    let kept = words[..span.start]
        .join(" ")
        .trim()
        .trim_end_matches([',', ';', ':', '-', '—'])
        .trim()
        .to_owned();
    (!kept.is_empty()).then_some(kept)
}

/// Find the first `"<preposition> [language] <name>"` span whose name may
/// actually be a language.
fn modifier_span(tokens: &[&str]) -> Option<ModifierSpan> {
    for (start, token) in tokens.iter().enumerate() {
        if !is_preposition(token) {
            continue;
        }
        let mut cursor = start + 1;
        // The head noun ("in language X", "на языке X") is explicit evidence
        // that whatever follows is meant as a language name.
        let mut named_by_noun = false;
        // A determiner between the marker and the name ("in the current
        // directory") is not evidence of anything: the span must earn the name
        // some other way.
        let mut skipped_function_word = false;
        while let Some(next) = tokens.get(cursor) {
            if is_language_noun(next) {
                named_by_noun = true;
                cursor += 1;
                continue;
            }
            if is_function_word(next) && !is_known(next) {
                skipped_function_word = true;
                cursor += 1;
                continue;
            }
            break;
        }
        let Some(candidate) = tokens.get(cursor) else {
            continue;
        };
        let mut end = cursor + 1;
        // "in the elvish language" names the noun after the candidate.
        if tokens.get(end).is_some_and(|next| is_language_noun(next)) {
            named_by_noun = true;
            end += 1;
        }
        if !could_name_a_language(candidate) {
            continue;
        }
        // A bare unknown name is read as a language only when it *ends* the
        // phrase: "hello world in elvish" names a language, "print the numbers
        // in reverse order" names an ordering. Anything a known name or the
        // explicit head noun already evidences is accepted wherever it sits.
        let phrase_final = tokens[end..].iter().all(|next| is_function_word(next));
        let accepted =
            is_known(candidate) || named_by_noun || (!skipped_function_word && phrase_final);
        if !accepted {
            continue;
        }
        return Some(ModifierSpan {
            start,
            end,
            name: (*candidate).to_owned(),
        });
    }
    None
}
