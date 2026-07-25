//! Seed-driven word lists shared by statement merging and the identifier rung.
//!
//! Three roles in `data/seed/meanings-statement-merge.lino` carry the
//! vocabulary this module exposes:
//!
//! - [`crate::seed::ROLE_STATEMENT_FUNCTION_WORD`] — words dropped before a
//!   statement signature is computed, so "the parser is fast" and "parser is
//!   fast" share one signature.
//! - [`crate::seed::ROLE_STATEMENT_NEGATION_CUE`] — words removed from the
//!   terms and folded into the statement's polarity, so an assertion and its
//!   denial reach the same signature with opposite signs.
//! - [`crate::seed::ROLE_IDENTIFIER_RESERVED_WORD`] — programming-language
//!   keywords the identifier rung must never emit bare.
//!
//! Nothing here is hardcoded: every surface lives in the seed and is loaded
//! through [`crate::seed::lexicon`]. The lists are cached in a `OnceLock` each
//! because the embedded seed is immutable at runtime.

use std::sync::OnceLock;

/// Function words dropped from statement terms and identifier candidates.
///
/// Only articles, prepositions, copulas and coordinators. Quantifiers ("all",
/// "some", "none") are deliberately *not* function words: dropping them would
/// merge "all tests pass" with "some tests pass".
#[must_use]
pub fn function_words() -> &'static [String] {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(|| words_for(crate::seed::ROLE_STATEMENT_FUNCTION_WORD))
}

/// Negation cues that flip a statement's polarity.
#[must_use]
pub fn negation_cues() -> &'static [String] {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(|| words_for(crate::seed::ROLE_STATEMENT_NEGATION_CUE))
}

/// Programming-language keywords an identifier must not collide with.
#[must_use]
pub fn reserved_words() -> &'static [String] {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(|| words_for(crate::seed::ROLE_IDENTIFIER_RESERVED_WORD))
}

fn words_for(role: &str) -> Vec<String> {
    crate::seed::lexicon()
        .words_for_role(role)
        .into_iter()
        .map(|word| word.to_lowercase())
        .collect()
}

/// Is `candidate` a reserved word of a supported programming language?
///
/// The comparison is exact, not case-insensitive: `type` is a Rust keyword and
/// `Type` is a perfectly good type name, so only the forms the seed records
/// (which include `Self`, `None`, `True`, `False`) are rejected.
#[must_use]
pub fn is_reserved_word(candidate: &str) -> bool {
    crate::seed::lexicon()
        .words_for_role(crate::seed::ROLE_IDENTIFIER_RESERVED_WORD)
        .iter()
        .any(|word| word == candidate)
}

/// Split `text` into lowercased tokens.
///
/// A token runs over alphanumeric characters plus the apostrophe (so the
/// negation cue `isn't` survives as one token); every other character is a
/// separator. Typographic apostrophes are normalized to ASCII so the seed's
/// `isn't` matches prose that writes `isn’t`. CJK text has no inter-word
/// spaces, so a CJK run arrives as a single token and is handled by substring
/// stripping in [`strip_words`].
#[must_use]
pub fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buffer = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            buffer.extend(ch.to_lowercase());
        } else if matches!(ch, '\'' | '\u{2019}') {
            buffer.push('\'');
        } else {
            flush(&mut buffer, &mut out);
        }
    }
    flush(&mut buffer, &mut out);
    out
}

fn flush(buffer: &mut String, out: &mut Vec<String>) {
    let token = buffer.trim_matches('\'').to_string();
    buffer.clear();
    if !token.is_empty() {
        out.push(token);
    }
}

/// Remove every token equal to a word in `vocabulary`.
///
/// For a CJK token — which packs several words into one run of characters — the
/// vocabulary words are instead removed as substrings, longest first, so
/// `解析器不快` loses `不` and yields `解析器快`. A CJK token that vanishes
/// entirely is dropped.
#[must_use]
pub fn strip_words(tokens: &[String], vocabulary: &[String]) -> Vec<String> {
    let cjk = cjk_words_longest_first(vocabulary);
    let mut out = Vec::with_capacity(tokens.len());
    for token in tokens {
        if vocabulary.iter().any(|word| word == token) {
            continue;
        }
        if crate::coding::contains_cjk(token) {
            let mut stripped = token.clone();
            for word in &cjk {
                stripped = stripped.replace(word.as_str(), "");
            }
            if !stripped.is_empty() {
                out.push(stripped);
            }
            continue;
        }
        out.push(token.clone());
    }
    out
}

/// Does any token carry a word of `vocabulary` — as a whole token, or (for CJK)
/// as a substring?
#[must_use]
pub fn mentions_word(tokens: &[String], vocabulary: &[String]) -> bool {
    let cjk = cjk_words_longest_first(vocabulary);
    tokens.iter().any(|token| {
        vocabulary.iter().any(|word| word == token)
            || (crate::coding::contains_cjk(token)
                && cjk.iter().any(|word| token.contains(word.as_str())))
    })
}

/// The CJK members of `vocabulary`, longest first, so `没有` is stripped before
/// `有` and a shorter word never eats a longer one's characters. `sort_by_key`
/// is stable, so equal-length words keep their seed declaration order and the
/// result stays deterministic.
fn cjk_words_longest_first(vocabulary: &[String]) -> Vec<String> {
    let mut out: Vec<String> = vocabulary
        .iter()
        .filter(|word| crate::coding::contains_cjk(word))
        .cloned()
        .collect();
    out.sort_by_key(|word| core::cmp::Reverse(word.chars().count()));
    out
}
