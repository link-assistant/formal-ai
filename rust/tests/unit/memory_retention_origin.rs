//! Proven reconstruction, not a cache-shaped label, grants eviction eligibility.
#[test]
fn seed_cache_events_are_stable_and_classified_recomputable() {
    // Issue #540 §4: imports materialize seed files as `seed_cache` events —
    // recomputable data with ids stable over the file name so re-import never
    // duplicates.
    let (path, content) = formal_ai::seed::seed_files()[0];
    let seed_files = vec![(path.to_owned(), content.to_owned())];
    let first = seed_cache_events(&seed_files);
    let second = seed_cache_events(&seed_files);
    assert_eq!(first.len(), 1);
    assert_eq!(
        first[0].id, second[0].id,
        "ids must be stable per file name"
    );
    assert_eq!(first[0].kind.as_deref(), Some("seed_cache"));
    assert_eq!(first[0].tool.as_deref(), Some(path));
    assert_eq!(first[0].content.as_deref(), Some(content));

    let plan = plan_memory_dreaming(&first, &DreamingConfig::default());
    let observation = plan
        .observations
        .iter()
        .find(|observation| observation.event_id == first[0].id)
        .expect("seed cache observed");
    assert_eq!(
        observation.durability,
        formal_ai::DreamingDurability::RecomputableCache
    );
}
use super::memory_maintenance::recomputable_event;

#[test]
fn memory_pressure_keeps_conversation_and_action_history_even_when_cache_shaped() {
    let mut events = Vec::new();
    for role in ["user", "assistant", "tool"] {
        for kind in ["source:http", "summary", "test_run", "analysis"] {
            events.push(MemoryEvent {
                id: format!("{role}-{kind}"),
                role: Some(role.to_owned()),
                kind: Some(kind.to_owned()),
                tool: Some("web_fetch".to_owned()),
                content: Some(
                    "Original observation that cannot be reconstructed from today's web."
                        .to_owned(),
                ),
                ..MemoryEvent::default()
            });
        }
    }
    // Compare persisted records, including the store's initial write count.
    let original = MemoryStore::from_events(events.clone()).events().to_vec();
    events.push(recomputable_event(
        "disposable-source",
        &"public source ".repeat(30),
    ));
    let mut store = MemoryStore::from_events(events);
    let config = DreamingConfig {
        storage_capacity_bytes: Some(10_000),
        free_bytes: Some(0),
        ..DreamingConfig::default()
    };
    let plan = plan_memory_dreaming(store.events(), &config);
    assert!(
        plan.actions
            .iter()
            .any(|action| action.event_id == "disposable-source")
    );
    for event in &original {
        assert!(
            !plan
                .actions
                .iter()
                .any(|action| action.event_id == event.id),
            "original experience was selected for eviction: {}",
            event.id
        );
    }
    let _ = apply_dreaming_plan(&mut store, &plan);
    let serialized = formal_ai::export_memory_links_notation(store.events());
    let restored = formal_ai::parse_memory_links_notation(&serialized);
    for event in &original {
        assert!(restored.iter().any(|retained| retained == event));
    }
}

#[test]
fn applying_an_old_eviction_plan_rechecks_the_records_current_origin() {
    let cache = recomputable_event("reclassified", &"cached value ".repeat(30));
    let plan = plan_memory_dreaming(
        std::slice::from_ref(&cache),
        &DreamingConfig {
            storage_capacity_bytes: Some(10_000),
            free_bytes: Some(0),
            ..DreamingConfig::default()
        },
    );
    assert!(!plan.actions.is_empty());
    let original = MemoryEvent {
        role: Some("tool".to_owned()),
        ..cache
    };
    let mut store = MemoryStore::from_events(vec![original]);
    let original = store.events()[0].clone();
    let result = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(result.removed_events, 0);
    assert!(store.events().contains(&original));
}
use formal_ai::{
    DreamingConfig, MemoryEvent, MemoryStore, apply_dreaming_plan, plan_memory_dreaming,
    seed_cache_events,
};

