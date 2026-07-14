//! Executable coverage for issue #686 — usage-weighted associative persistence.
//!
//! The [design case study](../../docs/case-studies/issue-686/README.md) maps every
//! concept the issue names onto the associative stack; this test exercises the
//! `formal_ai::associative_persistence` module that realizes it, proving each
//! headline requirement runs end to end:
//!
//! * meta-language expressions are **persisted** as content-addressed nodes in an
//!   associative links network,
//! * **reads (usages)** and **writes (changes)** are counted per expression,
//! * usage is **also derived from incoming and outgoing links** (degree),
//! * the **most used / most changed / most connected** expression is retained
//!   longest and evicted last,
//! * a world-model [`Context`] ingests into the store preserving statement ids,
//! * the whole store serializes as a **links network** (Links Notation).

use formal_ai::associative_persistence::{AssociativeMemory, RetentionWeights};
use formal_ai::world_model::{Context, Dependency, Statement};
use formal_ai::{
    parse_memory_links_notation, plan_memory_dreaming, DreamingConfig, MemoryEvent, MemoryStore,
};

#[test]
fn persist_is_content_addressed_and_deterministic() {
    let mut memory = AssociativeMemory::new();
    let first = memory.persist("all men are mortal");
    let second = memory.persist("all men are mortal");
    // Same text always maps to the same node in the links network.
    assert_eq!(first, second);
    // Persisting the same expression again is a change, so it counts as a write.
    assert_eq!(memory.len(), 1);
    assert_eq!(memory.writes(&first), 2);
    assert_eq!(memory.reads(&first), 0);
}

#[test]
fn writing_a_changed_identified_expression_persists_the_new_value() {
    let mut memory = AssociativeMemory::new();
    let id = "fact:door";
    memory.persist_identified(id, "the door is closed");

    memory.persist_identified(id, "the door is open");

    assert_eq!(
        memory.get(id).map(|entry| entry.text.as_str()),
        Some("the door is open")
    );
    assert_eq!(memory.writes(id), 2);
}

#[test]
fn new_expression_starts_with_one_write_and_zero_reads() {
    let mut memory = AssociativeMemory::new();
    let id = memory.persist("socrates is a man");
    assert_eq!(memory.writes(&id), 1);
    assert_eq!(memory.reads(&id), 0);
    assert!(memory.contains(&id));
}

#[test]
fn reads_and_writes_are_counted_independently() {
    let mut memory = AssociativeMemory::new();
    let id = memory.persist("knowledge persists");
    assert!(memory.note_read(&id));
    assert!(memory.note_read(&id));
    assert!(memory.note_write(&id));
    assert_eq!(memory.reads(&id), 2);
    // one initial assertion + one explicit write
    assert_eq!(memory.writes(&id), 2);
    // Unknown ids are rejected, not silently created.
    assert!(!memory.note_read("expression_does_not_exist"));
    assert!(!memory.note_write("expression_does_not_exist"));
}

#[test]
fn associations_form_a_links_network_with_degree() {
    let mut memory = AssociativeMemory::new();
    let a = memory.persist("A");
    let b = memory.persist("B");
    let c = memory.persist("C");

    assert!(memory.associate(&a, &b));
    assert!(memory.associate(&a, &c));
    assert!(memory.associate(&b, &c));
    // Re-adding an existing link is a no-op.
    assert!(!memory.associate(&a, &b));
    // Self-loops and unknown endpoints are rejected.
    assert!(!memory.associate(&a, &a));
    assert!(!memory.associate(&a, "expression_missing"));

    assert_eq!(memory.out_degree(&a), 2);
    assert_eq!(memory.in_degree(&a), 0);
    assert_eq!(memory.in_degree(&c), 2);
    assert_eq!(memory.out_degree(&c), 0);
    // link_usage folds incoming + outgoing — the issue's "usages based on
    // incoming and outgoing links".
    assert_eq!(memory.degree(&c), 2);
    assert_eq!(memory.link_usage(&c), memory.degree(&c));
    assert_eq!(memory.link_usage(&b), 2); // 1 in (a→b) + 1 out (b→c)
}

