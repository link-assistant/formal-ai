//! Scoped post-processing for explicit edits to generated code artifacts.

use crate::coding::ProgramSpec;
use crate::normal_markov::{quoted_segments, unwrap_transport_quotes};

pub fn apply_inline_hello_world_source_replacement(
    prompt: &str,
    source: &str,
    spec: ProgramSpec<'_>,
) -> String {
    if spec.task.slug != "hello_world" {
        return source.to_owned();
    }
    inline_hello_world_replacement(prompt).map_or_else(
        || source.to_owned(),
        |replacement| {
            crate::coding::catalog::literal_output_program(spec.language.slug, &replacement)
                .unwrap_or_else(|| source.to_owned())
        },
    )
}

pub(crate) fn inline_hello_world_replacement(prompt: &str) -> Option<String> {
    let prompt = unwrap_transport_quotes(prompt);
    if prompt.contains('?') || prompt.contains('？') {
        return None;
    }

    // A replacement literal is one line of text the template's string
    // literal can carry. A segment that spans a line break is a code block
    // the prompt supplies (a Markdown fence reads as backtick quotes), and an
    // empty one is a fence's delimiter pair: binding either would nest the
    // supplied code inside the generated program's output call.
    let quoted: Vec<String> = quoted_segments(prompt)
        .into_iter()
        .filter(|segment| {
            !segment.is_empty()
                && !segment
                    .chars()
                    .any(|character| character.is_control() && character != '\t')
        })
        .collect();
    match quoted.as_slice() {
        [] => None,
        [replacement] => Some(replacement.clone()),
        values if values.len() % 2 == 0 => values
            .as_chunks::<2>()
            .0
            .iter()
            .find(|[old, _]| {
                crate::coding::program_task_by_slug("hello_world")
                    .is_some_and(|task| old == task.output)
            })
            .map(|[_, new]| new.clone())
            .or_else(|| values.last().cloned()),
        _ => None,
    }
}
