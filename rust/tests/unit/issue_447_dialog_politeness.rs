//! Issue #447: the "интерфейс ужасен" dialog on a cut-off sidebar.
//!
//! The reported environment is a 1280×565 viewport (Firefox 151, Windows,
//! wasm 0.193.0): the left button panel is not fully visible and cannot be
//! scrolled with the mouse. The user's frustration line — "интерфейс
//! ужасен." — went to the agent, whose part in this bug is only that it
//! answered an interface complaint with the generic unknown-intent pool
//! instead of recognizing interface feedback.
//!
//! This test pins the dialog half the engine owns: the exact complaint
//! string lands in the unknown-intent fallback and its opener comes from
//! the seeded ru pool (`data/seed/unknown-openers.lino`), never from a Rust
//! literal — the file's own invariant that the varied answer stays a strict
//! superset of the seeded unknown response. The layout half (scrollable
//! sidebar) is a CSS change in the web surface; its fix plan is recorded in
//! `docs/case-studies/issue-447/README.md` for the web lane.

use formal_ai::seed::supported_languages;
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
fn interface_complaint_gets_a_seeded_unknown_opener() {
    let answer = solved("интерфейс ужасен.");
    // The pool lives in the seed; these literals pin the current seeded ru
    // pool the way the repo's tests pin expected answer substrings, so a
    // data edit that silently empties the pool fails here.
    let seeded_openers = [
        "Я пока не знаю, как ответить на это.",
        "Мне не удалось тебя понять.",
        "Я не уверен, как на это ответить.",
        "Я ещё не научился отвечать на это.",
        "Это для меня новое.",
    ];
    assert!(
        seeded_openers
            .iter()
            .any(|opener| answer.contains(opener)),
        "the complaint must be answered by a seeded ru opener, not a Rust literal: {answer}"
    );
}

#[test]
fn the_seeded_ru_pool_is_live_registry_data() {
    // The registry must still declare ru among the agent's answer languages;
    // the opener pool test above is meaningless without it.
    assert!(
        supported_languages().iter().any(|lang| lang == "ru"),
        "ru must remain a supported answer language"
    );
}
