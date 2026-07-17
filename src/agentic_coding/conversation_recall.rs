//! Bridge agentic chat messages into the shared conversation-history solver.
//!
//! Conversation recognition, multilingual surface forms, summarization, and
//! evidence tracing all belong to the universal solver. The agentic planner only
//! adapts protocol messages to [`ConversationTurn`] values and accepts an answer
//! when that solver classifies it as a conversation summary. This prevents the
//! Agent CLI surface from growing a second phrase table or a second memory model.

use crate::protocol::ChatMessage;
use crate::{solve_with_history, ConversationTurn};

/// Answer a conversation-summary request through the universal history solver.
pub(super) fn recall_answer_for(messages: &[ChatMessage]) -> Option<String> {
    let latest_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let prompt = messages[latest_user].content.plain_text();
    let history = messages[..latest_user]
        .iter()
        .filter_map(|message| {
            let text = message.content.plain_text();
            if text.trim().is_empty() {
                return None;
            }
            if message.role.eq_ignore_ascii_case("user") {
                Some(ConversationTurn::user(text))
            } else if message.role.eq_ignore_ascii_case("assistant") {
                Some(ConversationTurn::assistant(text))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let answer = solve_with_history(&prompt, &history);
    (answer.intent == "summarize_conversation").then_some(answer.answer)
}
