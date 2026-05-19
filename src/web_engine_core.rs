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

use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub use super::arithmetic::{evaluate_fallback_formatted, ArithmeticError};
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn normalize_collapses_punctuation_to_single_space() {
        assert_eq!(normalize_prompt("Hello,  world!"), "hello world");
        assert_eq!(normalize_prompt("  what's 2+2?"), "what s 2 2");
    }

    #[test]
    fn normalize_keeps_cjk_codepoints() {
        let out = normalize_prompt("你好，世界！");
        assert!(out.contains('你'));
        assert!(out.contains('好'));
        assert!(out.contains('世'));
        assert!(out.contains('界'));
    }

    #[test]
    fn normalize_handles_devanagari() {
        let out = normalize_prompt("नमस्ते, दुनिया!");
        assert!(out.contains('न'));
        assert!(out.contains('द'));
        assert!(!out.contains(','));
    }

    #[test]
    fn normalize_lowercases_cyrillic() {
        // `char::to_lowercase` handles Cyrillic correctly.
        let out = normalize_prompt("ПРИВЕТ, МИР!");
        assert!(out.contains("привет"));
        assert!(out.contains("мир"));
    }

    #[test]
    fn tokenize_returns_individual_words() {
        assert_eq!(
            tokenize_prompt("  Hello,  world  again!"),
            vec![
                "hello".to_string(),
                "world".to_string(),
                "again".to_string()
            ],
        );
    }

    #[test]
    fn detect_language_matches_existing_rules() {
        assert_eq!(detect_language("Hello"), Language::English);
        assert_eq!(detect_language("Привет"), Language::Russian);
        assert_eq!(detect_language("नमस्ते"), Language::Hindi);
        assert_eq!(detect_language("你好"), Language::Chinese);
    }

    #[test]
    fn evaluate_arithmetic_handles_word_operators() {
        assert_eq!(
            evaluate_arithmetic_expression("two plus two"),
            Ok("4".to_string())
        );
        assert_eq!(
            evaluate_arithmetic_expression("3 multiplied by 4"),
            Ok("12".to_string())
        );
    }

    #[test]
    fn evaluate_arithmetic_returns_localizable_errors() {
        assert!(evaluate_arithmetic_expression("1 / 0").is_err());
        assert!(evaluate_arithmetic_expression("").is_err());
    }
}
