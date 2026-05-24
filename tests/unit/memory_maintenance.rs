use formal_ai::{MemoryEvent, MemoryStore};

#[test]
fn library_memory_can_purge_soft_deleted_conversations() {
    // Issue #196: soft-delete markers are useful for hiding a conversation,
    // but users also need an explicit, physical deletion path for everything
    // attached to those already-deleted threads.
    let mut store = MemoryStore::from_events(vec![
        MemoryEvent {
            id: String::from("1"),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some(String::from("keep me")),
            conversation_id: Some(String::from("conv-keep")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("2"),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some(String::from("delete me")),
            conversation_id: Some(String::from("conv-delete")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("3"),
            kind: Some(String::from("conversation_deleted")),
            role: Some(String::from("system")),
            content: Some(String::from("Conversation deleted: delete me")),
            conversation_id: Some(String::from("conv-delete")),
            conversation_title: Some(String::from("delete me")),
            ..MemoryEvent::default()
        },
    ]);

    let removed = store.purge_deleted_conversations();

    assert_eq!(removed, 2);
    assert_eq!(store.len(), 1);
    assert_eq!(store.events()[0].content.as_deref(), Some("keep me"));
}

#[test]
fn library_memory_reset_clears_all_events() {
    let mut store = MemoryStore::from_events(vec![
        MemoryEvent::user("first"),
        MemoryEvent::assistant("second"),
    ]);

    let removed = store.reset();

    assert_eq!(removed, 2);
    assert!(store.is_empty());
}
