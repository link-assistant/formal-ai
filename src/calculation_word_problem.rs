//! Natural-language "word problem" normalization extracted from `calculation`
//! to keep each source file under the 1000-line cap enforced by
//! `scripts/check-file-size.rs`.
//!
//! Issue #334: the website demo asked to "calculate the 10th Fibonacci number
//! and multiply it by 8% of 500. Show me the code and the final result." That
//! text is not a calculator expression, but it reduces to one once the symbolic
//! Fibonacci reference is resolved (F(10) = 55), the spelled-out operator is
//! rewritten to `*`, and the trailing instruction sentence is dropped — yielding
//! `55 * 8% of 500`, which the calculator evaluates to 2200.

use std::collections::BTreeMap;

/// A normalized arithmetic word problem, ready for calculator evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordProblemNormalization {
    pub expression: String,
    pub reasoning_steps: Vec<String>,
    pub result_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoxRule {
    Known(i64),
    Multiple { factor: i64, source: String },
    Add { source: String, addend: i64 },
}

/// The `n`-th Fibonacci number under the convention F(1) = F(2) = 1 used across
/// the coding catalog (so F(10) = 55, matching the `fibonacci` program output).
fn fibonacci_value(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    let (mut previous, mut current) = (0u64, 1u64);
    for _ in 1..n {
        let next = previous + current;
        previous = current;
        current = next;
    }
    current
}

/// Parse a leading ordinal/cardinal token such as "10th", "10", "3rd" or the
/// spelled-out "tenth" into its numeric value. Returns `None` for anything else.
fn parse_ordinal(token: &str) -> Option<u32> {
    let token = token.trim_matches(|c: char| !c.is_alphanumeric());
    if token.is_empty() {
        return None;
    }
    let digits: String = token.chars().take_while(char::is_ascii_digit).collect();
    if !digits.is_empty() {
        let suffix = &token[digits.len()..];
        if suffix.is_empty() || matches!(suffix, "st" | "nd" | "rd" | "th") {
            return digits.parse().ok();
        }
        return None;
    }
    Some(match token.to_lowercase().as_str() {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "sixth" => 6,
        "seventh" => 7,
        "eighth" => 8,
        "ninth" => 9,
        "tenth" => 10,
        _ => return None,
    })
}

/// Lowercased, punctuation-trimmed view of a token for keyword comparisons.
fn bare_word(token: &str) -> String {
    token
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// Replace "(the) N-th Fibonacci number" references with their numeric value so
/// the surrounding arithmetic reduces to a calculator expression (issue #334).
fn resolve_fibonacci_references(text: &str) -> String {
    if !text.to_lowercase().contains("fibonacci") {
        return text.to_owned();
    }
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut out: Vec<String> = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if let Some(n) = parse_ordinal(tokens[index]) {
            if tokens
                .get(index + 1)
                .is_some_and(|next| bare_word(next) == "fibonacci")
            {
                // Drop a determiner we already emitted ("the 10th" -> "55").
                if out.last().is_some_and(|last| bare_word(last) == "the") {
                    out.pop();
                }
                out.push(fibonacci_value(n).to_string());
                index += 2;
                // Absorb a trailing "number" / "term" / "sequence" noun.
                if tokens.get(index).is_some_and(|next| {
                    matches!(bare_word(next).as_str(), "number" | "term" | "sequence")
                }) {
                    index += 1;
                }
                continue;
            }
        }
        out.push(tokens[index].to_owned());
        index += 1;
    }
    out.join(" ")
}

/// Split `text` into sentences on a period that ends a sentence (followed by
/// whitespace or the end of the string). A period flanked by digits ("3.14") is
/// kept inside its sentence so decimals are never broken apart.
fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut current = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        if ch == '.'
            && chars
                .get(index + 1)
                .map_or(true, |next| next.is_whitespace())
        {
            let sentence = current.trim().to_owned();
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
            current.clear();
            continue;
        }
        current.push(ch);
    }
    let sentence = current.trim().to_owned();
    if !sentence.is_empty() {
        sentences.push(sentence);
    }
    sentences
}

