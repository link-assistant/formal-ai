//! Exact scalar command aliases; additional operands retain unknown authority.
use super::{Capability, QualifiedToolAttempt};

pub(super) fn qualified_command_attempt<'a>(
    frames: &'a [QualifiedToolAttempt],
    command: &str,
) -> Option<&'a QualifiedToolAttempt> {
    let frame = frames.iter().rev().find(|frame| {
        frame.capability == Capability::Run
            && frame.arguments_value.as_object().is_some_and(|arguments| {
                ["command", "cmd", "script"].iter().any(|key| {
                    arguments.get(*key).and_then(serde_json::Value::as_str) == Some(command)
                })
            })
    })?;
    frame
        .arguments_value
        .as_object()
        .filter(|arguments| arguments.len() == 1)
        .map(|_| frame)
}
