//! Issue #657's own auto-learning report, and the shape that made it cheap.

use formal_ai::agentic_coding::learning_report::self_hosting_learning;
use formal_ai::agentic_coding::{
    run_agentic_task, LearningReport, REPORTS, SELF_HOSTING_LEARNING_PATH,
    SELF_HOSTING_LEARNING_TASK,
};

#[test]
fn self_hosting_learning_is_derived_and_review_gated() {
    let baseline = include_str!("../../data/meta/issue-657-self-hosting-learning.lino");
    let changed = baseline.replace("accessCount \"9\"", "accessCount \"14\"");
    let first = self_hosting_learning::render_document_from(baseline);
    let second = self_hosting_learning::render_document_from(&changed);

    // The ranking is derived from the persisted network, not canned: perturbing
    // one observation's usage has to move the document.
    assert_ne!(first, second);
    assert!(first.contains("self_hosting_learning_report"));
    assert!(first.contains("issue \"657\""));
    assert!(first.contains("decision \"awaiting_human_review\""));
    assert!(first
        .contains("promotion_gate \"metric_fixture_exact_share_and_honest_ledger_ratchet_pass\""));
    assert!(first.contains("lesson:trailer-provenance"));
    assert!(first.contains("lesson:honest-baseline"));
    assert!(first.contains("lesson:changed-line-weighting"));
    assert!(first.contains("lesson:monotonic-window"));

    // A report that promoted itself would defeat the gate it names.
    assert!(!first.contains("decision \"promoted\""));
}

#[test]
fn formal_ai_executes_self_hosting_learning_through_agent_cli() {
    let outcome = run_agentic_task(SELF_HOSTING_LEARNING_TASK).expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], SELF_HOSTING_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        self_hosting_learning::render_document()
    );
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("human-review-gated report"));
}

/// Every report is reachable through the table, which is what makes the table
/// the routing decision rather than a list someone has to remember to update.
#[test]
fn every_registered_report_routes_from_its_own_task() {
    assert!(
        REPORTS.len() >= 5,
        "expected every auto-learning report to be registered, found {}",
        REPORTS.len()
    );
    for report in REPORTS {
        let routed = formal_ai::agentic_coding::learning_report::route(report.task)
            .unwrap_or_else(|| panic!("{} does not route from its own task", report.head));
        assert_eq!(
            routed.head, report.head,
            "{} routed to {} instead of itself",
            report.head, routed.head
        );
    }
}

/// The identities must be distinct. Two reports sharing a head or a path would
/// route to whichever came first in the table and silently shadow the other.
#[test]
fn report_identities_are_unique() {
    for (index, report) in REPORTS.iter().enumerate() {
        for other in &REPORTS[index + 1..] {
            assert_ne!(report.head, other.head, "duplicate report head");
            assert_ne!(report.path, other.path, "duplicate report path");
            assert_ne!(report.issue, other.issue, "duplicate report issue");
        }
    }
}

/// Every report names its own issue and nothing else's.
///
/// This is the regression the descriptor exists to prevent. The renderer used to
/// hardcode issue #686, and each other report re-derived its identity by
/// `replacen`-ing that line back out — a patch that fails *silently*, because a
/// `replacen` matching nothing returns the string unchanged. A report could
/// therefore claim to answer issue #686 while ranking a different network.
#[test]
fn no_report_carries_another_reports_issue() {
    for report in REPORTS {
        let document = report.render_document();
        let issues: Vec<&str> = document
            .lines()
            .filter_map(|line| line.trim().strip_prefix("issue "))
            .collect();
        assert_eq!(
            issues,
            vec![format!("\"{}\"", report.issue)],
            "{} must state exactly its own issue",
            report.head
        );
    }
}

/// The head, the gate and the decision travel together.
#[test]
fn a_gated_report_states_the_decision_its_gate_implies() {
    for report in REPORTS {
        let document = report.render_document();
        assert!(
            document.starts_with(&format!("{}\n", report.head)),
            "{} must lead with its own head",
            report.head
        );
        let gated = report.promotion_gate.is_some();
        assert_eq!(
            gated,
            document.contains("decision \"awaiting_human_review\""),
            "{}: a promotion gate and a review decision must appear together",
            report.head
        );
        if let Some(gate) = report.promotion_gate {
            assert!(
                document.contains(&format!("promotion_gate \"{gate}\"")),
                "{} must publish the gate it waits on",
                report.head
            );
        }
    }
}

/// All reports share one field order, because they share one renderer.
#[test]
fn reports_share_a_canonical_field_order() {
    for report in REPORTS {
        let document = report.render_document();
        let head_fields: Vec<&str> = document
            .lines()
            .skip(1)
            .take_while(|line| !line.trim_start().starts_with("learned_expression_"))
            .filter_map(|line| line.trim().split_whitespace().next())
            .collect();
        let expected: &[&str] = if report.promotion_gate.is_some() {
            &[
                "issue",
                "decision",
                "promotion_gate",
                "record_type",
                "substrate",
                "retention_formula",
                "expression_count",
                "validation_warning_count",
                "multi_hop_seed",
                "multi_hop_recall",
            ]
        } else {
            &[
                "issue",
                "record_type",
                "substrate",
                "retention_formula",
                "expression_count",
                "validation_warning_count",
                "multi_hop_seed",
                "multi_hop_recall",
            ]
        };
        assert_eq!(
            head_fields, expected,
            "{} drifted from the canonical field order",
            report.head
        );
    }
}

/// The descriptor is the only place a report's identity lives.
///
/// A new report is a row, so the renderer must not learn any report's name. If
/// this fails, someone has re-introduced the identity patch.
#[test]
fn the_renderer_does_not_name_any_report() {
    let renderer = include_str!("../../src/agentic_coding/learning_report.rs");
    let code: String = renderer
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for report in REPORTS {
        assert!(
            !code.contains(&format!("\"{}\"", report.head)),
            "the renderer must not hardcode {}'s head; identity belongs in its descriptor",
            report.head
        );
    }
    assert!(
        !code.contains("replacen"),
        "a report's identity must be rendered, not patched back in afterwards",
    );
}

/// The fixture proves the derivation runs over an arbitrary network, not just
/// the committed ones.
#[test]
fn a_report_ranks_any_persisted_network() {
    static FIXTURE: LearningReport = LearningReport {
        head: "fixture_learning_report",
        issue: "0",
        promotion_gate: Some("fixture_gate"),
        path: "fixture-learning-report.lino",
        task: "write fixture-learning-report.lino",
        memory: "demo_memory\n  event \"observation:only\"\n    kind \"message\"\n    role \
                 \"user\"\n    content \"single observation\"\n    accessCount \"2\"\n    \
                 writeCount \"1\"\n",
        subject: "fixture observations",
    };

    let document = FIXTURE.render_document();
    assert!(document.starts_with("fixture_learning_report\n  issue \"0\"\n"));
    assert!(document.contains("promotion_gate \"fixture_gate\""));
    assert!(document.contains("expression_count \"1\""));
    assert!(document.contains("observation:only"));
    assert!(FIXTURE
        .final_answer(&document)
        .contains("ranked 1 fixture observations"));
}
