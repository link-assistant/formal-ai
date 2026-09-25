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

use crate::protocol::{chat_prompt_and_history, ChatMessage};
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
                SharedSolverStep::Ready(AgenticPlan::ToolCalls(vec![super::planner::PlannedToolCall {
                    tool: "translate".to_owned(),
                    arguments: format!(
                        "{{\"from\":\"{}\",\"to\":\"{}\",\"path\":\"{}\",\"write\":true}}",
                        request.from.name(),
                        request.to.name(),
                        request.path
                    ),
                }]))
            } else {
                SharedSolverStep::Ready(AgenticPlan::Final(answer.answer))
            }
        }
        _ => SharedSolverStep::NotOurs,
    }
}
