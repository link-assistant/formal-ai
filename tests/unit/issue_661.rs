//! Issue #661 (R384): probability-weighted statement formalization with
//! contradiction warnings.
//!
//! Two acceptance behaviours are covered here:
//! 1. `weighted_formalization` — every formalized statement in a
//!    multi-interpretation prompt carries a `statement_weight` link, and those
//!    weights sum to 1 across the candidates. The weights live in the trace
//!    (evidence links), never in the plain reply.
//! 2. `requirement_contradiction` — "always answer in Russian" followed by
//!    "never answer in Russian" yields a `requirement_contradiction` event and
//!    a reply that names both statements and proposes a resolution, in the
//!    prompt's language.

use formal_ai::{ConversationTurn, SolverConfig, UniversalSolver};

/// Collect the numeric weights carried by `statement_weight:` evidence links.
fn statement_weights(links: &[String]) -> Vec<f32> {
    links
        .iter()
        .filter_map(|link| link.strip_prefix("statement_weight:"))
        .filter_map(|payload| {
            payload.split_whitespace().find_map(|token| {
                token
                    .strip_prefix("weight=")
                    .and_then(|value| value.parse::<f32>().ok())
            })
        })
        .collect()
}

#[test]
fn weighted_formalization_attaches_statement_weight_links_summing_to_one() {
    // "apple is a fruit" is copula-ambiguous (P31 instance-of vs P279
    // subclass-of), so formalization exposes competing interpretations.
    let response = UniversalSolver::default().solve("apple is a fruit");

    let weights = statement_weights(&response.evidence_links);
    assert!(
        weights.len() >= 2,
        "a multi-interpretation prompt must carry a statement_weight link per candidate, \
         got weights {weights:?} from links {:?}",
        response.evidence_links,
    );

    let total: f32 = weights.iter().sum();
    assert!(
        (total - 1.0).abs() < 0.001,
        "statement weights must sum to one across candidates, got {weights:?} (sum {total})",
    );

    // Weights are diagnostics: they belong in the trace, not the plain reply.
    assert!(
        !response.answer.contains("statement_weight"),
        "statement weights must stay in the trace, not the plain reply: {}",
        response.answer,
    );
}

#[test]
fn weighted_formalization_stays_out_of_reply_when_diagnostics_default_off() {
    // Diagnostics are default-off; the weighted-statement machinery must not
    // leak the weight tokens into the user-facing thinking narrative either.
    let response = UniversalSolver::default().solve("apple is a fruit");
    assert!(
        !SolverConfig::default().diagnostic_mode,
        "diagnostics must default to off",
    );
    assert!(
        response
            .thinking_steps
            .iter()
            .all(|step| !step.detail.contains("weight=")),
        "statement weights must not appear in the default thinking narrative: {:?}",
        response.thinking_steps,
    );
}

#[test]
fn english_requirement_contradiction_warns_with_both_statements_and_a_resolution() {
    let solver = UniversalSolver::default();

    let first_prompt = "always answer in Russian";
    let first = solver.solve(first_prompt);

    let history = [
        ConversationTurn::user(first_prompt),
        ConversationTurn::assistant(first.answer),
    ];
    let response = solver.solve_with_history("never answer in Russian", &history);

    assert_eq!(
        response.intent, "requirement_contradiction",
        "the second, opposing directive must be flagged as a contradiction, got {}: {}",
        response.intent, response.answer,
    );

    // A `requirement_contradiction` event is recorded in the trace.
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("requirement_contradiction:")),
        "the clash must be recorded as a requirement_contradiction link, got {:?}",
        response.evidence_links,
    );

    // The reply names both statements verbatim.
    assert!(
        response.answer.contains("always answer in Russian"),
        "reply must quote the first requirement: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("never answer in Russian"),
        "reply must quote the second requirement: {}",
        response.answer,
    );

    // The reply proposes a concrete resolution reusing the retraction protocol.
    assert!(
        response.answer.contains("retract"),
        "reply must propose a resolution (retract one requirement): {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "policy:add_only_history"),
        "the resolution must reuse the append-only retraction protocol, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn requirement_contradiction_reply_is_in_the_prompt_language() {
    let solver = UniversalSolver::default();

    // Russian-language directives: "always answer in Russian" / "never answer
    // in Russian" phrased in Russian must produce a Russian warning.
    let first_prompt = "всегда отвечай на русском";
    let first = solver.solve(first_prompt);
    let history = [
        ConversationTurn::user(first_prompt),
        ConversationTurn::assistant(first.answer),
    ];
    let response = solver.solve_with_history("никогда не отвечай на русском", &history);

    assert_eq!(
        response.intent, "requirement_contradiction",
        "Russian directives must also be flagged, got {}: {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("Предупреждение"),
        "the warning must be rendered in the prompt's language (Russian): {}",
        response.answer,
    );
    assert!(
        response.answer.contains("всегда отвечай на русском")
            && response.answer.contains("никогда не отвечай на русском"),
        "the Russian reply must quote both directives: {}",
        response.answer,
    );
}

#[test]
fn requirement_contradiction_warning_has_seed_backed_hindi_and_chinese_variants() {
    let cases = [
        ("हमेशा हिंदी में उत्तर दें", "कभी नहीं हिंदी में उत्तर दें", "चेतावनी"),
        ("始终用中文回答", "永远不要用中文回答", "警告"),
    ];

    for (first_prompt, second_prompt, warning_marker) in cases {
        let solver = UniversalSolver::default();
        let first = solver.solve(first_prompt);
        let history = [
            ConversationTurn::user(first_prompt),
            ConversationTurn::assistant(first.answer),
        ];
        let response = solver.solve_with_history(second_prompt, &history);

        assert_eq!(
            response.intent, "requirement_contradiction",
            "opposing directives must be detected in every supported script: {}",
            response.answer,
        );
        assert!(
            response.answer.contains(warning_marker),
            "warning must use the prompt language: {}",
            response.answer,
        );
        assert!(
            response.answer.contains(first_prompt) && response.answer.contains(second_prompt),
            "warning must preserve both source statements: {}",
            response.answer,
        );
        assert!(
            response.answer.matches("0.500000").count() == 2,
            "the localized template must expose both calculated weights: {}",
            response.answer,
        );
    }
}
