use formal_ai::{
    apply_auto_free_space_with_snapshot, apply_dreaming_plan, auto_free_space_choice,
    auto_free_space_enabled, auto_free_space_preference_path,
    create_chat_completion_with_solver_and_memory, create_response_with_solver_and_memory,
    execute_memory_query, measure_storage, persist_auto_free_space_choice, plan_memory_dreaming,
    replay_answer_with_amendments, run_core_dreaming_once, seed_cache_events, AutoFreeSpaceChoice,
    ChatCompletionRequest, ChatMessage, DreamingActionKind, DreamingConfig, DreamingDurability,
    MemoryEvent, MemoryStore, ResponsesRequest, RetainedAmendment, SolverConfig, StorageSnapshot,
    SyncStore, UniversalSolver,
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
        verified_task_run_event(
            "run-1",
            "latex",
            "latex proof render pass 1",
            "Always compile proofs with LaTeX.",
        ),
        verified_task_run_event(
            "run-2",
            "latex",
            "latex proof render pass 2",
            "Always compile proofs with LaTeX.",
        ),
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
        verified_task_run_event(
            "run-1",
            "latex",
            "latex proof render pass 1",
            "Always compile proofs with LaTeX.",
        ),
        verified_task_run_event(
            "run-2",
            "latex",
            "latex proof render pass 2",
            "Always compile proofs with LaTeX.",
        ),
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
fn learned_amendment_changes_a_new_task_answer_without_repeating_requirement() {
    let mut store = MemoryStore::from_events(vec![
        requirement_event(
            "req-1",
            "latex",
            "Always include a LaTeX verification step in proof solutions.",
        ),
        verified_task_run_event(
            "run-1",
            "latex",
            "Explain a latex proof by induction",
            "Always include a LaTeX verification step in proof solutions.",
        ),
    ]);
    let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
    let _ = apply_dreaming_plan(&mut store, &plan);

    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage::user("latex: solve a new recurrence proof")],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };
    let completion = create_chat_completion_with_solver_and_memory(
        &request,
        &UniversalSolver::default(),
        store.events(),
    );
    let answer = completion.choices[0].message.content.plain_text();

    assert!(answer.contains("Learned standing requirement"));
    assert!(answer.contains("LaTeX verification step"));

    let responses_request = ResponsesRequest {
        input: serde_json::Value::String(String::from("latex: solve another recurrence proof")),
        ..ResponsesRequest::default()
    };
    let response = create_response_with_solver_and_memory(
        &responses_request,
        &UniversalSolver::default(),
        store.events(),
    );
    let response_text = &response.output_messages()[0].content[0].text;
    assert!(response_text.contains("Learned standing requirement"));
    assert!(response_text.contains("LaTeX verification step"));
}

#[test]
fn learned_amendment_changes_the_agentic_final_answer() {
    // Issue #540 §1: retained amendments must reach the agentic loop's final
    // answer (`AgenticPlan::Final`), not only the symbolic solver path — the
    // agentic surface must not be a side door around retained learning.
    let mut store = MemoryStore::from_events(vec![
        requirement_event(
            "req-1",
            "latex",
            "Always include a LaTeX verification step in proof solutions.",
        ),
        verified_task_run_event(
            "run-1",
            "latex",
            "Explain a latex proof by induction",
            "Always include a LaTeX verification step in proof solutions.",
        ),
    ]);
    let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
    let _ = apply_dreaming_plan(&mut store, &plan);

    // A recognised shell task with only a non-shell tool advertised makes the
    // planner emit `AgenticPlan::Final` immediately (it cannot call a run tool).
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "latex: run ls in the terminal"}],
        "tools": [{
            "type": "function",
            "function": {"name": "read_file", "parameters": {"type": "object"}}
        }]
    }))
    .expect("valid tool-bearing chat request");
    let agent_solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });

    let completion =
        create_chat_completion_with_solver_and_memory(&request, &agent_solver, store.events());

    assert_eq!(completion.choices[0].finish_reason, "stop");
    let answer = completion.choices[0].message.content.plain_text();
    assert!(
        answer.contains("Learned standing requirement"),
        "agentic final answer must carry the retained amendment: {answer}"
    );
    assert!(answer.contains("LaTeX verification step"), "{answer}");
}

