//! Traceability checks for issue #659's scanner and self-hosted evidence.

use std::fs;
use std::path::Path;

#[test]
fn issue_659_evidence_and_ci_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };

    let case_study = read("docs/case-studies/issue-659/README.md");
    for expected in [
        "774",
        "548",
        "1,322",
        "Sorry, I can't do that.",
        "byte-identical",
        "awaiting_human_review",
        "different wording",
    ] {
        assert!(
            case_study.contains(expected),
            "case study missing {expected}"
        );
    }

    let report = read(
        "docs/case-studies/issue-659/agent-cli-evidence/hardcoded-language-learning-report.lino",
    );
    assert_eq!(
        report,
        formal_ai::agentic_coding::learning_report::hardcoded_language_learning::render_document(),
        "committed Agent CLI evidence must reproduce byte for byte"
    );
    for expected in [
        "issue \"659\"",
        "decision \"awaiting_human_review\"",
        "lesson:context-sensitive-detection",
        "lesson:two-way-ratchet",
        "lesson:seed-first-migration",
    ] {
        assert!(report.contains(expected), "report missing {expected}");
    }
    assert!(!report.contains("decision \"promoted\""));

    for transcript in [
        "docs/case-studies/issue-659/agent-cli-evidence/agent-stream.jsonl",
        "docs/case-studies/issue-659/agent-cli-evidence/opencode-stream.jsonl",
    ] {
        assert!(
            root.join(transcript)
                .metadata()
                .map_or(0, |meta| meta.len())
                > 0,
            "{transcript} must preserve a real external CLI transcript"
        );
    }

    let workflow = read(".github/workflows/release.yml");
    assert!(workflow.contains("experiments/agent_cli_e2e/run_issue_659_learning.sh"));
    let contributing = read("CONTRIBUTING.md");
    assert!(contributing.contains("rust-script scripts/check-hardcoded-language.rs"));
}
