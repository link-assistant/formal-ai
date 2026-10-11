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
//! string is a complaint about the assistant's own surface, which the
//! issue-447 rows of `data/seed/capability-routing.lino` route to the
//! structured report (`report_issue`) -- the same `ui_complaint` class
//! `issue_1138_frontier_classes` pins in five languages. A chat surface that
//! does not advertise the report tool answers with the seeded honest gap
//! naming it, never with the generic unknown-intent pool that explained the
//! complaint away. The layout half (scrollable sidebar) is a CSS change in
//! the web surface; its fix plan is recorded in
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
fn interface_complaint_routes_to_the_structured_report() {
    let answer = solved("интерфейс ужасен.");
    assert!(
        answer.contains("`report_issue`"),
        "the complaint must reach the report route, not the unknown-intent pool: {answer}"
    );
    let unknown_openers = [
        "Я пока не знаю, как ответить на это.",
        "Мне не удалось тебя понять.",
        "Я не уверен, как на это ответить.",
        "Я ещё не научился отвечать на это.",
        "Это для меня новое.",
    ];
    assert!(
        !unknown_openers.iter().any(|opener| answer.contains(opener)),
        "a complaint about the assistant's own surface is recorded, not explained away: {answer}"
    );
}

#[test]
fn the_seeded_ru_pool_is_live_registry_data() {
    // The registry must still declare ru among the agent's answer languages:
    // the reported dialog was in Russian.
    assert!(
        supported_languages().iter().any(|lang| lang == "ru"),
        "ru must remain a supported answer language"
    );
}
