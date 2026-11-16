//! Issue #1166: independent per-clause task acceptance.
//!
//! Every clause of a task gets a verdict (discharged, contradicted, unresolved
//! or inapplicable) with its justification and source span, read from the task
//! text and the candidate's observed files and output only. The cases live in
//! `tests/fixtures/issue-1166/acceptance.json`, which the JavaScript twin
//! (`rust/tests/web/issue-1166-task-acceptance.test.mjs`) reads too, so both
//! roots pin the same canonical meanings and verdicts. The Python candidates
//! are executed here, row by row, and only their stdout is observed.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use formal_ai::agentic_coding::task_acceptance::{
    acceptance_report, check_task, specify_task, workflow_events,
};
use formal_ai::engine::{ExecutionRecipe, ExecutionRecipeFile};
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("../fixtures/issue-1166/acceptance.json");
const BODY: &str = include_str!("../fixtures/issue-1166/issue-1450-body.md");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("the fixture is JSON")
}

fn entries<'a>(root: &'a Value, section: &str) -> &'a [Value] {
    root[section].as_array().expect("a fixture section array")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("a string field")
}

fn task_of<'a>(root: &'a Value, id: &str) -> &'a str {
    let entry = entries(root, "specificationCases")
        .iter()
        .find(|entry| entry["id"] == id)
        .expect("a specification case");
    text(&entry["task"])
}

fn read_fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/issue-1166").join(name);
    std::fs::read_to_string(path).expect("a fixture file")
}

fn meaning(task: &str) -> String {
    text(&specify_task(task)["meaning"]).to_owned()
}

fn verdict_lines(report: &Value) -> Vec<String> {
    entries(report, "verdicts")
        .iter()
        .map(|verdict| {
            format!(
                "{}-{} {} {}",
                verdict["span"][0],
                verdict["span"][1],
                text(&verdict["verdict"]),
                text(&verdict["justification"])
            )
        })
        .collect()
}

fn expected_lines(entry: &Value) -> Vec<String> {
    entries(entry, "verdicts").iter().map(|line| text(line).to_owned()).collect()
}

#[test]
fn outputs_and_meanings_are_read_from_the_task_text() {
    let root = fixture();
    assert_eq!(entries(&root, "specificationCases").len(), 17);
    for entry in entries(&root, "specificationCases") {
        let specification = specify_task(text(&entry["task"]));
        let id = text(&entry["id"]);
        assert_eq!(specification["output"], entry["output"], "{id}");
        assert_eq!(specification["meaning"], entry["meaning"], "{id}");
    }
}

#[test]
fn equivalent_paraphrases_share_one_meaning_and_distinct_tasks_do_not() {
    let root = fixture();
    let meanings = |group: &Value| -> HashSet<String> {
        entries(group, "cases")
            .iter()
            .map(|name| meaning(task_of(&root, text(name))))
            .collect()
    };
    for group in entries(&root, "paraphraseGroups") {
        assert_eq!(meanings(group).len(), 1, "{}", group["id"]);
    }
    for group in entries(&root, "distinctGroups") {
        assert_eq!(meanings(group).len(), entries(group, "cases").len(), "{}", group["id"]);
    }
    let multilingual: HashSet<String> = entries(&root, "multilingual")
        .iter()
        .map(|name| meaning(&read_fixture(text(name))))
        .collect();
    assert_eq!(multilingual.len(), 1);
}

#[test]
fn every_clause_gets_a_verdict_with_a_justification_and_a_span() {
    let root = fixture();
    let specification = specify_task(BODY);
    assert_ne!(
        specification["task_identity"],
        specify_task(&format!("{BODY}\n"))["task_identity"]
    );
    for entry in entries(&root, "candidateCases") {
        let mut candidate = entry["candidate"].clone();
        if candidate.get("task_identity").is_none() {
            candidate["task_identity"] = specification["task_identity"].clone();
        }
        let report = check_task(&specification, &candidate);
        let id = text(&entry["id"]);
        assert_eq!(verdict_lines(&report), expected_lines(entry), "{id}");
        assert_eq!(report["complete"], entry["complete"], "{id}");
        for verdict in entries(&report, "verdicts") {
            let justification = text(&verdict["justification"]);
            assert!(!justification.is_empty() && !justification.contains("style"), "{id}");
        }
    }
}

