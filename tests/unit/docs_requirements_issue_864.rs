//! Executable traceability for issue #864's complete cross-surface contract.

use std::fs;
use std::path::PathBuf;

const CASE_STUDY: &str = "docs/case-studies/issue-864/README.md";
const REQUIREMENTS: &str = "docs/case-studies/issue-864/requirements.md";
const RUST_REGRESSIONS: &str = "tests/unit/issue_864.rs";
const BROWSER_REGRESSION: &str = "tests/e2e/tests/issue-864.spec.js";
const GLOBAL_REQUIREMENTS: &str = "REQUIREMENTS.md";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn assert_contains_all(relative: &str, needles: &[&str]) {
    let content = read(relative);
    for needle in needles {
        assert!(
            content.contains(needle),
            "{relative} must contain {needle:?}"
        );
    }
}

#[test]
fn r864_01_every_detected_failure_surface_proactively_asks_for_consent() {
    assert_contains_all(
        RUST_REGRESSIONS,
        &[
            "detected_tool_failures_proactively_ask_to_report_on_agentic_harnesses",
            "unresolved_reasoning_proactively_asks_to_report_on_every_rust_surface_language",
            "plain_text_agent_cli_failures_use_the_same_proactive_invitation",
        ],
    );
    assert_contains_all(
        BROWSER_REGRESSION,
        &["detected provider failures proactively offer a contextual issue report"],
    );
}

#[test]
fn r864_02_detection_is_semantic_and_expected_stops_are_not_failures() {
    assert_contains_all(
        "src/agentic_coding/tool_result.rs",
        &[
            "ROLE_TOOL_RESULT_FAILURE_SIGNAL",
            "expected_stop_status",
            "awaiting_approval",
        ],
    );
    assert_contains_all(
        "src/web/app/detected-failure.js",
        &[
            "FAILURE_INTENTS",
            "structuredResultHasFailure",
            "EXPECTED_STOP_STATUSES",
        ],
    );
    assert_contains_all(
        RUST_REGRESSIONS,
        &[
            "expected_tool_refusals_do_not_claim_formal_ai_detected_a_failure",
            "explicit_unsuccessful_results_invite_reports_but_pending_results_do_not",
        ],
    );
}

#[test]
fn r864_03_invitation_language_and_failure_state_survive_every_ui_path() {
    let response_seed = read("data/seed/multilingual-responses-agentic.lino");
    assert_eq!(
        response_seed
            .matches("intent detected_failure_report_invitation")
            .count(),
        6
    );
    assert_contains_all(
        "data/seed/multilingual-responses-agentic.lino",
        &[
            "language en",
            "language ru",
            "language hi",
            "language zh",
            "language es",
            "language unknown",
        ],
    );
    let ui_catalog = read("src/web/i18n-catalog-messages.lino");
    assert_eq!(ui_catalog.matches("detectedFailureReport ").count(), 4);
    assert_contains_all(
        "src/web/app/main.jsx",
        &[
            "detectedFailure: event.detectedFailure === true",
            "detectedFailure = detectedFailure || answerHasDetectedFailure(answer)",
            "data-testid=\"detected-failure-report\"",
        ],
    );
}

#[test]
fn r864_04_the_offer_reuses_the_contextual_report_without_auto_filing() {
    assert_contains_all(
        BROWSER_REGRESSION,
        &[
            "## Environment",
            "## User Context",
            "## Reproduction of dialog",
            "## Reasoning Trace",
            "## Description",
            "## Attach full memory (optional)",
        ],
    );
    assert_contains_all(
        "experiments/agent_cli_e2e/run_issue_864.sh",
        &[
            "gh issue create",
            "filed an issue without user confirmation",
        ],
    );
}

#[test]
fn r864_05_real_browser_and_agent_cli_evidence_are_replayable_in_ci() {
    for screenshot in [
        "docs/case-studies/issue-864/before.png",
        "docs/case-studies/issue-864/after.png",
    ] {
        let bytes = fs::read(root().join(screenshot)).expect("read screenshot");
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }
    assert_contains_all(
        ".github/workflows/proactive-failure-report-e2e.yml",
        &["@link-assistant/agent", "run_issue_864.sh"],
    );
    assert_contains_all(
        "docs/case-studies/issue-864/failure-e2e/final-answer.txt",
        &["The command failed:", "Report issue"],
    );
}

#[test]
fn the_complete_issue_864_task_is_traceable() {
    assert_contains_all(
        REQUIREMENTS,
        &["R864-01", "R864-02", "R864-03", "R864-04", "R864-05"],
    );
    assert_contains_all(
        GLOBAL_REQUIREMENTS,
        &["Issue #864", "R864-1", "Proactive", "Agent CLI"],
    );
    assert_contains_all(
        CASE_STUDY,
        &[
            "Root cause",
            "Semantic boundary",
            "Before",
            "After",
            "ses_03b54b716ffe3E7D9TZMDg6Evs",
        ],
    );
}
