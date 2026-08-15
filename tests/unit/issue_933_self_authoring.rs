//! Issue #933 executes a compound task through Formal AI and the real Agent
//! CLI, splits only after observed failure, and learns from the same sessions.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

#[test]
fn issue_933_formal_ai_authored_contract_is_preserved_byte_for_byte() {
    let canonical = read("data/meta/conversational-variation-floor-contract.lino");
    let authored =
        read("docs/case-studies/issue-933/self-hosting-authorship/variation-floor-contract.lino");
    assert_eq!(canonical.as_bytes(), authored.as_bytes());
    for invariant in [
        "minimum_per_language \"5\"",
        "languages \"en|ru|hi|zh\"",
        "normalization \"nfkc_lowercase_strip_punctuation_symbols_separators_whitespace\"",
        "execution \"attempt_whole_then_split_on_failure\"",
    ] {
        assert!(canonical.contains(invariant), "missing {invariant}");
    }
}

#[test]
fn issue_933_compound_agent_task_splits_from_failure_and_climbs_back_up() {
    let report: Value = serde_json::from_str(&read(
        "docs/case-studies/issue-933/self-hosting-authorship/dispatch-report.json",
    ))
    .expect("captured incremental dispatch report");
    assert_eq!(report["mode"], "incremental");
    let trace = &report["incremental"];
    assert_eq!(trace["solved"], true);
    assert!(trace["split_depth_reached"].as_u64().unwrap_or(0) >= 1);

    let steps = trace["steps"].as_array().expect("incremental steps");
    assert!(steps.len() >= 4, "{steps:#?}");
    assert_eq!(steps.first().unwrap()["passed"], false);
    assert_eq!(steps.first().unwrap()["cli"], "agent");
    assert_eq!(steps.last().unwrap()["passed"], true);
    assert_eq!(
        steps.first().unwrap()["task"],
        steps.last().unwrap()["task"]
    );

    let splits = trace["splits"].as_array().expect("incremental splits");
    let children = splits
        .iter()
        .find_map(|split| {
            split["children"]
                .as_array()
                .filter(|children| children.len() >= 2)
        })
        .unwrap_or_else(|| panic!("no productive split: {splits:#?}"));
    for target in [
        "variation-floor-contract.lino",
        "variation-floor-learning.lino",
    ] {
        assert!(
            children
                .iter()
                .any(|child| child.as_str().is_some_and(|task| task.contains(target))),
            "missing {target} leaf: {children:#?}"
        );
    }
    for step in steps {
        let relative = step["session_file"].as_str().expect("session path");
        let session: Value = serde_json::from_str(&read(&format!(
            "docs/case-studies/issue-933/self-hosting-authorship/{relative}"
        )))
        .expect("replayable Agent session");
        assert!(
            root()
                .join("docs/case-studies/issue-933/self-hosting-authorship")
                .join(relative)
                .is_file(),
            "missing replayable {relative}"
        );
        assert!(
            session["native_session"]["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("ses_")),
            "missing native session id: {session:#?}"
        );
        assert!(
            session["native_session"]["resume_command"]
                .as_str()
                .is_some_and(|command| command.contains("agent --resume ses_")),
            "missing native resume command: {session:#?}"
        );
    }
}

#[test]
fn issue_933_incremental_sessions_feed_human_gated_learning() {
    let learning = read("docs/case-studies/issue-933/self-hosting-authorship/learning.lino");
    let report: Value = serde_json::from_str(&read(
        "docs/case-studies/issue-933/self-hosting-authorship/dispatch-report.json",
    ))
    .expect("captured incremental dispatch report");
    let step_count = report["incremental"]["steps"]
        .as_array()
        .expect("incremental steps")
        .len();
    assert!(learning.contains("human_gated \"true\""), "{learning}");
    assert!(
        learning.contains(&format!("observation_count \"{step_count}\"")),
        "{learning}"
    );
    assert!(
        learning.contains("decision \"awaiting_human_review\"")
            || learning.contains("decision \"no_reviewable_change\""),
        "{learning}"
    );
    assert!(!learning.contains("decision \"approved\""), "{learning}");

    let recipe = read("experiments/issue_933_self_authoring/run.sh");
    for invariant in [
        "agent dispatch",
        "--incremental",
        "--cli agent",
        "--verify",
        "learning.lino",
    ] {
        assert!(recipe.contains(invariant), "recipe missing {invariant}");
    }
}

#[test]
fn issue_933_decomposition_assigns_at_least_twenty_percent_to_formal_ai() {
    let decomposition =
        read("docs/case-studies/issue-933/self-hosting-authorship/decomposition.lino");
    for invariant in [
        "leaf_count \"6\"",
        "formal_ai_authored_leaf_count \"2\"",
        "formal_ai_authored_percent \"33\"",
        "issue_933_leaf_contract_file",
        "issue_933_leaf_learning_observation",
    ] {
        assert!(decomposition.contains(invariant), "missing {invariant}");
    }
    assert_eq!(
        decomposition
            .matches("owner \"formal_ai_agent_cli\"")
            .count(),
        2
    );
    assert_eq!(
        decomposition
            .matches("record_type \"smallest_leaf\"")
            .count(),
        6
    );
}

#[test]
fn issue_933_documents_every_requirement() {
    let case_study = read("docs/case-studies/issue-933/README.md");
    let requirements = read("docs/requirements/issue-0933-conversational-variation-floor.md");
    for requirement in 1..=14 {
        let id = format!("R933-{requirement}");
        assert!(case_study.contains(&id), "case study is missing {id}");
        assert!(requirements.contains(&id), "requirements are missing {id}");
    }
}