#[test]
fn retention_score_combines_reads_writes_and_degree() {
    let mut memory = AssociativeMemory::new();
    let a = memory.persist("A"); // writes = 1
    let b = memory.persist("B"); // writes = 1
    memory.note_read(&a); // reads = 1
    memory.associate(&a, &b); // a: out 1, b: in 1

    // a: reads 1 + writes 1 + in 0 + out 1 = 3
    assert_eq!(memory.retention_score(&a), 3);
    // b: reads 0 + writes 1 + in 1 + out 0 = 2
    assert_eq!(memory.retention_score(&b), 2);
    // Unknown ids score zero.
    assert_eq!(memory.retention_score("expression_missing"), 0);

    // Weights let a caller make changes protect twice as strongly as reads.
    let weights = RetentionWeights {
        read: 1,
        write: 2,
        incoming: 1,
        outgoing: 1,
    };
    // a: 1*1 + 2*1 + 1*0 + 1*1 = 4
    assert_eq!(memory.retention_score_with(&a, weights), 4);
}

#[test]
fn most_used_data_persists_longer_and_is_evicted_last() {
    let mut memory = AssociativeMemory::new();
    let hot = memory.persist("frequently used");
    let warm = memory.persist("occasionally used");
    let cold = memory.persist("rarely used");

    for _ in 0..5 {
        memory.note_read(&hot);
    }
    memory.note_read(&warm);

    // Ranking is most-retained first.
    let ranking = memory.retention_ranking();
    assert_eq!(ranking.first(), Some(&hot));
    // Eviction order is least-retained first — cold goes first.
    let eviction = memory.eviction_order();
    assert_eq!(eviction.first(), Some(&cold));

    // Retaining a capacity of 1 keeps only the hottest expression.
    let evicted = memory.retain_most_used(1);
    assert_eq!(memory.len(), 1);
    assert!(memory.contains(&hot));
    assert!(!memory.contains(&warm));
    assert!(!memory.contains(&cold));
    // Evicted least-retained first.
    assert_eq!(evicted.len(), 2);
    assert_eq!(evicted[0].id, cold);
    assert_eq!(evicted[1].id, warm);
}

#[test]
fn forgetting_removes_incident_links() {
    let mut memory = AssociativeMemory::new();
    let a = memory.persist("A");
    let b = memory.persist("B");
    let c = memory.persist("C");
    memory.associate(&a, &b);
    memory.associate(&b, &c);

    let removed = memory.forget(&b).expect("b was persisted");
    assert_eq!(removed.id, b);
    assert!(!memory.contains(&b));
    // Links touching b are gone, so a and c lose their degree from b.
    assert_eq!(memory.out_degree(&a), 0);
    assert_eq!(memory.in_degree(&c), 0);
    // Forgetting an unknown id yields None.
    assert!(memory.forget("expression_missing").is_none());
}

#[test]
fn tie_breaks_are_deterministic() {
    let mut memory = AssociativeMemory::new();
    // Two expressions with identical usage must rank in a stable id order.
    let first = memory.persist("aaa");
    let second = memory.persist("bbb");
    let ranking = memory.retention_ranking();
    let mut sorted = vec![first, second];
    sorted.sort();
    assert_eq!(ranking, sorted);
}

#[test]
fn from_context_preserves_ids_and_dependency_links() {
    let mut context = Context::new("mortality");
    let premise = context.add_statement(Statement::new("all men are mortal"));
    let minor = context.add_statement(Statement::new("socrates is a man"));
    let conclusion = context.add_statement(
        Statement::new("socrates is mortal")
            .with_dependency(Dependency::supports(&premise))
            .with_dependency(Dependency::supports(&minor)),
    );

    let memory = AssociativeMemory::from_context(&context);
    // Every statement is persisted under its original id.
    assert!(memory.contains(&premise));
    assert!(memory.contains(&minor));
    assert!(memory.contains(&conclusion));
    // Dependency edges became associative links: conclusion → premises.
    assert_eq!(memory.out_degree(&conclusion), 2);
    assert_eq!(memory.in_degree(&premise), 1);
    assert_eq!(memory.in_degree(&minor), 1);
    // The well-connected premises outrank the isolated-in conclusion by degree.
    assert!(memory.retention_score(&premise) >= 1);
}

#[test]
fn links_notation_is_a_sorted_reproducible_links_network() {
    let mut memory = AssociativeMemory::new();
    let a = memory.persist("A");
    let b = memory.persist("B");
    memory.note_read(&a);
    memory.associate(&a, &b);

    let first = memory.links_notation();
    let second = memory.links_notation();
    // Deterministic: byte-for-byte identical across renders.
    assert_eq!(first, second);
    // Everything is emitted as a link.
    assert!(first.contains(&format!("expression: ({a} A)")));
    assert!(first.contains(&format!("reads: ({a} 1)")));
    assert!(first.contains(&format!("writes: ({a} 1)")));
    assert!(first.contains(&format!("associates: ({a} {b})")));
}

