//! Capability-first routing for the shared agentic CLI tool set (issue #758).

use serde_json::json;

use super::general_planner::compose_edit_request;
use super::planner::{plan_one, AgenticPlan, Capability};
use crate::protocol::ChatMessage;
use crate::seed;

/// The advertised tool name that provides `capability`. Client-executed MCP
/// tools take precedence over protocol-native hosted tools: a hosted search is
/// executed by the upstream model provider, while an MCP tool produces a result
/// that the CLI can return in the next turn. Canonical aliases remain ordered by
/// the registry, followed by the compatibility classifier for namespaced tools
/// used by other harnesses.
pub(super) fn tool_for<'a>(tool_names: &[&'a str], capability: Capability) -> Option<&'a str> {
    if let Some(name) = tool_names.iter().copied().find(|name| {
        name.to_ascii_lowercase().starts_with("mcp__") && classify_tool(name) == Some(capability)
    }) {
        return Some(name);
    }
    let registry = seed::agentic_tool_capabilities();
    let entry = registry
        .iter()
        .find(|entry| entry.id == capability.registry_id())?;
    entry
        .aliases
        .iter()
        .find_map(|alias| {
            tool_names
                .iter()
                .copied()
                .find(|name| alias.eq_ignore_ascii_case(name))
        })
        .or_else(|| {
            tool_names
                .iter()
                .copied()
                .find(|name| classify_tool(name) == Some(capability))
        })
}

fn tool_matches_capability(name: &str, capability: Capability) -> bool {
    seed::agentic_tool_capabilities()
        .into_iter()
        .find(|entry| entry.id == capability.registry_id())
        .is_some_and(|entry| {
            entry
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(name))
        })
}

/// Classify a tool name through the shared alias registry, with legacy
/// substring matching as a compatibility fallback.
pub(super) fn classify_tool(name: &str) -> Option<Capability> {
    for capability in [
        Capability::Search,
        Capability::Fetch,
        Capability::Read,
        Capability::Write,
        Capability::Edit,
        Capability::Run,
        Capability::Grep,
        Capability::Glob,
        Capability::ListDir,
        Capability::Todo,
        Capability::Subagent,
        Capability::ReadMany,
        Capability::MultiEdit,
    ] {
        if tool_matches_capability(name, capability) {
            return Some(capability);
        }
    }
    let lower = name.to_ascii_lowercase();
    if lower.contains("todo") {
        return None;
    }
    if matches!(lower.as_str(), "computer_use" | "code_interpreter") {
        Some(Capability::Run)
    } else if lower.contains("search") {
        (lower.contains("web") && lower != "tool_search").then_some(Capability::Search)
    } else if lower == "read"
        || lower.contains("read_file")
        || lower.contains("read_local_file")
        || lower.contains("file_read")
        || lower.contains("open_file")
        || lower.contains("view_file")
    {
        Some(Capability::Read)
    } else if lower.contains("fetch")
        || lower.contains("open")
        || lower.contains("browse")
        || lower.contains("get_url")
        || lower.contains("read_url")
    {
        Some(Capability::Fetch)
    } else if lower.contains("write") || lower.contains("create_file") {
        Some(Capability::Write)
    } else if lower.contains("edit") || lower.contains("patch") || lower.contains("replace") {
        Some(Capability::Edit)
    } else if lower.contains("run")
        || lower.contains("bash")
        || lower.contains("command")
        || lower.contains("exec")
        || lower.contains("shell")
    {
        Some(Capability::Run)
    } else {
        None
    }
}

