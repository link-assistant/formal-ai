use formal_ai::execute_memory_query_with_options;
use formal_ai::memory::{isoformat_now, MemoryEvent, MemoryStore};
use formal_ai::memory_program::{
    compile_memory_program, execute_memory_program, parse_memory_program_links_notation,
    MemoryProgramAuthorization, MemoryProgramHalt, MemoryProgramLimits,
};

const LIMITS: MemoryProgramLimits = MemoryProgramLimits {
    max_matches: 32,
    max_iterations: 4,
};

fn event(id: &str, kind: &str, role: &str, content: &str) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        kind: Some(kind.to_owned()),
        role: Some(role.to_owned()),
        content: Some(content.to_owned()),
        sent_at: Some(isoformat_now()),
        ..MemoryEvent::default()
    }
}

#[test]
fn fifteen_seeded_query_families_compile_with_a_program_trace() {
    let requests = [
        "List every fact I contributed about X and rename X to Y in all of them.",
        "Count events per topic this week and store the summary.",
        "For every meaning without a Russian label, add a todo link.",
        "List every fact about engines.",
        "Replace alpha with beta in every memory event.",
        "Archive every duplicate fact.",
        "Tag every fact about engines with reviewed.",
        "Copy every fact about engines to collection research.",
        "Delete every fact I contributed about engines.",
        "Normalize every label until no label changes.",
        "List this week's events by topic.",
        "Count facts per contributor and store the summary.",
        "Add a todo link for every meaning without an English label.",
        "Append reviewed to every fact dated 2026-08-01.",
        "Create a review link for every meaning with no links.",
    ];
    let mut families = std::collections::BTreeSet::new();
    for request in requests {
        let program = compile_memory_program(request, LIMITS).expect(request);
        families.insert(program.family.clone());
        let trace = program.links_notation();
        assert!(trace.starts_with("memory_program\n"), "{request}: {trace}");
        assert!(trace.contains("  step 1\n"), "{request}: {trace}");
        let execution = execute_memory_query_with_options(
            request,
            &mut MemoryStore::default(),
            None,
            LIMITS,
            MemoryProgramAuthorization::DestructiveConfirmed,
        )
        .expect("seeded family should route through the memory program surface");
        assert!(
            execution.answer.links_notation.contains(&program.id),
            "compiled program missing from trace for {request}: {}",
            execution.answer.links_notation
        );
    }
    assert_eq!(families.len(), 15);
}

#[test]
fn compiled_program_round_trips_replace_and_when_do_shapes() {
    let program = compile_memory_program(
        "List every fact I contributed about X and rename X to Y in all of them.",
        LIMITS,
    )
    .expect("seeded family");
    let notation = program.links_notation();
    assert!(notation.contains("  replace\n    old \"X\"\n    new \"Y\""));
    assert!(notation.contains("  when \"match\"\n    do \"update\""));
    assert_eq!(
        parse_memory_program_links_notation(&notation).expect("round trip"),
        program
    );

    let edited = notation.replacen("    new \"Y\"", "    new \"Z\"", 1);
    let edited = parse_memory_program_links_notation(&edited).expect("editable program");
    assert_ne!(edited.id, program.id);
    let mut store = MemoryStore::from_events(vec![event("fact", "fact", "user", "X")]);
    let outcome = execute_memory_program(&edited, &mut store, MemoryProgramAuthorization::Write);
    assert_eq!(outcome.halt, MemoryProgramHalt::Fixpoint);
    assert_eq!(store.events()[0].content.as_deref(), Some("Z"));
}

#[test]
fn contributed_facts_are_selectively_renamed_to_fixpoint() {
    let mut store = MemoryStore::from_events(vec![
        event("mine", "fact", "user", "X powers the engine"),
        event("assistant", "fact", "assistant", "X is only a draft"),
        event("other", "fact", "user", "Z is unrelated"),
    ]);
    let program = compile_memory_program(
        "List every fact I contributed about X and rename X to Y in all of them.",
        LIMITS,
    )
    .expect("seeded family");
    let outcome = execute_memory_program(&program, &mut store, MemoryProgramAuthorization::Write);

    assert_eq!(outcome.halt, MemoryProgramHalt::Fixpoint);
    assert_eq!(outcome.changed, 1);
    assert_eq!(
        store.events()[0].content.as_deref(),
        Some("Y powers the engine")
    );
    assert_eq!(
        store.events()[1].content.as_deref(),
        Some("X is only a draft")
    );
    assert!(outcome.links_notation().contains("halt fixpoint"));
}

