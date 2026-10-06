//! Issue #1186 (E150): formalization as a user-facing task.
//!
//! "Formalize in first-order logic: Every student who studies passes the
//! exam." must reach the formalization handler — never the canned
//! web-search paragraph — and render a quantified clause into every
//! target grammar the seed declares (FOL, Lean 4, Rocq, Links Notation).
//! Deformalization runs the grammar in reverse, and the round trip is
//! checked structurally: the deformalized sentence is re-formalized and
//! the re-rendered FOL must be identical.
//!
//! The tests pin the contract from two angles, mirroring the issue #1177
//! suite: handler level (each direction directly) and engine level (the
//! cued request through `UniversalSolver`, which runs once the dispatch
//! chain wires the handler). No test touches the network and no test
//! invokes a theorem prover.

use formal_ai::event_log::EventLog;
use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::{SolverConfig, UniversalSolver, handle_formalization_request};

/// The answer a hermetic (offline) solver gives to `prompt`.
fn solved(prompt: &str) -> String {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    solver.solve(prompt).answer
}

/// The handler's answer body for `prompt`.
fn handler_answer(prompt: &str) -> String {
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    handle_formalization_request(prompt, &normalized, &mut log)
        .unwrap_or_else(|| panic!("the formalization handler should answer: {prompt}"))
        .answer
}

/// The fenced block with the given tag, extracted from an answer body.
fn fenced(answer: &str, tag: &str) -> String {
    let open = ["```", tag, "\n"].concat();
    let start = answer
        .find(open.as_str())
        .unwrap_or_else(|| panic!("answer should carry a `{tag}` fence: {answer}"))
        + open.len();
    let end = start
        + answer[start..]
            .find("```")
            .unwrap_or_else(|| panic!("the `{tag}` fence should be closed: {answer}"));
    answer[start..end].to_owned()
}

// ---------------------------------------------------------------------------
// Formalization: natural sentence → FOL/Lean/Rocq/Links Notation.
// ---------------------------------------------------------------------------

#[test]
fn handler_formalizes_the_issue_probe_sentence() {
    let answer = handler_answer(
        "Formalize in first-order logic: Every student who studies passes the exam.",
    );
    let fol = fenced(&answer, "fol");
    assert!(
        fol.starts_with("∀x (Student(x) ∧ Studies(x) → Passes(x"),
        "the FOL shape must match ∀x (P(x) ∧ Q(x) → R(x)): {answer}"
    );
    let lean = fenced(&answer, "lean");
    assert!(
        lean.contains("∀ (x : U)"),
        "Lean renders a binder: {answer}"
    );
    assert!(lean.contains("Student x"), "{answer}");
    let rocq = fenced(&answer, "rocq");
    assert!(
        rocq.contains("forall (x : U)"),
        "Rocq renders a binder: {answer}"
    );
    let lino = fenced(&answer, "lino");
    assert!(lino.contains("formal_clause"), "{answer}");
    assert!(lino.contains("predicate Student"), "{answer}");
    assert!(
        answer.contains("No theorem prover was invoked"),
        "the honesty marker must state what was not run: {answer}"
    );
    assert!(
        answer.contains("Derivation:"),
        "the white-box derivation is part of the answer: {answer}"
    );
}

#[test]
fn handler_formalizes_objectless_conditional_exactly() {
    let answer = handler_answer("Formalize in FOL: Every student who studies passes");
    let fol = fenced(&answer, "fol");
    assert_eq!(
        fol, "∀x (Student(x) ∧ Studies(x) → Passes(x))",
        "without an object the FOL is exactly the issue's target shape: {answer}"
    );
}

#[test]
fn handler_formalizes_russian_conditional_in_russian() {
    let answer = handler_answer(
        "Формализуй в логике первого порядка: Каждый студент, который учится, сдаёт экзамен",
    );
    let fol = fenced(&answer, "fol");
    assert!(
        fol.starts_with("∀x (студент(x) ∧ учится(x) → сдаёт(x"),
        "Cyrillic predicates pass through capitalization unchanged: {answer}"
    );
    assert!(
        answer.contains("Формализованное утверждение:"),
        "the response prose is localized: {answer}"
    );
}

#[test]
fn handler_formalizes_hindi_and_chinese_conditionals() {
    let hindi = handler_answer("औपचारिक बनाओ: हर छात्र जो पढ़ता है, परीक्षा पास करता है");
    assert!(
        fenced(&hindi, "fol").starts_with("∀x (छात्र(x) ∧ पढ़ता(x) → "),
        "{hindi}"
    );
    let chinese = handler_answer("形式化：每个学习的学生都通过考试");
    assert!(
        fenced(&chinese, "fol").starts_with("∀x (学生(x) ∧ 学习(x) → "),
        "{chinese}"
    );
}

