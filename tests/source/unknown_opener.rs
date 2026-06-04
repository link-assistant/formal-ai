//! Deterministic opener variation for the unknown-intent fallback (issue #144).
//!
//! The chat surface should not return the same dead-end sentence for every
//! unknown prompt. This module picks an opener for the language from a small,
//! deterministic pool driven by the prompt's stable hash so a given prompt
//! always picks the same opener but different prompts can pick different ones.

use crate::engine::{
    chinese_unknown_answer, hindi_unknown_answer, russian_unknown_answer, unknown_answer,
    unknown_language_fallback_answer,
};
use crate::web_engine_core::{select_unknown_opener, unknown_openers_for};
use crate::Language;

/// Replace the leading opener of the cached seed answer with a deterministic
/// variation. The seed answer is split on the first sentence terminator so
/// the structured teaching instructions remain identical across variations.
fn unknown_answer_with_variation(prompt: &str, language: &str, seed_text: &str) -> String {
    let opener = select_unknown_opener(prompt, language);
    let body = strip_leading_opener(seed_text, unknown_openers_for(language));
    if body.is_empty() {
        return String::from(opener);
    }
    format!("{opener} {body}")
}

fn strip_leading_opener(text: &str, openers: &[&str]) -> String {
    let trimmed = text.trim_start();
    for known in openers {
        if let Some(rest) = trimmed.strip_prefix(known) {
            return rest.trim_start().to_owned();
        }
    }
    // Fallback: split on the first sentence boundary so the structured
    // instructions stay intact even when the seed opener drifts.
    for separator in [". ", "。", "। "] {
        if let Some(idx) = trimmed.find(separator) {
            let start = idx + separator.len();
            return trimmed[start..].trim_start().to_owned();
        }
    }
    trimmed.to_owned()
}

/// Public variation selector for the English unknown answer. The Rust side
/// uses this when no language-aware variant is needed; the worker mirrors the
/// behaviour in JavaScript.
#[must_use]
pub fn unknown_answer_variation_for(prompt: &str) -> String {
    unknown_answer_with_variation(prompt, "en", unknown_answer())
}

#[must_use]
pub fn language_aware_unknown_answer(prompt: &str, language: Language) -> String {
    let (seed_text, slug) = match language {
        Language::Russian => (russian_unknown_answer(), "ru"),
        Language::Hindi => (hindi_unknown_answer(), "hi"),
        Language::Chinese => (chinese_unknown_answer(), "zh"),
        Language::English => (unknown_answer(), "en"),
        Language::Unknown => return String::from(unknown_language_fallback_answer()),
    };
    unknown_answer_with_variation(prompt, slug, seed_text)
}
