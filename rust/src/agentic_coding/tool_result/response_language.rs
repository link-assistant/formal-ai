//! The language of a request whose script leaves it at the fallback language.
//!
//! Split out of `tool_result.rs` (the 1000-line Rust ceiling); mirrors
//! `lexicalLanguage` in `js/agentic/tool_result.mjs`. `Elimina las líneas 2 a
//! 3 de f.txt.` names no Spanish marker, so the script detector answers with
//! the fallback language; but `elimina`, `las` and `líneas` are seeded only as
//! Spanish words (PR #1188 G20).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

/// Fewer words seeded only in one language than this are no evidence.
const MIN_LEXICAL_EVIDENCE: usize = 2;

/// Every one-word surface of the meaning seed and the languages it is in.
fn seeded_word_languages() -> &'static BTreeMap<String, BTreeSet<String>> {
    static LANGUAGES: OnceLock<BTreeMap<String, BTreeSet<String>>> = OnceLock::new();
    LANGUAGES.get_or_init(|| {
        let mut languages: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for meaning in &crate::seed::lexicon().meanings {
            for lexeme in &meaning.lexemes {
                for word in &lexeme.words {
                    let surface = word.text.to_lowercase();
                    if surface.is_empty() || !surface.chars().all(char::is_alphabetic) {
                        continue;
                    }
                    languages
                        .entry(surface)
                        .or_default()
                        .insert(lexeme.language.clone());
                }
            }
        }
        languages
    })
}

/// The one language most of the request's unquoted words are seeded in alone.
///
/// Only when it is not the fallback, has at least `MIN_LEXICAL_EVIDENCE` such
/// words and more than the fallback has; a tie decides nothing.
pub(super) fn lexical_language(prompt: &str) -> Option<&'static str> {
    let seeded = seeded_word_languages();
    let lowered = crate::solver_handlers::text_outside_quoted_segments(prompt).to_lowercase();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for word in lowered.split(|character: char| !character.is_alphabetic()) {
        if let Some(languages) = seeded.get(word)
            && languages.len() == 1
            && let Some(language) = languages.first()
        {
            *counts.entry(language.as_str()).or_default() += 1;
        }
    }
    let top = counts.values().copied().max().unwrap_or(0);
    let mut leaders = counts.iter().filter(|(_, count)| **count == top);
    let (Some((leader, _)), None) = (leaders.next(), leaders.next()) else {
        return None;
    };
    let fallback = crate::language::fallback_language().slug();
    let fallback_count = counts.get(fallback).copied().unwrap_or(0);
    if top < MIN_LEXICAL_EVIDENCE || *leader == fallback || top <= fallback_count {
        return None;
    }
    crate::language::from_slug(leader).map(crate::language::Language::slug)
}
