//! Bridge agentic chat messages into the shared conversation-history solver.
//!
//! Conversation recognition, multilingual surface forms, summarization, and
//! evidence tracing all belong to the universal solver. The agentic planner only
//! adapts protocol messages to [`ConversationTurn`] values and accepts an answer
//! when that solver classifies it into a family the planner's own routes cannot
//! decide: a conversation summary, or a program-writing request whose typed
//! execution recipe has to be lowered into the client's real tools. This
//! prevents the Agent CLI surface from growing a second phrase table or a second
//! memory model.

use crate::protocol::{ChatMessage, chat_prompt_and_history};
use crate::solve_with_history;

use super::command_reroute;
use super::planner::AgenticPlan;

/// One consult of the shared history solver for the families the planner's own
/// routes cannot decide.
///
/// A conversation summary is answered as final prose. A program-writing request
/// -- `write_program`, or the `substitution_rule_export` follow-up that turns a
/// written program into its rule artifact -- is owned by that solver on every
/// surface: over a full toolset, *"Sort the results in reverse order and export
/// the substitution rule to JavaScript"* was answered by a web search for the
/// whole sentence because no agentic route asked the solver that owned it
/// (issue #936), and *"Write me a Rust program that lists the files in the
/// current directory"* planned a directory listing of the workspace instead of
/// the program. When the typed execution recipe can be lowered into the
/// client's own write and run tools, the planner *defers*: a template write
/// that never runs the program is not a completion the write-effect ladder
/// accepts, so the server's recipe path (`protocol::command_reroute_plan`)
/// owns the whole verified write--check--run chain (issue #916) and the planner
/// stops routing before any later arm can steal the prompt back. A client
/// without those tools still receives the composed program as the answer.
pub(super) enum SharedSolverStep {
    /// No family the planner delegated to the solver: keep routing.
    NotOurs,
    /// The solver's typed recipe owns this turn and the client can run it:
    /// yield the whole request so the server's recipe path lowers it.
    Defer,
    /// The solver answered directly.
    Ready(AgenticPlan),
}

pub(super) fn plan_shared_solver_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> SharedSolverStep {
    let (prompt, history) = chat_prompt_and_history(messages);
    if prompt.trim().is_empty() {
        return SharedSolverStep::NotOurs;
    }
    let owned = crate::meta_translate::owned_source_tree_request(&prompt);
    if owned.is_none() && crate::meta_translate::source_tree_request(&prompt).is_some() {
        return SharedSolverStep::NotOurs;
    }
    if tool_names.contains(&"translate")
        && let Some(owned) = owned
    {
        let expected = serde_json::json!({"from":owned.request.from.name(),"to":owned.request.to.name(),"path":&owned.request.path,"write":owned.write});
        if let Some(failure) = repeated_owned_operation_failure(messages, "translate", &expected) {
            return SharedSolverStep::Ready(AgenticPlan::Final(
                super::tool_result::render_failure("translate", &failure, &prompt),
            ));
        }
        return SharedSolverStep::Ready(AgenticPlan::ToolCalls(vec![
            super::planner::PlannedToolCall {
                tool: "translate".to_owned(),
                arguments: format!(
                    r#"{{"from":"{}","to":"{}","path":{},"write":{}}}"#,
                    owned.request.from.name(),
                    owned.request.to.name(),
                    serde_json::json!(owned.request.path),
                    owned.write
                ),
            },
        ]));
    }
    let answer = solve_with_history(&prompt, &history);
    match answer.intent.as_str() {
        "summarize_conversation" => SharedSolverStep::Ready(AgenticPlan::Final(answer.answer)),
        "write_program" | "substitution_rule_export" => {
            if command_reroute::plan_symbolic_command_reroute(messages, tool_names, &answer)
                .is_some()
            {
                SharedSolverStep::Defer
            } else {
                SharedSolverStep::Ready(AgenticPlan::Final(answer.answer))
            }
        }
        // Plan 16 L2g: the source-tree translation family. A client that
        // advertises the `translate` tool owns the read--translate--write
        // chain inside its own workspace, so the solver's in-process answer
        // (which reads the working directory, not the workspace) yields to
        // one tool call. Without the tool, the solver's rendered target (or
        // its honest gap) is the answer, exactly like `write_program`.
        "translate_source_tree" => {
            if tool_names.contains(&"translate")
                && let Some(request) = crate::meta_translate::source_tree_request(&prompt)
            {
                SharedSolverStep::Ready(AgenticPlan::ToolCalls(vec![
                    super::planner::PlannedToolCall {
                        tool: "translate".to_owned(),
                        arguments: format!(
                            "{{\"from\":\"{}\",\"to\":\"{}\",\"path\":\"{}\",\"write\":true}}",
                            request.from.name(),
                            request.to.name(),
                            request.path
                        ),
                    },
                ]))
            } else {
                SharedSolverStep::Ready(AgenticPlan::Final(answer.answer))
            }
        }
        _ => SharedSolverStep::NotOurs,
    }
}

fn repeated_owned_operation_failure(
    messages: &[ChatMessage],
    tool: &str,
    expected: &serde_json::Value,
) -> Option<String> {
    let start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    let mut seen = std::collections::HashSet::<String>::new();
    let mut failures = Vec::new();
    for (index, message) in messages.iter().enumerate().skip(start) {
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let Some(identifier) = message.tool_call_id.as_ref() else {
            continue;
        };
        if seen.contains(identifier) {
            continue;
        }
        let Some(call) = messages[start..index]
            .iter()
            .rev()
            .flat_map(|prior| prior.tool_calls.iter().rev())
            .find(|call| &call.id == identifier)
        else {
            continue;
        };
        if call.function.name != tool
            || message
                .name
                .as_ref()
                .is_some_and(|name| !name.eq_ignore_ascii_case(tool))
        {
            continue;
        }
        let Ok(observed) = serde_json::from_str::<serde_json::Value>(&call.function.arguments)
        else {
            continue;
        };
        if &observed != expected {
            continue;
        }
        seen.insert(identifier.clone());
        if let Some(failure) = super::tool_result::failure_message(
            &message.content.plain_text(),
            message.is_error,
            false,
        ) {
            failures.push(failure);
        } else {
            failures.clear();
        }
    }
    let latest = failures.last()?;
    (failures.iter().filter(|failure| *failure == latest).count() >= 2).then(|| latest.clone())
}