#[test]
fn dreaming_marks_only_replay_verified_specifics_as_covered() {
    let events = vec![
        requirement_event(
            "req-1",
            "latex",
            "Always include a LaTeX verification step in proof solutions.",
        ),
        verified_task_run_event(
            "run-verified",
            "latex",
            "Explain a latex proof by induction",
            "Always include a LaTeX verification step in proof solutions.",
        ),
        MemoryEvent {
            id: String::from("run-unverified"),
            kind: Some(String::from("test_run")),
            role: Some(String::from("assistant")),
            inputs: Some(String::from("Explain a contradiction proof")),
            outputs: Some(String::from(
                "an unrelated output that replay cannot reproduce",
            )),
            content: Some(String::from("unverified test run")),
            conversation_title: Some(String::from("latex")),
            ..MemoryEvent::default()
        },
    ];
    let plan = plan_memory_dreaming(
        &events,
        &DreamingConfig {
            storage_capacity_bytes: Some(1_000),
            free_bytes: Some(0),
            ..DreamingConfig::default()
        },
    );

    let verified = plan
        .observations
        .iter()
        .find(|observation| observation.event_id == "run-verified")
        .expect("verified run observation");
    let unverified = plan
        .observations
        .iter()
        .find(|observation| observation.event_id == "run-unverified")
        .expect("unverified run observation");
    assert!(verified.covered_by_amendment);
    assert!(!unverified.covered_by_amendment);
    assert!(!plan.actions.iter().any(|action| {
        action.event_id == "run-unverified"
            && action.kind == DreamingActionKind::ForgetCoveredSpecific
    }));
}

#[test]
fn dreaming_simulates_frequent_topic_tasks_and_mines_recurring_structures() {
    let events = vec![
        requirement_event(
            "req-1",
            "rust",
            "Always include a runnable test with Rust changes.",
        ),
        verified_task_run_event(
            "run-1",
            "rust",
            "refactor rust parser safely",
            "Always include a runnable test with Rust changes.",
        ),
        verified_task_run_event(
            "run-2",
            "rust",
            "refactor rust renderer safely",
            "Always include a runnable test with Rust changes.",
        ),
    ];

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());

    assert_eq!(plan.candidate_tasks.len(), 2);
    assert!(plan
        .candidate_tasks
        .iter()
        .all(|candidate| candidate.passed));
    assert!(plan.patterns.iter().any(|pattern| {
        pattern.topic == "rust"
            && pattern.occurrences == 2
            && pattern.structure.starts_with("refactor")
    }));
}

#[test]
fn storage_policy_measures_real_filesystem_and_persists_opt_in() {
    let memory_path = std::env::temp_dir().join(format!(
        "formal-ai-issue-540-{}-memory.lino",
        std::process::id()
    ));
    let preference_path = memory_path.with_file_name(format!(
        "{}.auto-free-space",
        memory_path.file_name().unwrap().to_string_lossy()
    ));
    let _ = std::fs::remove_file(&preference_path);

    let snapshot = measure_storage(&memory_path).expect("filesystem measurement");
    assert!(snapshot.capacity_bytes > 0);
    assert!(snapshot.free_bytes <= snapshot.capacity_bytes);
    assert!(!auto_free_space_enabled(&memory_path));
    persist_auto_free_space_choice(&memory_path, true).expect("persist opt-in");
    assert!(auto_free_space_enabled(&memory_path));
    persist_auto_free_space_choice(&memory_path, false).expect("persist opt-out");
    assert!(!auto_free_space_enabled(&memory_path));

    let _ = std::fs::remove_file(preference_path);
}

#[test]
fn dreaming_requirement_learning_uses_multilingual_data_cues() {
    let events = vec![
        requirement_event(
            "req-ru",
            "доказательства",
            "Всегда добавляй проверку результата.",
        ),
        verified_task_run_event(
            "run-ru",
            "доказательства",
            "Реши новую задачу",
            "Всегда добавляй проверку результата.",
        ),
    ];

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());

    assert!(plan.learned_requirements.iter().any(|requirement| {
        requirement.topic == "доказательства" && requirement.statement.contains("Всегда")
    }));
}

