//! Traceability checks for issue #660's importer and self-hosted evidence.

use std::fs;
use std::path::Path;

#[test]
fn issue_660_evidence_and_ci_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };

    let case_study = read("docs/case-studies/issue-660/README.md");
    for expected in [
        "208",
        "832",
        "1000‰",
        "import_rejected",
        "aliases.hi[0].value",
        "byte-identical",
        "awaiting_human_review",
    ] {
        assert!(
            case_study.contains(expected),
            "case study missing {expected}"
        );
    }

    let report =
        read("docs/case-studies/issue-660/agent-cli-evidence/lexeme-import-learning-report.lino");
    assert_eq!(
        report,
        formal_ai::agentic_coding::learning_report::lexeme_import_learning::render_document(),
        "committed Agent CLI evidence must reproduce byte for byte"
    );
    assert!(report.contains("issue \"660\""));
    assert!(report.contains("decision \"awaiting_human_review\""));
    assert!(report.contains("lesson:truthful-surface-source"));
    assert!(report.contains("lesson:durable-fail-closed-import"));
    assert!(!report.contains("decision \"promoted\""));

    for transcript in [
        "docs/case-studies/issue-660/agent-cli-evidence/agent-stream.jsonl",
        "docs/case-studies/issue-660/agent-cli-evidence/opencode-stream.jsonl",
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
    assert!(workflow.contains("experiments/agent_cli_e2e/run_issue_660_learning.sh"));
}
