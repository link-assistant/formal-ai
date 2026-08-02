//! Issue #680: the general, intent-based capability router.
//!
//! Tool-call emission in `formal-ai serve` is a function of **intent** — the
//! advertised tool set plus the request's semantics — not of a literal phrasing
//! or a pinned recipe. This module holds the three general capability probes
//! ([`plan_web_fetch_step`], [`plan_web_search_step`], [`plan_edit_step`]) that
//! [`super::planner::plan_chat_step`] runs for an arbitrary request: each fires
//! only when the request carries that capability's intent (recovered entirely
//! from the seed lexicon, never from hardcoded natural language — CONTRIBUTING
//! §2) *and* the CLI actually advertised a matching tool, otherwise it returns
//! [`None`] so the planner keeps looking and ultimately falls through to the
//! prose answer rather than fabricating a call the client cannot honour.
//!
//! The probes share the planner's own step primitives ([`super::planner`]'s
//! `Progress`, `tool_for`, `plan_one`, `fetch_arguments`) so routing here never
//! drifts from the recipe routing there.

use serde_json::json;

use super::general_planner::compose_edit_request;
use super::planner::{fetch_arguments, plan_one, tool_for, AgenticPlan, Capability, Progress};
use super::tool_result;
use crate::protocol::ChatMessage;

/// General web-fetch routing (issue #680): when the request carries HTTP-fetch
/// intent (any phrasing, any supported language) *and* the CLI advertised a fetch
/// tool, emit a real fetch `tool_call` for the named URL. Returns [`None`] when
/// there is no fetch intent or no fetch tool was advertised, so the planner keeps
/// looking (and ultimately falls through to the prose answer) rather than
/// fabricating a call the client cannot honour.
pub(super) fn plan_web_fetch_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let url = crate::solver_handlers::agentic_fetch_url_for(task)?;
    let tool = tool_for(tool_names, Capability::Fetch)?;
    let progress = Progress::scan(messages);
    if progress.done(Capability::Fetch) {
        return Some(AgenticPlan::Final(tool_result::render(
            "web_fetch",
            progress.fetch_result().unwrap_or_default(),
            task,
        )));
    }
    Some(plan_one(tool, fetch_arguments(&url)))
}

/// General web-search routing (issue #680): when the request carries web-search
/// intent (any phrasing, any supported language) *and* the CLI advertised a
/// search tool, emit a real search `tool_call` for the extracted query. Returns
/// [`None`] when there is no search intent or no search tool was advertised.
pub(super) fn plan_web_search_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let query = crate::solver_handlers::web_search_query_for(task)?;
    let Some(tool) = tool_for(tool_names, Capability::Search) else {
        let discovery = tool_names
            .iter()
            .copied()
            .find(|name| name.eq_ignore_ascii_case("tool_search"))?;
        if tool_result_exists(messages, discovery) {
            return None;
        }
        return Some(plan_one(
            discovery,
            json!({"query": "web search", "max_results": 5}).to_string(),
        ));
    };
    let progress = Progress::scan(messages);
    if progress.done(Capability::Search) {
        return Some(AgenticPlan::Final(tool_result::render(
            "web_search",
            progress.search_result().unwrap_or_default(),
            task,
        )));
    }
    Some(plan_one(tool, json!({ "query": query }).to_string()))
}

fn tool_result_exists(messages: &[ChatMessage], tool_name: &str) -> bool {
    let current_turn = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    messages
        .iter()
        .enumerate()
        .skip(current_turn)
        .any(|(index, message)| {
            if !message.role.eq_ignore_ascii_case("tool") {
                return false;
            }
            if message
                .name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(tool_name))
            {
                return true;
            }
            let Some(call_id) = message.tool_call_id.as_deref() else {
                return false;
            };
            messages[..index]
                .iter()
                .flat_map(|prior| &prior.tool_calls)
                .any(|call| {
                    call.id == call_id && call.function.name.eq_ignore_ascii_case(tool_name)
                })
        })
}

/// General file-edit routing (issue #680): when the request carries a
/// file-modification intent — an edit action, a replacement lead, and a named
/// target file, in any phrasing/language — *and* the CLI advertised an edit tool,
/// read the target first when a read tool is available, then emit a real edit
/// `tool_call` that replaces the recovered old text with the new text. Reading
/// first satisfies editing clients that enforce read-before-write and grounds the
/// replacement in the current file. Returns [`None`] when there is no edit intent
/// or no edit tool was advertised, so the planner keeps looking (and ultimately
/// falls through to the prose answer) rather than fabricating a call the client
/// cannot honour.
///
/// The `(target, old, new)` triple is recovered entirely from the seed lexicon
/// (the `file_edit_*` roles), not from any pinned phrasing, and the edit tool's
/// arguments are emitted under every common key alias so one shape drives any
/// CLI's edit/patch/replace tool (see [`edit_arguments`]).
pub(super) fn plan_edit_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let (target, old, new) = compose_edit_request(task)?;
    let tool = tool_for(tool_names, Capability::Edit)?;
    let progress = Progress::scan(messages);
    if progress.done(Capability::Edit) {
        return tool_result::latest_turn_answer(messages, tool_names, task).map(AgenticPlan::Final);
    }
    if let Some(read_tool) =
        tool_for(tool_names, Capability::Read).filter(|_| !progress.done(Capability::Read))
    {
        return Some(plan_one(read_tool, read_arguments(&target)));
    }
    Some(plan_one(tool, edit_arguments(&target, &old, &new)))
}

fn read_arguments(path: &str) -> String {
    json!({
        "path": path,
        "filePath": path,
        "file_path": path,
    })
    .to_string()
}

/// Arguments for an edit step that satisfy whichever key an advertised edit tool
/// expects. Agentic CLIs disagree on the parameter names — opencode's `edit`
/// wants `filePath`/`oldString`/`newString`, Gemini/Qwen's `replace` wants
/// `file_path`/`old_string`/`new_string`, and Anthropic's `str_replace` wants
/// `path`/`old_str`/`new_str`. All aliases are emitted; a schema-validating CLI
/// keeps the ones it declared and strips the rest, so the same plan drives any of
/// them without a per-CLI special case (issue #680), mirroring
/// [`super::planner`]'s `write_arguments`.
fn edit_arguments(path: &str, old: &str, new: &str) -> String {
    json!({
        "path": path,
        "filePath": path,
        "file_path": path,
        "oldString": old,
        "old_string": old,
        "old_str": old,
        "old": old,
        "newString": new,
        "new_string": new,
        "new_str": new,
        "new": new,
    })
    .to_string()
}
