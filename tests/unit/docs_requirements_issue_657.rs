use std::fs;
use std::path::Path;

/// Pin the issue #657 evidence that a passing unit test cannot produce.
///
/// The auto-learning report claims an external Agent CLI can derive it, and the
/// case study claims two harnesses agree on it. Those are claims about a real
/// run over the wire, so the committed run is what backs them — an in-process
/// test asserting its own harness works would be the circularity issue #657
/// exists to rule out.
#[test]
fn issue_657_self_hosting_evidence_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };

    let case_study = read("docs/case-studies/issue-657/README.md");
    for expected in [
        "0.00%",
        "awaiting_human_review",
        "metric_fixture_exact_share_and_honest_ledger_ratchet_pass",
        "their model provider",
        "byte-identical",
        // Short enough to survive markdown line-wrapping.
        "does not promote",
    ] {
        assert!(
            case_study.contains(expected),
            "case study missing {expected}"
        );
    }

    // The report the two external harnesses actually derived.
    let report =
        read("docs/case-studies/issue-657/agent-cli-learning/self-hosting-learning-report.lino");
    assert!(report.starts_with("self_hosting_learning_report\n"));
    for expected in [
        "issue \"657\"",
        "decision \"awaiting_human_review\"",
        "promotion_gate \"metric_fixture_exact_share_and_honest_ledger_ratchet_pass\"",
        "retention_formula \"reads + writes + incoming_links + outgoing_links\"",
        "lesson:trailer-provenance",
        "lesson:honest-baseline",
        "lesson:changed-line-weighting",
        "lesson:monotonic-window",
    ] {
        assert!(
            report.contains(expected),
            "committed report missing {expected}"
        );
    }
    assert!(
        !report.contains("decision \"promoted\""),
        "the committed report promoted itself past the gate it names"
    );

    // The committed evidence must be the report the code renders today, or the
    // case study is describing a run that no longer reproduces.
    assert_eq!(
        report,
        formal_ai::agentic_coding::learning_report::self_hosting_learning::render_document(),
        "the committed Agent CLI evidence has drifted from the rendered report; \
         re-run experiments/agent_cli_e2e/run_issue_657_metric.sh"
    );

    // Both harnesses' transcripts are kept: parity is only checkable if both sides survive.
    for stream in [
        "docs/case-studies/issue-657/agent-cli-learning/agent-stream.jsonl",
        "docs/case-studies/issue-657/agent-cli-learning/opencode-stream.jsonl",
    ] {
        assert!(
            root.join(stream).metadata().map_or(0, |meta| meta.len()) > 0,
            "{stream} must preserve the harness transcript"
        );
    }

    // The E2E is wired into CI; an unrun E2E gate is not a gate.
    let workflow = read(".github/workflows/release.yml");
    assert!(
        workflow.contains("experiments/agent_cli_e2e/run_issue_657_metric.sh"),
        "the issue #657 Agent CLI E2E must run in CI"
    );

    let ledger = read("data/meta/self-hosting-ledger.lino");
    assert!(ledger.contains("attribution_policy \"commit_trailers\""));
    assert!(ledger.contains("percentage_basis_points \"0\""));
}
