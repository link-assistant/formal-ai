//! Literal extraction shared by numeric-list requests.
use super::{ListValue, Operation, ParsedListItem};

pub(crate) fn parse_list_items(prompt: &str, operation: Operation) -> Vec<ParsedListItem> {
    let quoted = parse_quoted_strings(prompt);
    if matches!(operation, Operation::Transform(_)) && quoted.len() >= 2 {
        return quoted;
    }
    parse_numbers(prompt)
}

/// Extract every quoted string literal from the raw prompt, in order. Quoted
/// strings let the same list algorithm handle non-numeric values without trying
/// to infer arbitrary unquoted prose as data.
fn parse_quoted_strings(prompt: &str) -> Vec<ParsedListItem> {
    let chars: Vec<char> = prompt.chars().collect();
    let mut items = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let quote = chars[index];
        if quote != '"' && quote != '\'' {
            index += 1;
            continue;
        }
        index += 1;
        let mut text = String::new();
        while index < chars.len() {
            let ch = chars[index];
            if ch == '\\'
                && let Some(next) = chars.get(index + 1)
            {
                text.push(*next);
                index += 2;
                continue;
            }
            if ch == quote {
                break;
            }
            text.push(ch);
            index += 1;
        }
        if index < chars.len() && chars[index] == quote {
            index += 1;
            if !text.is_empty() {
                items.push(ParsedListItem {
                    text,
                    value: ListValue::Text,
                });
            }
        }
    }
    items
}

/// Whether the request lists numbers, or at least two quoted values.
///
/// The `list_items` claim evidence of issue #1175 R3.
#[must_use]
pub fn names_list_items(prompt: &str) -> bool {
    !parse_numbers(prompt).is_empty() || parse_quoted_strings(prompt).len() >= 2
}

/// Extract every number token from the raw prompt, in order of appearance.
///
/// Recognizes optionally-signed integers and decimals (e.g. `-3`, `5`, `7.5`).
/// The surface text is preserved so the echo and the generated code use exactly
/// what the user typed; the parsed `f64` value drives the computation.
pub fn parse_numbers(prompt: &str) -> Vec<ParsedListItem> {
    let chars: Vec<char> = prompt.chars().collect();
    let mut numbers = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        let sign = if (ch == '-' || ch == '+')
            && chars.get(index + 1).is_some_and(char::is_ascii_digit)
            && !preceded_by_digit_or_word(&chars, index)
        {
            let s = ch;
            index += 1;
            Some(s)
        } else if ch.is_ascii_digit() {
            None
        } else {
            index += 1;
            continue;
        };

        let start = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
        }
        // Optional single decimal point followed by digits.
        if index < chars.len()
            && chars[index] == '.'
            && chars.get(index + 1).is_some_and(char::is_ascii_digit)
        {
            index += 1;
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
            }
        }

        if start == index {
            continue;
        }
        let mut text = String::new();
        if let Some(s) = sign {
            text.push(s);
        }
        text.extend(&chars[start..index]);
        if let Ok(value) = text.parse::<f64>() {
            numbers.push(ParsedListItem {
                text,
                value: ListValue::Number(value),
            });
        }
    }
    numbers
}

/// A leading sign only introduces a negative/positive literal when it is not
/// glued to a preceding digit or letter (so `main2` or `a-b` do not spawn a
/// bogus number, but a standalone `-3` does).
fn preceded_by_digit_or_word(chars: &[char], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|prev| chars.get(prev))
        .is_some_and(|prev| prev.is_alphanumeric())
}
