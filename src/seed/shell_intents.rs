//! Semantic shell-intent vocabulary loaded from `data/seed/shell-intents.lino`
//! (issue #680).
//!
//! The terminal-command vocabulary in [`super::terminal_commands`] only reaches a
//! command when the prompt *names* it after a run verb (`run pwd`) or asks, in
//! prose, to list a directory. That leaves the common case the issue calls out —
//! *"Print the current working directory"*, *"How much disk space is free?"*,
//! *"What is my username?"* — unrouted: the user expresses an **intent** without
//! ever naming the command. This vocabulary maps each such intent to its concrete
//! command through multilingual cue phrases, so a request routes on meaning rather
//! than on the exact tool word.
//!
//! Like every other trigger vocabulary the natural language lives in seed data, not
//! in the planner, so a maintainer retunes coverage by editing a `.lino` file and
//! the project rule against hardcoded natural language in the solver is upheld. The
//! JavaScript worker loads the synced deployment copy through
//! `src/web/seed_loader.js`.

use super::parser::{parse_lino, LinoNode};
use super::SHELL_INTENTS_LINO;

/// How a matched intent recovers the command's argument from the prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShellIntentArgument {
    /// The command takes no argument (`pwd`, `whoami`, `df -h`).
    #[default]
    None,
    /// The command takes a file path recovered from a filename-looking token
    /// (`wc -l Cargo.toml`).
    Path,
    /// The command takes a name introduced by a name-lead cue such as
    /// *called*/*named* (`mkdir build`).
    NameLead,
}

/// One intent → command mapping: the command to emit, how to fill its argument, and
/// the multilingual cue phrases (lowercased) that signal the intent.
#[derive(Debug, Clone, Default)]
pub struct ShellIntent {
    /// The concrete command to emit, e.g. `pwd`, `df -h`, `wc -l`.
    pub command: String,
    /// How the command's argument (if any) is recovered from the prompt.
    pub argument: ShellIntentArgument,
    /// Cue phrases, pooled across languages and lowercased for substring matching.
    pub cues: Vec<String>,
}

/// The semantic shell-intent vocabulary: the ordered intent table plus the
/// name-lead cue words that introduce a `NameLead` argument.
#[derive(Debug, Clone, Default)]
pub struct ShellIntentVocabulary {
    /// Name-lead cue words (*called*, *named*, …), lowercased, pooled across
    /// languages.
    pub name_leads: Vec<String>,
    /// Intent → command mappings in declaration (most-specific-first) order.
    pub intents: Vec<ShellIntent>,
}

/// Parse `data/seed/shell-intents.lino` into the semantic shell-intent vocabulary.
#[must_use]
pub fn shell_intent_vocabulary() -> ShellIntentVocabulary {
    let tree = parse_lino(SHELL_INTENTS_LINO);
    let mut vocab = ShellIntentVocabulary::default();
    let Some(root) = tree.children.first() else {
        return vocab;
    };
    for group in &root.children {
        match group.name.as_str() {
            "name_leads" => vocab.name_leads = collect_language_values(group, "lead"),
            "intents" => {
                vocab.intents = group
                    .children
                    .iter()
                    .filter(|c| c.name == "intent")
                    .map(parse_intent)
                    .collect();
            }
            _ => {}
        }
    }
    vocab
}

/// Parse a single `intent` node into a [`ShellIntent`].
fn parse_intent(node: &LinoNode) -> ShellIntent {
    ShellIntent {
        command: node.find_child_value("command").to_owned(),
        argument: match node.find_child_value("argument") {
            "path" => ShellIntentArgument::Path,
            "name_lead" => ShellIntentArgument::NameLead,
            _ => ShellIntentArgument::None,
        },
        cues: collect_language_values(node, "cue")
            .into_iter()
            .map(|cue| cue.to_lowercase())
            .collect(),
    }
}

/// Collect every `<child_name>` id nested under the `language` children of `group`,
/// pooled across all languages (intent detection is language-agnostic).
fn collect_language_values(group: &LinoNode, child_name: &str) -> Vec<String> {
    group
        .children
        .iter()
        .filter(|c| c.name == "language")
        .flat_map(|lang| {
            lang.children
                .iter()
                .filter(|c| c.name == child_name)
                .map(|c| c.id.clone())
        })
        .collect()
}
