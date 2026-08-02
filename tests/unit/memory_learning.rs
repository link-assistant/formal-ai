//! Issue #540 §2–§3: verified learning regression tests — coverage revocation,
//! eviction fallback, the organic record→dream→apply loop, refinement
//! resurrection, durable failure records, trial synthesis, and multilingual
//! task-kind gating.

use formal_ai::{
    apply_dreaming_plan, create_chat_completion_with_solver_and_memory, plan_memory_dreaming,
    replay_answer_with_amendments, ChatCompletionRequest, ChatMessage, DreamingActionKind,
    DreamingConfig, MemoryEvent, MemoryStore, RetainedAmendment, SyncStore, UniversalSolver,
};
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

#[test]
fn hindi_task_kinds_are_replayed_as_candidates() {
    // Issue #540 §3/§7: the same lexicon-grounded gating must hold for Hindi —
    // a run recorded with the Hindi kind "परीक्षण" (test) is replayed as a
    // candidate exactly like English and Russian kinds.
    let rule = "हमेशा परिणाम की जाँच जोड़ें।";
    let output = replay_answer_with_amendments(
        "आगमन द्वारा प्रमाण हल करें",
        &[RetainedAmendment {
            id: String::from("hi-amendment"),
            topic: String::from("प्रमाण"),
            rule: String::from(rule),
        }],
    );
    let events = vec![
        requirement_event("req-hi", "प्रमाण", rule),
        MemoryEvent {
            id: String::from("run-hi"),
            kind: Some(String::from("परीक्षण")),
            role: Some(String::from("assistant")),
            inputs: Some(String::from("आगमन द्वारा प्रमाण हल करें")),
            outputs: Some(output),
            conversation_title: Some(String::from("प्रमाण")),
            ..MemoryEvent::default()
        },
    ];

    let plan = plan_memory_dreaming(&events, &DreamingConfig::default());
    let candidate = plan
        .candidate_tasks
        .iter()
        .find(|candidate| candidate.source_event_id == "run-hi")
        .expect("Hindi-kind run must be replayed as a candidate");
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
