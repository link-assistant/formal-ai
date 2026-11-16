//! Issue #1558: file names from issue text are resolved before they are read.
//!
//! Each test replays the Agent CLI session recorded for PR #1493 (its system
//! prompts, the solve prompt and the issue fetch) with the recorded text of the
//! work item under test, through the public chat-completion entry that binds a
//! relative read to `<workspace>/<path>` the way the Agent CLI `read` tool needs
//! it. The read, glob and `find` calls are answered from a workspace the way
//! the Agent CLI tools answer them. Twin of
//! `rust/tests/web/issue-1558-reference-resolution.test.mjs`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use formal_ai::agentic_coding::repetition_guard::repeated_action_stop;
use formal_ai::agentic_coding::work_item_report::{
    latest_work_item_record, parses_as_json_object, reported_work_item_record,
};
use formal_ai::protocol::{ChatCompletionRequest, ChatMessage, create_chat_completion_with_solver};
use formal_ai::{SolverConfig, UniversalSolver};
use serde_json::Value;

const RECORDED_REQUEST: &str =
    include_str!("../../../../data/benchmarks/fixtures/issue-1558/agent-cli-request.json");
const RECORDED_WORKSPACE: &str = "/tmp/gh-issue-solver-1793442214945";
const VIRTUAL_ROOT: &str = "/tmp/issue-1558-virtual-workspace";

fn recorded_issue(issue: u32) -> &'static str {
    match issue {
        1484 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1484.txt"),
        1485 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1485.txt"),
        1487 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1487.txt"),
        1488 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1488.txt"),
        1507 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/issue-1507.txt"),
        4033 => include_str!("../../../../data/benchmarks/fixtures/issue-1558/hive-mind-4033.txt"),
        _ => panic!("no recorded issue {issue}"),
    }
}

/// The files a session can see: this checkout (tracked files) or a small virtual tree.
pub(super) struct Workspace {
    pub(super) root: String,
    files: BTreeMap<String, Option<String>>,
}

impl Workspace {
    pub(super) fn checkout() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the repository root")
            .canonicalize()
            .expect("a canonical repository root");
        let listed = std::process::Command::new("git")
            .args(["ls-files", "-z"])
            .current_dir(&root)
            .output()
            .expect("git ls-files");
        let files = String::from_utf8(listed.stdout)
            .expect("UTF-8 file names")
            .split('\0')
            .filter(|file| !file.is_empty())
            .map(|file| (file.to_owned(), None))
            .collect();
        Self {
            root: root.to_str().expect("a UTF-8 root").to_owned(),
            files,
        }
    }

    fn with(files: &[(&str, &str)]) -> Self {
        Self {
            root: VIRTUAL_ROOT.to_owned(),
            files: files
                .iter()
                .map(|(path, content)| ((*path).to_owned(), Some((*content).to_owned())))
                .collect(),
        }
    }

    fn relative<'a>(&self, path: &'a str) -> &'a str {
        path.strip_prefix(&format!("{}/", self.root))
            .unwrap_or(path)
    }

    fn content(&self, relative: &str) -> Option<String> {
        match self.files.get(relative)? {
            Some(content) => Some(content.clone()),
            None => std::fs::read_to_string(Path::new(&self.root).join(relative)).ok(),
        }
    }

    fn named(&self, name: &str) -> Vec<&str> {
        self.files
            .keys()
            .map(String::as_str)
            .filter(|file| file.rsplit('/').next() == Some(name))
            .take(100)
            .collect()
    }

    /// The answer to one call and whether it failed, or `None` for an action the emulation does not perform.
    pub(super) fn answer(&self, tool: &str, arguments: &Value) -> Option<(String, bool)> {
        if tool == "read" {
            let path = arguments["filePath"].as_str()?;
            let absolute = if path.starts_with('/') {
                path.to_owned()
            } else {
                format!("{}/{path}", self.root)
            };
            let Some(source) = self.content(self.relative(&absolute)) else {
                return Some((format!("Error: File not found: {absolute}"), true));
            };
            let lines = source.split('\n').take(2000).collect::<Vec<_>>();
            let body = lines
                .iter()
                .enumerate()
                .map(|(index, line)| format!("{:05}| {line}", index + 1))
                .collect::<Vec<_>>()
                .join("\n");
            return Some((
                format!(
                    "<file>\n{body}\n\n(End of file - total {} lines)\n</file>",
                    lines.len()
                ),
                false,
            ));
        }
        if tool == "glob" {
            let name = arguments["pattern"].as_str()?.strip_prefix("**/")?;
            let files = self
                .named(name)
                .into_iter()
                .map(|file| format!("{}/{file}", self.root))
                .collect::<Vec<_>>();
            let listing = if files.is_empty() {
                "No files found".to_owned()
            } else {
                files.join("\n")
            };
            return Some((listing, false));
        }
        let command = arguments["command"].as_str()?;
        let name = command
            .strip_prefix("find . -type f -name '")?
            .split('\'')
            .next()?;
        (tool == "bash").then(|| {
            let mut listing = self
                .named(name)
                .into_iter()
                .map(|file| format!("./{file}"))
                .collect::<Vec<_>>()
                .join("\n");
            listing.push('\n');
            (listing, false)
        })
    }
}

