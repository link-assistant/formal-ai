//! Issue #901: automate TRIZ principles and general paradox resolution.
//!
//! The ask, in three parts: (1) contradictions are links carrying a 0-1
//! value chosen from the requirements, which rust/src/selection_heuristics.rs
//! already computes for candidate sets; (2) collect the *other* common
//! ways paradoxes and contradictions dissolve — different dimensions,
//! alternation, points on the range — as data; (3) replay it on a top-20
//! corpus of commonly discussed invention tasks and expose it from a
//! prompt. These tests pin the seed additions (twelve resolution
//! families, the twenty-task benchmark, the cues) and the user-facing
//! entry in rust/src/triz_solver.rs.

use formal_ai::event_log::EventLog;
use formal_ai::triz_solver::{handle_triz, triz_benchmark_tasks, triz_families};
use formal_ai::web_engine_core::normalize_prompt;

/// The handler answer for a raw prompt.
fn handled(prompt: &str) -> Option<formal_ai::engine::SymbolicAnswer> {
    handle_triz(prompt, &normalize_prompt(prompt), &mut EventLog::new())
}

#[test]
fn the_seed_carries_twelve_families_and_twenty_benchmark_tasks() {
    let families = triz_families();
    assert!(families.len() >= 12, "got {}", families.len());
    // The two shapes #901 names explicitly must be present.
    assert!(
        families
            .iter()
            .any(|family| family.method_id == "family_range_selection")
    );
    assert!(
        families
            .iter()
            .any(|family| family.method_id == "family_dimension_change")
    );
    let tasks = triz_benchmark_tasks();
    assert_eq!(tasks.len(), 20, "the top-20 corpus");
    for task in &tasks {
        assert!(
            !task.contradiction.is_empty(),
            "{} states its contradiction",
            task.task_id
        );
        assert!(
            !task.methods.is_empty(),
            "{} names its methods",
            task.task_id
        );
    }
}

#[test]
fn a_russian_contradiction_prompt_gets_the_family_map() {
    let answer =
        handled("Как разрешить противоречие: деталь должна быть жёсткой и одновременно гибкой?")
            .expect("a contradiction question must be handled");
    assert_eq!(answer.intent, "triz_resolution");
    assert!(
        answer.answer.contains("Range selection") && answer.answer.contains("Dimension change"),
        "the family map must be in the answer: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("no default 50 %"),
        "the 0-1 link value note must be stated: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("methods: family_"),
        "precedents must cite their methods: {}",
        answer.answer
    );
}

#[test]
fn an_umbrella_prompt_finds_the_umbrella_precedent() {
    let answer = handled(
        "Inventive problem: an umbrella must be big enough in rain yet small enough in a crowded bus.",
    )
    .expect("handled");
    assert!(
        answer.answer.contains("umbrella") && answer.answer.contains("crowded bus"),
        "the matched benchmark task must be cited: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("family_space_separation"),
        "the umbrella task's methods must be named: {}",
        answer.answer
    );
}

#[test]
fn the_triz_acronym_and_english_paradox_cues_trigger() {
    assert!(handled("Apply TRIZ to smartwatch battery vs thickness").is_some());
    assert!(handled("How do we resolve this paradox: transparency vs strength?").is_some());
}

#[test]
fn ordinary_prompts_decline() {
    assert!(handled("What is the capital of France?").is_none());
    assert!(handled("Напомни мне встречу в 20:00").is_none());
}

#[test]
fn the_handler_logs_its_cue_and_precedents() {
    let prompt = "Engineering contradiction: coat tiny pills evenly without waste";
    let mut log = EventLog::new();
    let answer = handle_triz(prompt, &normalize_prompt(prompt), &mut log).expect("handled");
    assert!(log.first_of("triz_solver:cued").is_some());
    assert!(
        log.first_of("triz_solver:precedent").is_some(),
        "a matched precedent must be logged"
    );
    assert!(
        answer.answer.contains("pill"),
        "the pill-coating task is the overlapping precedent: {}",
        answer.answer
    );
}