pub(super) fn plan_shared_capability_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let capability = [
        Capability::Grep,
        Capability::MultiEdit,
        Capability::ReadMany,
        Capability::Glob,
        Capability::ListDir,
        Capability::Todo,
        Capability::Subagent,
    ]
    .into_iter()
    .find(|capability| task_matches(task, *capability))?;
    if super::tool_result::has_latest_turn_result(messages) {
        return super::tool_result::latest_turn_answer(messages, tool_names, task)
            .map(AgenticPlan::Final);
    }
    if let Some(tool) = tool_for(tool_names, capability) {
        return Some(plan_one(tool, arguments_for(capability, task)));
    }
    if let Some(tool) = tool_for(tool_names, Capability::Run) {
        if let Some(command) = shell_fallback(capability, task) {
            return Some(plan_one(tool, json!({"command": command}).to_string()));
        }
    }
    None
}

fn task_matches(task: &str, capability: Capability) -> bool {
    let lower = task.to_lowercase();
    if capability == Capability::MultiEdit && file_tokens(task).len() > 1 {
        return compose_edit_request(task).is_some();
    }
    let id = capability.registry_id();
    seed::agentic_tool_capabilities()
        .into_iter()
        .find(|entry| entry.id == id)
        .is_some_and(|entry| entry.cues.iter().any(|cue| lower.contains(cue)))
}

fn arguments_for(capability: Capability, task: &str) -> String {
    match capability {
        Capability::Grep => {
            let query = super::shell_command::code_search_query_for_task(task)
                .unwrap_or_else(|| task.to_owned());
            json!({"query": query, "pattern": query}).to_string()
        }
        Capability::Glob => {
            let pattern = wildcard_token(task).unwrap_or("*");
            json!({"pattern": pattern, "path": "."}).to_string()
        }
        Capability::ListDir => json!({"path": "."}).to_string(),
        Capability::Todo => json!({
            "todos": [{"content": task, "status": "pending"}],
            "plan": [{"step": task, "status": "pending"}],
        })
        .to_string(),
        Capability::Subagent => json!({
            "description": task,
            "prompt": task,
            "input": task,
            "subagent_type": "general",
        })
        .to_string(),
        Capability::ReadMany => {
            let paths = file_tokens(task);
            json!({"paths": paths, "file_paths": paths}).to_string()
        }
        Capability::MultiEdit => {
            let (path, old, new) = compose_edit_request(task)
                .unwrap_or_else(|| (String::from("."), String::new(), String::new()));
            let paths = file_tokens(task);
            json!({
                "path": path,
                "paths": paths,
                "edits": [{
                    "old": old,
                    "new": new,
                    "old_string": old,
                    "new_string": new,
                }],
            })
            .to_string()
        }
        _ => json!({"prompt": task}).to_string(),
    }
}

fn wildcard_token(task: &str) -> Option<&str> {
    task.split_whitespace()
        .map(|token| token.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '`' | '"' | '\'')))
        .find(|token| token.contains('*') || token.contains('?') || token.contains('['))
}

fn file_tokens(task: &str) -> Vec<&str> {
    task.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '`' | '"' | '\'' | '(' | ')'))
        })
        .filter(|token| {
            token.contains('.')
                && !token.starts_with('.')
                && !token.ends_with('.')
                && !token.contains("//")
        })
        .collect()
}

fn shell_fallback(capability: Capability, task: &str) -> Option<String> {
    match capability {
        Capability::Grep => super::shell_command::shell_command_for_task(task),
        Capability::Glob => {
            let pattern = wildcard_token(task).unwrap_or("*").replace('\'', "'\\''");
            let mut command = String::from("find");
            command.push_str(" .");
            command.push_str(" -path ");
            command.push('\'');
            command.push_str(&pattern);
            command.push('\'');
            Some(command)
        }
        Capability::ListDir => Some(String::from("ls")),
        Capability::ReadMany => {
            let paths = file_tokens(task);
            (!paths.is_empty()).then(|| {
                let paths = paths
                    .into_iter()
                    .map(shell_quote)
                    .collect::<Vec<_>>()
                    .join(" ");
                let mut command = String::from("cat");
                command.push(' ');
                command.push_str(&paths);
                command
            })
        }
        _ => None,
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