#[test]
fn handler_formalizes_existential_as_conjunction() {
    let answer = handler_answer("Formalize in first-order logic: Some bird that sings flies");
    assert_eq!(
        fenced(&answer, "fol"),
        "∃x (Bird(x) ∧ Sings(x) ∧ Flies(x))",
        "existential readings are conjunctive: {answer}"
    );
}

#[test]
fn handler_formalizes_negative_quantifier() {
    let answer = handler_answer("Formalize in FOL: No cat that sleeps hunts");
    assert_eq!(
        fenced(&answer, "fol"),
        "¬∃x (Cat(x) ∧ Sleeps(x) ∧ Hunts(x))",
        "negative readings are conjunctive under ¬∃: {answer}"
    );
}

#[test]
fn handler_renders_every_target_with_the_named_one_first() {
    let answer = handler_answer("Formalize in Lean: Every student who studies passes");
    assert!(
        answer.contains("```lean\n"),
        "the named target is rendered: {answer}"
    );
    let lean_at = answer.find("```lean").expect("lean fence");
    let fol_at = answer.find("```fol").expect("fol fence");
    assert!(
        lean_at < fol_at,
        "the named target renders before the others: {answer}"
    );
}

// ---------------------------------------------------------------------------
// Deformalization and the structural round trip.
// ---------------------------------------------------------------------------

#[test]
fn deformalization_round_trip_preserves_structure() {
    let fol = "∀x (Student(x) ∧ Studies(x) → Passes(x, exam))";
    let back = handler_answer(&format!("Deformalize in plain English: {fol}"));
    assert!(
        back.contains("every student that studies passes the exam"),
        "the natural reading uses the seeded en template: {back}"
    );
    assert!(
        back.contains("structure preserved"),
        "the round-trip verdict is stated: {back}"
    );
    // The structural check, end to end: re-formalizing the rendered
    // sentence must reproduce the original FOL exactly.
    let again = handler_answer(
        "Formalize in first-order logic: every student that studies passes the exam",
    );
    assert_eq!(
        fenced(&again, "fol"),
        fol,
        "re-formalization reproduces the clause structure: {again}"
    );
}

#[test]
fn deformalization_localizes_the_wrapper_prose() {
    let answer = handler_answer("Деформализуй: ∀x (Student(x) ∧ Studies(x) → Passes(x))");
    assert!(
        answer.contains("Прочтение на естественном языке"),
        "the prose wrapper is Russian: {answer}"
    );
    assert!(
        answer.contains("каждый") && answer.contains("который"),
        "the seeded Russian quantifier and relative marker render: {answer}"
    );
    // Predicate symbols are carried in their formal surface — translating
    // predicate vocabulary is the translation pipeline's task, honestly
    // outside this handler's grammar.
    assert!(answer.contains("studies"), "{answer}");
}

// ---------------------------------------------------------------------------
// Honesty and routing.
// ---------------------------------------------------------------------------

#[test]
fn unparseable_sentence_gets_an_honest_refusal() {
    let answer = handler_answer("Formalize in first-order logic: hello world");
    assert!(
        answer.contains("quantified clause"),
        "the reason names what could not be parsed: {answer}"
    );
    assert!(
        !answer.contains("Web search requested"),
        "the canned search paragraph must never answer: {answer}"
    );
}

#[test]
fn unrelated_prompts_are_not_claimed() {
    for prompt in [
        "Hello, how are you today?",
        "What is the capital of France?",
        "Write a regular expression that matches five digits",
    ] {
        let normalized = normalize_prompt(prompt);
        let mut log = EventLog::new();
        assert!(
            handle_formalization_request(prompt, &normalized, &mut log).is_none(),
            "formalization must not claim: {prompt}"
        );
    }
}

// ---------------------------------------------------------------------------
// Engine level: the cued request is answered by the solver (this runs once
// the dispatch chain wires the handler before the search fallback).
// ---------------------------------------------------------------------------

#[test]
fn engine_answers_formalization_request() {
    let answer =
        solved("Formalize in first-order logic: Every student who studies passes the exam.");
    assert!(answer.contains("∀x"), "the FOL binder renders: {answer}");
    assert!(
        !answer.contains("Web search requested"),
        "never the canned search paragraph: {answer}"
    );
}
