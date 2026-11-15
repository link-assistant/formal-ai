//! Issue #1560 outcome: a Hive Mind session on issues 1486, 1489 and 1501
//! reads repository files instead of refusing every read; a work-item final is
//! a localized report, never a JSON object; a fetched issue page yields only
//! its title and body; a page without a body is rejected and the API read is
//! used; a user turn that forbids a read is still honoured.
//! JavaScript twin: rust/tests/web/work-item-report-outcome.test.mjs.

use formal_ai::agentic_coding::tool_result::complete_owned_source_read_frame;
use formal_ai::agentic_coding::work_item_page::work_item_page_text;
use formal_ai::agentic_coding::work_item_report::{
    parses_as_json_object, reported_work_item_record, work_item_report,
};
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const ISSUES: &str =
    include_str!("../../../../data/benchmarks/fixtures/work-item-1560/issues.json");
const PAGE_1501: &str =
    include_str!("../../../../data/benchmarks/fixtures/work-item-1560/page-1501.txt");
const SIGN_IN: &str =
    include_str!("../../../../data/benchmarks/fixtures/work-item-1560/signin.txt");
const PROMPT: &str =
    include_str!("../../../../data/benchmarks/fixtures/work-item-1560/hive-mind-prompt.txt");
/// Hive Mind's tool set, recorded from the pull request 1502 session.
const TOOLS: [&str; 14] = [
    "bash",
    "read",
    "glob",
    "grep",
    "list",
    "edit",
    "write",
    "task",
    "webfetch",
    "websearch",
    "codesearch",
    "batch",
    "todowrite",
    "todoread",
];
const CHROME: [&str; 3] = ["· GitHub", "Skip to content", "Sign in"];
/// `gh issue view` without a login.
const GH_SIGNED_OUT: &str =
    "gh: To get started with GitHub CLI, please run:  gh auth login\n\n__formal_ai_exit=4\n";
const TITLE_1501: &str = "Agent mode 0.355.27 misroutes common file requests";

fn prompt(number: u32) -> String {
    PROMPT
        .replace("{number}", &number.to_string())
        .replace("{pull}", &(number + 1).to_string())
        .replace("{branch}", "0123456789ab")
}

fn issue(number: &str) -> String {
    let issues: Value = serde_json::from_str(ISSUES).expect("issue fixtures");
    let issue = &issues["issues"][number];
    format!(
        "{}\n\n{}",
        issue["title"].as_str().expect("title"),
        issue["body"].as_str().expect("body")
    )
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root")
        .to_path_buf()
}

/// A repository file of this checkout, or `None`.
fn repository_file(path: &str) -> Option<String> {
    let relative = path.trim_start_matches('/');
    if relative.is_empty() {
        return None;
    }
    let full = repository_root().join(relative);
    full.is_file()
        .then(|| std::fs::read_to_string(full).ok())
        .flatten()
}

struct Session {
    final_answer: String,
    calls: Vec<(String, Value)>,
    read: Vec<String>,
}

/// Drives a session to its final answer. `answer(tool, args)` returns a tool
/// result, or `None` for the default: a repository file in the agent's
/// numbered read frame for a read, empty output for anything else.
fn drive(
    mut messages: Vec<ChatMessage>,
    tools: &[&str],
    answer: impl Fn(&str, &Value) -> Option<String>,
) -> Session {
    let mut calls = Vec::new();
    let mut read = Vec::new();
    for step in 0..24 {
        match plan_chat_step(&messages, tools) {
            Some(AgenticPlan::ToolCalls(planned)) => {
                let call = planned[0].clone();
                let args: Value = serde_json::from_str(&call.arguments).expect("call arguments");
                let path = ["filePath", "file_path", "path"]
                    .iter()
                    .find_map(|key| args[*key].as_str())
                    .unwrap_or_default()
                    .to_owned();
                let file = (call.tool == "read")
                    .then(|| repository_file(&path))
                    .flatten();
                if file.is_some() {
                    read.push(path.clone());
                }
                let content = answer(&call.tool, &args).unwrap_or_else(|| match &file {
                    Some(source) => complete_owned_source_read_frame(source),
                    None if call.tool == "read" => format!("Error: File not found: {path}"),
                    None => String::new(),
                });
                let id = format!("outcome-{step}");
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    id.clone(),
                    call.tool.clone(),
                    call.arguments.clone(),
                )]));
                messages.push(ChatMessage::tool_result(id, &call.tool, &content));
                calls.push((call.tool, args));
            }
            Some(AgenticPlan::Final(final_answer)) => {
                return Session {
                    final_answer,
                    calls,
                    read,
                };
            }
            None => panic!("the session ended without a final answer: {calls:?}"),
        }
    }
    panic!("the session did not end within 24 steps: {calls:?}");
}

fn issue_read(number: &'static str) -> impl Fn(&str, &Value) -> Option<String> {
    move |tool, args| {
        let command = args["command"].as_str().unwrap_or_default();
        if tool == "bash" && (command.contains("gh issue ") || command.contains("gh api ")) {
            Some(format!("{}\n\n__formal_ai_exit=0\n", issue(number)))
        } else {
            None
        }
    }
}

/// A work-item final is a localized report; its record stays in the trace.
fn reported(answer: &str) -> Value {
    assert!(!parses_as_json_object(answer), "{answer}");
    assert!(!answer.contains("ReadPolicyRefused"), "{answer}");
    let record = reported_work_item_record(answer).expect("a work-item final keeps its record");
    assert_ne!(record["reason"], "ReadPolicyRefused", "{answer}");
    record
}