#[test]
fn multilingual_topics_receive_distinct_stable_amendment_ids() {
    let events = vec![
        requirement_event("req-proof", "证明", "始终添加验证步骤。"),
        verified_task_run_event("run-proof", "证明", "解释归纳法", "始终添加验证步骤。"),
        requirement_event("req-code", "编码", "始终添加测试步骤。"),
        verified_task_run_event("run-code", "编码", "重构解析器", "始终添加测试步骤。"),
    ];

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());
    assert_eq!(plan.amendments.len(), 2);
    assert_ne!(plan.amendments[0].id, plan.amendments[1].id);
}

#[test]
fn core_background_dreaming_learns_but_does_not_free_without_consent() {
    let memory_path = std::env::temp_dir().join(format!(
        "formal-ai-core-dreaming-{}-memory.lino",
        std::process::id()
    ));
    let store = MemoryStore::from_events(vec![
        requirement_event(
            "req-core",
            "rust",
            "Always include a runnable test with Rust changes.",
        ),
        verified_task_run_event(
            "run-core",
            "rust",
            "refactor rust parser safely",
            "Always include a runnable test with Rust changes.",
        ),
        recomputable_event("duplicate-a", "same public cache"),
        recomputable_event("duplicate-b", "same public cache"),
    ]);
    store.save_to_file(&memory_path).expect("write fixture");
    persist_auto_free_space_choice(&memory_path, false).expect("persist default-off");

    let outcome = run_core_dreaming_once(&memory_path).expect("core dreaming run");
    let after = MemoryStore::load_from_file(&memory_path).expect("load dreamed store");

    assert_eq!(outcome.removed_events, 0);
    assert!(after.events().iter().any(|event| event.id == "duplicate-a"));
    assert!(after.events().iter().any(|event| event.id == "duplicate-b"));
    assert!(after
        .events()
        .iter()
        .any(|event| { event.kind.as_deref() == Some("meta_algorithm_amendment") }));

    let _ = std::fs::remove_file(&memory_path);
    let _ = std::fs::remove_file(format!("{}.auto-free-space", memory_path.display()));
}