fn pressure() -> DreamingConfig {
    DreamingConfig {
        storage_capacity_bytes: Some(100_000),
        free_bytes: Some(0),
        ..DreamingConfig::default()
    }
}

#[test]
fn reclaimed_bytes_include_all_new_learning_records() {
    // R710-R7: compact reconstruction metadata is not the only retained cost.
    // Learning during eviction must not turn gross payload bytes into a false
    // net-saving claim, and the original requirement remains historical data.
    for payload_copies in [5, 2_000] {
        let requirement = MemoryEvent {
            id: "original-requirement".to_owned(),
            kind: Some("message".to_owned()),
            role: Some("user".to_owned()),
            content: Some(
                "Always include a LaTeX verification step in proof solutions.".to_owned(),
            ),
            conversation_title: Some("latex".to_owned()),
            ..MemoryEvent::default()
        };
        let cache = recomputable_event("public-cache", &"public payload ".repeat(payload_copies));
        let mut store = MemoryStore::from_events(vec![requirement, cache]);
        let original = store.events()[0].clone();
        let plan = plan_memory_dreaming(store.events(), &pressure());
        let before_bytes: u64 = plan
            .observations
            .iter()
            .map(|event| event.estimated_bytes)
            .sum();
        let outcome = apply_dreaming_plan(&mut store, &plan);
        assert_eq!(outcome.removed_events, 1);
        assert!(
            outcome.learned_amendments > 0,
            "must exercise new retained learning"
        );
        assert!(store.events().contains(&original));
        let after = plan_memory_dreaming(store.events(), &pressure());
        let after_bytes: u64 = after
            .observations
            .iter()
            .map(|event| event.estimated_bytes)
            .sum();
        if payload_copies == 5 {
            assert!(
                after_bytes > before_bytes,
                "exercise a net-growing learning pass"
            );
        } else {
            assert!(after_bytes < before_bytes, "exercise positive net savings");
        }
        assert_eq!(
            outcome.estimated_reclaimed_bytes,
            before_bytes.saturating_sub(after_bytes),
            "every new retained record consumes space, not just reconstruction metadata"
        );
    }
}

#[test]
fn legacy_cache_labels_and_unproved_derived_records_are_retained() {
    let events = [
        "source:http",
        "analysis",
        "summary",
        "seed_cache",
        "test_run",
    ]
    .into_iter()
    .flat_map(|kind| {
        [None, Some("cache"), Some("derived")].map(|role| MemoryEvent {
            id: format!("{kind}-{role:?}"),
            kind: Some(kind.to_owned()),
            role: role.map(str::to_owned),
            tool: Some("web_fetch".to_owned()),
            content: Some("Original or unknown data; no reconstruction contract".to_owned()),
            ..MemoryEvent::default()
        })
    })
    .collect::<Vec<_>>();
    let mut store = MemoryStore::from_events(events);
    let before = store.events().to_vec();
    let plan = plan_memory_dreaming(store.events(), &pressure());
    assert!(
        plan.actions.is_empty(),
        "unknown provenance must fail closed: {:?}",
        plan.actions
    );
    assert!(plan.requires_bigger_storage);
    assert_eq!(apply_dreaming_plan(&mut store, &plan).removed_events, 0);
    for event in &before {
        assert!(
            store.events().contains(event),
            "original record changed: {}",
            event.id
        );
    }
}

#[test]
fn modified_imported_seeds_are_not_reconstructable_from_the_embedded_seed() {
    let mut store = MemoryStore::from_events(seed_cache_events(&[(
        "data/seed/roles.lino".to_owned(),
        "roles\n  my_private_extension\n".to_owned(),
    )]));
    let before = store.events().to_vec();
    let plan = plan_memory_dreaming(store.events(), &pressure());
    assert!(
        plan.actions.is_empty(),
        "imported custom knowledge is not an embedded cache"
    );
    let _ = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(store.events(), before);
}

