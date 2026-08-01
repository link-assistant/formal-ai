//! Grounded structural edits over client-supplied workspace bytes.
//!
//! Unlike literal replacement, collection insertion has no complete old value
//! in the request. The planner must read the target, derive the replacement
//! from those bytes, write the full updated source, and observe the result.

use serde_json::json;

use super::code_artifact::{latest_result, source_from_read_result};
use super::code_task::render_seeded_outcome;
use super::planner::{plan_one, tool_for, write_arguments, AgenticPlan, Capability};
use crate::normal_markov::{quoted_segments, unwrap_transport_quotes};
use crate::protocol::ChatMessage;
use crate::seed;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CollectionInsertion {
    target: String,
    collection: String,
    value: String,
}

pub(super) fn plan_structured_edit_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let task = unwrap_transport_quotes(task);
    let edit = collection_insertion(task)?;
    let latest_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let current_turn = &messages[latest_user + 1..];
    let source = latest_result(current_turn, Capability::Read)
        .as_deref()
        .map(source_from_read_result)
        .filter(|source| !source.is_empty());

    let Some(source) = source else {
        let read_tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(read_tool, read_arguments(&edit.target)));
    };
    let updated = insert_rust_array_value(&source, &edit.collection, &edit.value)?;

    if latest_result(current_turn, Capability::Write).is_none() {
        let write_tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(
            write_tool,
            write_arguments(&edit.target, &updated),
        ));
    }
    if let Some(observed) = latest_result(current_turn, Capability::Run) {
        if observed == updated {
            return Some(AgenticPlan::Final(render_seeded_outcome(
                "coding_workspace_effect_observed",
                task,
                &edit.target,
            )?));
        }
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &edit.target,
        )?));
    }
    let run_tool = tool_for(tool_names, Capability::Run)?;
    Some(plan_one(
        run_tool,
        json!({"command": format!("cat {}", edit.target)}).to_string(),
    ))
}

fn collection_insertion(task: &str) -> Option<CollectionInsertion> {
    // Seed matching is token-bounded. Collapse punctuation to token separators
    // so a concept at the end of a sentence (for example `array.`) remains
    // evidence for the same meaning without adding language-specific syntax.
    let normalized = task
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role(seed::ROLE_FILE_WRITE_ACTION_CUE, &normalized)
        || !lexicon
            .first_role_match(seed::ROLE_CODING_SEARCH_SUBJECT_KIND, &normalized)
            .is_some_and(|meaning| meaning.slug == "coding_search_array")
    {
        return None;
    }
    let values = quoted_segments(task);
    let [value] = values.as_slice() else {
        return None;
    };
    if value.contains(['"', '\\', '\n', '\r']) {
        return None;
    }
    let target = rust_path(task)?;
    let without_literals = task.replacen(&target, "", 1).replacen(value, "", 1);
    let collection = without_literals
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| token.contains('_'))
        .find(|token| valid_identifier(token))?
        .to_owned();
    Some(CollectionInsertion {
        target,
        collection,
        value: value.clone(),
    })
}

fn insert_rust_array_value(source: &str, collection: &str, value: &str) -> Option<String> {
    let declaration = source.find(collection)?;
    let open = source[declaration + collection.len()..]
        .find('[')?
        .checked_add(declaration + collection.len())?;
    let close = source[open + 1..].find(']')?.checked_add(open + 1)?;
    let existing = &source[open + 1..close];
    let literal = format!("\"{value}\"");
    if existing
        .split(',')
        .any(|item| item.trim() == literal.as_str())
    {
        return Some(source.to_owned());
    }
    let insertion = if existing.trim().is_empty() {
        literal
    } else {
        format!(", {literal}")
    };
    let mut updated = String::with_capacity(source.len() + insertion.len());
    updated.push_str(&source[..close]);
    updated.push_str(&insertion);
    updated.push_str(&source[close..]);
    Some(updated)
}

fn rust_path(task: &str) -> Option<String> {
    let end = task.find(".rs")?.checked_add(3)?;
    let start = task[..end]
        .char_indices()
        .rev()
        .take_while(|(_, character)| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
        })
        .last()
        .map_or(0, |(index, _)| index);
    let path = task.get(start..end)?;
    (!path.is_empty()
        && !path.starts_with('/')
        && !path.split('/').any(|component| component == ".."))
    .then(|| path.to_owned())
}

fn valid_identifier(identifier: &str) -> bool {
    let mut characters = identifier.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn read_arguments(path: &str) -> String {
    json!({"path": path, "filePath": path, "file_path": path}).to_string()
}