#[test]
fn usage_recalculation_covers_cached_and_seed_links() {
    let events = vec![
        MemoryEvent {
            id: String::from("seed-unused"),
            kind: Some(String::from("seed_cache")),
            content: Some(String::from("reconstructable seeded catalog entry")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("seed-used"),
            kind: Some(String::from("seed_cache")),
            content: Some(String::from("another seeded catalog entry")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("task-reference"),
            kind: Some(String::from("task")),
            content: Some(String::from("solve using seed-used")),
            evidence: vec![String::from("seed-used")],
            ..MemoryEvent::default()
        },
    ];
    let plan = plan_memory_dreaming(
        &events,
        &DreamingConfig {
            storage_capacity_bytes: Some(1_000),
            free_bytes: Some(0),
            ..DreamingConfig::default()
        },
    );

    assert_eq!(plan.event_usage("seed-unused"), Some(0));
    assert!(plan.event_usage("seed-used").unwrap_or_default() > 0);
    let unused_position = plan
        .actions
        .iter()
        .position(|action| action.event_id == "seed-unused")
        .expect("unused seed cache selected");
    let used_position = plan
        .actions
        .iter()
        .position(|action| action.event_id == "seed-used");
    assert!(used_position.is_none_or(|position| unused_position < position));
}

#[test]
fn dreaming_forgets_covered_specifics_first_under_pressure() {
    // Under storage pressure, specifics reproducible from a retained amendment
    // are forgotten before other recomputable data, while raw/learning stays.
    let events = vec![
        requirement_event("req-1", "latex", "Always compile proofs with LaTeX."),
        verified_task_run_event(
            "run-1",
            "latex",
            &format!("latex {}", "proof render specifics ".repeat(20)),
            "Always compile proofs with LaTeX.",
        ),
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
        verified_task_run_event(
            "run-1",
            "latex",
            "latex proof render pass 1",
            "Always compile proofs with LaTeX.",
        ),
        verified_task_run_event(
            "run-2",
            "latex",
            "latex proof render pass 2",
            "Always compile proofs with LaTeX.",
        ),
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

#[test]
fn auto_free_space_choice_distinguishes_never_asked_from_declined() {
    // Issue #540 §4: the CLI must not re-prompt a user who already declined,
    // which requires a persisted tri-state, not a boolean.
    let memory_path = std::env::temp_dir().join(format!(
        "formal-ai-issue-540-tri-state-{}-memory.lino",
        std::process::id()
    ));
    let preference_path = auto_free_space_preference_path(&memory_path);
    let _ = std::fs::remove_file(&preference_path);

    assert_eq!(
        auto_free_space_choice(&memory_path),
        AutoFreeSpaceChoice::NeverAsked
    );
    persist_auto_free_space_choice(&memory_path, false).expect("persist decline");
    assert_eq!(
        auto_free_space_choice(&memory_path),
        AutoFreeSpaceChoice::Declined
    );
    assert!(!auto_free_space_enabled(&memory_path));
    persist_auto_free_space_choice(&memory_path, true).expect("persist consent");
    assert_eq!(
        auto_free_space_choice(&memory_path),
        AutoFreeSpaceChoice::Enabled
    );
    assert!(auto_free_space_enabled(&memory_path));

    let _ = std::fs::remove_file(preference_path);
}

#[test]
fn auto_free_space_for_write_stops_at_target_with_nonzero_incoming_bytes() {
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
    assert!(store
        .events()
        .iter()
        .any(|event| event.content.as_deref() == Some("irreplaceable raw question")));
    assert!(store
        .events()
        .iter()
        .any(|event| event.content.as_deref() == Some("irreplaceable raw answer")));

    let _ = std::fs::remove_file(preference_path);
}

#[test]
fn seed_cache_events_are_stable_and_classified_recomputable() {
    // Issue #540 §4: imports materialize seed files as `seed_cache` events —
    // recomputable data with ids stable over the file name so re-import never
    // duplicates.
    let seed_files = vec![(
        String::from("data/seed/roles.lino"),
        String::from("roles\n  example\n"),
    )];
    let first = seed_cache_events(&seed_files);
    let second = seed_cache_events(&seed_files);
    assert_eq!(first.len(), 1);
    assert_eq!(
        first[0].id, second[0].id,
        "ids must be stable per file name"
    );
    assert_eq!(first[0].kind.as_deref(), Some("seed_cache"));
    assert_eq!(first[0].tool.as_deref(), Some("data/seed/roles.lino"));
    assert_eq!(first[0].content.as_deref(), Some("roles\n  example\n"));

    let plan = plan_memory_dreaming(&first, &DreamingConfig::default());
    let observation = plan
        .observations
        .iter()
        .find(|observation| observation.event_id == first[0].id)
        .expect("seed cache observed");
    assert_eq!(
        observation.durability,
        DreamingDurability::RecomputableCache
    );
}

#[test]
fn recall_counts_access_and_dreaming_treats_read_data_as_used() {
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
    assert_eq!(plan.event_usage("never-read"), Some(0));
}

#[test]
fn adding_a_requirement_revokes_coverage_and_preserves_the_stale_specific() {
    // Issue #540 §2: coverage is checked against the *current* rule set via the
    // production replay path, so changing the retained rules revokes coverage
    // of specifics recorded under the old rules — and revoked specifics must
    // no longer be forgotten as "covered".
    let statement_one = "Always include a LaTeX verification step in proof solutions.";
    let mut events = vec![
        requirement_event("req-1", "latex", statement_one),
        verified_task_run_event(
            "run-1",
            "latex",
            "Explain a latex proof by induction",
            statement_one,
        ),
    ];
    let pressure = DreamingConfig {
        storage_capacity_bytes: Some(1_000),
        free_bytes: Some(0),
        ..DreamingConfig::default()
    };

    let before = plan_memory_dreaming(&events, &pressure);
    let observed = |plan: &formal_ai::DreamingPlan| {
        plan.observations
            .iter()
            .find(|observation| observation.event_id == "run-1")
            .expect("run-1 observed")
            .covered_by_amendment
    };
    assert!(observed(&before), "run-1 starts replay-verified as covered");
    assert!(before.actions.iter().any(|action| {
        action.event_id == "run-1" && action.kind == DreamingActionKind::ForgetCoveredSpecific
    }));

    // A second requirement on the same topic changes the joined rule text, so
    // the stored output (produced under the old rule) is no longer re-derived.
    let statement_two = "Always cite the numbered theorem being applied.";
    events.push(requirement_event("req-2", "latex", statement_two));
    let after = plan_memory_dreaming(&events, &pressure);

    let amendment = after
        .amendments
        .iter()
        .find(|amendment| amendment.topic == "latex")
        .expect("latex amendment");
    assert!(amendment.rule.contains(statement_one), "{}", amendment.rule);
    assert!(amendment.rule.contains(statement_two), "{}", amendment.rule);
    assert!(
        !observed(&after),
        "a rule change must revoke coverage of specifics recorded under the old rule"
    );
    assert!(
        !after.actions.iter().any(|action| {
            action.event_id == "run-1" && action.kind == DreamingActionKind::ForgetCoveredSpecific
        }),
        "revoked specifics must not be forgotten as covered"
    );
}

#[test]
fn failed_verification_falls_back_to_normal_eviction_ordering_under_pressure() {
    // Issue #540 §2: a specific that fails replay verification loses only its
    // forget-first status — under pressure its actual fate is *normal*
    // eviction, ranked after covered specifics and after cheaper-to-restore
    // recomputable caches.
    let statement = "Always include a LaTeX verification step in proof solutions.";
    let events = vec![
        requirement_event("req-1", "latex", statement),
        verified_task_run_event(
            "run-covered",
            "latex",
            "Explain a latex proof by induction",
            statement,
        ),
        MemoryEvent {
            id: String::from("run-unverified"),
            kind: Some(String::from("test_run")),
            role: Some(String::from("assistant")),
            inputs: Some(String::from("Explain a latex contradiction proof")),
            outputs: Some(String::from(
                "an unrelated output that replay cannot reproduce",
            )),
            content: Some(String::from("unverified test run")),
            conversation_title: Some(String::from("latex")),
            ..MemoryEvent::default()
        },
        recomputable_event("public-cache", "refetchable public source payload"),
    ];
    // Pressure far beyond the store's total size, so every reclaimable event
    // is selected and the plan exposes the full eviction ordering.
    let plan = plan_memory_dreaming(
        &events,
        &DreamingConfig {
            storage_capacity_bytes: Some(1_000_000),
            free_bytes: Some(0),
            ..DreamingConfig::default()
        },
    );

    let position = |id: &str| {
        plan.actions
            .iter()
            .position(|action| action.event_id == id)
            .unwrap_or_else(|| panic!("{id} must be selected under this pressure"))
    };
    let unverified = &plan.actions[position("run-unverified")];
    assert_eq!(
        unverified.kind,
        DreamingActionKind::EvictLowUseRecomputable,
        "the failed-verification specific falls back to normal eviction"
    );
    assert_eq!(
        plan.actions[position("run-covered")].kind,
        DreamingActionKind::ForgetCoveredSpecific
    );
    assert!(
        position("run-covered") < position("run-unverified"),
        "covered specifics are reclaimed before unverified ones"
    );
    assert!(
        position("public-cache") < position("run-unverified"),
        "refetchable caches are reclaimed before unverified derived runs"
    );
    assert!(!plan.actions.iter().any(|action| action.event_id == "req-1"));
}

#[test]
fn organically_recorded_chat_dreams_amendments_that_replay_through_production() {
    // Issue #540 §2: the loop must work on *organically recorded* data, not
    // fixtures shaped like the simulator output — record live chat turns via
    // `SyncStore`, dream over the recorded file, and verify the learned rule
    // both changes a later production answer and replays exactly.
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-issue-540-organic-loop-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("memory.lino");
    let solver = UniversalSolver::default();
    let chat_answer = |prompt: &str, events: &[MemoryEvent]| {
        let request = ChatCompletionRequest {
            model: None,
            messages: vec![ChatMessage::user(prompt)],
            temperature: None,
            stream: false,
            tools: Vec::new(),
            tool_choice: None,
            functions: Vec::new(),
            function_call: None,
            stream_options: None,
        };
        create_chat_completion_with_solver_and_memory(&request, &solver, events).choices[0]
            .message
            .content
            .plain_text()
    };

    // Live turn 1: the user states a requirement; the production surface
    // answers and the exchange is recorded through the live-recording path.
    let requirement_prompt = "latex proofs: Always include a LaTeX verification step.";
    let mut sync = SyncStore::open_at(&path);
    let requirement_answer = chat_answer(requirement_prompt, sync.events());
    sync.record_chat_exchange(requirement_prompt, &requirement_answer)
        .expect("record requirement exchange");

    // Dream over the recorded file: the organically recorded requirement must
    // generalize into a retained amendment even though pure chat stores hold
    // no reclaimable specifics yet.
    let mut dreamed = MemoryStore::load_from_file(&path).expect("load recorded store");
    let plan = plan_memory_dreaming(dreamed.events(), &DreamingConfig::default());
    assert!(
        plan.learned_requirements
            .iter()
            .any(|requirement| requirement.topic == "latex"),
        "requirement learning must lift the recorded chat turn"
    );
    let outcome = apply_dreaming_plan(&mut dreamed, &plan);
    assert_eq!(outcome.learned_amendments, 1);
    dreamed.save_to_file(&path).expect("persist dreamed store");

    // Live turn 2 on the topic: the learned rule now shapes the answer.
    let task_prompt = "latex: prove the sum formula";
    let task_answer = chat_answer(task_prompt, dreamed.events());
    assert!(
        task_answer.contains("Learned standing requirement (latex)"),
        "{task_answer}"
    );
    let mut sync = SyncStore::open_at(&path);
    sync.record_chat_exchange(task_prompt, &task_answer)
        .expect("record task exchange");

    // Re-dream: the organically recorded exchange replays through the same
    // production application path and verifies.
    let replan = plan_memory_dreaming(sync.events(), &DreamingConfig::default());
    let candidate = replan
        .candidate_tasks
        .iter()
        .find(|candidate| candidate.input == task_prompt)
        .expect("recorded live exchange becomes a replay candidate");
    assert!(
        candidate.passed,
        "production-recorded output must replay exactly: {}",
        candidate.simulated_output
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn failed_replay_refines_the_amendment_back_from_the_recorded_marker() {
    // Issue #540 §3: the auto-learning loop must consume failed simulations.
    // A historical run whose output carries the production compliance marker
    // proves a rule was once in force; when the requirement events themselves
    // are gone, refinement folds the statement back, re-verifies, and the
    // specific becomes covered again.
    let rule = "Always cite the numbered theorem being applied.";
    let historical = verified_task_run_event(
        "run-historical",
        "latex",
        "Explain a latex proof by induction",
        rule,
    );
    // No requirement events exist for the topic — only the historical run.
    let plan = plan_memory_dreaming(&[historical], &DreamingConfig::default());

    let amendment = plan
        .amendments
        .iter()
        .find(|amendment| amendment.topic == "latex")
        .expect("refinement must recreate the lost amendment");
    assert_eq!(amendment.rule, rule);
    assert!(amendment
        .covered_event_ids
        .contains(&String::from("run-historical")));
    let candidate = plan
        .candidate_tasks
        .iter()
        .find(|candidate| candidate.source_event_id == "run-historical")
        .expect("historical run replayed");
    assert!(candidate.passed, "{}", candidate.simulated_output);
    assert!(plan.learned_requirements.iter().any(|requirement| {
        requirement.statement == rule
            && requirement
                .source_event_ids
                .contains(&String::from("run-historical"))
    }));
    assert!(
        plan.observations
            .iter()
            .find(|observation| observation.event_id == "run-historical")
            .expect("observed")
            .covered_by_amendment
    );
}

#[test]
fn failed_replays_are_preserved_as_refinement_records() {
    // Issue #540 §3: failed simulations must not be discarded — applying the
    // plan materializes each as a `dreaming_candidate_failure` event carrying
    // the diverging input/output pair, and re-applying does not duplicate it.
    let mut store = MemoryStore::from_events(vec![
        requirement_event(
            "req-1",
            "latex",
            "Always include a LaTeX verification step in proof solutions.",
        ),
        MemoryEvent {
            id: String::from("run-unverified"),
            kind: Some(String::from("test_run")),
            role: Some(String::from("assistant")),
            inputs: Some(String::from("Explain a latex contradiction proof")),
            outputs: Some(String::from(
                "an unrelated output that replay cannot reproduce",
            )),
            content: Some(String::from("unverified test run")),
            conversation_title: Some(String::from("latex")),
            ..MemoryEvent::default()
        },
    ]);
    let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
    assert!(plan
        .candidate_tasks
        .iter()
        .any(|candidate| candidate.source_event_id == "run-unverified" && !candidate.passed));

    let outcome = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(outcome.recorded_failures, 1);
    let failure = store
        .events()
        .iter()
        .find(|event| event.kind.as_deref() == Some("dreaming_candidate_failure"))
        .expect("failed replay preserved");
    assert_eq!(
        failure.inputs.as_deref(),
        Some("Explain a latex contradiction proof")
    );
    assert_eq!(
        failure.outputs.as_deref(),
        Some("an unrelated output that replay cannot reproduce")
    );

    let replan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
    let second = apply_dreaming_plan(&mut store, &replan);
    assert_eq!(
        second.recorded_failures, 0,
        "failure records are idempotent"
    );
}

#[test]
fn dreaming_synthesizes_new_trials_from_numeric_patterns_on_top_topics() {
    // Issue #540 §3: dreaming must find *new* tasks on the most-used topics —
    // a recurring numeric structure spawns a never-seen trial, solved through
    // the production amendment path and retained as a `dreaming_trial` event.
    let task = |id: &str, input: &str| MemoryEvent {
        id: String::from(id),
        kind: Some(String::from("test_run")),
        role: Some(String::from("assistant")),
        inputs: Some(String::from(input)),
        outputs: Some(String::from("done")),
        content: Some(String::from(input)),
        conversation_title: Some(String::from("math")),
        ..MemoryEvent::default()
    };
    let mut store =
        MemoryStore::from_events(vec![task("run-1", "add 1 2"), task("run-2", "add 3 4")]);

    let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
    assert!(plan
        .patterns
        .iter()
        .any(|pattern| pattern.topic == "math" && pattern.structure == "add # #"));
    let trial = plan
        .synthesized_tasks
        .iter()
        .find(|trial| trial.topic == "math")
        .expect("numeric pattern must synthesize a trial");
    assert_eq!(trial.input, "add 2 3", "advances numbers past seen inputs");
    assert!(!trial.answer.is_empty());

    let outcome = apply_dreaming_plan(&mut store, &plan);
    assert_eq!(outcome.recorded_trials, 1);
    assert!(store.events().iter().any(|event| {
        event.kind.as_deref() == Some("dreaming_trial")
            && event.inputs.as_deref() == Some("add 2 3")
    }));
}

#[test]
fn multilingual_task_kinds_are_replayed_as_candidates() {
    // Issue #540 §3/§7: task detection is grounded in the multilingual data
    // lexicon — a run recorded with a Russian kind must be visible to replay
    // verification, not only English `task`/`test_run` kinds.
    let rule = "Всегда добавляй проверку результата.";
    let output = replay_answer_with_amendments(
        "Реши задачу по индукции",
        &[RetainedAmendment {
            id: String::from("ru-amendment"),
            topic: String::from("доказательства"),
            rule: String::from(rule),
        }],
    );
    let events = vec![
        requirement_event("req-ru", "доказательства", rule),
        MemoryEvent {
            id: String::from("run-ru"),
            kind: Some(String::from("проверка")),
            role: Some(String::from("assistant")),
            inputs: Some(String::from("Реши задачу по индукции")),
            outputs: Some(output),
            conversation_title: Some(String::from("доказательства")),
            ..MemoryEvent::default()
        },
    ];

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());
    let candidate = plan
        .candidate_tasks
        .iter()
        .find(|candidate| candidate.source_event_id == "run-ru")
        .expect("Russian-kind run must be replayed as a candidate");
    assert!(candidate.passed, "{}", candidate.simulated_output);
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

fn verified_task_run_event(id: &str, topic: &str, input: &str, requirement: &str) -> MemoryEvent {
    // The expected output is produced by the production application path
    // itself (issue #540 §2): replay verification then compares like with
    // like, and a fixture whose input does not actually match the topic will
    // fail replay exactly as an organic record would.
    let output = replay_answer_with_amendments(
        input,
        &[RetainedAmendment {
            id: format!("{id}-amendment"),
            topic: String::from(topic),
            rule: String::from(requirement),
        }],
    );
    MemoryEvent {
        id: String::from(id),
        kind: Some(String::from("test_run")),
        role: Some(String::from("assistant")),
        content: Some(String::from(input)),
        inputs: Some(String::from(input)),
        outputs: Some(output),
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
