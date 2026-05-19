//! Deterministic opener variation for the unknown-intent fallback (issue #144).
//!
//! The chat surface should not return the same dead-end sentence for every
//! unknown prompt. This module picks an opener for the language from a small,
//! deterministic pool driven by the prompt's stable hash so a given prompt
//! always picks the same opener but different prompts can pick different ones.

use crate::engine::stable_id;
use crate::engine::{
    chinese_unknown_answer, hindi_unknown_answer, russian_unknown_answer, unknown_answer,
    unknown_language_fallback_answer,
};
use crate::Language;

/// Alternative opening phrases prepended to the deterministic unknown-answer
/// body. The active opener is chosen by [`select_unknown_opener`] from the
/// prompt's stable hash so a given prompt always returns the same variation
/// but the surface response can differ across distinct prompts. The first
/// entry of each language matches the opener already embedded in the seed
/// text so the "with-variations" answer is a strict superset of the seed.
const UNKNOWN_OPENERS_EN: &[&str] = &[
    "I don't know how to answer that yet.",
    "I didn't understand you.",
    "I'm not sure how to respond to that yet.",
    "I haven't learned to answer that yet.",
    "That one is new to me.",
];
const UNKNOWN_OPENERS_RU: &[&str] = &[
    "Я пока не знаю, как ответить на это.",
    "Я тебя не понял.",
    "Я не уверен, как на это ответить.",
    "Я ещё не научился отвечать на это.",
    "Это для меня новое.",
];
const UNKNOWN_OPENERS_HI: &[&str] = &[
    "मुझे अभी इसका उत्तर देना नहीं आता।",
    "मैं समझ नहीं पाया।",
    "मुझे यकीन नहीं है कि कैसे उत्तर दूँ।",
    "मैंने अभी तक यह उत्तर देना नहीं सीखा।",
    "यह मेरे लिए नया है।",
];
const UNKNOWN_OPENERS_ZH: &[&str] = &[
    "我还不知道如何回答这个问题。",
    "我不太明白你说的意思。",
    "我不确定该如何回答。",
    "我还没有学会回答这个问题。",
    "这对我来说是新的。",
];

fn unknown_openers_for(language: &str) -> &'static [&'static str] {
    match language {
        "ru" => UNKNOWN_OPENERS_RU,
        "hi" => UNKNOWN_OPENERS_HI,
        "zh" => UNKNOWN_OPENERS_ZH,
        _ => UNKNOWN_OPENERS_EN,
    }
}

/// Pick a deterministic opener variation for the given prompt and language.
/// Different prompts get different openers, but the same prompt always picks
/// the same one. The returned slice index never exceeds the language pool
/// length and is safe to index without bounds checks.
#[must_use]
pub fn select_unknown_opener(prompt: &str, language: &str) -> &'static str {
    let pool = unknown_openers_for(language);
    debug_assert!(!pool.is_empty(), "unknown opener pool must be non-empty");
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return pool[0];
    }
    let id = stable_id("unknown_opener", trimmed);
    let hex = id.rsplit('_').next().unwrap_or("0");
    let value = u64::from_str_radix(hex, 16).unwrap_or(0);
    let pool_len = pool.len() as u64;
    let index = usize::try_from(value % pool_len).unwrap_or(0);
    pool[index]
}

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