#[test]
fn independent_execution_rejects_a_shared_misunderstanding() {
    let root = fixture();
    let directory = std::env::temp_dir().join(format!("issue-1166-acceptance-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let program = directory.join("main.py");
    let python = |arguments: &[&str]| {
        let output = Command::new("python3").args(arguments).output().expect("python3 runs");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).expect("UTF-8 stdout")
    };
    let path = program.to_str().expect("a UTF-8 path");
    for entry in entries(&root, "executionCases") {
        let specification = specify_task(task_of(&root, text(&entry["task"])));
        std::fs::write(&program, text(&entry["program"])).expect("the program is written");
        if let Some(generated) = entry["generated_test"].as_str() {
            // The test generated from the same misunderstanding passes on its own.
            let test = directory.join("test_main.py");
            std::fs::write(&test, generated).expect("the test is written");
            python(&[test.to_str().expect("a UTF-8 path"), path]);
        }
        let rows: Vec<Vec<String>> = specification["output"]["table"]["rows"].as_array().map_or_else(
            || vec![Vec::new()],
            |rows| {
                rows.iter()
                    .map(|row| entries(row, "args").iter().map(|value| text(value).to_owned()).collect())
                    .collect()
            },
        );
        let observations: Vec<Value> = rows
            .iter()
            .map(|arguments| {
                let mut command = vec![path];
                command.extend(arguments.iter().map(String::as_str));
                json!({ "args": arguments, "stdout": python(&command) })
            })
            .collect();
        let report = check_task(
            &specification,
            &json!({
                "task_identity": specification["task_identity"],
                "program_path": "main.py",
                "files": [{ "path": "main.py", "source": entry["program"] }],
                "observations": observations,
            }),
        );
        let id = text(&entry["id"]);
        assert_eq!(verdict_lines(&report), expected_lines(entry), "{id}");
        assert_eq!(report["complete"], entry["complete"], "{id}");
    }
    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn workflow_triggers_are_read_in_every_yaml_form() {
    assert_eq!(workflow_events("on: [push, pull_request]\njobs:\n"), ["push", "pull_request"]);
    assert_eq!(workflow_events("\"on\": push\njobs:\n"), ["push"]);
    assert_eq!(
        workflow_events("on:\n  push:\n    branches: [main]\n  pull_request:\njobs:\n"),
        ["push", "pull_request"]
    );
    assert_eq!(
        workflow_events("on:\n  - push\n  - workflow_dispatch\njobs:\n"),
        ["push", "workflow_dispatch"]
    );
    assert!(workflow_events("jobs:\n").is_empty());
}

#[test]
fn a_finished_recipe_reports_only_actionable_verdicts() {
    let root = fixture();
    let first = &entries(&root, "candidateCases")[0]["candidate"]["files"];
    let workflow = ExecutionRecipeFile {
        path: text(&first[1]["path"]).to_owned(),
        source: text(&first[1]["source"]).to_owned(),
    };
    let recipe = |language: &str, path: &str, source: &str, supporting_files: Vec<ExecutionRecipeFile>| {
        ExecutionRecipe {
            language: language.to_owned(),
            source: source.to_owned(),
            path: path.to_owned(),
            supporting_files,
            commands: vec![String::new()],
        }
    };
    let greeting = vec!["Hello, World!\n".to_owned()];
    let commented = recipe("kotlin", "Main.kt", text(&first[0]["source"]), vec![workflow.clone()]);
    let report = acceptance_report(BODY, &commented, &greeting);
    assert!(report.starts_with(
        "Independent acceptance of the task: 16 discharged, 0 contradicted, 10 unresolved, 6 inapplicable.\n"
    ));
    assert!(report.contains(
        "\n- unresolved (workflow_status_unobserved, bytes 817-856): Show a green check mark when tests pass\n"
    ));
    assert!(report.contains("The task is not complete"));
    let bare = recipe("kotlin", "Main.kt", "fun main() {\n    println(\"Hello, World!\")\n}\n", vec![workflow]);
    assert!(acceptance_report(BODY, &bare, &greeting).contains(
        "\n- contradicted (comments_missing, bytes 192-230): Add clear comments explaining the code\n"
    ));
    let twice = task_of(&root, "twice");
    let honoured = recipe("python", "main.py", "print(\"Hi\")\nprint(\"Hi\")\n", Vec::new());
    assert_eq!(acceptance_report(twice, &honoured, &["Hi\nHi\n".to_owned()]), "");
    let once = recipe("python", "main.py", "print(\"Hi\")\n", Vec::new());
    assert!(acceptance_report(twice, &once, &["Hi\n".to_owned()]).contains("contradicted (output_differs"));
}
