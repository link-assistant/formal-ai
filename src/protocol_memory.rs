use crate::engine::SymbolicAnswer;
use crate::memory::{MemoryEvent, MemoryStore};
use crate::memory_program::{MemoryProgramAuthorization, MemoryProgramLimits};
use crate::solver::ConversationTurn;
use crate::solver_handlers::{
    answer_memory_recall, execute_memory_query_with_options, is_exact_memory_query,
};

const REQUEST_HISTORY_CONVERSATION_ID: &str = "request_history";

pub fn answer_from_memory_if_requested(
    prompt: &str,
    history: &[ConversationTurn],
    memory_events: &[MemoryEvent],
) -> Option<SymbolicAnswer> {
    let events = memory_events_with_request_history(memory_events, history);
    if is_exact_memory_query(prompt) {
        // Protocol functions receive an immutable event slice. Execute exact
        // reads over an isolated projection and default mutations to read-only;
        // native/browser callers with explicit authorization retain CRUD.
        let mut store = MemoryStore::from_events(events);
        return execute_memory_query_with_options(
            prompt,
            &mut store,
            Some(REQUEST_HISTORY_CONVERSATION_ID),
            MemoryProgramLimits::default(),
            MemoryProgramAuthorization::ReadOnly,
        )
        .map(|execution| execution.answer);
    }
    if memory_events.is_empty() {
        return None;
    }
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
