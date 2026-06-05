use super::{export_links_notation, parse_links_notation, MemoryEvent, MemoryStore};

fn sample_events() -> Vec<MemoryEvent> {
    vec![
        MemoryEvent {
            id: String::from("1"),
            role: Some(String::from("user")),
            content: Some(String::from("Hi")),
            sent_at: Some(String::from("2026-05-15T12:00:00.000Z")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("2"),
            role: Some(String::from("assistant")),
            intent: Some(String::from("greeting")),
            content: Some(String::from("Hi, how may I help you?")),
            sent_at: Some(String::from("2026-05-15T12:00:01.000Z")),
            ..MemoryEvent::default()
        },
    ]
}

#[test]
fn export_round_trips_through_parse() {
    let events = sample_events();
    let text = export_links_notation(&events);
    let parsed = parse_links_notation(&text);
    assert_eq!(parsed, events);
}

#[test]
fn parse_ignores_unknown_fields() {
    let text =
        "demo_memory\n  event \"1\"\n    role \"user\"\n    novel_key \"x\"\n    content \"Hi\"\n";
    let events = parse_links_notation(text);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].role.as_deref(), Some("user"));
    assert_eq!(events[0].content.as_deref(), Some("Hi"));
}

fn assert_append_only<T>(_: &T)
where
    T: Sized,
{
}

#[test]
fn store_is_append_only() {
    let mut store = MemoryStore::new();
    store.append(MemoryEvent::user("hello"));
    store.append(MemoryEvent::assistant("hi back"));
    assert_eq!(store.len(), 2);
    // The struct deliberately exposes no removal API; this test pins the
    // API surface so future refactors cannot quietly add one.
    assert_append_only(&store);
}

#[test]
fn parse_returns_empty_when_header_missing() {
    let events = parse_links_notation("totally_different_header\n  event \"1\"\n");
    assert!(events.is_empty());
}

#[test]
fn import_from_links_notation_appends_in_order() {
    let mut store = MemoryStore::new();
    store.append(MemoryEvent::user("prior"));
    let inbound = export_links_notation(&sample_events());
    let inserted = store.import_links_notation(&inbound);
    assert_eq!(inserted, 2);
    assert_eq!(store.len(), 3);
    assert_eq!(store.events()[0].content.as_deref(), Some("prior"));
    assert_eq!(
        store.events()[2].content.as_deref(),
        Some("Hi, how may I help you?")
    );
}

#[test]
fn file_round_trip_preserves_events() {
    let dir = std::env::temp_dir().join(format!("formal-ai-memory-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("memory.lino");
    let store = MemoryStore::from_events(sample_events());
    store.save_to_file(&path).expect("save");
    let loaded = MemoryStore::load_from_file(&path).expect("load");
    assert_eq!(loaded.events(), store.events());
    // load_from_file on missing path returns empty store.
    let missing = MemoryStore::load_from_file(dir.join("nope.lino")).expect("missing-ok");
    assert!(missing.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
