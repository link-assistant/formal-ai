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
#[allow(unused_imports)]
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

/// Result of applying the deterministic arithmetic decision procedure to a
/// fact-check statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticClaimOutcome {
    /// Both closed expressions reduce and the asserted relation holds.
    Unrefuted,
    /// Both closed expressions reduce and contradict the asserted relation.
    Refuted,
    /// The text has an arithmetic relation, but at least one side does not
    /// reduce to a numeric value.
    Inconclusive,
}

/// Structured arithmetic assessment shared with the browser through WASM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArithmeticClaimAssessment {
    pub outcome: ArithmeticClaimOutcome,
    pub left_expression: String,
    pub left_value: String,
    pub right_expression: String,
    pub right_value: String,
    pub relation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArithmeticComparison {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

/// Assess a closed arithmetic equality or inequality without any I/O.
///
/// `None` means that the statement is outside this decision procedure. A
/// recognized relation that cannot be evaluated returns an `Inconclusive`
/// assessment, preserving the fact checker's prior-only semantics.
#[must_use]
pub fn assess_arithmetic_claim(statement: &str) -> Option<ArithmeticClaimAssessment> {
    let (left, right, comparison) = split_arithmetic_comparison(statement)?;
    let left_expression = normalize_arithmetic_claim_side(left);
    let right_expression = normalize_arithmetic_claim_side(right);
    if left_expression.is_empty()
        || right_expression.is_empty()
        || !left_expression.chars().any(|ch| ch.is_ascii_digit())
        || !right_expression.chars().any(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    let left_result = evaluate_arithmetic_expression(&left_expression);
    let right_result = evaluate_arithmetic_expression(&right_expression);
    let (Ok(left_value), Ok(right_value)) = (left_result, right_result) else {
        return Some(ArithmeticClaimAssessment {
            outcome: ArithmeticClaimOutcome::Inconclusive,
            left_expression,
            left_value: String::new(),
            right_expression,
            right_value: String::new(),
            relation: arithmetic_comparison_symbol(comparison),
        });
    };
    let equal = arithmetic_values_equal(&left_value, &right_value);
    let less = arithmetic_value_less_than(&left_value, &right_value);
    let holds = match comparison {
        ArithmeticComparison::Equal => equal,
        ArithmeticComparison::NotEqual => !equal,
        ArithmeticComparison::Less => less,
        ArithmeticComparison::Greater => arithmetic_value_less_than(&right_value, &left_value),
        ArithmeticComparison::LessOrEqual => equal || less,
        ArithmeticComparison::GreaterOrEqual => {
            equal || arithmetic_value_less_than(&right_value, &left_value)
        }
    };
    Some(ArithmeticClaimAssessment {
        outcome: if holds {
            ArithmeticClaimOutcome::Unrefuted
        } else {
            ArithmeticClaimOutcome::Refuted
        },
        left_expression,
        left_value,
        right_expression,
        right_value,
        relation: arithmetic_comparison_symbol(comparison),
    })
}

fn split_arithmetic_comparison(statement: &str) -> Option<(&str, &str, ArithmeticComparison)> {
    const COMPARISONS: &[(&str, ArithmeticComparison)] = &[
        ("==", ArithmeticComparison::Equal),
        ("!=", ArithmeticComparison::NotEqual),
        ("≠", ArithmeticComparison::NotEqual),
        ("<=", ArithmeticComparison::LessOrEqual),
        (">=", ArithmeticComparison::GreaterOrEqual),
        ("≤", ArithmeticComparison::LessOrEqual),
        ("≥", ArithmeticComparison::GreaterOrEqual),
        ("=", ArithmeticComparison::Equal),
        ("<", ArithmeticComparison::Less),
        (">", ArithmeticComparison::Greater),
    ];
    for (token, comparison) in COMPARISONS {
        if let Some(index) = statement.find(token) {
            let (left, remainder) = statement.split_at(index);
            return Some((left, &remainder[token.len()..], *comparison));
        }
    }
    None
}

fn normalize_arithmetic_claim_side(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for ch in text.trim().chars() {
        match ch {
            '×' | '·' => output.push('*'),
            '÷' => output.push('/'),
            '−' | '–' | '—' => output.push('-'),
            ',' => output.push(' '),
            _ => output.push(ch),
        }
    }
    output.trim().to_string()
}

fn arithmetic_values_equal(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let (Ok(left), Ok(right)) = (left.parse::<f64>(), right.parse::<f64>()) else {
        return false;
    };
    (left - right).abs() < 1e-9
}

fn arithmetic_value_less_than(left: &str, right: &str) -> bool {
    let (Ok(left), Ok(right)) = (left.parse::<f64>(), right.parse::<f64>()) else {
        return false;
    };
    left < right
}

const fn arithmetic_comparison_symbol(comparison: ArithmeticComparison) -> &'static str {
    match comparison {
        ArithmeticComparison::Equal => "=",
        ArithmeticComparison::NotEqual => "≠",
        ArithmeticComparison::Less => "<",
        ArithmeticComparison::Greater => ">",
        ArithmeticComparison::LessOrEqual => "≤",
        ArithmeticComparison::GreaterOrEqual => "≥",
    }
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

/// The opener pools, embedded so the browser worker (`no_std` + `alloc`, no
/// filesystem) varies its unknown answers from exactly the same data as the
/// native build. Issue #706: a language's openers are a seed edit, so this
/// module holds no per-language branch.
const UNKNOWN_OPENERS: &str = include_str!("../data/seed/unknown-openers.lino");

/// Strip the surrounding quotes of a seed value, if any.
fn unquote_seed_value(value: &'static str) -> &'static str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

/// One `pool` record: a language slug and its openers, in declaration order.
struct OpenerPool {
    language: &'static str,
    openers: Vec<&'static str>,
}

/// The parsed opener registry: the pools plus the fallback language a slug
/// without its own pool borrows from.
struct OpenerRegistry {
    pools: Vec<OpenerPool>,
    fallback_language: &'static str,
    sentence_separators: Vec<&'static str>,
}

fn parse_opener_registry() -> OpenerRegistry {
    let mut registry = OpenerRegistry {
        pools: Vec::new(),
        fallback_language: "en",
        sentence_separators: Vec::new(),
    };
    for line in UNKNOWN_OPENERS.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let (key, value) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        match key {
            "pool" => registry.pools.push(OpenerPool {
                language: "",
                openers: Vec::new(),
            }),
            "fallback_language" => registry.fallback_language = unquote_seed_value(value),
            "sentence_separator" => registry
                .sentence_separators
                .push(unquote_seed_value(value)),
            "language" => {
                if let Some(pool) = registry.pools.last_mut() {
                    pool.language = unquote_seed_value(value);
                }
            }
            "opener" => {
                if let Some(pool) = registry.pools.last_mut() {
                    pool.openers.push(unquote_seed_value(value));
                }
            }
            _ => {}
        }
    }
    registry.pools.retain(|pool| !pool.openers.is_empty());
    registry
}

// Parsed once per process natively. The WASM worker resets its bump allocator
// at the start of every exported call, so a cached allocation there would
// dangle; it reparses inside the caller's epoch.
#[cfg(not(target_arch = "wasm32"))]
fn with_opener_registry<R>(action: impl FnOnce(&OpenerRegistry) -> R) -> R {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<OpenerRegistry> = OnceLock::new();
    action(REGISTRY.get_or_init(parse_opener_registry))
}

#[cfg(target_arch = "wasm32")]
fn with_opener_registry<R>(action: impl FnOnce(&OpenerRegistry) -> R) -> R {
    action(&parse_opener_registry())
}

/// The opener pool for `language`, or the fallback language's pool when the
/// seed declares none for it.
///
/// Returns an owned vector rather than a slice because the pools are parsed
/// from seed data, not from per-language constants.
#[must_use]
pub fn unknown_openers_for(language: &str) -> Vec<&'static str> {
    with_opener_registry(|registry| {
        registry
            .pools
            .iter()
            .find(|pool| pool.language == language)
            .or_else(|| {
                registry
                    .pools
                    .iter()
                    .find(|pool| pool.language == registry.fallback_language)
            })
            .map(|pool| pool.openers.clone())
            .unwrap_or_default()
    })
}

/// The sentence terminators the unknown-answer body is split on when the seed
/// opener has drifted from every pool entry.
#[must_use]
pub fn unknown_opener_sentence_separators() -> Vec<&'static str> {
    with_opener_registry(|registry| registry.sentence_separators.clone())
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
