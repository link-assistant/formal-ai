use super::EventLog;
use crate::memory::MemoryStore;

#[test]
fn append_returns_stable_ids_for_distinct_events() {
    let mut log = EventLog::new();
    let first = log.append("impulse", "hi");
    let second = log.append("impulse", "hi");
    assert_ne!(first, second, "appending twice must produce distinct ids");
    assert_eq!(log.events().len(), 2);
}

#[test]
fn evidence_links_round_trip_event_kinds() {
    let mut log = EventLog::new();
    log.append("impulse", "hello");
    log.append("intent", "greeting");
    let links = log.evidence_links();
    assert_eq!(links.len(), 2);
    assert!(links[0].starts_with("impulse:"));
    assert!(links[1].starts_with("intent:"));
}

#[test]
fn steps_block_lists_events_in_insertion_order() {
    let mut log = EventLog::new();
    log.append("impulse", "x");
    log.append("trace", "y");
    let block = log.steps_block();
    assert!(block.contains("step_0 impulse x"));
    assert!(block.contains("step_1 trace y"));
}

#[test]
fn event_log_replays_into_link_store() {
    let mut log = EventLog::new();
    log.append("impulse", "hello");
    let mut store = MemoryStore::new();
    let inserted = log.append_to_link_store(&mut store).expect("replay");
    assert_eq!(inserted, 1);
    assert_eq!(store.events()[0].kind.as_deref(), Some("impulse"));
    assert_eq!(store.link_records().len(), 1);
}
