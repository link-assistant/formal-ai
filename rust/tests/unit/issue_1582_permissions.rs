//! Shared positive and negative permission requests, with networking disabled.
use formal_ai::{SolverConfig, UniversalSolver};

#[test]
fn permission_guidance_owns_direct_requests_in_every_supported_language() {
    let fixtures: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../data/parity/issue-1582-permissions.json"
        ))
        .expect("shared fixture"),
    )
    .expect("fixture JSON");
    // Frozen exact examples are shared with the serving-worker test, not read from the seed.
    let examples: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/parity/issue-1582-permission-answers.json"
    ))
    .expect("exact answer examples");
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    for row in fixtures.as_array().expect("fixture rows") {
        let prompt = row["prompt"].as_str().expect("prompt");
        let answer = solver.solve(prompt);
        if row["guidance"] == true {
            let language = row["language"].as_str().expect("language");
            assert_eq!(answer.answer, examples[language].as_str().expect("example"));
            assert_eq!(answer.intent, "capabilities", "{prompt}: {}", answer.answer);
            assert!(
                answer.answer.contains("/permissions"),
                "{prompt}: {}",
                answer.answer
            );
            assert!(answer.answer.contains("FORMAL_AI_OFFLINE"));
        } else {
            assert!(
                !answer.answer.contains("/permissions"),
                "{prompt}: {}",
                answer.answer
            );
        }
    }
}
