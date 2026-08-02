//! Grounded, verified workspace transformations learned from coding-task runs.
//!
//! The planner never edits request prose. It compiles a bounded transformation,
//! reads the client-owned bytes, executes the transformation in memory, writes
//! the complete result, and accepts success only after an exact read-back. The
//! same state machine composes source creation with a second module-registration
//! edit, so a multi-file request cannot stop after its first observable effect.

use serde_json::{json, Value};

use super::code_artifact::source_from_read_result;
use super::code_task::{render_rust_template, render_seeded_outcome, rust_source_for_task};
use super::general_planner::compose_edit_request;
use super::planner::{
    plan_one, tool_capability, tool_for, write_arguments, AgenticPlan, Capability,
};
use crate::normal_markov::{quoted_segments, unwrap_transport_quotes};
use crate::protocol::ChatMessage;
use crate::seed;
use crate::workspace_change_learning::execute_workspace_rewrite;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroundedRewrite {
    target: String,
    pattern: String,
    replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompositeModuleChange {
    source_path: String,
    source: String,
    registration_path: String,
    registration: String,
}

pub(super) fn plan_workspace_change_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let task = unwrap_transport_quotes(task);
    let latest_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let current_turn = &messages[latest_user + 1..];

    if let Some(change) = composite_module_change(task) {
        return plan_composite_step(task, current_turn, tool_names, &change);
    }
    let rewrite = grounded_rewrite(task)?;
    plan_rewrite_step(task, current_turn, tool_names, &rewrite)
}

fn plan_rewrite_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    rewrite: &GroundedRewrite,
) -> Option<AgenticPlan> {
    let Some(read) = result_for_path(current_turn, Capability::Read, &rewrite.target, None) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(&rewrite.target)));
    };
    let source = source_from_read_result(&read);
    let Ok(execution) = execute_workspace_rewrite(&source, &rewrite.pattern, &rewrite.replacement)
    else {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &rewrite.target,
        )?));
    };
    let updated = execution.output;

    if result_for_path(
        current_turn,
        Capability::Write,
        &rewrite.target,
        Some(&updated),
    )
    .is_none()
    {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(tool, write_arguments(&rewrite.target, &updated)));
    }
    let command = format!("cat {}", rewrite.target);
    let Some(observed) = result_for_command(current_turn, &command) else {
        let tool = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(tool, json!({"command": command}).to_string()));
    };
    let intent = if observed == updated {
        "coding_workspace_effect_observed"
    } else {
        "coding_workspace_verification_failed"
    };
    Some(AgenticPlan::Final(render_seeded_outcome(
        intent,
        task,
        &rewrite.target,
    )?))
}

fn plan_composite_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    change: &CompositeModuleChange,
) -> Option<AgenticPlan> {
    if result_for_path(
        current_turn,
        Capability::Write,
        &change.source_path,
        Some(&change.source),
    )
    .is_none()
    {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(
            tool,
            write_arguments(&change.source_path, &change.source),
        ));
    }

    let source_command = format!("cat {}", change.source_path);
    let Some(observed_source) = result_for_command(current_turn, &source_command) else {
        let tool = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(
            tool,
            json!({"command": source_command}).to_string(),
        ));
    };
    if observed_source != change.source {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &change.source_path,
        )?));
    }

    let Some(read) = result_for_path(
        current_turn,
        Capability::Read,
        &change.registration_path,
        None,
    ) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(&change.registration_path)));
    };
    let current = source_from_read_result(&read);
    let updated = insert_registration(&current, &change.registration);
    if updated == current {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_effect_observed",
            task,
            &change.registration_path,
        )?));
    }

    if result_for_path(
        current_turn,
        Capability::Write,
        &change.registration_path,
        Some(&updated),
    )
    .is_none()
    {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(
            tool,
            write_arguments(&change.registration_path, &updated),
        ));
    }

    let registration_command = format!("cat {}", change.registration_path);
    let Some(observed_registration) = result_for_command(current_turn, &registration_command)
    else {
        let tool = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(
            tool,
            json!({"command": registration_command}).to_string(),
        ));
    };
    let intent = if observed_registration == updated {
        "coding_workspace_effect_observed"
    } else {
        "coding_workspace_verification_failed"
    };
    Some(AgenticPlan::Final(render_seeded_outcome(
        intent,
        task,
        &change.registration_path,
    )?))
}

