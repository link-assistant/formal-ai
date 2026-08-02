//! Bridge agentic chat messages into the shared conversation-history solver.
//!
//! Conversation recognition, multilingual surface forms, summarization, and
//! evidence tracing all belong to the universal solver. The agentic planner only
//! adapts protocol messages to [`ConversationTurn`] values and accepts an answer
//! when that solver classifies it as a conversation summary. This prevents the
//! Agent CLI surface from growing a second phrase table or a second memory model.

use crate::protocol::{chat_prompt_and_history, ChatMessage};
use crate::solve_with_history;

/// Answer a conversation-summary request through the universal history solver.
pub(super) fn recall_answer_for(messages: &[ChatMessage]) -> Option<String> {
    let (prompt, history) = chat_prompt_and_history(messages);
    if prompt.trim().is_empty() {
        return None;
    }
    let answer = solve_with_history(&prompt, &history);
    (answer.intent == "summarize_conversation").then_some(answer.answer)
}
