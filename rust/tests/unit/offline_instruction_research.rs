//! Offline research keeps its cache boundary and nominal unknown responses.
use formal_ai::{SolverConfig, UniversalSolver};

#[test]
fn seeded_action_requests_check_offline_sources_in_both_word_orders() {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..Default::default()
    });
    for prompt in [
        "Calibrate the snorflax against silent teal weather",
        "Откалибруй снорфлакс для тихой бирюзовой погоды",
        "स्नोरफ्लैक्स कैलिब्रेट करें",
        "校准斯诺弗拉克斯",
        "Calibra el snorflax para el tiempo turquesa silencioso",
    ] {
        let answer = solver.solve(prompt);
        assert_eq!(answer.intent, "web_search", "{prompt}: {}", answer.answer);
        assert!(
            answer
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:unknown_reasoning_fallback"),
            "{prompt}: {:?}",
            answer.evidence_links
        );
        assert!(
            !answer
                .evidence_links
                .iter()
                .any(|link| link.starts_with("web_search:executed:")),
            "offline cache miss cannot claim captured sources: {prompt}"
        );
    }
}

#[test]
fn action_word_substrings_and_nominal_phrases_keep_the_unknown_guide() {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..Default::default()
    });
    for prompt in [
        "calibrated snorflax silent teal weather",
        "recalibrate snorflax silent teal weather",
        "snorflax silent teal weather without rules",
    ] {
        let answer = solver.solve(prompt);
        assert_eq!(answer.intent, "unknown", "{prompt}: {}", answer.answer);
        assert!(
            !answer
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:unknown_reasoning_fallback")
        );
    }
}
