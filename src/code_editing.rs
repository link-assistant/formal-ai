//! Scoped post-processing for explicit edits to generated code artifacts.

use crate::coding::ProgramSpec;

pub fn apply_inline_hello_world_output_replacement(
    prompt: &str,
    answer: &str,
    spec: ProgramSpec,
) -> Option<String> {
    if spec.task.slug != "hello_world" {
        return None;
    }
    let replacement = inline_hello_world_replacement(prompt)?;
    Some(answer.replace("Hello, world!", &replacement))
}

pub fn apply_inline_hello_world_source_replacement(
    prompt: &str,
    source: &str,
    spec: ProgramSpec,
) -> String {
    if spec.task.slug != "hello_world" {
        return source.to_owned();
    }
    inline_hello_world_replacement(prompt).map_or_else(
        || source.to_owned(),
        |replacement| source.replace("Hello, world!", &replacement),
    )
}

fn inline_hello_world_replacement(prompt: &str) -> Option<String> {
    let normalized = normalize_replacement_prompt(prompt);
    if !mentions_replacement(&normalized) {
        return None;
    }

    let quoted = quoted_segments(prompt);
    match quoted.len() {
        0 => None,
        1 => non_empty(quoted[0].clone()),
        _ if normalized.contains("replace") || normalized.contains("замен") => {
            non_empty(quoted[1].clone())
        }
        _ => quoted.last().cloned().and_then(non_empty),
    }
}

fn mentions_replacement(normalized: &str) -> bool {
    normalized.contains("replace")
        || normalized.contains("change")
        || normalized.contains("instead")
        || normalized.contains("rather than")
        || normalized.contains("вместо")
        || normalized.contains("замен")
        || normalized.contains("बदल")
        || normalized.contains("替换")
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn normalize_replacement_prompt(prompt: &str) -> String {
    let mut normalized = String::with_capacity(prompt.len());
    for character in prompt.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            normalized.push(character);
        } else {
            normalized.push(' ');
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn quoted_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        let Some((relative_start, open, close)) =
            text[cursor..]
                .char_indices()
                .find_map(|(index, character)| {
                    quote_close_for(character).map(|close| (index, character, close))
                })
        else {
            break;
        };
        let content_start = cursor + relative_start + open.len_utf8();
        let Some(relative_end) = text[content_start..].find(close) else {
            break;
        };
        let content_end = content_start + relative_end;
        segments.push(text[content_start..content_end].to_owned());
        cursor = content_end + close.len_utf8();
    }
    segments
}

const fn quote_close_for(open: char) -> Option<char> {
    match open {
        '\'' => Some('\''),
        '"' => Some('"'),
        '`' => Some('`'),
        '«' => Some('»'),
        '“' => Some('”'),
        '‘' => Some('’'),
        '「' => Some('」'),
        '『' => Some('』'),
        _ => None,
    }
}
