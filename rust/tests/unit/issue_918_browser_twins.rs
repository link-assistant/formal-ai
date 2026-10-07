//! Issue #918 (E71, R914-6): the browser-only rows of the handler-precedence
//! table whose native twin already runs on the memory surface.
//!
//! `memory_program` and `memory_program_gap` are `browser_only` in
//! `data/seed/handler-precedence.lino` because the worker holds the memory
//! store its solve table reads. The native runtime runs the same interpreter
//! on its own memory surface (`execute_memory_query_with_options`, behind the
//! CLI's `memory query` and the protocol's memory events). Both runtimes
//! compile a request against the reviewed templates and steps of
//! `data/seed/memory-programs.lino` and render the outcome through the seeded
//! `memory_program_*` responses, so neither holds vocabulary or prose.
//!
//! The answers below are byte-identical to the ones the browser worker gives
//! for the same request over the same store; the browser twin is
//! `rust/tests/web/issue-0918-browser-twins.test.mjs`, which also recomputes
//! the program id from the canonical program text both runtimes hash.

use formal_ai::execute_memory_query_with_options;
use formal_ai::memory::{MemoryEvent, MemoryStore};
use formal_ai::memory_program::{MemoryProgramAuthorization, MemoryProgramLimits};

const RENAME_REQUEST: &str =
    "List every fact I contributed about X and rename X to Y in all of them.";
const RENAME_EN: &str = "Memory program memory_program_41ef0c9602340676 matched 1 event(s), changed 1, and stopped at fixpoint after 2 iteration(s).";
const RENAME_EMPTY_RU: &str = "Программа памяти memory_program_41ef0c9602340676 сопоставила событий: 0, изменила: 0 и остановилась с результатом fixpoint после итераций: 1.";
const DESTRUCTIVE_EN: &str = "Destructive memory actions require explicit human confirmation. No memory event was erased or retracted.";
const GAP_EN: &str = "I could not compile this memory request without dropping a step. program_gap:no_complete_seeded_family";

fn fact_store() -> MemoryStore {
    MemoryStore::from_events(vec![MemoryEvent {
        id: "fact".to_owned(),
        kind: Some("fact".to_owned()),
        role: Some("user".to_owned()),
        content: Some("X".to_owned()),
        ..MemoryEvent::default()
    }])
}

fn run(prompt: &str, store: &mut MemoryStore) -> (String, String, bool) {
    let execution = execute_memory_query_with_options(
        prompt,
        store,
        None,
        MemoryProgramLimits::default(),
        MemoryProgramAuthorization::Write,
    )
    .unwrap_or_else(|| panic!("{prompt} must reach the memory surface"));
    (
        execution.answer.intent,
        execution.answer.answer,
        execution.changed,
    )
}

#[test]
fn a_seeded_rename_program_runs_to_fixpoint_with_the_seeded_wording() {
    let mut store = fact_store();
    let (intent, answer, changed) = run(RENAME_REQUEST, &mut store);
    assert_eq!(intent, "memory_program");
    assert_eq!(answer, RENAME_EN);
    assert!(changed);
    assert_eq!(store.events()[0].content.as_deref(), Some("Y"));
}

#[test]
fn the_russian_request_compiles_to_the_same_program_and_answers_in_russian() {
    let (intent, answer, changed) = run(
        "Перечисли все факты, которые я добавил о X, и переименуй X в Y во всех них.",
        &mut MemoryStore::default(),
    );
    assert_eq!(intent, "memory_program");
    assert_eq!(answer, RENAME_EMPTY_RU);
    assert!(!changed);
}

#[test]
fn a_destructive_program_is_refused_without_confirmation() {
    let mut store = fact_store();
    let (intent, answer, changed) = run("Delete every fact I contributed about X.", &mut store);
    assert_eq!(intent, "memory_program_refused");
    assert_eq!(answer, DESTRUCTIVE_EN);
    assert!(!changed);
    assert_eq!(store.len(), 1);
}

#[test]
fn a_memory_request_no_seeded_family_covers_names_its_gap() {
    let (intent, answer, changed) = run(
        "Transpose every fact matrix in memory.",
        &mut MemoryStore::default(),
    );
    assert_eq!(intent, "memory_program_gap");
    assert_eq!(answer, GAP_EN);
    assert!(!changed);
}