pub(super) struct Call {
    pub(super) tool: String,
    pub(super) arguments: Value,
    pub(super) failed: bool,
}

pub(super) struct Session {
    pub(super) calls: Vec<Call>,
    pub(super) final_answer: Option<String>,
    pub(super) messages: Vec<ChatMessage>,
}

pub(super) fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

/// The recorded request, rewritten to `issue` of `repository` in `workspace`, ending with `text`.
pub(super) fn recorded_request(
    issue: u32,
    repository: &str,
    workspace: &Workspace,
    text: &str,
    tools: Option<&[&str]>,
) -> ChatCompletionRequest {
    let rewritten = RECORDED_REQUEST
        .replace(RECORDED_WORKSPACE, &workspace.root)
        .replace(
            "link-assistant/formal-ai/issues/1485",
            &format!("{repository}/issues/{issue}"),
        )
        .replace("issue-1485-", &format!("issue-{issue}-"));
    let mut body: Value = serde_json::from_str(&rewritten).expect("the recorded request");
    let call = body["messages"]
        .as_array()
        .and_then(|messages| messages.last())
        .and_then(|message| message["tool_calls"][0]["id"].as_str())
        .expect("the recorded issue fetch")
        .to_owned();
    body["messages"]
        .as_array_mut()
        .expect("messages")
        .push(serde_json::json!({"role":"tool","tool_call_id":call,"content":text}));
    if let Some(tools) = tools {
        body["tools"]
            .as_array_mut()
            .expect("tools")
            .retain(|tool| tools.contains(&tool["function"]["name"].as_str().unwrap_or_default()));
    }
    body["stream"] = Value::Bool(false);
    serde_json::from_value(body).expect("a chat completion request")
}

/// Plan the session step by step until a final answer, an action outside the emulation, or 40 steps.
pub(super) fn run_session(
    issue: u32,
    repository: &str,
    workspace: &Workspace,
    text: &str,
    tools: Option<&[&str]>,
) -> Session {
    let mut request = recorded_request(issue, repository, workspace, text, tools);
    let solver = agent_solver();
    let mut calls = Vec::new();
    for _ in 0..40 {
        let completion = create_chat_completion_with_solver(&request, &solver);
        let message = completion.choices[0].message.clone();
        if message.tool_calls.is_empty() {
            return Session {
                calls,
                final_answer: Some(message.content.plain_text()),
                messages: request.messages,
            };
        }
        request.messages.push(message.clone());
        for call in &message.tool_calls {
            let arguments: Value =
                serde_json::from_str(&call.function.arguments).expect("JSON arguments");
            let Some((content, failed)) = workspace.answer(&call.function.name, &arguments) else {
                panic!(
                    "issue {issue}: the session planned an action outside the emulation: {} {}",
                    call.function.name, call.function.arguments
                );
            };
            calls.push(Call {
                tool: call.function.name.clone(),
                arguments,
                failed,
            });
            request.messages.push(ChatMessage::tool_result(
                call.id.clone(),
                call.function.name.clone(),
                content,
            ));
        }
    }
    panic!("issue {issue}: the session did not end within 40 steps");
}

/// The invariants every session of this issue keeps, and its evidence report.
pub(super) fn settled(session: &Session) -> Value {
    let answer = session
        .final_answer
        .as_deref()
        .expect("the session ends with an answer");
    assert!(
        !answer.starts_with("Repeated action stopped"),
        "the repetition guard must not end the session: {answer}"
    );
    let mut seen = BTreeSet::new();
    for call in &session.calls {
        let key = format!("{} {}", call.tool, call.arguments);
        assert!(
            seen.insert(key.clone()),
            "an answered action was planned again: {key}"
        );
    }
    assert_eq!(
        repeated_action_stop(&session.messages),
        None,
        "the transcript holds no repeated action"
    );
    // Issue #1560: the answer is a localized report; the record stays in the trace.
    assert!(!parses_as_json_object(answer), "{answer}");
    let report = reported_work_item_record(answer)
        .or_else(latest_work_item_record)
        .expect("a structured evidence report");
    assert_eq!(report["schema"], "repository-work-item-evidence/v1");
    assert_eq!(report["authored"], false);
    assert_eq!(report["verified"], false);
    report
}

