//! Scoped post-processing for explicit edits to generated code artifacts.

use crate::coding::ProgramSpec;
use crate::normal_markov::quoted_segments;

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
    let prompt = unwrap_transport_quotes(prompt);
    if prompt.contains('?') || prompt.contains('？') {
        return None;
    }

    let quoted = quoted_segments(prompt);
    match quoted.as_slice() {
        [] => None,
        [replacement] => Some(replacement.clone()),
        values if values.len() % 2 == 0 => values
            .chunks_exact(2)
            .find(|pair| pair[0] == "Hello, world!")
            .map(|pair| pair[1].clone())
            .or_else(|| values.last().cloned()),
        _ => None,
    }
}

fn unwrap_transport_quotes(text: &str) -> &str {
    let trimmed = text.trim();
    for quote in ['"', '\''] {
        if let Some(inner) = trimmed
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
        {
            return inner;
        }
    }
    trimmed
}
