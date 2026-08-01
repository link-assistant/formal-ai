//! Documentation contracts for the issue #710 closure audit.

const CASE_STUDY: &str = include_str!("../../docs/case-studies/issue-710/README.md");
const REQUIREMENTS: &str = include_str!("../../REQUIREMENTS.md");
const ROADMAP: &str = include_str!("../../ROADMAP.md");
const AGENT_AUTHORED_VERDICT: &str = include_str!(
    "../../docs/case-studies/issue-710/agent-cli-evidence/verdict-contract/agent-authored-verdict-definition.md"
);
const AGENT_STREAM: &str = include_str!(
    "../../docs/case-studies/issue-710/agent-cli-evidence/verdict-contract/agent-stream.raw.log"
);
const AGENT_SESSION: &str = "ses_04171e114ffeADit1lxRlTHs5A";

const VERDICTS: [&str; 4] = [
    "`works-now`",
    "`still-broken`",
    "`superseded`",
    "`blocked-upstream`",
];

#[test]
fn every_issue_710_checklist_row_has_one_allowed_verdict_and_evidence() {
    let audit = CASE_STUDY
        .split("## 2026-08-01 re-verification")
        .nth(1)
        .expect("case study should contain the dated re-verification")
        .split("Totals:")
        .next()
        .expect("case study should contain audit totals");
    let rows = audit
        .lines()
        .filter(|line| {
            let first_cell = line
                .trim_start_matches('|')
                .split('|')
                .next()
                .unwrap_or_default()
                .trim();
            first_cell.parse::<usize>().is_ok()
        })
        .collect::<Vec<_>>();

    assert_eq!(rows.len(), 32, "the issue checklist has exactly 32 rows");
    for (index, row) in rows.iter().enumerate() {
        assert!(
            row.starts_with(&format!("| {} |", index + 1)),
            "audit rows must remain in source order: {row}"
        );
        assert_eq!(
            VERDICTS
                .iter()
                .filter(|verdict| row.contains(**verdict))
                .count(),
            1,
            "row must carry exactly one allowed verdict: {row}"
        );
        assert!(
            row.contains("](") || row.contains("[`"),
            "row must link its evidence or focused owner: {row}"
        );
    }
}

#[test]
fn no_conversational_gap_is_left_without_a_green_specification() {
    let chat_rows = CASE_STUDY
        .lines()
        .filter(|line| line.starts_with('|') && line.contains("| Chat |"))
        .collect::<Vec<_>>();

    assert_eq!(chat_rows.len(), 10);
    assert!(chat_rows.iter().all(|row| !row.contains("`still-broken`")));
    assert!(CASE_STUDY.contains("reproduction-before.log"));
    assert!(CASE_STUDY.contains("reproduction-after.log"));
    assert!(CASE_STUDY.contains("one of five named smallest leaves (**20%**)"));
}

#[test]
fn requirements_and_roadmap_report_the_same_partial_reality() {
    let rows = REQUIREMENTS
        .lines()
        .filter(|line| line.starts_with("| R710-"))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 32);
    assert_eq!(
        rows.iter()
            .filter(|row| row.contains("`works-now`"))
            .count(),
        21
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.contains("`still-broken`"))
            .count(),
        10
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.contains("`superseded`"))
            .count(),
        1
    );
    assert!(ROADMAP.contains("21 works now, 1 superseded, 10 still broken"));
    assert!(
        !ROADMAP.contains("Silently-dropped chat/UX/process requirements re-verified | Not done")
    );
}

#[test]
fn formal_ai_authored_verdict_leaf_is_byte_exact_and_has_session_evidence() {
    assert_eq!(
        AGENT_AUTHORED_VERDICT,
        "A works-now verdict requires a passing regression test against the current production path; a still-broken verdict requires an open focused tracking issue."
    );
    assert!(AGENT_STREAM.contains("formal-ai"));
    assert!(AGENT_STREAM.contains(AGENT_SESSION));
}
