//! Issue #715: the link dialect must be reachable from the surface that owns
//! the links.
//!
//! `issue_715_link_substitution_query` pins the dialect itself. This pins its
//! route into the product: `parse_link_substitution_query` and `matched_links`
//! were public library API with no caller in `src/`, so the half of the query
//! language that operates on links could not be reached from the CLI at all.
//! Review feedback on #727 asked to "convert from it to actual read/write/update
//! tools" — that conversion is only real if a request can actually travel it.

use formal_ai::memory::{MemoryEvent, MemoryStore};
use formal_ai::{execute_memory_query, MemoryQueryExecution};

/// Two events, so a read has something to select *between* rather than merely
/// something to return.
fn store() -> MemoryStore {
    let mut store = MemoryStore::default();
    for (id, content) in [("e1", "the kettle is on"), ("e2", "the cat is asleep")] {
        store.append(MemoryEvent {
            id: String::from(id),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some(String::from(content)),
            conversation_id: Some(String::from("c1")),
            ..MemoryEvent::default()
        });
    }
    store
}

fn query(prompt: &str) -> MemoryQueryExecution {
    let mut store = store();
    execute_memory_query(prompt, &mut store, None)
        .unwrap_or_else(|| panic!("{prompt} should be recognized as a memory query"))
}

/// link-cli documents `(($i: $s $t)) (($i: $s $t))` as reading "all links
/// without modification", so it must return the store rather than rewrite it.
#[test]
fn a_read_query_returns_every_projected_link() {
    let execution = query("(($i: $s $t)) (($i: $s $t))");

    assert!(
        !execution.changed,
        "a read must not mark the store dirty: {}",
        execution.answer.answer
    );
    assert_eq!(execution.answer.intent, "memory_link_query");

    let body = &execution.answer.answer;
    let matched = body
        .lines()
        .next()
        .expect("the answer should open with a count");
    // Both events project, and every projected link matches the all-variables
    // pattern, so matched and total have to agree.
    let (count, total) = matched
        .strip_prefix("matched ")
        .and_then(|rest| rest.strip_suffix(" links"))
        .and_then(|rest| rest.split_once(" of "))
        .unwrap_or_else(|| panic!("unexpected count line: {matched:?}"));
    assert_eq!(count, total, "a read of every link must match every link");
    assert!(
        count.parse::<usize>().is_ok_and(|value| value > 0),
        "the projection should not be empty: {matched:?}"
    );

    // The answer is written in the notation the question was asked in: every
    // stored link carries an index, so it comes back in the `(index: source
    // target)` form rather than the two-slot one.
    assert!(
        body.contains(": Type MemoryEvent)"),
        "the projection's own doublets should come back rendered: {body}"
    );
}

/// A literal value in a slot selects, so a read can ask for one link rather than
/// all of them. This is the case that proves matching is real and not a dump.
#[test]
fn a_read_query_can_select_a_single_link() {
    let execution = query("(($i: Type MemoryEvent)) (($i: Type MemoryEvent))");

    let body = &execution.answer.answer;
    assert!(
        body.starts_with("matched 2 of "),
        "both events project the same Type -> MemoryEvent doublet: {body}"
    );
    assert!(
        !body.contains("SubType"),
        "a selective read must not return links it did not match: {body}"
    );
}

/// The projection is derived one-way, so a write has no inverse back to the
/// event that produced the link. Refusing is the honest answer; the message has
/// to name the boundary rather than fail silently.
#[test]
fn link_level_writes_are_refused_and_leave_the_store_alone() {
    for (label, prompt) in [
        ("update", "((1: Type MemoryEvent)) ((1: Type Rewritten))"),
        ("delete", "(($i: Type MemoryEvent)) ()"),
        ("create", "() ((Type Fabricated))"),
    ] {
        let mut store = store();
        let before = store.events().to_vec();
        let execution = execute_memory_query(prompt, &mut store, None)
            .unwrap_or_else(|| panic!("{label}: {prompt} should be recognized"));

        assert!(!execution.changed, "{label}: a refusal cannot be a change");
        assert_eq!(
            execution.answer.intent, "memory_link_query_rejected",
            "{label}: should be refused, got {:?}",
            execution.answer.answer
        );
        assert!(
            execution.answer.answer.contains("one-way projection"),
            "{label}: the refusal must name the boundary, got {:?}",
            execution.answer.answer
        );
        assert_eq!(
            store.events(),
            before.as_slice(),
            "{label}: a refused write must not touch the store"
        );
    }
}

/// Parsing is the recogniser, so prose that merely opens with a parenthesis must
/// fall through to natural language instead of being answered with a parse
/// error. Without this the route would capture ordinary turns.
#[test]
fn prose_that_opens_with_a_parenthesis_is_not_captured() {
    let mut store = store();
    let execution = execute_memory_query("(remember that the kettle is on)", &mut store, None);

    let intent = execution.map(|execution| execution.answer.intent);
    assert_ne!(
        intent.as_deref(),
        Some("memory_link_query"),
        "prose must not be read as a link query"
    );
    assert_ne!(
        intent.as_deref(),
        Some("memory_link_query_rejected"),
        "prose must not be refused as a link query"
    );
}