fn grounded_rewrite(task: &str) -> Option<GroundedRewrite> {
    let (target, old_clause, new_clause) = compose_edit_request(task)?;
    let quoted = quoted_segments(task);
    let operands = match quoted.as_slice() {
        [old, new] => Some((old.clone(), new.clone())),
        _ if seed::lexicon().mentions_role(
            seed::ROLE_CODING_IDENTIFIER_RENAME_ACTION,
            &task.to_lowercase(),
        ) =>
        {
            Some((
                identifier_tokens(&old_clause).next_back()?.to_owned(),
                identifier_tokens(&new_clause).next()?.to_owned(),
            ))
        }
        _ => None,
    }?;
    let (old, new) = operands;
    if old.is_empty() || old == new || new.contains(&old) {
        return None;
    }
    Some(GroundedRewrite {
        target,
        pattern: old,
        replacement: new,
    })
}

// This is a seed-template placeholder, not a Rust formatting argument.
#[allow(clippy::literal_string_with_formatting_args)]
fn composite_module_change(task: &str) -> Option<CompositeModuleChange> {
    let lowered = task.to_lowercase();
    if !seed::lexicon().mentions_role(seed::ROLE_CODING_MODULE_REGISTRATION_ACTION, &lowered) {
        return None;
    }
    let generated = rust_source_for_task(task)?;
    let registration_path = rust_paths(task)
        .into_iter()
        .find(|path| path != &generated.path)?;
    let module = generated.path.rsplit('/').next()?.strip_suffix(".rs")?;
    if !valid_identifier(module) {
        return None;
    }
    let registration =
        render_rust_template("coding_source_module_registration", &[("{module}", module)])?;
    Some(CompositeModuleChange {
        source_path: generated.path,
        source: generated.content,
        registration_path,
        registration,
    })
}

fn insert_registration(source: &str, registration: &str) -> String {
    if source
        .lines()
        .any(|line| line.trim() == registration.trim())
    {
        return source.to_owned();
    }
    let mut updated = source.to_owned();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(registration);
    updated
}

fn identifier_tokens(text: &str) -> impl DoubleEndedIterator<Item = &str> {
    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| valid_identifier(token))
}

fn valid_identifier(identifier: &str) -> bool {
    let mut characters = identifier.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
        && !seed::lexicon()
            .words_for_role(seed::ROLE_IDENTIFIER_RESERVED_WORD)
            .iter()
            .any(|reserved| reserved == identifier)
}

fn rust_paths(task: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for (suffix, _) in task.match_indices(".rs") {
        let end = suffix + 3;
        let start = task[..end]
            .char_indices()
            .rev()
            .take_while(|(_, character)| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
            })
            .last()
            .map_or(0, |(index, _)| index);
        let path = &task[start..end];
        if !path.is_empty()
            && !path.starts_with('/')
            && !path.split('/').any(|component| component == "..")
            && !paths.iter().any(|existing| existing == path)
        {
            paths.push(path.to_owned());
        }
    }
    paths
}

fn result_for_path(
    messages: &[ChatMessage],
    capability: Capability,
    path: &str,
    expected_content: Option<&str>,
) -> Option<String> {
    matching_result(messages, |name, arguments| {
        if tool_capability(name) != Some(capability) {
            return false;
        }
        let Ok(value) = serde_json::from_str::<Value>(arguments) else {
            return false;
        };
        let matches_path = ["path", "filePath", "file_path"]
            .iter()
            .any(|key| value.get(key).and_then(Value::as_str) == Some(path));
        matches_path
            && expected_content.is_none_or(|expected| {
                value.get("content").and_then(Value::as_str) == Some(expected)
            })
    })
}

fn result_for_command(messages: &[ChatMessage], command: &str) -> Option<String> {
    matching_result(messages, |name, arguments| {
        if tool_capability(name) != Some(Capability::Run) {
            return false;
        }
        serde_json::from_str::<Value>(arguments)
            .ok()
            .and_then(|value| {
                value
                    .get("command")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .as_deref()
            == Some(command)
    })
}

fn matching_result(
    messages: &[ChatMessage],
    matches: impl Fn(&str, &str) -> bool,
) -> Option<String> {
    messages
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, message)| {
            if !message.role.eq_ignore_ascii_case("tool") {
                return None;
            }
            let id = message.tool_call_id.as_deref()?;
            let call = messages[..index]
                .iter()
                .rev()
                .flat_map(|prior| prior.tool_calls.iter().rev())
                .find(|call| call.id == id)?;
            matches(&call.function.name, &call.function.arguments)
                .then(|| message.content.plain_text())
        })
}

fn read_arguments(path: &str) -> String {
    json!({"path": path, "filePath": path, "file_path": path}).to_string()
}
