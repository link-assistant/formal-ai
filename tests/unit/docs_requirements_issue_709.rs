use std::fs;
use std::path::{Path, PathBuf};

const CASE_STUDY: &str = "docs/case-studies/issue-709/README.md";
const REQUIREMENTS: &str = "docs/case-studies/issue-709/requirements.md";
const REGRESSIONS: &str = "tests/unit/issue_709_search_fusion.rs";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

#[test]
fn every_issue_709_requirement_maps_to_a_regression_test_that_exists() {
    let requirements = read(REQUIREMENTS);
    let regressions = [
        read(REGRESSIONS),
        read("tests/unit/issue_709_search_fusion_learning.rs"),
        read("tests/unit/docs_requirements_issue_709.rs"),
    ]
    .join("\n");
    let mut ids = Vec::new();
    for line in requirements
        .lines()
        .filter(|line| line.starts_with("| R709-"))
    {
        let columns: Vec<_> = line.split('|').map(str::trim).collect();
        ids.push(columns[1].to_owned());
        for name in columns[4].split(',').map(str::trim) {
            let name = name.trim_matches('`');
            if Path::new(name).extension().is_none() {
                assert!(
                    regressions.contains(&format!("fn {name}(")),
                    "requirements.md names missing test {name}"
                );
            }
        }
    }
    assert_eq!(
        ids,
        (1..=9)
            .map(|index| format!("R709-{index:02}"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn web_and_telegram_acceptance_evidence_is_committed() {
    let playwright = read("tests/e2e/tests/issue-709.spec.js");
    for phrase in [
        "Apple is a fruit.",
        "Яблоко это фрукт.",
        "conflict=source_disagreement",
        "source_tier=original_first_party",
        "Read more",
    ] {
        assert!(
            playwright.contains(phrase),
            "Playwright fixture must pin {phrase}"
        );
    }
    assert!(
        root()
            .join("docs/screenshots/issue-709-search-fusion.png")
            .is_file()
    );
    assert!(read(REGRESSIONS).contains("telegram_html_from_markdown"));
}

#[test]
fn case_study_release_and_agent_authorship_evidence_are_committed() {
    let study = read(CASE_STUDY).to_lowercase();
    for phrase in [
        "root cause",
        "formalize",
        "merge",
        "rank",
        "cross-language",
        "contradiction",
        "offline",
        "agent cli",
    ] {
        assert!(study.contains(phrase), "case study must explain {phrase}");
    }
    for raw in [
        "issue.json",
        "issue-comments.json",
        "pull-request.json",
        "pull-conversation-comments.json",
        "pull-review-comments.json",
        "pull-reviews.json",
    ] {
        assert!(
            root()
                .join("docs/case-studies/issue-709/raw-data")
                .join(raw)
                .is_file()
        );
    }
    let fragments = fs::read_dir(root().join("changelog.d")).expect("read changelog fragments");
    let unreleased = fragments.filter_map(Result::ok).any(|entry| {
        fs::read_to_string(entry.path())
            .is_ok_and(|body| body.contains("#709") && body.contains("bump: minor"))
    });
    assert!(unreleased || read("CHANGELOG.md").contains("#709"));
    let decomposition =
        read("docs/case-studies/issue-709/self-hosting-authorship/decomposition.lino");
    assert!(decomposition.contains("reviewed_smallest_leaves 13"));
    assert!(decomposition.contains("formal_ai_authored_leaves 7"));
    assert!(decomposition.contains("formal_ai_authored_percent 54"));
    assert!(
        root()
            .join("data/meta/search-fusion-provenance-invariant.lino")
            .is_file()
    );
    let agent_leaves = [
        (
            "data/meta/search-fusion-learning-contract.lino",
            "docs/case-studies/issue-709/agent-cli-evidence/learning-contract/search-fusion-learning-contract.lino",
            "ses_03f0edb46ffesIq1tz5G0zcTuV",
        ),
        (
            "data/meta/search-fusion-source-policy.lino",
            "docs/case-studies/issue-709/agent-cli-evidence/source-policy/search-fusion-source-policy.lino",
            "ses_03f0ec980ffeTwKWTHsJV8L3tk",
        ),
        (
            "data/benchmarks/search-fusion-learning-generalization.lino",
            "docs/case-studies/issue-709/agent-cli-evidence/generalization-fixture/search-fusion-learning-generalization.lino",
            "ses_03f0eb7f9ffes4J25WB6ozqWjQ",
        ),
        (
            "data/meta/issue-709-search-fusion-learning.lino",
            "docs/case-studies/issue-709/agent-cli-evidence/learning-report/issue-709-search-fusion-learning.lino",
            "ses_03f015388ffej7CYn8EdCRJjRm",
        ),
        (
            "data/seed/search-fusion-language-grammar.lino",
            "docs/case-studies/issue-709/agent-cli-evidence/language-grammar/search-fusion-language-grammar.lino",
            "ses_03efa9270ffeDsKblsJY05z2bo",
        ),
    ];
    for (canonical, evidence, session) in agent_leaves {
        assert_eq!(
            read(canonical),
            read(evidence),
            "Agent leaf drifted: {canonical}"
        );
        assert!(
            decomposition.contains(session),
            "decomposition must attribute {session}"
        );
    }
    assert!(decomposition.contains("ses_03eff26d2ffeDtvQk3uW203zgj"));
}
