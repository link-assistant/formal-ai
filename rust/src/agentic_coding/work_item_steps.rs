//! The shell steps and file templates a work item ends with, read from
//! `data/meta/work-item-steps.lino` (issue #1133).
//!
//! The commit that lands a result, the `gh` read of an issue, the commit
//! subject a listing of changed files earns, the workflow an issue asks for:
//! each is a line of wording with placeholders, and the wording is data. This
//! module only fills the placeholders.

const STEPS: &str = include_str!("../../embedded/data/meta/work-item-steps.lino");

/// The template registered under `key`, with `{placeholders}` filled in.
///
/// Panics when the ledger lacks the key: the ledger is compiled in, so a
/// missing key is a defect of this repository, not of a request.
pub(super) fn fill(key: &str, values: &[(&str, &str)]) -> String {
    let mut text = template(key)
        .unwrap_or_else(|| panic!("data/meta/work-item-steps.lino lacks `{key}`"));
    for (placeholder, value) in values {
        text = text.replace(placeholder, value);
    }
    text
}

fn template(key: &str) -> Option<String> {
    STEPS.lines().find_map(|line| {
        let (candidate, value) = line.strip_prefix("  ")?.split_once(' ')?;
        (candidate == key).then(|| unescape(unquote(value.trim())))
    })
}

fn unquote(value: &str) -> &str {
    let mut chars = value.chars();
    match (chars.next(), chars.next_back()) {
        (Some(open), Some(close)) if open == close && matches!(open, '"' | '\'') => {
            &value[1..value.len() - 1]
        }
        _ => value,
    }
}

fn unescape(value: &str) -> String {
    value.replace("\\n", "\n").replace("\\t", "\t")
}
