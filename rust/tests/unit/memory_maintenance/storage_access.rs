use super::*;

pub(super) fn auto_free_space_for_write_stops_at_target_with_nonzero_incoming_bytes() {
    // Issue #540 §4: the write-driven freeing path must reclaim just enough
    // for the next write plus the 20% target, not everything reclaimable.
    let memory_path = std::env::temp_dir().join(format!(
        "formal-ai-issue-540-stop-at-target-{}-memory.lino",
        std::process::id()
    ));
    let preference_path = auto_free_space_preference_path(&memory_path);

    // Five distinct recomputable caches (~600 bytes each) plus irreplaceable
    // raw messages that must never be selected under pressure.
    let mut events: Vec<MemoryEvent> = (0..5)
        .map(|index| {
            recomputable_event(
                &format!("cache-{index}"),
                &format!("distinct cached payload {index} {}", "x".repeat(600)),
            )
        })
        .collect();
    events.push(MemoryEvent::user("irreplaceable raw question"));
    events.push(MemoryEvent::assistant("irreplaceable raw answer"));
    let recomputable_count = 5;

    // capacity 100_000 → 20% target = 20_000; free 20_500 with 1_000 incoming
    // leaves a 500-byte deficit — far less than one cache entry.
    let snapshot = StorageSnapshot {
        capacity_bytes: 100_000,
        free_bytes: 20_500,
    };

    // Declined (and never-asked) consent must block freeing even under
    // identical pressure.
    let _ = std::fs::remove_file(&preference_path);
    let mut store = MemoryStore::from_events(events.clone());
    assert!(
        apply_auto_free_space_with_snapshot(&mut store, &memory_path, 1_000, snapshot).is_none()
    );
    persist_auto_free_space_choice(&memory_path, false).expect("persist decline");
    assert!(
        apply_auto_free_space_with_snapshot(&mut store, &memory_path, 1_000, snapshot).is_none()
    );
    assert_eq!(store.len(), events.len());

    persist_auto_free_space_choice(&memory_path, true).expect("persist consent");
    let (plan, outcome) =
        apply_auto_free_space_with_snapshot(&mut store, &memory_path, 1_000, snapshot)
            .expect("consented freeing runs");

    assert_eq!(plan.incoming_bytes, 1_000);
    assert_eq!(plan.required_reclaim_bytes, 500);
    assert!(plan.selected_reclaim_bytes >= plan.required_reclaim_bytes);
    // Stops at the target: one cache covers the 500-byte deficit, so most
    // recomputable data survives and nothing irreplaceable is touched.
    assert!(outcome.removed_events >= 1);
    assert!(outcome.removed_events < recomputable_count);
    let remaining_caches = store
        .events()
        .iter()
        .filter(|event| event.id.starts_with("cache-"))
        .count();
    assert!(remaining_caches >= recomputable_count - outcome.removed_events);
    assert!(
        store
            .events()
            .iter()
            .any(|event| event.content.as_deref() == Some("irreplaceable raw question"))
    );
    assert!(
        store
            .events()
            .iter()
            .any(|event| event.content.as_deref() == Some("irreplaceable raw answer"))
    );

    let _ = std::fs::remove_file(preference_path);
}

pub(super) fn recall_counts_access_and_dreaming_treats_read_data_as_used() {
    // Issue #494 via #540 §4: usage is counted when data is *read back*, not
    // only when other events cite it — a recall bumps `access_count`, the
    // caller persists the store, and dreaming ranks the read event as used.
    let mut store = MemoryStore::from_events(vec![
        MemoryEvent {
            id: String::from("tool-1"),
            kind: Some(String::from("source:http")),
            role: Some(String::from("tool")),
            tool: Some(String::from("web_search")),
            content: Some(String::from("Found Rust memory references.")),
            conversation_id: Some(String::from("conv-tools")),
            conversation_title: Some(String::from("Tool Trace")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("never-read"),
            kind: Some(String::from("analysis")),
            content: Some(String::from("unrelated derived summary")),
            ..MemoryEvent::default()
        },
    ]);

    let execution = execute_memory_query("recall web_search", &mut store, Some("conv-current"))
        .expect("recall query recognized");
    assert_eq!(execution.answer.intent, "conversation_recall");
    assert!(
        execution.changed,
        "a recall that read events must mark the store changed so access counts persist"
    );
    let read_event = store
        .events()
        .iter()
        .find(|event| event.id == "tool-1")
        .expect("read event survives");
    assert_eq!(read_event.access_count, 1);
    assert_eq!(
        store
            .events()
            .iter()
            .find(|event| event.id == "never-read")
            .expect("unread event survives")
            .access_count,
        0
    );

    // Access counts must round-trip through serialization.
    let serialized = store.export_links_notation();
    assert!(serialized.contains("accessCount \"1\""));
    let reloaded = MemoryStore::from_events(formal_ai::parse_memory_links_notation(&serialized));
    assert_eq!(
        reloaded
            .events()
            .iter()
            .find(|event| event.id == "tool-1")
            .expect("reload keeps event")
            .access_count,
        1
    );

    // Dreaming sees the read event as used even though nothing cites it.
    let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
    assert!(plan.event_usage("tool-1").unwrap_or_default() >= 1);
    assert_eq!(plan.event_usage("never-read"), Some(1));
}
