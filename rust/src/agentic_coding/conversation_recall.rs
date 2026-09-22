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
/// the program. The solver's typed execution recipe is lowered into the
/// client's own write and run tools when both are advertised; a client without
/// them still receives the composed program as the answer.
pub(super) fn plan_shared_solver_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let (prompt, history) = chat_prompt_and_history(messages);
    if prompt.trim().is_empty() {
        return None;
    }
    let answer = solve_with_history(&prompt, &history);
    match answer.intent.as_str() {
        "summarize_conversation" => Some(AgenticPlan::Final(answer.answer)),
        "write_program" | "substitution_rule_export" => Some(
            command_reroute::plan_symbolic_command_reroute(messages, tool_names, &answer)
                .unwrap_or(AgenticPlan::Final(answer.answer)),
        ),
        _ => None,
    }
}
