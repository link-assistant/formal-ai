//! Issue #1162: the delivery commands of one Hello World work item agree
//! across runtimes and hold on the evidence the 2026-11 test runs left.
//!
//! `rust/tests/fixtures/issue-1162/delivery-commands.json` is written by
//! `experiments/issue-1162/regenerate-delivery-fixture.mjs` from the JS
//! runtime; `rust/tests/web/issue-1162-delivery-commands.test.mjs` asserts
//! the same file there. `own-status-notices.json` holds the shortest real
//! orchestrator notice per own-status pattern from the three test pull
//! requests (339 comments in all) plus negatives that must stay feedback.

use super::{pr_completion_commands, superseded_workflow_step, CommitTarget};
use crate::agentic_coding::restart_feedback::{feedback_needs_changes, pull_request_feedback_filter};
use crate::agentic_coding::tool_result::exact_successful_stdout;
use crate::event_log::EventLog;

const DELIVERY: &str = include_str!("../../fixtures/issue-1162/delivery-commands.json");
const NOTICES: &str = include_str!("../../fixtures/issue-1162/own-status-notices.json");
const KOTLIN_ISSUE: &str = include_str!("../../fixtures/issue-1162/hello-world-kotlin-issue.md");
const WORKFLOW_NAMES: &str = include_str!("../../fixtures/issue-1162/workflow-names.json");

fn json(text: &str) -> serde_json::Value {
    serde_json::from_str(text).expect("the issue #1162 fixtures are JSON")
}

fn text<'a>(value: &'a serde_json::Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_else(|| panic!("`{key}` is a string"))
}

fn strings(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .expect("a list of commands")
        .iter()
        .map(|item| item.as_str().expect("a command").to_owned())
        .collect()
}

fn recipe_for(prompt: &str) -> crate::engine::ExecutionRecipe {
    let mut recipe = *crate::coding::program_contract::answer(prompt, &mut EventLog::default())
        .and_then(|answer| answer.execution_recipe)
        .expect("the Hello World prompt has a program recipe");
    if crate::agentic_coding::ci_workflow::requested_in(prompt) {
        crate::agentic_coding::ci_workflow::attach(&mut recipe, prompt);
    }
    recipe
}

#[test]
fn delivery_commands_match_the_js_runtime() {
    let fixture = json(DELIVERY);
    let workflows = fixture["workflows"].as_array().expect("workflow cases");
    assert_eq!(workflows.len(), 3);
    for case in workflows {
        let id = text(case, "id");
        let prompt = case["prompt"].as_str().unwrap_or(KOTLIN_ISSUE);
        let recipe = recipe_for(prompt);
        assert_eq!(recipe.path, text(case, "program"), "{id}");
        let workflow = recipe
            .supporting_files
            .iter()
            .find(|file| {
                std::path::Path::new(&file.path)
                    .extension()
                    .is_some_and(|ext| ext == "yml" || ext == "yaml")
            })
            .map(|file| file.path.as_str());
        assert_eq!(workflow, case["workflow"].as_str(), "{id}");
        assert_eq!(superseded_workflow_step(&recipe), text(case, "retire_step"), "{id}");
        if id == "kotlin-issue-body" {
            let pull = &fixture["pull_request"];
            let target = CommitTarget {
                binding_refused: false,
                branch: None,
                reference: text(pull, "target").to_owned(),
            };
            assert_eq!(
                pr_completion_commands(&recipe, &target, text(pull, "issue")),
                strings(&pull["with_issue"])
            );
            assert_eq!(
                pr_completion_commands(&recipe, &target, text(pull, "target")),
                strings(&pull["without_issue"])
            );
        }
    }
    assert_eq!(pull_request_feedback_filter(), text(&fixture, "feedback_filter"));
    for case in fixture["stdout"].as_array().expect("stdout cases") {
        assert_eq!(
            exact_successful_stdout(text(case, "raw")).as_deref(),
            case["exact"].as_str(),
            "{}",
            text(case, "id")
        );
    }
}

#[test]
fn the_workflow_takes_the_suggested_file_name_never_over_a_literal_path() {
    let fixture = json(WORKFLOW_NAMES);
    for case in fixture["cases"].as_array().expect("naming cases") {
        let recipe = recipe_for(text(case, "prompt"));
        let workflow = recipe
            .supporting_files
            .iter()
            .find(|file| file.path.ends_with(".yml") || file.path.ends_with(".yaml"))
            .map(|file| file.path.as_str());
        assert_eq!(workflow, case["workflow"].as_str(), "{}", text(case, "id"));
    }
}

#[test]
fn the_retire_step_only_targets_the_generated_workflow_directory() {
    let fixture = json(DELIVERY);
    let kotlin = &fixture["workflows"][0];
    let step = text(kotlin, "retire_step");
    assert!(step.contains("for formal_ai_workflow in '.github/workflows'/*.yml"));
    assert!(step.contains("test \"$formal_ai_workflow\" != '.github/workflows/test-hello-world.yml'"));
    assert!(step.contains("git diff --quiet --"));
    assert!(step.contains("grep -qxF -e 'name: "));
    // A recipe without a generated workflow retires nothing.
    assert_eq!(text(&fixture["workflows"][2], "retire_step"), "");
}

fn single(item: &serde_json::Value) -> String {
    serde_json::json!({ "comments": [item], "reviews": [] }).to_string()
}

#[test]
fn every_observed_orchestrator_notice_is_bookkeeping_and_every_negative_is_feedback() {
    let fixture = json(NOTICES);
    let notices = fixture["own_notices"].as_array().expect("own notices");
    assert_eq!(notices.len(), 14, "one notice per own-status pattern");
    assert_eq!(fixture["observed"]["unmatched"], 0);
    for notice in notices {
        let item = serde_json::json!({
            "author": { "login": "konard" },
            "viewerDidAuthor": true,
            "includesCreatedEdit": false,
            "body": text(notice, "body"),
        });
        assert!(!feedback_needs_changes(&single(&item)), "{}", text(notice, "key"));
    }
    for negative in fixture["negatives"].as_array().expect("negatives") {
        assert!(
            feedback_needs_changes(&single(&negative["item"])),
            "{}",
            text(negative, "id")
        );
    }
}

/// Issue #1157 / #1162 acceptance item 5 (already-complete no-op): a repeated
/// or resumed run whose verified files already match the last commit is a
/// success stated in a sentence, and an actual commit is never reported as
/// unchanged.
#[test]
fn an_unchanged_tree_on_a_repeated_run_is_reported_as_success() {
    let recipe = recipe_for(KOTLIN_ISSUE);
    let target = CommitTarget {
        binding_refused: false,
        branch: Some("issue-1162".to_owned()),
        reference: "https://github.com/example/repo/issues/1162".to_owned(),
    };
    let mut outputs = vec![String::new(); recipe.commands.len()];
    outputs.push("__formal_ai_commit_unchanged\n0123456789abcdef".to_owned());
    let answer = recipe.final_answer(&outputs, Some(&target));
    assert!(
        answer.contains("Nothing to commit: the verified files already match the last commit on `issue-1162`"),
        "{answer}"
    );
    assert!(answer.contains("is a success, not a failure"), "{answer}");

    outputs.pop();
    outputs.push("0123456789abcdef".to_owned());
    let answer = recipe.final_answer(&outputs, Some(&target));
    assert!(!answer.contains("Nothing to commit"), "{answer}");
    assert!(answer.contains("Committed and pushed to `issue-1162`"), "{answer}");
}
