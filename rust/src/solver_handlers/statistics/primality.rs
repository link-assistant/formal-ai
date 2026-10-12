//! Primality of one stated whole number (R1017).
//!
//! "Is 97 a prime number?" is decided by trial division: no integer from 2 to
//! ⌊√97⌋ = 9 divides 97, so it is prime; 91 = 7 × 13 is not, and the answer
//! names the smallest factor with its cofactor. The property surfaces ("prime",
//! "простое", "质数", …) are the `number_property_prime` meaning in
//! `data/seed/meanings-statistics.lino`; the prose is the
//! `number_primality_*` templates in
//! `data/seed/multilingual-responses-quantities.lino`.
//!
//! The question must state exactly one whole number and name the property
//! after it ("is 97 prime", "является ли 97 простым", "97是质数吗"), so "the
//! largest prime below 100" or "the 10th prime" (a property word before the
//! number, an ordinal glued to it) are not read as asking about 100 or 10. A
//! code request ("write a function that checks whether 97 is prime") is left
//! to the coding handlers.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{lexicon, localized_response};
use crate::solver_handlers::calendar::contains_term;
use crate::solver_handlers::finalize_simple;

/// The seed meaning carrying the "prime" surfaces.
const PRIME_SLUG: &str = "number_property_prime";

/// Largest number tested (10¹²): trial division then needs at most 10⁶ steps.
const MAX_TESTED: u64 = 1_000_000_000_000;

/// Answer "is <n> prime?" or decline with `None`.
pub(super) fn handle_primality(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let lowered = prompt.to_lowercase();
    let (start, end) = single_stated_integer(&lowered)?;
    let after = &lowered[end..];
    let names_property = lexicon()
        .meaning(PRIME_SLUG)?
        .words()
        .any(|word| contains_term(after, word));
    if !names_property {
        return None;
    }
    let normalized = crate::web_engine_core::normalize_prompt(prompt);
    let vocabulary = crate::seed::operation_vocabulary();
    if ["code_request", "function", "implement"]
        .into_iter()
        .any(|slug| vocabulary.matches(slug, &normalized))
    {
        return None;
    }
    let number = lowered[start..end].parse::<u64>().ok()?;
    if number > MAX_TESTED {
        return None;
    }
    let language = detect_language(prompt).slug();
    let limit = number.isqrt();
    let subject = number.to_string();
    log.append("number_property:subject", subject.clone());
    log.append("number_property:property", PRIME_SLUG);
    let (intent, factor) = if number < 2 {
        ("number_primality_below_two", None)
    } else if let Some(factor) = smallest_factor(number, limit) {
        ("number_primality_composite", Some(factor))
    } else if limit < 2 {
        ("number_primality_prime_small", None)
    } else {
        ("number_primality_prime", None)
    };
    log.append("number_property:trial_division", format!("2..={limit}"));
    if let Some(factor) = factor {
        log.append(
            "number_property:factor",
            format!("{subject} = {factor} × {}", number / factor),
        );
    }
    log.append(
        "number_property:result",
        (factor.is_none() && number >= 2).to_string(),
    );
    log.append("language", language.to_owned());
    let template = localized_response(intent, language)?;
    let mut body = template
        .replace(concat!("{", "n}"), &subject)
        .replace(concat!("{", "limit}"), &limit.to_string());
    if let Some(factor) = factor {
        body = body
            .replace(concat!("{", "factor}"), &factor.to_string())
            .replace(concat!("{", "cofactor}"), &(number / factor).to_string());
    }
    Some(finalize_simple(
        prompt,
        log,
        "number_primality",
        "response:number_primality",
        &body,
        1.0,
    ))
}

/// The byte span of the prompt's only standalone whole number, or `None` when
/// it states none, several, a negative one, or a decimal. A digit run glued to
/// an ASCII letter ("10th", "x2") is not a standalone number.
fn single_stated_integer(lowered: &str) -> Option<(usize, usize)> {
    let bytes = lowered.as_bytes();
    let glued = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'.';
    let mut found = None;
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let before = start.checked_sub(1).map(|at| bytes[at]);
        let after = bytes.get(index).copied();
        if before == Some(b'-') {
            return None;
        }
        if before.is_some_and(glued) || after.is_some_and(glued) {
            continue;
        }
        if found.replace((start, index)).is_some() {
            return None;
        }
    }
    found
}

/// The smallest divisor of `number` in `2..=limit`, if any.
fn smallest_factor(number: u64, limit: u64) -> Option<u64> {
    (2..=limit).find(|divisor| number.is_multiple_of(*divisor))
}
