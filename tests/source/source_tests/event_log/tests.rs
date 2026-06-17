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
fn thinking_steps_project_events_to_canonical_user_steps() {
    let mut log = EventLog::new();
    log.append("impulse", "Hi");
    log.append("language", "en");
    log.append("intent_formalization:route", "greeting");
    log.append("validation", "accepted");
    log.append("response", "response:greeting");

    let steps = log.thinking_steps();

    assert_eq!(steps.len(), 5);
    assert_eq!(steps[0].order, 0);
    assert_eq!(steps[0].step, "impulse");
    assert_eq!(steps[1].step, "detect_language");
    assert_eq!(steps[2].step, "formalize");
    assert_eq!(steps[3].step, "rule_verification");
    assert_eq!(steps[4].step, "deformalize");
    assert_eq!(steps[4].source_event, "response");
}

#[test]
fn thinking_steps_fold_calculation_trace_into_composite_compute_step() {
    let mut log = EventLog::new();
    log.append("impulse", "What is 8% of $50?");
    log.append("language", "en");
    log.append("calculation:request", "8% of $50");
    log.append("calculation:engine", "link-calculator");
    log.append("calculation:lino", "((8 / 100) * (50 USD))");
    log.append("calculation:steps", "9");
    log.append("calculation", "8% of $50 = 4 USD");
    log.append("intent", "calculation");
    log.append("validation", "accepted_without_extra_constraints");
    log.append("response", "response:calculation");

    let steps = log.thinking_steps();

    // The calculator trace collapses into one composite `compute` parent...
    let compute = steps
        .iter()
        .find(|step| step.step == "compute")
        .expect("a compute step is projected");
    assert_eq!(compute.detail, "8% of $50 = 4 USD");
    assert_eq!(compute.level, "high");
    assert!(
        compute.parent_id.is_none(),
        "the composite compute step is itself a top-level parent"
    );
    assert!(
        compute.summary.contains("8% of $50 = 4 USD"),
        "the compute summary is concrete: {:?}",
        compute.summary
    );

    // ...with engine/expression/steps as detailed children pointing at it (R11).
    let children: Vec<_> = steps
        .iter()
        .filter(|step| step.parent_id.as_deref() == Some(compute.id.as_str()))
        .collect();
    assert_eq!(children.len(), 3, "engine + expression + steps children");
    assert!(children.iter().all(|child| child.level == "detailed"));
    assert!(steps
        .iter()
        .any(|step| step.step == "compute_expression" && step.detail == "((8 / 100) * (50 USD))"));

    // Internal acceptance bookkeeping is dropped from the curated view.
    assert!(
        !steps
            .iter()
            .any(|step| step.detail.contains("accepted_without_extra_constraints")),
        "validation payload noise must not leak into the curated steps"
    );
}

#[test]
fn thinking_steps_for_answer_makes_the_closing_step_concrete() {
    let mut log = EventLog::new();
    log.append("impulse", "Hi");
    log.append("language", "en");
    log.append("intent_formalization:route", "greeting");
    log.append("response", "response:greeting");

    // Without an answer the closing step echoes the opaque response link...
    let opaque = log.thinking_steps();
    let last_opaque = opaque.last().expect("a closing step");
    assert_eq!(last_opaque.step, "deformalize");
    assert_eq!(last_opaque.detail, "response:greeting");

    // ...but `thinking_steps_for_answer` substitutes the real composed answer.
    let concrete = log.thinking_steps_for_answer("Hi, how may I help you?");
    let last = concrete.last().expect("a closing step");
    assert_eq!(last.step, "deformalize");
    assert_eq!(last.detail, "Hi, how may I help you?");
    assert!(
        last.summary.contains("Hi, how may I help you?"),
        "the closing summary contains the actual answer: {:?}",
        last.summary
    );
}

#[test]
fn thinking_steps_drop_internal_bookkeeping_noise() {
    let mut log = EventLog::new();
    log.append("impulse", "Hi");
    log.append("intent_formalization", "{big formalization blob}");
    log.append("formalization:0", "score=520 probability=1.0");
    log.append("candidate", "greeting");
    log.append("selected_rule", "greeting");
    log.append("policy:checked", "ok");
    log.append("trace:simplification", "smallest_sufficient");
    log.append("response", "response:greeting");

    let steps = log.thinking_steps();
    let kinds: Vec<&str> = steps
        .iter()
        .map(|step| step.source_event.as_str())
        .collect();
    assert!(kinds.contains(&"impulse"));
    assert!(kinds.contains(&"response"));
    // None of the noisy internal kinds survive curation (allowlist approach).
    for noisy in [
        "intent_formalization",
        "formalization:0",
        "candidate",
        "selected_rule",
        "policy:checked",
        "trace:simplification",
    ] {
        assert!(
            !kinds.contains(&noisy),
            "noisy event {noisy:?} must be dropped from the curated steps"
        );
    }
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
