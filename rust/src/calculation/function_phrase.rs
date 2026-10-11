//! Spelled function applications rewritten into call syntax (R1017).
//!
//! "What is the square root of 144?" reaches the calculator as the expression
//! "the square root of 144", which no evaluator parses. A function named in
//! words and joined to its argument by a marker word is the same request as
//! the call `sqrt(144)`, so the phrase is rewritten into that call before
//! evaluation. The function surfaces are the `math_function_name` meanings
//! (`square_root`, `sine`, `natural_logarithm`, …, each written as its first
//! English surface — `sqrt`, `sin`, `ln`) and the joining words the
//! `math_function_argument_marker` role ("of", "из", "de"), both in
//! `data/seed/meanings-calculator.lino`. Words before the function that state
//! no number ("the") are dropped; a phrase followed by more prose ("the square
//! root of 2 is irrational") is left alone, since that is a statement about
//! the value rather than a request to compute it.

use std::cmp::Reverse;

use crate::seed;

/// The rewritten expression, or `None` when no spelled function application
/// with a numeric argument is present.
pub(super) fn rewrite_function_phrases(expression: &str) -> Option<String> {
    let lex = seed::lexicon();
    let mut names: Vec<(String, &str)> = Vec::new();
    for meaning in lex.meanings_with_role(seed::ROLE_MATH_FUNCTION_NAME) {
        let Some(canonical) = meaning.word_in("en") else {
            continue;
        };
        if canonical.is_empty() || !canonical.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        for word in meaning.words() {
            names.push((word.to_lowercase(), canonical));
        }
    }
    // Longest surface first, so "natural logarithm" wins over "logarithm" and
    // "квадратный корень" over "корень".
    names.sort_by_key(|(surface, _)| Reverse(surface.chars().count()));
    let markers = lex.words_for_role(seed::ROLE_MATH_FUNCTION_ARGUMENT_MARKER);
    let lower = expression.to_lowercase();
    for (surface, canonical) in &names {
        for (start, _) in lower.match_indices(surface.as_str()) {
            if let Some(rewritten) = rewrite_at(&lower, start, surface, canonical, &markers) {
                return Some(rewritten);
            }
        }
    }
    None
}

/// Rewrite the function surface found at `start`, if a numeric argument
/// follows it (optionally after one marker word) and only arithmetic follows
/// the argument.
fn rewrite_at(
    lower: &str,
    start: usize,
    surface: &str,
    canonical: &str,
    markers: &[String],
) -> Option<String> {
    let prefix = &lower[..start];
    if prefix
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric)
    {
        return None;
    }
    let after_name = &lower[start + surface.len()..];
    if !after_name.starts_with(char::is_whitespace) {
        return None;
    }
    let mut rest = after_name.trim_start();
    for marker in markers {
        if let Some(tail) = rest.strip_prefix(marker.as_str())
            && tail.starts_with(char::is_whitespace)
        {
            rest = tail.trim_start();
            break;
        }
    }
    let argument_end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(rest.len());
    let argument = &rest[..argument_end];
    if !argument.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let suffix = &rest[argument_end..];
    if suffix.chars().any(char::is_alphabetic) {
        return None;
    }
    let kept_prefix = if prefix.chars().any(|c| c.is_ascii_digit()) {
        prefix
    } else {
        ""
    };
    Some(
        format!("{kept_prefix}{canonical}({argument}){suffix}")
            .trim()
            .to_owned(),
    )
}