#[test]
fn declared_public_cache_can_be_forgotten_without_forgetting_its_origin() {
    let origin = MemoryEvent {
        id: "original-observation".to_owned(),
        role: Some("tool".to_owned()),
        kind: Some("source:http".to_owned()),
        content: Some("A historical observation".to_owned()),
        ..MemoryEvent::default()
    };
    let cache = MemoryEvent {
        id: "rediscoverable-copy".to_owned(),
        role: Some("cache".to_owned()),
        evidence: vec!["rediscover:https://doc.rust-lang.org/std/".to_owned()],
        content: Some("Disposable public documentation ".repeat(200)),
        ..origin.clone()
    };
    let mut store = MemoryStore::from_events(vec![origin, cache]);
    let original = store.events()[0].clone();
    let plan = plan_memory_dreaming(store.events(), &pressure());
    let result = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(result.removed_events, 1);
    assert!(store.events().contains(&original));
    let reconstruction = store
        .events()
        .iter()
        .find(|event| {
            event.kind.as_deref() == Some("cache_reconstruction")
                && event.inputs.as_deref() == Some("rediscoverable-copy")
        })
        .expect("forgetting a cache must retain its reconstruction edge");
    assert!(reconstruction.content.is_none());
    assert!(
        reconstruction
            .evidence
            .contains(&"rediscover:https://doc.rust-lang.org/std/".to_owned())
    );
    assert_eq!(reconstruction.role.as_deref(), Some("system"));
    let gross_bytes = plan
        .observations
        .iter()
        .find(|event| event.event_id == "rediscoverable-copy")
        .unwrap()
        .estimated_bytes;
    assert!(result.estimated_reclaimed_bytes > 0);
    assert!(
        result.estimated_reclaimed_bytes < gross_bytes,
        "retained reconstruction metadata costs space"
    );
    let serialized = formal_ai::export_memory_links_notation(store.events());
    let mut restored =
        MemoryStore::from_events(formal_ai::parse_memory_links_notation(&serialized));
    let next = plan_memory_dreaming(restored.events(), &pressure());
    assert!(
        !next
            .actions
            .iter()
            .any(|action| action.event_id == reconstruction.id)
    );
    let _ = apply_dreaming_plan(&mut restored, &next);
    assert!(restored.events().contains(reconstruction));
    assert!(restored.events().contains(&original));
}

#[test]
fn shared_event_id_cannot_make_an_original_observation_disposable() {
    let original = MemoryEvent {
        id: "shared".to_owned(),
        role: Some("tool".to_owned()),
        kind: Some("source:http".to_owned()),
        content: Some("Original observation".to_owned()),
        ..MemoryEvent::default()
    };
    let cache = MemoryEvent {
        role: Some("cache".to_owned()),
        evidence: vec!["rediscover:https://doc.rust-lang.org/std/".to_owned()],
        ..original.clone()
    };
    let mut store = MemoryStore::from_events(vec![original, cache]);
    let before = store.events().to_vec();
    let plan = plan_memory_dreaming(store.events(), &pressure());
    let result = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(result.removed_events, 0);
    assert_eq!(result.estimated_reclaimed_bytes, 0);
    assert_eq!(store.events(), before);
}

#[test]
fn memory_pressure_does_not_replace_a_tiny_cache_with_larger_metadata() {
    let cache = MemoryEvent {
        id: "tiny".to_owned(),
        kind: Some("source:http".to_owned()),
        role: Some("cache".to_owned()),
        content: Some("x".to_owned()),
        evidence: vec!["rediscover:https://doc.rust-lang.org/std/".to_owned()],
        ..MemoryEvent::default()
    };
    let mut store = MemoryStore::from_events(vec![cache]);
    let before = store.events().to_vec();
    let plan = plan_memory_dreaming(store.events(), &pressure());
    assert!(
        plan.actions.is_empty(),
        "a non-positive net saving cannot help disk pressure"
    );
    assert!(plan.requires_bigger_storage);
    let outcome = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(outcome.removed_events, 0);
    assert_eq!(store.events(), before);
}
