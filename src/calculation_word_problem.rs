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

/// Rewrite a natural-language "word problem" into a calculator expression.
///
/// Issue #334 step 2: see the module-level documentation. Returns `None` when
/// no rewrite applies so callers can fall through unchanged.
#[must_use]
pub fn normalize_word_problem(expression: &str) -> Option<String> {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return None;
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
    Some(working)
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
