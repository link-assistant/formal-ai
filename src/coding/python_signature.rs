//! Python signature and import handling shared by the synthesis handlers
//! (issue #1085, D5.3).
//!
//! Two upstream prompt shapes defeated the curated synthesis path for reasons
//! that had nothing to do with the tasks themselves:
//!
//! * `HumanEval` prompts annotate `numbers: List[float]` and open with
//!   `from typing import List`. A candidate that copies the signature but not
//!   the import raises `NameError` the moment Python evaluates the annotation,
//!   so the verification failed on every upstream case, including the seeded
//!   ones, while the curated wording (`list[float]`) passed. The imports are
//!   part of the specification and travel with the candidate.
//! * `MBPP` prompts carry no signature, only tests, and
//!   `similar_elements((3, 4, 5, 6), (5, 7, 4, 10))` inside an `assert` is a
//!   call. Copying it produced `def similar_elements((3, 4, 5, 6), ...)`, which
//!   does not parse. Only a parameter list is a signature.

/// The import statements a prompt declares, in order, without duplicates.
#[must_use]
pub fn import_preamble(prompt: &str) -> String {
    let mut seen: Vec<String> = Vec::new();
    for line in prompt.lines().map(str::trim) {
        let is_import =
            line.starts_with("import ") || (line.starts_with("from ") && line.contains(" import "));
        if is_import && !seen.iter().any(|known| known == line) {
            seen.push(line.to_owned());
        }
    }
    seen.join("\n")
}

/// Whether the text between a signature's parentheses is a parameter list.
///
/// Comma-separated names, each optionally starred, annotated or given a
/// default, or nothing at all. A literal, a nested call or a tuple is not.
#[must_use]
pub fn is_parameter_list(inside: &str) -> bool {
    if inside.trim().is_empty() {
        return true;
    }
    split_top_level_commas(inside).iter().all(|parameter| {
        let name = parameter
            .trim()
            .trim_start_matches('*')
            .split([':', '='])
            .next()
            .unwrap_or_default()
            .trim();
        let mut characters = name.chars();
        characters
            .next()
            .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
            && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
    })
}

/// Split on commas that are not inside brackets, so a default such as
/// `x=(1, 2)` or an annotation such as `dict[str, int]` stays whole.
#[must_use]
pub fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (index, character) in text.char_indices() {
        match character {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&text[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

/// The first identifier a prompt applies to an argument list, which is the
/// function the prompt declares or exercises; Python keywords and builtins are
/// skipped.
#[must_use]
pub fn declared_function_name(prompt: &str) -> Option<String> {
    let bytes = prompt.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !is_ascii_identifier_start(bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && is_ascii_identifier_continue(bytes[index]) {
            index += 1;
        }
        let identifier = &prompt[start..index];
        let mut cursor = index;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor < bytes.len()
            && bytes[cursor] == b'('
            && !is_reserved_python_identifier(identifier)
        {
            return Some(identifier.to_owned());
        }
    }
    None
}

const fn is_ascii_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

const fn is_ascii_identifier_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn is_reserved_python_identifier(identifier: &str) -> bool {
    matches!(
        identifier,
        "if" | "for"
            | "while"
            | "return"
            | "def"
            | "list"
            | "tuple"
            | "set"
            | "dict"
            | "str"
            | "int"
            | "float"
            | "bool"
            | "sum"
            | "abs"
            | "range"
            | "print"
    )
}

/// The declared signature of `function_name` in `prompt`, from its name through
/// its return annotation, or `None` when the prompt applies the name to
/// arguments rather than parameters.
#[must_use]
pub fn declared_signature(prompt: &str, function_name: &str) -> Option<String> {
    let marker = format!("{function_name}(");
    let lower = prompt.to_ascii_lowercase();
    let start = lower.find(&marker.to_ascii_lowercase())?;
    let after_name = start + function_name.len();
    let close = matching_close_paren(prompt, after_name)?;
    let mut end = close;
    let tail = &prompt[end..];
    let trimmed = tail.trim_start();
    if trimmed.starts_with("->") {
        let return_start = end + (tail.len() - trimmed.len());
        end = return_annotation_end(prompt, return_start).unwrap_or(end);
    }
    // Only a parameter list is a signature; an `assert` call is not (issue #1085).
    if !is_parameter_list(&prompt[after_name + 1..close - 1]) {
        return None;
    }
    Some(prompt[start..end].trim().trim_end_matches('.').to_owned())
}

fn return_annotation_end(prompt: &str, arrow_start: usize) -> Option<usize> {
    let arrow_end = arrow_start + "->".len();
    let mut seen_annotation = false;
    let mut last_annotation_end = None;
    let mut cursor = arrow_end;

    while cursor < prompt.len() {
        let character = prompt[cursor..].chars().next()?;
        let next = cursor + character.len_utf8();
        if character.is_whitespace() {
            if seen_annotation
                && next_non_whitespace(prompt, next).is_some_and(is_return_annotation_char)
            {
                cursor = next;
                continue;
            }
            if seen_annotation {
                break;
            }
            cursor = next;
            continue;
        }
        if !is_return_annotation_char(character) {
            break;
        }
        seen_annotation = true;
        last_annotation_end = Some(next);
        cursor = next;
    }

    last_annotation_end
}

fn next_non_whitespace(prompt: &str, start: usize) -> Option<char> {
    prompt[start..]
        .chars()
        .find(|character| !character.is_whitespace())
}

const fn is_return_annotation_char(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(
            character,
            '_' | '[' | ']' | '(' | ')' | ',' | '\'' | '"' | '|'
        )
}

#[must_use]
pub fn matching_close_paren(prompt: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, character) in prompt[open_index..].char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open_index + offset + character.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}