#[test]
fn topic_counts_and_missing_labels_create_deduplicated_links() {
    let mut topics = MemoryStore::from_events(vec![
        MemoryEvent {
            intent: Some(String::from("engines")),
            ..event("a", "event", "user", "first")
        },
        MemoryEvent {
            intent: Some(String::from("engines")),
            ..event("b", "event", "user", "second")
        },
        MemoryEvent {
            intent: Some(String::from("memory")),
            ..event("c", "event", "user", "third")
        },
    ]);
    let count = compile_memory_program(
        "Count events per topic this week and store the summary.",
        LIMITS,
    )
    .expect("seeded family");
    let outcome = execute_memory_program(&count, &mut topics, MemoryProgramAuthorization::Write);
    assert_eq!(outcome.halt, MemoryProgramHalt::Fixpoint);
    let summary = topics
        .events()
        .iter()
        .find(|event| event.kind.as_deref() == Some("topic_summary"))
        .and_then(|event| event.content.as_deref())
        .expect("stored topic summary");
    assert!(summary.contains("engines=2"), "{summary}");
    assert!(summary.contains("memory=1"), "{summary}");

    let mut meanings = MemoryStore::from_events(vec![
        event("missing", "meaning", "user", "engine"),
        MemoryEvent {
            evidence: vec![String::from("label:ru=двигатель")],
            ..event("labeled", "meaning", "user", "motor")
        },
    ]);
    let todo = compile_memory_program(
        "For every meaning without a Russian label, add a todo link.",
        LIMITS,
    )
    .expect("seeded family");
    let _ = execute_memory_program(&todo, &mut meanings, MemoryProgramAuthorization::Write);
    let todos = meanings
        .events()
        .iter()
        .filter(|event| event.kind.as_deref() == Some("todo"))
        .collect::<Vec<_>>();
    assert_eq!(todos.len(), 1);
    assert!(todos[0].inputs.as_deref().is_some_and(|id| id == "missing"));
}

#[test]
fn bounds_stop_honestly_before_partial_writes() {
    let limits = MemoryProgramLimits {
        max_matches: 1,
        max_iterations: 3,
    };
    let program = compile_memory_program("Replace X with XX in every memory event.", limits)
        .expect("seeded family");
    let mut too_many = MemoryStore::from_events(vec![
        event("a", "fact", "user", "X"),
        event("b", "fact", "user", "X"),
    ]);
    let outcome =
        execute_memory_program(&program, &mut too_many, MemoryProgramAuthorization::Write);
    assert_eq!(
        outcome.halt,
        MemoryProgramHalt::MatchLimit {
            matched: 2,
            max_matches: 1,
        }
    );
    assert_eq!(outcome.changed, 0);
    assert!(outcome
        .links_notation()
        .contains("matched 2 exceeds max_matches 1"));

    let mut unbounded = MemoryStore::from_events(vec![event("a", "fact", "user", "X")]);
    let outcome =
        execute_memory_program(&program, &mut unbounded, MemoryProgramAuthorization::Write);
    assert_eq!(
        outcome.halt,
        MemoryProgramHalt::IterationLimit { max_iterations: 3 }
    );
    assert!(outcome
        .links_notation()
        .contains("max_iterations 3 reached"));
}

#[test]
fn destructive_program_requires_confirmation_and_appends_a_retraction() {
    let program = compile_memory_program("Delete every fact I contributed about engines.", LIMITS)
        .expect("seeded family");
    let mut store =
        MemoryStore::from_events(vec![event("fact-1", "fact", "user", "engines need fuel")]);

    let refused = execute_memory_program(&program, &mut store, MemoryProgramAuthorization::Write);
    assert_eq!(
        refused.halt,
        MemoryProgramHalt::PermissionDenied {
            required: String::from("destructive"),
        }
    );
    assert_eq!(store.len(), 1);
    assert!(refused
        .links_notation()
        .contains("policy destructive_action_requires_confirmation"));

    let applied = execute_memory_program(
        &program,
        &mut store,
        MemoryProgramAuthorization::DestructiveConfirmed,
    );
    assert_eq!(applied.halt, MemoryProgramHalt::Fixpoint);
    assert_eq!(store.len(), 2);
    assert_eq!(store.events()[0].id, "fact-1");
    assert_eq!(store.events()[1].kind.as_deref(), Some("memory_retraction"));
    assert_eq!(store.events()[1].inputs.as_deref(), Some("fact-1"));
}

#[test]
fn query_surface_traces_the_compiled_program_and_names_program_gaps() {
    let mut store = MemoryStore::from_events(vec![event("fact", "fact", "user", "X")]);
    let execution = execute_memory_query_with_options(
        "List every fact I contributed about X and rename X to Y in all of them.",
        &mut store,
        None,
        LIMITS,
        MemoryProgramAuthorization::Write,
    )
    .expect("memory program route");
    assert_eq!(execution.answer.intent, "memory_program");
    assert!(execution
        .answer
        .links_notation
        .contains("memory_program_compiled"));
    assert!(execution
        .answer
        .evidence_links
        .iter()
        .any(|link| link.starts_with("memory_program_compiled:")));

    let gap = execute_memory_query_with_options(
        "Transpose every fact matrix in memory.",
        &mut store,
        None,
        LIMITS,
        MemoryProgramAuthorization::Write,
    )
    .expect("honest memory program gap");
    assert_eq!(gap.answer.intent, "memory_program_gap");
    assert!(gap.answer.answer.contains("program_gap"));
    assert!(!gap.changed);
}