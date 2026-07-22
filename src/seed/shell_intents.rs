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
    /// One path/name recovered from the request (`rm old.txt`).
    OnePath,
    /// Two path/name operands recovered in request order (`cp a.txt b.txt`).
    TwoPaths,
    /// The command takes the text after the matched intent cue (`rg QUERY .`).
    Remainder,
    /// A local-code search query, with local-scope words removed.
    SearchQuery,
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

/// Commands selected from a workspace's package-manager marker file.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceCommands {
    /// File whose presence identifies the workspace toolchain.
    pub marker: String,
    /// Test command for this workspace.
    pub test: String,
    /// Dependency-install command for this workspace.
    pub install: String,
    /// Build command for this workspace.
    pub build: String,
}

/// One local filesystem scope and the shell root searched for that scope.
#[derive(Debug, Clone, Default)]
pub struct LocalPathSearchScope {
    /// Shell expression used as the first `find` operand.
    pub root: String,
    /// Multilingual phrases that identify this as a local filesystem request.
    pub cues: Vec<String>,
}

/// One requested filesystem object kind and its portable `find` predicate.
#[derive(Debug, Clone, Default)]
pub struct LocalPathSearchKind {
    /// Predicate such as `-type d` or `-type f`.
    pub predicate: String,
    /// Multilingual nouns that identify the requested kind.
    pub cues: Vec<String>,
}

/// The semantic shell-intent vocabulary: the ordered intent table plus the
/// name-lead cue words that introduce a `NameLead` argument.
#[derive(Debug, Clone, Default)]
pub struct ShellIntentVocabulary {
    /// Name-lead cue words (*called*, *named*, …), lowercased, pooled across
    /// languages.
    pub name_leads: Vec<String>,
    /// Natural-language glue ignored while recovering path and search operands.
    pub argument_noise: Vec<String>,
    /// Phrases that distinguish repository/file search from internet search.
    pub local_search_scopes: Vec<String>,
    /// Seed-defined portable command with root, predicate, and pattern slots.
    pub local_path_search_command_template: String,
    /// Verbs that ask to discover a path by name.
    pub local_path_search_actions: Vec<String>,
    /// Local filesystem scope phrases and the roots they map to.
    pub local_path_search_scopes: Vec<LocalPathSearchScope>,
    /// File/folder kind phrases and their `find` predicates.
    pub local_path_search_kinds: Vec<LocalPathSearchKind>,
    /// Workspace marker → test/install/build mappings in preference order.
    pub workspace_commands: Vec<WorkspaceCommands>,
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
            "argument_noise" => {
                vocab.argument_noise = collect_language_values(group, "word");
            }
            "local_search_scopes" => {
                vocab.local_search_scopes = collect_language_values(group, "scope");
            }
            "local_path_search" => {
                group
                    .find_child_value("command_template")
                    .clone_into(&mut vocab.local_path_search_command_template);
                vocab.local_path_search_actions = group
                    .children
                    .iter()
                    .find(|child| child.name == "actions")
                    .map(|node| collect_language_values(node, "action"))
                    .unwrap_or_default();
                vocab.local_path_search_scopes = group
                    .children
                    .iter()
                    .find(|child| child.name == "scopes")
                    .map(|node| {
                        node.children
                            .iter()
                            .filter(|child| child.name == "scope")
                            .map(|scope| LocalPathSearchScope {
                                root: scope.find_child_value("root").to_owned(),
                                cues: collect_language_values(scope, "cue")
                                    .into_iter()
                                    .map(|cue| cue.to_lowercase())
                                    .collect(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                vocab.local_path_search_kinds = group
                    .children
                    .iter()
                    .find(|child| child.name == "kinds")
                    .map(|node| {
                        node.children
                            .iter()
                            .filter(|child| child.name == "kind")
                            .map(|kind| LocalPathSearchKind {
                                predicate: kind.find_child_value("predicate").to_owned(),
                                cues: collect_language_values(kind, "cue")
                                    .into_iter()
                                    .map(|cue| cue.to_lowercase())
                                    .collect(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
            }
            "workspace_commands" => {
                vocab.workspace_commands = group
                    .children
                    .iter()
                    .filter(|child| child.name == "workspace")
                    .map(|node| WorkspaceCommands {
                        marker: node.find_child_value("marker").to_owned(),
                        test: node.find_child_value("test").to_owned(),
                        install: node.find_child_value("install").to_owned(),
                        build: node.find_child_value("build").to_owned(),
                    })
                    .collect();
            }
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
            "one_path" => ShellIntentArgument::OnePath,
            "two_paths" => ShellIntentArgument::TwoPaths,
            "remainder" => ShellIntentArgument::Remainder,
            "search_query" => ShellIntentArgument::SearchQuery,
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