fn requirement_texts(record: &Value) -> Vec<String> {
    record["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .map(|requirement| requirement["text"].as_str().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn hive_mind_sessions_read_a_repository_file_and_do_not_refuse_the_read() {
    for number in ["1486", "1489", "1501"] {
        let session = drive(
            vec![ChatMessage::user(prompt(number.parse().unwrap()))],
            &TOOLS,
            issue_read(number),
        );
        let (tool, args) = &session.calls[0];
        assert_eq!(tool, "bash", "{number}");
        assert!(
            args["command"]
                .as_str()
                .unwrap_or_default()
                .contains(&format!("issues/{number}")),
            "{number}: the issue is read first"
        );
        assert!(
            !session.read.is_empty(),
            "{number}: no repository file was read: {:?}",
            session.calls
        );
        let record = reported(&session.final_answer);
        assert!(
            record["observations"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|observation| observation["complete"] == json!(true)
                    && session
                        .read
                        .iter()
                        .any(|path| observation["path"] == json!(path))),
            "{number}: {record}"
        );
        assert!(
            session
                .final_answer
                .contains(&format!("`{}`", session.read[0])),
            "{number}: the report names the file it read: {}",
            session.final_answer
        );
        assert!(
            requirement_texts(&record).len() > 1,
            "{number}: the issue text is split into requirements"
        );
    }
}

#[test]
fn issue_1501_page_keeps_only_its_title_and_body() {
    let text = work_item_page_text(PAGE_1501).expect("the page has a body");
    assert!(text.starts_with(TITLE_1501), "{}", &text[..200]);
    assert!(text.contains("Summary"));
    for marker in CHROME {
        assert!(!text.contains(marker), "{marker}");
    }
    assert!(!text.contains("Sign up for free"));
    assert!(!text.contains("&#39;"), "entities are decoded");
}

#[test]
fn issue_1501_page_session_reads_a_file_and_its_requirements_carry_no_chrome() {
    let fetch_only: Vec<&str> = TOOLS
        .iter()
        .copied()
        .filter(|tool| *tool != "bash")
        .collect();
    let session = drive(
        vec![ChatMessage::user(prompt(1501))],
        &fetch_only,
        |tool, _| (tool == "webfetch").then(|| PAGE_1501.to_owned()),
    );
    assert_eq!(session.calls[0].0, "webfetch", "{:?}", session.calls);
    assert!(
        !session.read.is_empty(),
        "no repository file was read: {:?}",
        session.calls
    );
    let record = reported(&session.final_answer);
    let requirements = requirement_texts(&record);
    assert!(
        requirements[0].starts_with(TITLE_1501),
        "{}",
        requirements[0]
    );
    for requirement in &requirements {
        for marker in CHROME {
            assert!(!requirement.contains(marker), "{marker} in {requirement}");
        }
    }
    for marker in CHROME {
        assert!(!session.final_answer.contains(marker), "{marker}");
    }
}

#[test]
fn a_page_without_an_issue_body_is_rejected_and_the_api_read_is_used() {
    assert_eq!(work_item_page_text(SIGN_IN), None);
    let session = drive(
        vec![ChatMessage::user(prompt(1501))],
        &TOOLS,
        |tool, args| {
            let command = args["command"].as_str().unwrap_or_default();
            if tool == "webfetch" {
                Some(SIGN_IN.to_owned())
            } else if tool == "bash" && command.contains("gh issue view") {
                Some(GH_SIGNED_OUT.to_owned())
            } else if tool == "bash" && (command.contains("curl ") || command.contains("gh api ")) {
                Some(format!("{}\n\n__formal_ai_exit=0\n", issue("1501")))
            } else {
                None
            }
        },
    );
    let used: Vec<String> = session
        .calls
        .iter()
        .map(|(tool, args)| {
            if tool == "bash" {
                args["command"].as_str().unwrap_or_default().to_owned()
            } else {
                tool.clone()
            }
        })
        .collect();
    let webfetch = used
        .iter()
        .position(|used| used == "webfetch")
        .unwrap_or_else(|| panic!("the page is fetched: {used:?}"));
    assert!(
        used[webfetch + 1..]
            .iter()
            .any(|command| command.contains("curl ") || command.contains("gh api ")),
        "the REST or gh api read follows the rejected page: {used:?}"
    );
    assert!(
        !session.read.is_empty(),
        "the session continues from the API body: {used:?}"
    );
    let record = reported(&session.final_answer);
    for requirement in requirement_texts(&record) {
        assert!(!requirement.contains("Sign in"), "{requirement}");
    }
}

#[test]
fn a_user_turn_that_says_not_to_read_a_file_is_still_honoured() {
    let messages = vec![ChatMessage::user(
        "Do not read package.json. Investigate package.json.",
    )];
    match plan_chat_step(&messages, &["read"]) {
        Some(AgenticPlan::ToolCalls(calls)) => assert!(
            !calls
                .iter()
                .any(|call| call.tool == "read" && call.arguments.contains("package.json")),
            "{calls:?}"
        ),
        Some(AgenticPlan::Final(answer)) => assert!(!parses_as_json_object(&answer), "{answer}"),
        None => {}
    }
}

#[test]
fn a_refusal_names_the_requirement_it_concerns_in_the_localized_report() {
    let record = json!({
        "schema": "repository-work-item-evidence/v1",
        "reason": "ReadPolicyRefused",
        "authored": false,
        "observations": [],
        "requirements": [
            {"text": "Keep the release notes short."},
            {"text": "Do not read secrets/token.txt.\nIt holds a credential."}
        ],
        "refused": {"path": "secrets/token.txt", "requirement": 1}
    });
    let report = work_item_report(&record, "Solve the issue.");
    assert!(!parses_as_json_object(&report));
    assert!(report.contains("secrets/token.txt"));
    assert!(report.contains("Do not read secrets/token.txt. It holds a credential."));
    assert!(report.contains("ReadPolicyRefused"));
}
