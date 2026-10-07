//! Input-token usage for the chat and responses surfaces.
//!
//! Split out of `protocol.rs` (the 1000-line Rust ceiling).

use super::{ChatMessage, MessageContent, ResponsesRequest};
use crate::engine::estimate_tokens;

fn message_content_tokens(content: &MessageContent) -> u32 {
    match content {
        MessageContent::Text(text) => estimate_tokens(text),
        MessageContent::Parts(parts) => parts.iter().fold(0, |total, part| {
            total.saturating_add(part.text.as_deref().map_or(0, estimate_tokens))
        }),
    }
}

/// Count only role-visible message content. Tool call names and arguments are
/// excluded from input usage because they are protocol metadata, while tool
/// result content is included like every other message body.
pub(super) fn message_input_tokens(messages: &[ChatMessage]) -> u32 {
    messages.iter().fold(0, |total, message| {
        total.saturating_add(message_content_tokens(&message.content))
    })
}

pub(super) fn responses_input_tokens(request: &ResponsesRequest) -> u32 {
    message_input_tokens(&request.to_chat_completion_request().messages)
}
