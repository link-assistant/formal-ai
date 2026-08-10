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
const AGENT_AUTHORED_AUDIT: &str = include_str!(
    "../../docs/case-studies/issue-710/agent-cli-evidence/audit-contract/agent-authored-audit-contract.lino"
);
const AUDIT_AGENT_STREAM: &str = include_str!(
    "../../docs/case-studies/issue-710/agent-cli-evidence/audit-contract/agent-stream.raw.log"
);
const AUDIT_AGENT_SESSION: &str = "ses_0410e92d7ffe8KC9TV3V6UvJXM";

const VERDICTS: [&str; 4] = [
    "`works-now`",
    "`still-broken`",
    "`superseded`",
    "`blocked-upstream`",
];

#[test]
fn every_issue_710_checklist_row_has_one_allowed_verdict_and_evidence() {
    let audit = CASE_STUDY
        .split("## 2026-08-10 follow-up re-verification")
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
    let current_audit = CASE_STUDY
        .split("## 2026-08-10 follow-up re-verification")
        .nth(1)
        .expect("case study should contain the follow-up re-verification");
    let chat_rows = current_audit
        .lines()
        .filter(|line| line.starts_with('|') && line.contains("| Chat |"))
        .collect::<Vec<_>>();

    assert_eq!(chat_rows.len(), 10);
    assert!(chat_rows.iter().all(|row| !row.contains("`still-broken`")));
    assert!(CASE_STUDY.contains("reproduction-before.log"));
    assert!(CASE_STUDY.contains("reproduction-after.log"));
    assert!(CASE_STUDY.contains("two of five named smallest leaves (**40%**)"));
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
        29
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.contains("`still-broken`"))
            .count(),
        2
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.contains("`superseded`"))
            .count(),
        1
    );
    assert!(ROADMAP.contains("29 works now, 1 superseded, 2 still broken"));
    assert!(
        !ROADMAP.contains("Silently-dropped chat/UX/process requirements re-verified | Not done")
    );
}

#[test]
fn current_open_gaps_have_focused_open_owners() {
    assert!(CASE_STUDY.contains("[#990](https://github.com/link-assistant/formal-ai/issues/990)"));
    assert!(CASE_STUDY.contains("[#991](https://github.com/link-assistant/formal-ai/issues/991)"));
    assert!(REQUIREMENTS.contains("R710-20 | How-to multi-source synthesis and seven-day availability cache. | `still-broken` — [#991]"));
    assert!(REQUIREMENTS.contains(
        "R710-30 | link-foundation/start and command-stream adoption. | `still-broken` — [#990]"
    ));
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

#[test]
fn formal_ai_authored_audit_leaf_matches_all_reconciled_requirements() {
    // This 2026-08-01 Agent-authored artifact is intentionally immutable even
    // though the current follow-up audit records later merged implementations.
    let requirements = AGENT_AUTHORED_AUDIT
        .split("\n  requirement\n")
        .skip(1)
        .collect::<Vec<_>>();

    assert_eq!(requirements.len(), 32);
    for (offset, requirement) in requirements.iter().enumerate() {
        assert!(requirement.contains(&format!("index \"{}\"", offset + 1)));
        assert!(requirement.contains(&format!("identifier \"R710-{:02}\"", offset + 1)));
        assert!(requirement.contains(&format!(
            "evidence_ref \"case-study-R710-{:02}\"",
            offset + 1
        )));
    }
    assert_eq!(
        AGENT_AUTHORED_AUDIT
            .matches("verdict \"works-now\"")
            .count(),
        21
    );
    assert_eq!(
        AGENT_AUTHORED_AUDIT
            .matches("verdict \"still-broken\"")
            .count(),
        10
    );
    assert_eq!(
        AGENT_AUTHORED_AUDIT
            .matches("verdict \"superseded\"")
            .count(),
        1
    );
    assert!(!AGENT_AUTHORED_AUDIT.contains("blocked-upstream"));
    assert!(AUDIT_AGENT_STREAM.contains("formal-ai"));
    assert!(AUDIT_AGENT_STREAM.contains(AUDIT_AGENT_SESSION));
}
