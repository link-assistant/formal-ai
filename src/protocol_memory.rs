use crate::engine::SymbolicAnswer;
use crate::memory::MemoryEvent;
use crate::solver::ConversationTurn;
use crate::solver_handlers::answer_memory_recall;

const REQUEST_HISTORY_CONVERSATION_ID: &str = "request_history";

pub fn answer_from_memory_if_requested(
    prompt: &str,
    history: &[ConversationTurn],
    memory_events: &[MemoryEvent],
) -> Option<SymbolicAnswer> {
    if memory_events.is_empty() {
        return None;
    }
    let events = memory_events_with_request_history(memory_events, history);
    answer_memory_recall(prompt, &events, Some(REQUEST_HISTORY_CONVERSATION_ID))
}

fn memory_events_with_request_history(
    memory_events: &[MemoryEvent],
    history: &[ConversationTurn],
) -> Vec<MemoryEvent> {
    let mut events = Vec::with_capacity(memory_events.len() + history.len());
    events.extend_from_slice(memory_events);
    for (index, turn) in history.iter().enumerate() {
        events.push(MemoryEvent {
            id: format!("request-history-{index}"),
            kind: Some(String::from("message")),
            role: Some(turn.role.slug().to_owned()),
            content: Some(turn.content.clone()),
            conversation_id: Some(String::from(REQUEST_HISTORY_CONVERSATION_ID)),
            conversation_title: Some(String::from("Current request")),
            ..MemoryEvent::default()
        });
    }
    events
}