#[test]
fn durable_memory_writes_round_trip_and_protect_changed_knowledge() {
    let mut store = MemoryStore::from_events(vec![MemoryEvent {
        id: String::from("fact:door"),
        content: Some(String::from("the door is closed")),
        ..MemoryEvent::default()
    }]);
    assert_eq!(store.events()[0].write_count, 1);

    assert_eq!(store.apply_substitution("closed", "open"), 1);
    assert_eq!(store.events()[0].write_count, 2);
    let serialized = store.export_links_notation();
    assert!(serialized.contains("writeCount \"2\""));
    let reloaded = parse_memory_links_notation(&serialized);
    assert_eq!(reloaded[0].write_count, 2);
    let projection = formal_ai::link_store::memory_event_to_link_record(&reloaded[0], 0);
    assert!(projection
        .links
        .iter()
        .any(|link| link.from == "field:writeCount" && link.to == "value:2"));

    let plan = plan_memory_dreaming(&reloaded, &DreamingConfig::default());
    assert_eq!(plan.event_usage("fact:door"), Some(2));
}

#[test]
fn durable_memory_retention_uses_both_directions_of_evidence_links() {
    let events = vec![
        MemoryEvent {
            id: String::from("conclusion"),
            content: Some(String::from("the derived conclusion")),
            evidence: vec![String::from("premise")],
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("premise"),
            content: Some(String::from("the grounded premise")),
            ..MemoryEvent::default()
        },
    ];

    let associative = AssociativeMemory::from_memory_events(&events);
    assert_eq!(associative.out_degree("conclusion"), 1);
    assert_eq!(associative.in_degree("premise"), 1);
    assert_eq!(associative.retention_score("conclusion"), 2);
    assert_eq!(associative.retention_score("premise"), 2);

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());
    assert_eq!(plan.event_usage("conclusion"), Some(2));
    assert_eq!(plan.event_usage("premise"), Some(2));
}

#[test]
fn sync_merge_counts_uncounted_edits_and_preserves_monotone_peer_counts() {
    let base = MemoryEvent {
        id: String::from("fact"),
        content: Some(String::from("old")),
        write_count: 1,
        ..MemoryEvent::default()
    };
    let edited = MemoryEvent {
        id: String::from("fact"),
        content: Some(String::from("new")),
        write_count: 1,
        ..MemoryEvent::default()
    };
    let merged = formal_ai::memory_sync::merge_event(&base, &edited);
    assert_eq!(merged.content.as_deref(), Some("new"));
    assert_eq!(merged.write_count, 2);

    let peer = MemoryEvent {
        write_count: 7,
        ..edited
    };
    assert_eq!(
        formal_ai::memory_sync::merge_event(&merged, &peer).write_count,
        7
    );
}

#[test]
fn ingestion_preserves_qualifiers_and_retains_misaligned_candidates() {
    let events = vec![MemoryEvent {
        id: String::from("claim"),
        kind: Some(String::from("task")),
        role: Some(String::from("assistant")),
        content: Some(String::from("qualified claim")),
        sent_at: Some(String::from("2026-07-14T12:00:00Z")),
        conversation_id: Some(String::from("case-study")),
        evidence: vec![String::from("missing-source")],
        ..MemoryEvent::default()
    }];

    let memory = AssociativeMemory::from_memory_events(&events);
    let claim = memory.get("claim").expect("candidate is retained");
    assert_eq!(
        claim.qualifiers.get("kind").map(String::as_str),
        Some("task")
    );
    assert_eq!(
        claim.qualifiers.get("conversation_id").map(String::as_str),
        Some("case-study")
    );
    assert_eq!(claim.validation_issues.len(), 1);
    assert!(claim.validation_issues[0].contains("missing-source"));
}

#[test]
fn multi_hop_recall_is_bounded_deterministic_and_counts_reads() {
    let mut memory = AssociativeMemory::new();
    memory.persist_identified("a", "first");
    memory.persist_identified("b", "second");
    memory.persist_identified("c", "third");
    memory.associate("a", "b");
    memory.associate("b", "c");

    assert_eq!(memory.recall_related("a", 1), vec!["a", "b"]);
    assert_eq!(memory.reads("a"), 1);
    assert_eq!(memory.reads("b"), 1);
    assert_eq!(memory.reads("c"), 0);
    assert_eq!(memory.recall_related("a", 2), vec!["a", "b", "c"]);
    assert_eq!(memory.reads("a"), 2);
    assert_eq!(memory.reads("b"), 2);
    assert_eq!(memory.reads("c"), 1);
}