fn sentence_words(sentence: &str) -> Vec<String> {
    sentence
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn parse_int_token(token: &str) -> Option<i64> {
    if let Ok(value) = token.parse::<i64>() {
        return Some(value);
    }
    match token {
        "zero" => Some(0),
        "one" | "a" | "an" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        _ => None,
    }
}

fn canonical_box_id(token: &str) -> Option<String> {
    let cleaned = token
        .trim_matches(|ch: char| !ch.is_alphanumeric())
        .to_ascii_uppercase();
    (!cleaned.is_empty() && cleaned.chars().count() <= 3).then_some(cleaned)
}

fn parse_declared_box_count(words: &[String]) -> Option<usize> {
    if words.len() >= 4
        && words[0] == "i"
        && words[1] == "have"
        && matches!(words[3].as_str(), "box" | "boxes")
    {
        let count = parse_int_token(&words[2])?;
        usize::try_from(count).ok()
    } else {
        None
    }
}

fn parse_box_rule(words: &[String]) -> Option<(String, BoxRule, Option<String>)> {
    let mut index = usize::from(words.first().is_some_and(|word| word == "if"));
    if words.get(index)? != "box" {
        return None;
    }
    let target = canonical_box_id(words.get(index + 1)?)?;
    if words.get(index + 2)? != "has" {
        return None;
    }
    index += 3;

    if words.get(index).is_some_and(|word| word == "twice")
        && words.get(index + 1).is_some_and(|word| word == "as")
        && words.get(index + 2).is_some_and(|word| word == "many")
        && words.get(index + 4).is_some_and(|word| word == "as")
        && words.get(index + 5).is_some_and(|word| word == "box")
    {
        let item = words.get(index + 3).cloned();
        let source = canonical_box_id(words.get(index + 6)?)?;
        return Some((target, BoxRule::Multiple { factor: 2, source }, item));
    }

    let value = parse_int_token(words.get(index)?)?;
    if words.get(index + 1).is_some_and(|word| word == "more")
        && words.get(index + 3).is_some_and(|word| word == "than")
        && words.get(index + 4).is_some_and(|word| word == "box")
    {
        let item = words.get(index + 2).cloned();
        let source = canonical_box_id(words.get(index + 5)?)?;
        return Some((
            target,
            BoxRule::Add {
                source,
                addend: value,
            },
            item,
        ));
    }

    let item = words.get(index + 1).cloned();
    Some((target, BoxRule::Known(value), item))
}

fn resolve_box_value(
    id: &str,
    rules: &BTreeMap<String, BoxRule>,
    memo: &mut BTreeMap<String, i64>,
    stack: &mut Vec<String>,
    reasoning_steps: &mut Vec<String>,
    result_label: &str,
) -> Option<i64> {
    if let Some(value) = memo.get(id) {
        return Some(*value);
    }
    if stack.iter().any(|existing| existing == id) {
        return None;
    }
    let rule = rules.get(id)?;
    stack.push(id.to_owned());
    let value =
        match rule {
            BoxRule::Known(value) => {
                reasoning_steps.push(format!("Box {id} = {value} {result_label}."));
                Some(*value)
            }
            BoxRule::Multiple { factor, source } => {
                resolve_box_value(source, rules, memo, stack, reasoning_steps, result_label)
                    .and_then(|source_value| {
                        let value = source_value.checked_mul(*factor)?;
                        reasoning_steps.push(format!(
                            "Box {id} = {factor} * {source_value} = {value} {result_label}."
                        ));
                        Some(value)
                    })
            }
            BoxRule::Add { source, addend } => {
                resolve_box_value(source, rules, memo, stack, reasoning_steps, result_label)
                    .and_then(|source_value| {
                        let value = source_value.checked_add(*addend)?;
                        reasoning_steps.push(format!(
                            "Box {id} = {source_value} + {addend} = {value} {result_label}."
                        ));
                        Some(value)
                    })
            }
        };
    stack.pop();
    let value = value?;
    memo.insert(id.to_owned(), value);
    Some(value)
}

fn normalize_box_total_problem(text: &str) -> Option<WordProblemNormalization> {
    let lower = text.to_lowercase();
    if !(lower.contains("box")
        && lower.contains("how many")
        && lower.contains("total")
        && (lower.contains("twice as many") || lower.contains("more") && lower.contains("than")))
    {
        return None;
    }

    let mut declared_count = None;
    let mut rules = BTreeMap::new();
    let mut result_label = None;
    for sentence in split_sentences(text) {
        let words = sentence_words(&sentence);
        if words.is_empty() {
            continue;
        }
        declared_count = declared_count.or_else(|| parse_declared_box_count(&words));
        if let Some((target, rule, item)) = parse_box_rule(&words) {
            if let Some(item) = item.filter(|value| !matches!(value.as_str(), "box" | "boxes")) {
                result_label = Some(item);
            }
            rules.insert(target, rule);
        }
    }

    if rules.len() < 2 {
        return None;
    }
    if let Some(count) = declared_count {
        if rules.len() < count {
            return None;
        }
    }

    let result_label = result_label.unwrap_or_else(|| String::from("items"));
    let mut memo = BTreeMap::new();
    let mut reasoning_steps = Vec::new();
    let ids = rules.keys().cloned().collect::<Vec<_>>();
    for id in &ids {
        resolve_box_value(
            id,
            &rules,
            &mut memo,
            &mut Vec::new(),
            &mut reasoning_steps,
            &result_label,
        )?;
    }
    let values = ids
        .iter()
        .map(|id| memo.get(id).copied())
        .collect::<Option<Vec<_>>>()?;
    if values.is_empty() {
        return None;
    }
    let expression = values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" + ");
    reasoning_steps.push(format!("Total = {expression} {result_label}."));
    Some(WordProblemNormalization {
        expression,
        reasoning_steps,
        result_label: Some(result_label),
    })
}

