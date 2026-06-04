//! Shared symbolic engine primitives reused by the CLI, the HTTP server, and
//! the browser worker via Rust→WASM.
//!
//! Issue #133 (R194) wants every non-UI primitive — language detection,
//! prompt normalization, arithmetic evaluation — to live in Rust and be
//! exposed to the browser through the WASM bridge. JavaScript is reserved
//! for UI, transport, and orchestration; data processing happens in this
//! module so the offline trace and the live answer agree byte-for-byte.
//!
//! The module is `no_std` + `alloc` compatible so the WASM worker can
//! `#[path]`-include it without pulling in the standard library. The
//! sibling modules `language` and `arithmetic` are reached through `super::`
//! so the same source file compiles inside both the host crate (where the
//! modules live at `crate::language` / `crate::arithmetic`) and the
//! wasm-worker crate (which mounts them via `#[path]`).

#![allow(clippy::module_name_repetitions)]

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryFrom;

pub use super::arithmetic::evaluate_fallback_formatted;
pub use super::language::{detect as detect_language, Language};

/// Normalize an arbitrary prompt to a lowercase, single-space-delimited stream.
///
/// This matches the behaviour of the legacy JavaScript `normalizePrompt(prompt)`
/// helper in the browser worker — keeping a single implementation in Rust
/// eliminates the drift that produced different traces in #133.
///
/// The rules:
///   * Unicode letters and digits are kept (preserving every script — Cyrillic,
///     Devanagari, CJK, Latin).
///   * Every other Unicode codepoint becomes a single space.
///   * Adjacent spaces collapse, leading and trailing spaces are stripped.
///   * ASCII uppercase letters fold to lowercase. Non-ASCII case folding is
///     applied through `char::to_lowercase` so the result is locale-agnostic.
#[must_use]
pub fn normalize_prompt(prompt: &str) -> String {
    let mut out = String::with_capacity(prompt.len());
    let mut last_was_space = true;
    for ch in prompt.chars() {
        if is_unicode_letter_or_digit(ch) {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            last_was_space = false;
        } else if !last_was_space {
            out.push(' ');
            last_was_space = true;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Tokenize a normalized prompt into whitespace-separated tokens. Used by the
/// JS worker to feed the existing intent matchers; centralising the rule keeps
/// the JS and Rust paths aligned.
#[must_use]
pub fn tokenize_prompt(prompt: &str) -> Vec<String> {
    normalize_prompt(prompt)
        .split(' ')
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect()
}

/// Evaluate an arithmetic expression and return the formatted result.
///
/// The helper accepts the same word-form operators (`plus`, `minus`, `плюс`,
/// `умножить на`, …) as the legacy JS path so prompts like "what is two plus
/// two" round-trip to "4" through the WASM bridge.
///
/// `Ok(string)` carries the rendered numeric result. `Err(string)` carries
/// the error reason from `ArithmeticError::Display`.
pub fn evaluate_arithmetic_expression(expression: &str) -> Result<String, String> {
    evaluate_fallback_formatted(expression).map_err(|err| err.to_string())
}

/// Stable FNV-1a 64-bit id used by Rust answers and browser-worker memory.
///
/// JavaScript strings are UTF-16 internally, so the browser worker must call
/// this WASM export or use an explicit UTF-8 byte fallback. Hashing UTF-16 code
/// units changes non-ASCII ids and breaks parity for multilingual prompts.
#[must_use]
pub fn stable_id(prefix: &str, text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    format!("{prefix}_{hash:016x}")
}

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

#[must_use]
pub fn unknown_openers_for(language: &str) -> &'static [&'static str] {
    match language {
        "ru" => UNKNOWN_OPENERS_RU,
        "hi" => UNKNOWN_OPENERS_HI,
        "zh" => UNKNOWN_OPENERS_ZH,
        _ => UNKNOWN_OPENERS_EN,
    }
}

/// Pick the deterministic unknown-answer opener for a prompt/language pair.
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

/// Match a prompt against intent-route fields using the browser/Rust route
/// semantics: exact keyword/phrase match, token match, or all-token combo.
#[must_use]
pub fn matches_intent_route_parts(
    normalized_prompt: &str,
    raw_prompt: &str,
    keywords: &[String],
    phrases: &[String],
    tokens: &[String],
    combos: &[Vec<String>],
) -> bool {
    if keywords
        .iter()
        .any(|keyword| normalized_prompt == keyword || raw_prompt == keyword)
    {
        return true;
    }
    if phrases
        .iter()
        .any(|phrase| normalized_prompt == phrase || raw_prompt == phrase)
    {
        return true;
    }
    if tokens
        .iter()
        .any(|token| contains_route_token(normalized_prompt, token))
    {
        return true;
    }
    combos.iter().any(|combo| {
        !combo.is_empty()
            && combo
                .iter()
                .all(|token| contains_route_token(normalized_prompt, token))
    })
}

/// Parse the line protocol used by the JS→WASM route matcher and return the
/// canonical match result.
///
/// Format:
/// `normalized\nraw\nK\tkeyword\nP\tphrase\nT\ttoken\nC\ttoken1\ttoken2...`
#[must_use]
pub fn matches_intent_route_payload(payload: &str) -> bool {
    let mut lines = payload.lines();
    let normalized = lines.next().unwrap_or("");
    let raw = normalize_route_raw_prompt(lines.next().unwrap_or(""));
    let mut keywords = Vec::new();
    let mut phrases = Vec::new();
    let mut tokens = Vec::new();
    let mut combos = Vec::new();

    for line in lines {
        let mut fields = line.split('\t');
        let Some(kind) = fields.next() else {
            continue;
        };
        match kind {
            "K" => {
                if let Some(value) = fields.next() {
                    keywords.push(value.to_string());
                }
            }
            "P" => {
                if let Some(value) = fields.next() {
                    phrases.push(value.to_string());
                }
            }
            "T" => {
                if let Some(value) = fields.next() {
                    tokens.push(value.to_string());
                }
            }
            "C" => {
                let combo = fields
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                if !combo.is_empty() {
                    combos.push(combo);
                }
            }
            _ => {}
        }
    }

    matches_intent_route_parts(normalized, &raw, &keywords, &phrases, &tokens, &combos)
}

fn contains_route_token(normalized_prompt: &str, expected: &str) -> bool {
    normalized_prompt
        .split_whitespace()
        .any(|token| token == expected)
}

fn normalize_route_raw_prompt(prompt: &str) -> String {
    let mut out = String::with_capacity(prompt.len());
    for ch in prompt.chars() {
        for lower in ch.to_lowercase() {
            out.push(lower);
        }
    }
    let trimmed = out.trim();
    let trimmed = trimmed.trim_end_matches(['?', '。', '.', '!', ',', ';', ':']);
    trimmed.trim().to_string()
}

fn is_unicode_letter_or_digit(ch: char) -> bool {
    if ch.is_ascii_alphanumeric() {
        return true;
    }
    let cp = ch as u32;
    // Cyrillic block (basic + supplement).
    if (0x0400..=0x04FF).contains(&cp) || (0x0500..=0x052F).contains(&cp) {
        return true;
    }
    // Devanagari block.
    if (0x0900..=0x097F).contains(&cp) {
        return true;
    }
    // CJK Unified Ideographs and the Bopomofo/CJK extension blocks.
    if (0x3400..=0x4DBF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0x3040..=0x30FF).contains(&cp)
        || (0x3100..=0x312F).contains(&cp)
        || (0xAC00..=0xD7AF).contains(&cp)
    {
        return true;
    }
    // Latin extended (Á, ñ, ü, …) and Greek for completeness.
    if (0x00C0..=0x024F).contains(&cp) || (0x0370..=0x03FF).contains(&cp) {
        return true;
    }
    false
}

#[path = "source_tests/web_engine_core/tests.rs"]
mod tests;
