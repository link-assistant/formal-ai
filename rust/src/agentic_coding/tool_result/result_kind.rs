//! Whether an empty tool result came from a listing or a search (PR #1188
//! G75).
//!
//! A label with operands is a shell command, read by its own word through the
//! seeded `result_command_words` of `data/seed/shell-intents.lino`, never by a
//! word inside its operands (`sed … allowlist.txt` is no listing). A one-word
//! label is a tool's name. Split out of `tool_result.rs` (the 1000-line Rust
//! ceiling).

use std::sync::OnceLock;

use crate::seed::SHELL_INTENTS_LINO;
use crate::seed::parser::parse_lino;

/// The seeded `(word, kind)` pairs of the shell commands whose result kind is
/// known.
fn command_words() -> &'static [(String, String)] {
    static WORDS: OnceLock<Vec<(String, String)>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let tree = parse_lino(SHELL_INTENTS_LINO);
        tree.children
            .first()
            .and_then(|root| {
                root.children
                    .iter()
                    .find(|group| group.name == "result_command_words")
            })
            .map(|group| {
                group
                    .children
                    .iter()
                    .filter(|node| node.name == "command")
                    .map(|node| {
                        (
                            node.find_child_value("word").to_owned(),
                            node.find_child_value("kind").to_owned(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// Whether the label has operands, so it is a shell command, not a tool name.
fn is_command(lower: &str) -> bool {
    lower.split_whitespace().nth(1).is_some()
}

/// The seeded kind of a command's own word.
fn command_kind(lower: &str) -> Option<&'static str> {
    let word = lower.split_whitespace().next()?;
    command_words()
        .iter()
        .find(|(candidate, _)| candidate == word)
        .map(|(_, kind)| kind.as_str())
}

/// Whether the result labelled `label` is a listing.
pub(super) fn is_listing(label: &str) -> bool {
    let lower = label.to_ascii_lowercase();
    if is_command(&lower) {
        return command_kind(&lower) == Some("listing");
    }
    lower.contains("list") || lower.contains("glob")
}

/// Whether the result labelled `label` is a search.
pub(super) fn is_search(label: &str) -> bool {
    let lower = label.to_ascii_lowercase();
    if is_command(&lower) {
        return command_kind(&lower) == Some("search");
    }
    ["grep", "find", "search"]
        .iter()
        .any(|kind| lower.contains(kind))
}