/// Rewrite a natural-language "word problem" into a calculator expression.
///
/// Issue #334 step 2: see the module-level documentation. Returns `None` when
/// no rewrite applies so callers can fall through unchanged.
#[must_use]
pub fn normalize_word_problem_detailed(expression: &str) -> Option<WordProblemNormalization> {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(normalized) = normalize_box_total_problem(trimmed) {
        return Some(normalized);
    }
    // Keep only sentence fragments that carry arithmetic content, dropping pure
    // instruction clauses such as "Show me the code and the final result". The
    // split treats a period as a sentence boundary only when it ends a sentence
    // (followed by whitespace or the end of the string) so decimals like "3.14"
    // survive intact.
    let arithmetic: Vec<String> = split_sentences(trimmed)
        .into_iter()
        .filter(|sentence| {
            !sentence.is_empty()
                && (sentence.chars().any(|c| c.is_ascii_digit()) || sentence.contains('%'))
        })
        .collect();
    if arithmetic.is_empty() {
        return None;
    }
    let mut working = resolve_fibonacci_references(&arithmetic.join(". "));
    // Rewrite spelled-out operators the calculator does not accept. Longer
    // phrases come first so "and multiply it by" wins over "multiply by".
    for (phrase, symbol) in [
        (" and multiply it by ", " * "),
        (" and multiply by ", " * "),
        (" multiply it by ", " * "),
        (" multiplied by ", " * "),
        (" multiply by ", " * "),
        (" and divide it by ", " / "),
        (" and divide by ", " / "),
        (" divide it by ", " / "),
        (" divided by ", " / "),
        (" divide by ", " / "),
    ] {
        let lower = working.to_lowercase();
        if let Some(position) = lower.find(phrase) {
            working = format!(
                "{}{symbol}{}",
                &working[..position],
                &working[position + phrase.len()..]
            );
        }
    }
    let working = working.split_whitespace().collect::<Vec<_>>().join(" ");
    if working.is_empty() || working.eq_ignore_ascii_case(trimmed) {
        return None;
    }
    Some(WordProblemNormalization {
        expression: working,
        reasoning_steps: Vec::new(),
        result_label: None,
    })
}

#[must_use]
#[cfg(test)]
pub fn normalize_word_problem(expression: &str) -> Option<String> {
    normalize_word_problem_detailed(expression).map(|normalization| normalization.expression)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibonacci_convention_matches_catalog() {
        assert_eq!(fibonacci_value(1), 1);
        assert_eq!(fibonacci_value(2), 1);
        assert_eq!(fibonacci_value(5), 5);
        assert_eq!(fibonacci_value(10), 55);
    }

    #[test]
    fn resolves_fibonacci_and_rewrites_operator() {
        // The Fibonacci reference becomes 55, the spelled-out operator becomes
        // `*`, and the trailing instruction sentence is dropped. The leading
        // "calculate" verb is left for the calculator wrapper-stripping stage.
        assert_eq!(
            normalize_word_problem(
                "calculate the 10th Fibonacci number and multiply it by 8% of 500. \
                 Show me the code and the final result."
            )
            .as_deref(),
            Some("calculate 55 * 8% of 500"),
        );
        assert_eq!(
            normalize_word_problem("the fifth Fibonacci number multiplied by 10").as_deref(),
            Some("5 * 10"),
        );
    }

    #[test]
    fn resolves_box_relations_and_total() {
        let normalized = normalize_word_problem_detailed(
            "I have 3 boxes. Box A has twice as many apples as Box B. \
             Box C has 5 more apples than Box A. If Box B has 10 apples, \
             how many apples are there in total? Show your reasoning step by step.",
        )
        .expect("box total problem should normalize");
        assert_eq!(normalized.expression, "20 + 10 + 25");
        assert_eq!(
            normalized.reasoning_steps,
            vec![
                "Box B = 10 apples.",
                "Box A = 2 * 10 = 20 apples.",
                "Box C = 20 + 5 = 25 apples.",
                "Total = 20 + 10 + 25 apples.",
            ],
        );
        assert_eq!(normalized.result_label.as_deref(), Some("apples"));
    }

    #[test]
    fn decimals_are_never_split_on_their_dot() {
        // "3.14" must not become "3. 14" — the period is flanked by digits, so it
        // stays inside its sentence and the whole expression is unchanged.
        assert_eq!(normalize_word_problem("What is 3.14 * 2"), None);
    }

    #[test]
    fn pure_instruction_text_is_left_alone() {
        assert_eq!(normalize_word_problem("Show me the code"), None);
        assert_eq!(normalize_word_problem(""), None);
    }
}
