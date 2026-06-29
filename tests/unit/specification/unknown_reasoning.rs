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
fn unresolved_bare_term_routes_to_web_search_after_local_sources_miss() {
    let cases = [
        ("English", "cursor"),
        ("Russian", "курсор"),
        ("Hindi", "कर्सर"),
        ("Chinese", "光标"),
    ];

    for (language, prompt) in cases {
        let response = answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "{language} bare unresolved term should search instead of returning unknown: {}",
            response.answer,
        );
        assert!(
            has_evidence(&response, &format!("web_search:request:{prompt}")),
            "{language} web search should use the term itself as the query: {:?}",
            response.evidence_links,
        );
        assert!(
            has_evidence(&response, "web_search:query_kind:unresolved_bare_term"),
            "{language} trace should explain why the fallback searched: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .links_notation
                .contains("reasoning:gather_result public_knowledge_cache:miss"),
            "{language} local/public cache misses should precede the search handoff: {}",
            response.links_notation,
        );
    }
}

#[test]
fn seed_backed_bare_terms_still_use_public_knowledge_cache() {
    let response = answer("Rust");

    assert_eq!(response.intent, "concept_lookup");
    assert!(
        has_evidence(&response, "concept_lookup:hit:"),
        "known concept terms should not be stolen by bare-term web search: {:?}",
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
fn russian_unknown_reasoning_hint_uses_russian_rule_commands() {
    let response = answer("снорфлакс тихая бирюзовая погода без правила");

    assert_eq!(response.intent, "unknown");
    assert!(
        response.answer.contains("Покажи правила поведения"),
        "Russian unknown-reasoning hint should list rules in Russian, got: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("Покажи правило unknown"),
        "Russian unknown-reasoning hint should inspect rules in Russian, got: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("Когда я скажу"),
        "Russian unknown-reasoning hint should teach dialog rules in Russian, got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("List behavior rules")
            && !response.answer.contains("Show behavior rule unknown")
            && !response.answer.contains("When I say"),
        "Russian unknown-reasoning hint should not switch to English command examples, got: {}",
        response.answer,
    );
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
