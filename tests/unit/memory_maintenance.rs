use formal_ai::{
    apply_dreaming_plan, plan_memory_dreaming, DreamingActionKind, DreamingConfig, MemoryEvent,
    MemoryStore,
};

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

#[test]
fn dreaming_restructures_recomputable_duplicates_by_recalculated_use_frequency() {
    let events = vec![
        recomputable_event("cache-low-use", "same cache payload"),
        recomputable_event("cache-high-use", "same cache payload"),
        MemoryEvent {
            id: String::from("analysis-1"),
            kind: Some(String::from("analysis")),
            content: Some(String::from(
                "Prefer the frequently reused cache-high-use record.",
            )),
            evidence: vec![String::from("source:http:cache-high-use")],
            ..MemoryEvent::default()
        },
    ];

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());

    assert!(
        plan.event_usage("cache-high-use").unwrap_or_default()
            > plan.event_usage("cache-low-use").unwrap_or_default(),
        "dreaming should recalculate use frequency from event text and evidence",
    );
    assert!(plan.actions.iter().any(|action| {
        action.kind == DreamingActionKind::RemoveDuplicateRecomputable
            && action.event_id == "cache-low-use"
    }));
    assert!(!plan
        .actions
        .iter()
        .any(|action| action.event_id == "cache-high-use"));
}

#[test]
fn dreaming_preserves_raw_events_and_learning_when_reclaiming_space() {
    let events = vec![
        MemoryEvent {
            id: String::from("raw-user-message"),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some("irreplaceable user experience ".repeat(20)),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("learned-skill"),
            kind: Some(String::from("learning_ledger")),
            content: Some("promoted learned experience ".repeat(20)),
            ..MemoryEvent::default()
        },
        recomputable_event("external-cache", &"cached public source ".repeat(20)),
        MemoryEvent {
            id: String::from("intermediate-summary"),
            kind: Some(String::from("intermediate_conclusion")),
            content: Some("recomputable intermediate conclusion ".repeat(20)),
            ..MemoryEvent::default()
        },
    ];
    let config = DreamingConfig {
        storage_capacity_bytes: Some(1_000),
        free_bytes: Some(0),
        incoming_bytes: 0,
        ..DreamingConfig::default()
    };

    let plan = plan_memory_dreaming(&events, &config);

    assert!(plan.required_reclaim_bytes > 0);
    assert!(plan
        .actions
        .iter()
        .any(|action| action.event_id == "external-cache"));
    assert!(!plan
        .actions
        .iter()
        .any(|action| action.event_id == "raw-user-message"));
    assert!(!plan
        .actions
        .iter()
        .any(|action| action.event_id == "learned-skill"));
}

#[test]
fn dreaming_reports_bigger_storage_when_recomputable_data_cannot_satisfy_target() {
    let events = vec![
        MemoryEvent {
            id: String::from("raw-only"),
            kind: Some(String::from("message")),
            role: Some(String::from("assistant")),
            content: Some("raw retained experience ".repeat(10)),
            ..MemoryEvent::default()
        },
        recomputable_event("small-cache", "tiny cache"),
    ];
    let config = DreamingConfig {
        storage_capacity_bytes: Some(10_000),
        free_bytes: Some(0),
        incoming_bytes: 0,
        ..DreamingConfig::default()
    };

    let plan = plan_memory_dreaming(&events, &config);

    assert!(plan.required_reclaim_bytes > plan.selected_reclaim_bytes);
    assert!(plan.requires_bigger_storage);
    assert!(!plan
        .actions
        .iter()
        .any(|action| action.event_id == "raw-only"));
}

#[test]
fn applying_dreaming_plan_removes_only_selected_recomputable_events() {
    let mut store = MemoryStore::from_events(vec![
        recomputable_event("cache-low-use", "same cache payload"),
        recomputable_event("cache-high-use", "same cache payload"),
        MemoryEvent {
            id: String::from("raw-user-message"),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some(String::from("raw event stays")),
            evidence: vec![String::from("source:http:cache-high-use")],
            ..MemoryEvent::default()
        },
    ]);
    let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());

    let outcome = apply_dreaming_plan(&mut store, &plan);

    assert_eq!(outcome.removed_events, 1);
    assert!(store
        .events()
        .iter()
        .any(|event| event.id == "cache-high-use"));
    assert!(store
        .events()
        .iter()
        .any(|event| event.id == "raw-user-message"));
    assert!(!store
        .events()
        .iter()
        .any(|event| event.id == "cache-low-use"));
}