/// The workspace-relative paths the session read, in order.
fn reads(session: &Session, workspace: &Workspace) -> Vec<String> {
    session
        .calls
        .iter()
        .filter(|call| call.tool == "read")
        .map(|call| {
            workspace
                .relative(call.arguments["filePath"].as_str().expect("a bound path"))
                .to_owned()
        })
        .collect()
}

fn recorded_session(issue: u32) -> (Session, Workspace) {
    let workspace = Workspace::checkout();
    let session = run_session(
        issue,
        "link-assistant/formal-ai",
        &workspace,
        recorded_issue(issue),
        None,
    );
    (session, workspace)
}

#[test]
fn issue_1485_linked_name_is_read_at_its_linked_path() {
    let (session, workspace) = recorded_session(1485);
    let report = settled(&session);
    assert_eq!(
        reads(&session, &workspace),
        ["scripts/hive-delivery-attempt.mjs"]
    );
    assert_eq!(report["reason"], "EvidenceCollected");
    assert_eq!(
        report["observations"][0]["reference"],
        "hive-delivery-attempt.mjs"
    );
}

#[test]
fn issue_1487_linked_name_is_read_under_js_agentic_crate() {
    let (session, workspace) = recorded_session(1487);
    settled(&session);
    let read = reads(&session, &workspace);
    assert_eq!(read[0], "js/agentic/crate/text_formalization.mjs");
    assert!(
        !read.iter().any(|file| file == "text_formalization.mjs"),
        "the bare name is not read at the workspace root: {read:?}"
    );
}

#[test]
fn issue_1507_runtime_name_is_a_term_and_other_files_are_still_read() {
    let (session, workspace) = recorded_session(1507);
    let report = settled(&session);
    let read = reads(&session, &workspace);
    assert!(
        !read
            .iter()
            .any(|file| file.rsplit('/').next() == Some("Node.js")),
        "{read:?}"
    );
    assert!(
        read.iter().any(|file| file == "js/agentic/node-host.mjs"),
        "{read:?}"
    );
    assert_eq!(report["terms"], serde_json::json!(["Node.js"]));
    assert_eq!(report["unresolved"], serde_json::json!([]));
}

#[test]
fn issues_1484_and_1488_report_other_repository_files_once() {
    for issue in [1484, 1488] {
        let (session, workspace) = recorded_session(issue);
        let report = settled(&session);
        assert!(reads(&session, &workspace).is_empty(), "issue {issue}");
        assert_eq!(report["reason"], "UnresolvedReferences", "issue {issue}");
        let unresolved = report["unresolved"].as_array().expect("unresolved items");
        assert!(!unresolved.is_empty(), "issue {issue}");
        let mut names = BTreeSet::new();
        for row in unresolved {
            assert!(
                names.insert(row["reference"].as_str().expect("a name").to_owned()),
                "issue {issue}: {row} is reported more than once"
            );
            assert_eq!(row["reason"], "OutsideRepository", "issue {issue}");
            assert_eq!(
                row["repository"], "link-assistant/hive-mind",
                "issue {issue}"
            );
        }
    }
}

fn virtual_session(
    issue: u32,
    workspace: &Workspace,
    text: &str,
    tools: Option<&[&str]>,
) -> Session {
    run_session(issue, "example/project", workspace, text, tools)
}

#[test]
fn bare_name_missing_at_the_root_is_searched_and_ranked_by_context() {
    let workspace = Workspace::with(&[
        ("scripts/runner/helper.mjs", "export const runner = 1;\n"),
        ("docs/examples/helper.mjs", "export const example = 1;\n"),
    ]);
    let session = virtual_session(
        7,
        &workspace,
        "Fix the runner\n\nThe runner script `helper.mjs` exits early; see the scripts directory.\n\n__formal_ai_exit=0\n",
        None,
    );
    let report = settled(&session);
    assert_eq!(
        session
            .calls
            .iter()
            .map(|call| call.tool.as_str())
            .collect::<Vec<_>>(),
        ["glob", "read"]
    );
    assert_eq!(reads(&session, &workspace), ["scripts/runner/helper.mjs"]);
    assert_eq!(report["observations"][0]["resolvedBy"], "ranked-name-match");
    assert_eq!(report["reason"], "EvidenceCollected");
}

