use super::*;

fn event(id: &str, content: &str) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        content: Some(content.to_owned()),
        ..MemoryEvent::default()
    }
}

#[test]
fn events_since_returns_tail_after_marker() {
    let log = vec![event("a", "1"), event("b", "2"), event("c", "3")];
    let delta = events_since(&log, Some("a"));
    assert_eq!(delta.len(), 2);
    assert_eq!(delta[0].id, "b");
    assert_eq!(delta[1].id, "c");
}

#[test]
fn events_since_none_returns_everything() {
    let log = vec![event("a", "1"), event("b", "2")];
    assert_eq!(events_since(&log, None).len(), 2);
    assert_eq!(events_since(&log, Some("")).len(), 2);
}

#[test]
fn events_since_unknown_marker_returns_everything() {
    let log = vec![event("a", "1"), event("b", "2")];
    // The puller references an id this log never had — return all so nothing
    // is skipped.
    assert_eq!(events_since(&log, Some("zzz")).len(), 2);
}

#[test]
fn merge_union_appends_new_ids_only() {
    let base = vec![event("a", "1"), event("b", "2")];
    let incoming = vec![event("b", "2"), event("c", "3")];
    let merged = merge_union_by_id(&base, &incoming);
    let ids: Vec<&str> = merged.iter().map(|event| event.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b", "c"]);
}

#[test]
fn merge_event_lets_incoming_fields_win() {
    let base = event("a", "original");
    let mut incoming = event("a", "edited");
    incoming.intent = Some(String::from("greeting"));
    let merged = merge_union_by_id(&[base], &[incoming]);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].content.as_deref(), Some("edited"));
    assert_eq!(merged[0].intent.as_deref(), Some("greeting"));
}

#[test]
fn sync_store_round_trips_through_file() {
    let dir = std::env::temp_dir().join(format!("formal-ai-sync-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("memory.lino");

    let mut store = SyncStore::open_at(&path);
    let inbound = export_links_notation(&[event("a", "1"), event("b", "2")]);
    let added = store.import_links_notation(&inbound).expect("import");
    assert_eq!(added, 2);

    // A fresh open observes the persisted events.
    let reopened = SyncStore::open_at(&path);
    assert_eq!(reopened.events().len(), 2);

    // The delta after "a" is just "b".
    let delta = reopened.delta_links_notation(Some("a"));
    let parsed = parse_links_notation(&delta);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].id, "b");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sync_store_without_path_is_empty_and_safe() {
    let mut store = SyncStore::default();
    assert!(store.events().is_empty());
    // Importing without a configured path does not error (no-op persist).
    let added = store
        .import_links_notation(&export_links_notation(&[event("a", "1")]))
        .expect("import without path");
    assert_eq!(added, 1);
    assert_eq!(store.events().len(), 1);
}
