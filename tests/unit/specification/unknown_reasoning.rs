//! Unknown-prompt reasoning tests (issue #298).
//!
//! When no specialized route claims a prompt, the solver must still run the
//! universal unknown-handling loop: state what is known, identify the missing
//! piece, try reachable sources, and only then ask a minimal question or fall
//! back to the legacy unknown guide.

use formal_ai::{ConversationTurn, SolverConfig, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::new(SolverConfig {
        questioning_rigor: 0.8,
        ..Default::default()
    })
    .solve(prompt)
}

fn has_evidence(response: &SymbolicAnswer, prefix: &str) -> bool {
    response
        .evidence_links
        .iter()
        .any(|link| link.starts_with(prefix))
}

#[test]
fn unmatched_prompt_records_unknown_reasoning_trace() {
    let response = answer("Calibrate the snorflax against silent teal weather");

    assert!(
        has_evidence(&response, "reasoning:known:"),
        "unknown reasoning must record known context: {:?}",
        response.evidence_links,
    );
    assert!(
        has_evidence(&response, "reasoning:unknown:"),
        "unknown reasoning must record the missing piece: {:?}",
        response.evidence_links,
    );
    assert!(
        has_evidence(&response, "reasoning:candidate_source:"),
        "unknown reasoning must record candidate sources: {:?}",
        response.evidence_links,
    );
    assert!(
        has_evidence(&response, "reasoning:gather_attempt:"),
        "unknown reasoning must record gather attempts: {:?}",
        response.evidence_links,
    );
    assert!(response.links_notation.contains("reasoning:known"));
}

#[test]
fn unknown_reasoning_retries_public_knowledge_cache() {
    let response = answer("Use cached public knowledge about WebAssembly");

    assert_eq!(response.intent, "concept_lookup");
    assert!(response.answer.contains("WebAssembly"));
    assert!(
        has_evidence(&response, "reasoning:gather_result:"),
        "public-cache retry must be recorded: {:?}",
        response.evidence_links,
    );
    assert!(
        has_evidence(&response, "concept_lookup:hit:"),
        "public-cache retry should reuse concept lookup evidence: {:?}",
        response.evidence_links,
    );
}

#[test]
fn unknown_reasoning_uses_one_minimal_question_when_unreachable() {
    let response = answer("How should snorflax be calibrated for teal silence");

    assert_eq!(response.intent, "unknown");
    assert!(response.answer.contains("snorflax"));
    assert!(response.answer.contains("could not determine"));
    assert!(
        response.answer.matches('?').count() <= 1,
        "unknown reasoning should ask at most one question: {}",
        response.answer,
    );
}

#[test]
fn unknown_reasoning_records_trace_for_every_supported_language() {
    let cases = [
        ("English", "snorflax silent teal weather without rules"),
        ("Russian", "снорфлакс тихая бирюзовая погода без правила"),
        ("Hindi", "स्नोरफ्लैक्स शांत नीला मौसम बिना नियम"),
        ("Chinese", "斯诺弗拉克斯 安静 蓝绿色 天气 无规则"),
    ];

    for (language, prompt) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "unknown",
            "{language} prompt should stay on the unknown reasoning path"
        );
        assert!(
            has_evidence(&response, "reasoning:known:"),
            "{language} prompt should record known reasoning evidence: {:?}",
            response.evidence_links,
        );
        assert!(
            has_evidence(&response, "reasoning:unknown:"),
            "{language} prompt should record the missing unknown: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn unknown_reasoning_answers_from_link_memory() {
    let solver = UniversalSolver::new(SolverConfig {
        questioning_rigor: 0.8,
        ..Default::default()
    });
    let response = solver.solve_with_history(
        "What is the launch code?",
        &[ConversationTurn::user("The launch code is DELTA-7.")],
    );

    assert_eq!(response.intent, "memory_fact_lookup");
    assert!(response.answer.contains("DELTA-7"));
    assert!(
        has_evidence(&response, "cache_hit:link_memory"),
        "link-memory gather must be recorded as a cache hit: {:?}",
        response.evidence_links,
    );
}

#[test]
fn unknown_reasoning_records_last_resort_fallback() {
    let response = UniversalSolver::new(SolverConfig::default()).solve("");

    assert_eq!(response.intent, "unknown");
    assert!(response.answer.contains("Links Notation"));
    assert!(
        has_evidence(&response, "reasoning:gave_up:"),
        "legacy fallback must be explicitly recorded: {:?}",
        response.evidence_links,
    );
}