#[test]
fn failed_read_is_searched_once_then_unresolved_and_the_rest_is_read() {
    let workspace = Workspace::with(&[("src/present.mjs", "export const present = 1;\n")]);
    let session = virtual_session(
        8,
        &workspace,
        "Two files\n\nCompare docs/missing-notes.md with src/present.mjs.\n\n__formal_ai_exit=0\n",
        None,
    );
    let report = settled(&session);
    assert_eq!(
        session
            .calls
            .iter()
            .map(|call| (call.tool.as_str(), call.failed))
            .collect::<Vec<_>>(),
        [("read", true), ("glob", false), ("read", false)]
    );
    assert_eq!(
        reads(&session, &workspace),
        ["docs/missing-notes.md", "src/present.mjs"]
    );
    assert_eq!(report["reason"], "UnresolvedReferences");
    assert_eq!(
        report["unresolved"],
        serde_json::json!([{"reference":"docs/missing-notes.md","target":"docs/missing-notes.md","reason":"NotFound"}])
    );
    assert_eq!(report["observations"][0]["readFailed"], true);
}

#[test]
fn without_a_glob_tool_the_search_runs_through_the_shell() {
    let workspace = Workspace::with(&[("lib/tool/config.json", "{\"a\":1}\n")]);
    let session = virtual_session(
        9,
        &workspace,
        "Config\n\nThe `config.json` of the tool is wrong.\n\n__formal_ai_exit=0\n",
        Some(&["bash", "read"]),
    );
    settled(&session);
    assert_eq!(
        session
            .calls
            .iter()
            .map(|call| call.tool.as_str())
            .collect::<Vec<_>>(),
        ["read", "bash", "read"]
    );
    assert_eq!(
        reads(&session, &workspace),
        ["config.json", "lib/tool/config.json"]
    );
}

#[test]
fn external_reference_is_reported_once_and_the_repository_file_is_read() {
    let workspace = Workspace::with(&[("src/app.mjs", "export default 1;\n")]);
    let session = virtual_session(
        10,
        &workspace,
        "Port\n\nPort [draft.mjs](https://github.com/other/upstream/blob/abc123/scripts/draft.mjs) into src/app.mjs.\n\n__formal_ai_exit=0\n",
        None,
    );
    let report = settled(&session);
    assert_eq!(reads(&session, &workspace), ["src/app.mjs"]);
    assert_eq!(
        report["unresolved"],
        serde_json::json!([{"reference":"draft.mjs","target":"scripts/draft.mjs","reason":"OutsideRepository","repository":"other/upstream"}])
    );
}

#[test]
fn hive_mind_4033_successful_read_is_not_planned_again() {
    let workspace = Workspace::with(&[
        (
            "src/solve.results.lib.mjs",
            "// results\nexport const placeholder = true;\n",
        ),
        (
            "src/pull-request-changes.lib.mjs",
            "// changes\nexport const changes = true;\n",
        ),
    ]);
    let session = run_session(
        4033,
        "link-assistant/hive-mind",
        &workspace,
        recorded_issue(4033),
        None,
    );
    let report = settled(&session);
    let read = reads(&session, &workspace);
    assert_eq!(
        read.iter()
            .filter(|file| *file == "src/solve.results.lib.mjs")
            .count(),
        1,
        "{read:?}"
    );
    assert!(
        read.iter()
            .any(|file| file == "src/pull-request-changes.lib.mjs"),
        "{read:?}"
    );
    assert!(
        report["observations"]
            .as_array()
            .expect("observations")
            .iter()
            .any(|row| row["path"] == "src/solve.results.lib.mjs" && row["complete"] == true),
        "{report}"
    );
}

#[test]
fn change_request_naming_an_existing_file_reads_it_once() {
    let workspace = Workspace::with(&[(
        "src/greeting.mjs",
        "const greeting = \"hello\";\nexport default greeting;\n",
    )]);
    let session = virtual_session(
        11,
        &workspace,
        "Rename the greeting\n\nIn src/greeting.mjs, replace «const greeting = \"hello\";» with «const greeting = \"hello, world\";». Preserve all other content.\n\n__formal_ai_exit=0\n",
        None,
    );
    let report = settled(&session);
    assert_eq!(reads(&session, &workspace), ["src/greeting.mjs"]);
    assert_eq!(report["observations"][0]["complete"], true);
}

#[test]
fn repetition_guard_stays_in_place() {
    let mut messages = vec![ChatMessage::user("Read missing.mjs")];
    for id in ["a", "b", "c"] {
        messages.push(ChatMessage::assistant_tool_calls(vec![
            formal_ai::protocol::ToolCall::function(
                id,
                "read",
                r#"{"filePath":"/ws/missing.mjs"}"#,
            ),
        ]));
        messages.push(ChatMessage::tool_result(
            id,
            "read",
            "Error: File not found: /ws/missing.mjs",
        ));
    }
    let stop = repeated_action_stop(&messages).expect("three identical failed reads stop");
    assert!(stop.starts_with("Repeated action stopped after"), "{stop}");
}
