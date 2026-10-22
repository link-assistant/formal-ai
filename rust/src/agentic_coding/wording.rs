//! Shared agentic wording and cue data reader.

pub(super) fn agentic_message(key: &str, values: &[(&str, &str)]) -> String {
    crate::seed::agentic_message(key, values)
}
