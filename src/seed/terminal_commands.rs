//! Terminal-command trigger vocabulary loaded from
//! `data/seed/terminal-commands.lino` (issue #513, visible fix for #511).
//!
//! The natural-language triggers that classify a prompt as a terminal-command
//! request — terminal/shell context phrases, run verbs, Chinese run verbs, and
//! leading shell tokens — used to live as inline `const` arrays in
//! [`crate::solver_terminal`]. They now live once in seed data so a user can
//! retune detection by editing a `.lino` file, and so the project rule against
//! hardcoded natural language in the solver/worker is upheld and CI-enforced.
//!
//! The JavaScript worker loads the synced `src/web/seed/terminal-commands.lino`
//! deployment copy through `src/web/seed_loader.js`, with drift guarded by
//! `experiments/issue-513-sync-worker-terminal.mjs`, so both engines detect
//! terminal commands from one source of truth.

use super::parser::{parse_lino, LinoNode};
use super::TERMINAL_COMMANDS_LINO;

/// The multilingual terminal-command trigger vocabulary, pooled across every
/// supported language because detection itself is language-agnostic.
#[derive(Debug, Clone, Default)]
pub struct TerminalCommandVocabulary {
    /// Terminal/shell/console context phrases (substring-matched).
    pub terminal_phrases: Vec<String>,
    /// Run/execute verbs for space-delimited scripts (word-token matched).
    pub run_verbs: Vec<String>,
    /// Chinese run/execute verbs (substring-matched — CJK has no word breaks).
    pub cjk_run_verbs: Vec<String>,
    /// Explicit command-introducing prefixes whose complete remainder is passed
    /// through without consulting the command-token vocabulary.
    pub passthrough_prefixes: Vec<String>,
    /// Leading shell command tokens (e.g. `ls`, `git`).
    pub shell_tokens: Vec<String>,
}

/// Parse `data/seed/terminal-commands.lino` into the trigger vocabulary.
#[must_use]
pub fn terminal_command_vocabulary() -> TerminalCommandVocabulary {
    let tree = parse_lino(TERMINAL_COMMANDS_LINO);
    let mut vocab = TerminalCommandVocabulary::default();
    let Some(root) = tree.children.first() else {
        return vocab;
    };
    for group in &root.children {
        match group.name.as_str() {
            "terminal_phrases" => vocab.terminal_phrases = collect_language_values(group, "phrase"),
            "run_verbs" => vocab.run_verbs = collect_language_values(group, "verb"),
            "cjk_run_verbs" => vocab.cjk_run_verbs = collect_values(group, "verb"),
            "passthrough_prefixes" => {
                vocab.passthrough_prefixes = collect_language_values(group, "prefix");
            }
            "shell_tokens" => vocab.shell_tokens = collect_values(group, "token"),
            _ => {}
        }
    }
    vocab
}

/// Collect every `<child_name>` id directly under `group`, in declaration order.
fn collect_values(group: &LinoNode, child_name: &str) -> Vec<String> {
    group
        .children
        .iter()
        .filter(|c| c.name == child_name)
        .map(|c| c.id.clone())
        .collect()
}

/// Collect every `<child_name>` id nested under the `language` children of
/// `group`, pooled across all languages (detection is language-agnostic).
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
