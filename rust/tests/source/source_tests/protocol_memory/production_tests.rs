use super::{
    ConversationTurn, MemoryEvent, REQUEST_HISTORY_CONVERSATION_ID,
    memory_events_with_request_history,
};

#[test]
fn request_history_preserves_saved_events_and_projects_ordered_turns() {
    let saved = MemoryEvent {
        id: String::from("saved-event"),
        conversation_id: Some(String::from("saved-conversation")),
        content: Some(String::from("Previously retained evidence")),
        ..MemoryEvent::default()
    };
    let history = [
        ConversationTurn::user("Show the retained evidence"),
        ConversationTurn::assistant("Here is the recorded observation"),
    ];
    let events = memory_events_with_request_history(std::slice::from_ref(&saved), &history);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0], saved);
    for (index, (event, turn)) in events[1..].iter().zip(&history).enumerate() {
        assert_eq!(event.id, format!("request-history-{index}"));
        assert_eq!(event.kind.as_deref(), Some("message"));
        assert_eq!(event.role.as_deref(), Some(turn.role.slug()));
        assert_eq!(event.content.as_deref(), Some(turn.content.as_str()));
        assert_eq!(
            event.conversation_id.as_deref(),
            Some(REQUEST_HISTORY_CONVERSATION_ID)
        );
    }
}
