use super::{
    AGENT_TOOLS, BRANCH, ChatMessage, ISSUE, PR, PlannedToolCall, command_of, drive,
    is_work_item_read, page, path_of, prompt,
};

// Canonical template keys are literal placeholders, not Rust format captures.
const FILES_PLACEHOLDER: &str = "{files}";
const SUBJECT_PLACEHOLDER: &str = "{subject}";
const BODY_PLACEHOLDER: &str = "{body}";

// Reuse the production reader and canonical templates without exporting a planner helper.
use crate::canonical_work_item_steps;

// This fixture stops at the named-branch commit phase. Prepared-PR delivery,
// feedback and readiness are exercised with real Git in issue_1158/1160/1162.
pub(super) fn commit_phase_prompt() -> String {
    prompt().replace(&format!("Update the pull request at {PR}.\n"), "")
}

pub(super) fn expected_owned_recipe_commit(planned: &[PlannedToolCall]) -> String {
    let paths = planned
        .iter()
        .filter(|call| call.tool == "write")
        .filter(|call| {
            let arguments: serde_json::Value =
                serde_json::from_str(&call.arguments).expect("write arguments");
            arguments["append_mode"] != "atomic_record_append"
        })
        .map(path_of)
        .collect::<Vec<_>>();
    assert_eq!(paths.first().map(String::as_str), Some("Main.scala"));
    assert_eq!(
        paths
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        paths.len(),
        "one ordered write per recipe path"
    );
    let quote = |value: &str| {
        assert!(
            !value.contains('\''),
            "these fixture operands contain no shell quote"
        );
        format!("'{value}'")
    };
    let files = paths
        .iter()
        .map(|value| quote(value))
        .collect::<Vec<_>>()
        .join(" ");
    let subject = canonical_work_item_steps::fill("subject_added", &[("{path}", &paths[0])]);
    let body = canonical_work_item_steps::fill("body_resolves", &[("{reference}", ISSUE)]);
    let inner = canonical_work_item_steps::fill(
        "recipe_commit_command",
        &[
            (FILES_PLACEHOLDER, "\"$@\""),
            (SUBJECT_PLACEHOLDER, &quote(&subject)),
            (BODY_PLACEHOLDER, &quote(&body)),
            ("{branch}", &quote(BRANCH)),
        ],
    );
    let template = canonical_work_item_steps::fill("workflow_template", &[("{path}", &paths[0])]);
    let header = template.lines().next().expect("workflow provenance header");
    let name = template
        .lines()
        .find(|line| line.starts_with("name: "))
        .expect("workflow name");
    let workflow = planned
        .iter()
        .find(|call| {
            let args: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
            call.tool == "write"
                && args["path"]
                    .as_str()
                    .is_some_and(|path| path.starts_with(".github/workflows/"))
        })
        .expect("recipe workflow write");
    let args: serde_json::Value = serde_json::from_str(&workflow.arguments).unwrap();
    let source = args["content"].as_str().expect("workflow source");
    assert!(source.lines().any(|line| line == header));
    assert!(source.lines().any(|line| line == name));
    let path = path_of(workflow);
    let (directory, _) = path.rsplit_once('/').expect("workflow directory");
    let retire = canonical_work_item_steps::fill(
        "superseded-workflow-retire-command",
        &[
            ("{dir}", &quote(directory)),
            ("{workflow}", &quote(&path)),
            ("{header}", &quote(header)),
            ("{name}", &quote(name)),
        ],
    );
    canonical_work_item_steps::fill(
        "task-scaffold-commit-command",
        &[
            (FILES_PLACEHOLDER, &files),
            ("{reference}", &quote(ISSUE)),
            ("{retire}", &retire),
            ("{commit}", &inner),
        ],
    )
}

pub(super) fn assert_named_branch_push(language: &str, request: String) {
    let scala = page("Scala");
    let mut messages = vec![ChatMessage::user(request)];
    let (planned, _) = drive(
        AGENT_TOOLS.as_slice(),
        &mut messages,
        |tool, arguments| {
            if is_work_item_read(tool, arguments) {
                scala.clone()
            } else {
                "Hello, World!\n".to_owned()
            }
        },
        10,
    );
    let commit = planned
        .iter()
        .map(command_of)
        .find(|command| command == &expected_owned_recipe_commit(&planned))
        .unwrap_or_else(|| panic!("{language}: no commit step in {planned:?}"));
    assert!(
        commit.contains(&format!("__formal_ai_push_ref='{BRANCH}'")),
        "{language}: named branch binding: {commit}"
    );
    assert!(
        commit.contains("git push -q origin \"$__formal_ai_push_ref\""),
        "{language}: the bound branch must be the actual push operand: {commit}"
    );
    assert!(
        commit.contains("git symbolic-ref --quiet --short HEAD"),
        "{language}: detached HEAD must be refused: {commit}"
    );
}
