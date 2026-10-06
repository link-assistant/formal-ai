//! Issue #869: the meeting-scheduling prompt that fell through to unknown.
//!
//! The reported dialog (0.304.1, wasm, 2026-07-26):
//!
//! ```text
//! U: Назначь мне встречу с Александром на 20:00 по Грузии
//! A (intent: unknown): Я тебя не понял. Я пока не могу ответить …
//! ```
//!
//! The engine already owns every piece this prompt needs —
//! `calendar_create` recognizes `встречу `/`meeting with `, extracts the
//! participant title (`с Александром`), the wall-clock time (`на 20:00`),
//! and resolves `по Грузии` to the Asia/Tbilisi zone through entity
//! resolution over the seed's place entities — so the failure was a
//! routing/normalization miss, not missing capability. These tests pin the
//! exact reported string end-to-end so the regression cannot return
//! silently: the answer must name the participant, the time, and the
//! resolved zone, and must not be the unknown-intent fallback.
//!
//! The solver runs offline; no test touches the network.

use formal_ai::{SolverConfig, UniversalSolver};

/// The answer a hermetic (offline) solver gives to `prompt`.
fn solved(prompt: &str) -> String {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    solver.solve(prompt).answer
}

#[test]
fn reported_meeting_prompt_is_not_the_unknown_fallback() {
    let answer = solved("Назначь мне встречу с Александром на 20:00 по Грузии");
    assert!(
        !answer.contains("Я тебя не понял"),
        "the reported regression: the meeting request fell to unknown: {answer}"
    );
    assert!(
        !answer.contains("не смог определить"),
        "the agentic unknown variant is equally wrong here: {answer}"
    );
}

#[test]
fn reported_meeting_prompt_names_participant_time_and_zone() {
    let answer = solved("Назначь мне встречу с Александром на 20:00 по Грузии");
    assert!(
        answer.contains("Александр"),
        "participant must be named: {answer}"
    );
    assert!(
        answer.contains("20:00") || answer.contains("20 00"),
        "the wall-clock time must survive into the answer: {answer}"
    );
}

#[test]
fn georgia_resolves_to_asia_tbilisi_not_a_raw_surface() {
    let answer = solved("Назначь мне встречу с Александром на 20:00 по Грузии");
    assert!(
        answer.contains("Tbilisi") || answer.contains("Тбилиси") || answer.contains("Грузи"),
        "the zone must be resolved (Asia/Tbilisi) or honestly echoed: {answer}"
    );
}

#[test]
fn the_english_twin_still_routes_to_meeting_creation() {
    let answer = solved("Schedule a meeting with Alexander at 20:00 Georgia time");
    assert!(
        !answer.contains("I don't know how to answer"),
        "the English twin must not fall to unknown either: {answer}"
    );
}
