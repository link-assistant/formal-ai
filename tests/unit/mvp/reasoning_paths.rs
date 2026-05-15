//! Reasoning-path tests (R85–R88).
//!
//! These tests pin down the universal solver's new specialized handlers and
//! prove that each interface (library and the convenience module-level
//! entry points) routes through the same loop, without any hardcoded
//! demo-style responses. Every test exercises the event-log projection so a
//! regression to memoized answers would break here first.

use formal_ai::{
    solve, solve_with_history, ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver,
};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// R85: arithmetic — symbols, words, parentheses, errors.
// ---------------------------------------------------------------------------

#[test]
fn arithmetic_handles_basic_addition() {
    let response = answer("What is 2 + 2?");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains('4'));
    assert!((response.confidence - 1.0).abs() < f32::EPSILON);
}

#[test]
fn arithmetic_handles_parentheses_and_precedence() {
    let response = answer("Calculate 7 * (3 + 4)");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("49"));
}

#[test]
fn arithmetic_handles_word_operators() {
    let response = answer("What is 10 plus 20 times 3?");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("70"));
}

#[test]
fn arithmetic_handles_division_remainder() {
    let response = answer("Compute 100 - 25 % 7");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("96"));
}

#[test]
fn arithmetic_handles_decimals() {
    let response = answer("How much is 1.5 + 2.5?");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains('4'));
}

#[test]
fn arithmetic_reports_division_by_zero_without_panicking() {
    let response = answer("What is 5 / 0?");
    assert_eq!(response.intent, "calculation_error");
    assert!(response.answer.to_lowercase().contains("division by zero"));
    assert!(response.confidence < 1.0);
}

#[test]
fn arithmetic_records_calculation_event_in_evidence_log() {
    let response = answer("What is 6 * 7?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("calculation")),
        "evidence links should include the calculation event so the answer is a \
         projection of the log, not a memoized constant: {:?}",
        response.evidence_links,
    );
}

#[test]
fn arithmetic_never_fires_on_plain_greetings() {
    let response = answer("Hi");
    assert_eq!(response.intent, "greeting");
}

// ---------------------------------------------------------------------------
// R86: concept lookup against the offline seed.
// ---------------------------------------------------------------------------

#[test]
fn concept_lookup_answers_what_is_wikipedia() {
    let response = answer("What is Wikipedia?");
    assert_eq!(response.intent, "concept_lookup");
    assert!(response.answer.to_lowercase().contains("encyclopedia"));
    assert!(response
        .answer
        .contains("https://en.wikipedia.org/wiki/Wikipedia"));
}

#[test]
fn concept_lookup_handles_tell_me_about_links_notation() {
    let response = answer("Tell me about Links Notation");
    assert_eq!(response.intent, "concept_lookup");
    assert!(response.answer.contains("Links Notation"));
    assert!(response.answer.to_lowercase().contains("indentation"));
}

#[test]
fn concept_lookup_handles_what_does_x_mean() {
    let response = answer("What does Wikidata mean?");
    assert_eq!(response.intent, "concept_lookup");
    assert!(response.answer.contains("Wikidata"));
}

#[test]
fn concept_lookup_includes_source_citation() {
    let response = answer("What is WebAssembly?");
    assert!(response.answer.contains("Source:"));
}

#[test]
fn concept_lookup_does_not_fire_for_identity_questions() {
    let response = answer("What is formal-ai?");
    assert_eq!(
        response.intent, "identity",
        "identity rule must win over concept lookup for formal-ai questions"
    );
}

#[test]
fn concept_lookup_records_concept_event_in_evidence_log() {
    let response = answer("What is Rust?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("concept_lookup")),
        "evidence links should include the concept_lookup event: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// R87: multi-turn conversation memory via solve_with_history.
// ---------------------------------------------------------------------------

#[test]
fn solve_with_history_recalls_name_across_turns() {
    let history = [ConversationTurn::user("Hi, my name is Ada Lovelace.")];
    let response = solve_with_history("What is my name?", &history);
    assert_eq!(response.intent, "recall_name");
    assert!(response.answer.contains("Ada"));
}

#[test]
fn solve_with_history_recalls_last_question() {
    let history = [
        ConversationTurn::user("What is 2 + 2?"),
        ConversationTurn::assistant("2 + 2 = 4"),
    ];
    let response = solve_with_history("What was my previous question?", &history);
    assert_eq!(response.intent, "recall_last_question");
    assert!(response.answer.contains("2 + 2"));
}

#[test]
fn solve_with_history_summarizes_conversation() {
    let history = [
        ConversationTurn::user("Hi"),
        ConversationTurn::assistant("Hi, how may I help you?"),
        ConversationTurn::user("What is 2 + 2?"),
        ConversationTurn::assistant("2 + 2 = 4"),
    ];
    let response = solve_with_history("Summarize the conversation so far.", &history);
    assert_eq!(response.intent, "summarize_conversation");
    assert!(response.answer.contains("Hi"));
    assert!(response.answer.contains("2 + 2"));
}

#[test]
fn solve_with_history_falls_through_for_unrelated_prompts() {
    let history = [ConversationTurn::user("My name is Ada.")];
    let response = solve_with_history("Hi", &history);
    assert_eq!(response.intent, "greeting");
}

#[test]
fn solve_without_history_matches_legacy_entry_point() {
    let a = solve("Hi");
    let b = FormalAiEngine.answer("Hi");
    assert_eq!(a, b);
}

#[test]
fn prior_turns_appear_in_evidence_log() {
    let history = [ConversationTurn::user("My name is Ada.")];
    let response = UniversalSolver::default().solve_with_history("Hi", &history);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("prior_turn:user")),
        "prior turns must be recorded as events so memory recall is a projection \
         of the append-only log: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// R88: JavaScript execution — explicit declaration, no silent failure.
// ---------------------------------------------------------------------------

#[test]
fn javascript_request_returns_explicit_unavailability() {
    let prompt = "Please run this javascript:\n```js\nconsole.log(1 + 2);\n```";
    let response = answer(prompt);
    assert_eq!(response.intent, "javascript_execution_unavailable");
    assert!(response
        .answer
        .to_lowercase()
        .contains("do not embed a javascript"));
    assert!(response.answer.contains("console.log(1 + 2);"));
}

#[test]
fn javascript_request_records_execution_status_event() {
    let prompt = "Please execute this javascript:\n```js\nlet x = 5;\n```";
    let response = answer(prompt);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("execution_status")),
        "the JS handler must emit an execution_status event so refusal is \
         auditable: {:?}",
        response.evidence_links,
    );
}

#[test]
fn javascript_handler_does_not_intercept_unrelated_code_blocks() {
    // No "run this" cue, so the handler must not steal the prompt from the
    // generic algorithm/code-fence flow.
    let prompt = "Here is some javascript:\n```js\nconsole.log(1);\n```";
    let response = answer(prompt);
    assert_ne!(response.intent, "javascript_execution_unavailable");
}

// ---------------------------------------------------------------------------
// Cross-handler sanity: every reasoning path projects from a non-empty event
// log, so the answer is never memoized.
// ---------------------------------------------------------------------------

#[test]
fn every_specialized_handler_emits_a_trace_link() {
    let prompts = [
        "Hi",
        "What is 2 + 2?",
        "What is Wikipedia?",
        "Please run this javascript:\n```js\n1+1;\n```",
        "Write me hello world program in Rust",
    ];
    for prompt in prompts {
        let response = answer(prompt);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("trace:")),
            "prompt {prompt:?} must emit a trace link: {:?}",
            response.evidence_links,
        );
    }
}
