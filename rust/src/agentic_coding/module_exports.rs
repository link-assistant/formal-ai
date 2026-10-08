//! Which functions a module exports (PR #1188 T99, gap G26).
//!
//! "Which functions does src/m.mjs export?" read the module and dumped it. A
//! question that names a module, the seeded export relation
//! (`module_export_question`) and functions (`coding_declaration_noun`) is
//! answered from the module's own declarations: a line that opens with a
//! seeded export marker (`module_export_marker`: export, pub) and declares a
//! function within its next words (`function_declaration_keyword`) exports
//! that function. Twin of `js/agentic/module_exports.mjs`.

use super::module_function::{paths_in, read_source};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use crate::protocol::ChatMessage;
use crate::seed;

const MARKER_REACH: usize = 3;
const CHANGE_ROLES: [&str; 3] = [
    "file_write_action_cue",
    "file_edit_action_cue",
    "coding_text_remove_action",
];

const fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '$'
}

/// The module an export question names.
fn export_question(task: &str) -> Option<String> {
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(task);
    if !lexicon.mentions_role("module_export_question", &normalized)
        || !lexicon.mentions_role("coding_declaration_noun", &normalized)
        // A request that changes the module (add an exported function) is not
        // a question about it.
        || CHANGE_ROLES
            .iter()
            .any(|role| lexicon.mentions_role(role, &normalized))
    {
        return None;
    }
    paths_in(task).into_iter().next()
}

/// The names of the functions `source` declares behind a seeded export
/// marker, each once, in order.
fn exported_functions(source: &str) -> Vec<String> {
    let lexicon = seed::lexicon();
    let markers = lexicon.words_for_role("module_export_marker");
    let keywords = lexicon.words_for_role("function_declaration_keyword");
    let mut names: Vec<String> = Vec::new();
    for line in source.split('\n') {
        let words: Vec<&str> = line.split_whitespace().collect();
        let Some(first) = words.first() else {
            continue;
        };
        if !markers.iter().any(|marker| marker == first) {
            continue;
        }
        let Some(at) = words.iter().enumerate().position(|(index, word)| {
            index > 0 && index <= MARKER_REACH && keywords.iter().any(|keyword| keyword == word)
        }) else {
            continue;
        };
        let Some(next) = words.get(at + 1) else {
            continue;
        };
        let name: String = next
            .chars()
            .take_while(|c| is_identifier_char(*c))
            .collect();
        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// The arm: read the named module, then answer with its exported functions.
pub(super) fn plan_module_exports_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let path = export_question(task)?;
    let current_turn = &messages[super::planner::evidence_window_start(messages)..];
    let Some(source) = read_source(current_turn, &path) else {
        let read = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(
            read,
            super::workspace_change::read_arguments(&path),
        ));
    };
    let names = exported_functions(&source);
    let count = names.len().to_string();
    let listed = names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = [
        ("path", path.as_str()),
        ("count", count.as_str()),
        ("names", listed.as_str()),
    ];
    let intent = if names.is_empty() {
        "module_exports_none"
    } else {
        "module_exports_listed"
    };
    let language = crate::language::detect(task).slug();
    seed::render_response(intent, language, &values)
        .or_else(|| seed::render_response(intent, "en", &values))
        .map(AgenticPlan::Final)
}