#[test]
fn dreaming_recalculates_topic_frequency_and_learns_durable_requirements() {
    // Issue #540: while idle, dreaming should learn which topics the user
    // interacts with most and remember the requirements stated on them so the
    // user never has to repeat himself.
    let events = vec![
        requirement_event(
            "req-1",
            "latex",
            "Always compile proofs with LaTeX before answering.",
        ),
        task_run_event("run-1", "latex", "proof render pass 1"),
        task_run_event("run-2", "latex", "proof render pass 2"),
        MemoryEvent {
            id: String::from("chit-chat"),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some(String::from("hello there")),
            conversation_title: Some(String::from("smalltalk")),
            ..MemoryEvent::default()
        },
    ];

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());

    assert_eq!(plan.topic_interactions("latex"), Some(3));
    assert!(
        plan.topic_interactions("latex").unwrap_or_default()
            > plan.topic_interactions("smalltalk").unwrap_or_default(),
        "the most-used topic should rank above incidental chatter",
    );
    assert!(
        plan.learned_requirements
            .iter()
            .any(|requirement| requirement.topic == "latex"
                && requirement.statement.contains("LaTeX"))
    );
}

#[test]
fn dreaming_generalizes_requirements_into_meta_algorithm_amendments() {
    // The learned requirement must be baked into a meta-algorithm amendment that
    // covers the specific test-run records it can reproduce.
    let events = vec![
        requirement_event("req-1", "latex", "Always compile proofs with LaTeX."),
        task_run_event("run-1", "latex", "proof render pass 1"),
        task_run_event("run-2", "latex", "proof render pass 2"),
    ];

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());

    let amendment = plan
        .amendment_for("latex")
        .expect("dreaming should generalize the latex requirement");
    assert!(amendment.rule.contains("LaTeX"));
    assert!(amendment
        .source_requirement_ids
        .contains(&String::from("req-1")));
    assert!(amendment.covered_event_ids.contains(&String::from("run-1")));
    assert!(amendment.covered_event_ids.contains(&String::from("run-2")));
}

#[test]
fn dreaming_forgets_covered_specifics_first_under_pressure() {
    // Under storage pressure, specifics reproducible from a retained amendment
    // are forgotten before other recomputable data, while raw/learning stays.
    let events = vec![
        requirement_event("req-1", "latex", "Always compile proofs with LaTeX."),
        task_run_event("run-1", "latex", &"proof render specifics ".repeat(20)),
        recomputable_event("unrelated-cache", &"unrelated cached source ".repeat(5)),
    ];
    let config = DreamingConfig {
        storage_capacity_bytes: Some(1_000),
        free_bytes: Some(0),
        incoming_bytes: 0,
        ..DreamingConfig::default()
    };

    let plan = plan_memory_dreaming(&events, &config);

    assert!(plan.required_reclaim_bytes > 0);
    assert!(
        plan.actions.iter().any(|action| {
            action.kind == DreamingActionKind::ForgetCoveredSpecific && action.event_id == "run-1"
        }),
        "covered test-run specifics should be forgotten under pressure",
    );
    assert!(!plan.actions.iter().any(|action| action.event_id == "req-1"));
}

#[test]
fn applying_dreaming_plan_bakes_amendments_and_is_idempotent() {
    // Applying the plan materializes each amendment as a retained learning event;
    // re-applying an unchanged plan must not duplicate it.
    let mut store = MemoryStore::from_events(vec![
        requirement_event("req-1", "latex", "Always compile proofs with LaTeX."),
        task_run_event("run-1", "latex", "proof render pass 1"),
        task_run_event("run-2", "latex", "proof render pass 2"),
    ]);
    let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());

    let outcome = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(outcome.learned_amendments, 1);
    let amendment_count = store
        .events()
        .iter()
        .filter(|event| event.kind.as_deref() == Some("meta_algorithm_amendment"))
        .count();
    assert_eq!(amendment_count, 1);

    // Re-planning against the mutated store keeps the amendment retained and does
    // not append a second copy.
    let replan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
    let second = apply_dreaming_plan(&mut store, &replan);
    assert_eq!(second.learned_amendments, 0);
    let amendment_count_after = store
        .events()
        .iter()
        .filter(|event| event.kind.as_deref() == Some("meta_algorithm_amendment"))
        .count();
    assert_eq!(amendment_count_after, 1);
}

fn requirement_event(id: &str, topic: &str, statement: &str) -> MemoryEvent {
    MemoryEvent {
        id: String::from(id),
        kind: Some(String::from("message")),
        role: Some(String::from("user")),
        content: Some(String::from(statement)),
        conversation_title: Some(String::from(topic)),
        ..MemoryEvent::default()
    }
}

fn task_run_event(id: &str, topic: &str, payload: &str) -> MemoryEvent {
    MemoryEvent {
        id: String::from(id),
        kind: Some(String::from("test_run")),
        role: Some(String::from("assistant")),
        content: Some(String::from(payload)),
        outputs: Some(String::from(payload)),
        conversation_title: Some(String::from(topic)),
        ..MemoryEvent::default()
    }
}

fn recomputable_event(id: &str, payload: &str) -> MemoryEvent {
    MemoryEvent {
        id: String::from(id),
        kind: Some(String::from("source:http")),
        content: Some(String::from(payload)),
        tool: Some(String::from("web_search")),
        outputs: Some(String::from(payload)),
        ..MemoryEvent::default()
    }
}
