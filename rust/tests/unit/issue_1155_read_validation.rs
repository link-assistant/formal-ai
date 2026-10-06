//! Issue #1155: a work-item read is validated before it becomes page
//! evidence, and a failed read walks the fallbacks instead of ending the
//! retrieval.
//!
//! The 2026-09-27 incident (Scala run, `--tool agent`): an unauthenticated
//! `gh issue view` printed its how-to-authenticate banner, the Agent CLI
//! echoed that output with the exit status dropped (`"metadata": {"exit": 4}`
//! in its own record, plain text in the tool message), and the banner was
//! accepted as the issue body — the run then wrote its plan record and
//! answered `planned_not_executed` without ever trying the `webfetch` tool
//! the client offered. Every read the planner plans now prints its own exit
//! status as a sentinel line, the output must still have the title-and-body
//! shape of a work item, and a failed read walks the fallbacks in order —
//! the client's fetch tool, the credential-free REST read, `gh api`, then the
//! prepared pull request — before the run closes by listing every read tried
//! and its result. The full account is `docs/case-studies/issue-1155/`.

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

/// The Agent CLI's tool set in the incident run (the fetch-and-run subset its
/// `tools` array carried that matters here).
const AGENT_TOOLS: [&str; 6] = ["bash", "read", "write", "edit", "webfetch", "websearch"];
/// Codex's tool set in the sibling run: its shell, its patch tool, and the
/// operator's GitHub connector (a remote fetch, not a hosted one).
const CODEX_TOOLS: [&str; 5] = [
    "shell",
    "apply_patch",
    "update_plan",
    "mcp__codex_apps__github_fetch",
    "mcp__codex_apps__github_search",
];

const ISSUE: &str =
    "https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/issues/1";
const PR: &str =
    "https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2";

/// Hive Mind's regular work-item prompt for the incident repository.
fn solve_prompt(target: &str) -> String {
    format!(
        "Issue to solve: {target}\nYour prepared branch: issue-1-9f127a3c593c\n\
         Your prepared working directory: /tmp/hive-mind-workspace\n\nProceed.\n"
    )
}

/// The same prompt with the prepared pull request named, as Hive Mind sends
/// it once the PR exists.
fn solve_prompt_with_pr(target: &str, prepared_pr: &str) -> String {
    format!(
        "Issue to solve: {target}\nYour prepared branch: issue-1-9f127a3c593c\n\
         Your prepared working directory: /tmp/hive-mind-workspace\n\
         Your prepared Pull Request: {prepared_pr}\n\nProceed.\n"
    )
}

/// The banner an unauthenticated `gh` prints instead of an issue, exactly as
/// the incident client echoed it: no exit status, no error flag.
const GH_AUTH_BANNER: &str = "To get started with GitHub CLI, please run:  gh auth login\nAlternatively, populate the GH_TOKEN environment variable with a GitHub API authentication token.";

/// The issue body the successful reads return.
fn issue_body() -> String {
    "Implement Hello World in Rust\n\n## Task\nPlease implement a \"Hello World\" program in Rust.\n\n## Requirements\n1. Create a file with the appropriate extension for Rust\n2. The program should print exactly: `Hello, World!`\n3. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n".to_owned()
}

fn command_of(call: &PlannedToolCall) -> String {
    call.arguments
        .parse::<serde_json::Value>()
        .ok()
        .and_then(|value| {
            ["command", "cmd", "script"]
                .iter()
                .find_map(|key| value.get(*key))
                .and_then(|argument| argument.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

/// Record one planned call and the harness's result for it, as the harness
/// would echo it: the call's own arguments, and plain output with no status —
/// the incident shape.
fn record(messages: &mut Vec<ChatMessage>, id: &str, call: &PlannedToolCall, result: &str) {
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.to_owned(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id.to_owned(), &call.tool, result));
}

/// Drive a session: `results(tool, command)` says what the harness answers.
fn drive(
    tools: &[&str],
    messages: &mut Vec<ChatMessage>,
    results: impl Fn(&str, &str) -> String,
    max_steps: usize,
) -> (Vec<PlannedToolCall>, Option<String>) {
    let mut planned = Vec::new();
    for step in 0..max_steps {
        match plan_chat_step(messages, tools) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                let call = calls[0].clone();
                let result = results(&call.tool, &command_of(&call));
                record(messages, &format!("c{step}"), &call, &result);
                planned.push(call);
            }
            Some(AgenticPlan::Final(text)) => return (planned, Some(text)),
            None => return (planned, None),
        }
    }
    (planned, None)
}

/// Requirement 5, first half — the incident replay: the Agent CLI's tool set,
/// the `gh` result echoed without its exit status. The banner must not become
/// the issue body; the next plan is the `webfetch` of the issue URL.
#[test]
fn a_status_less_gh_banner_is_a_failed_read_and_webfetch_of_the_issue_follows() {
    let mut messages = vec![ChatMessage::user(solve_prompt(ISSUE))];
    let (planned, _) = drive(
        &AGENT_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "bash" && command.starts_with("gh issue view") {
                GH_AUTH_BANNER.to_owned()
            } else {
                String::new()
            }
        },
        2,
    );
    assert!(
        planned.len() >= 2,
        "the read failure must not end the run: {planned:?}"
    );
    assert_eq!(planned[1].tool, "webfetch", "{planned:?}");
    assert!(
        planned[1].arguments.contains(ISSUE),
        "the fetch fallback reads the issue URL itself: {}",
        planned[1].arguments
    );
}

/// Requirement 5, second half: with the sentinel line `__formal_ai_exit=4`
/// — the status the incident client held in its own metadata — the read is a
/// failure, and the fetch fallback follows all the same.
#[test]
fn with_the_sentinel_line_the_read_is_a_failure_and_the_fetch_follows() {
    let mut messages = vec![ChatMessage::user(solve_prompt(ISSUE))];
    let banner_with_sentinel = format!("{GH_AUTH_BANNER}\n__formal_ai_exit=4\n");
    let (planned, _) = drive(
        &AGENT_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "bash" && command.starts_with("gh issue view") {
                banner_with_sentinel.clone()
            } else {
                String::new()
            }
        },
        2,
    );
    assert!(
        planned.len() >= 2,
        "the sentinel failure must not end the run: {planned:?}"
    );
    assert_eq!(planned[1].tool, "webfetch", "{planned:?}");
    assert!(
        planned[1].arguments.contains(ISSUE),
        "the fetch fallback reads the issue URL: {}",
        planned[1].arguments
    );
}

/// A read that prints its own non-zero exit status is a failed read even when
/// the client's echo carries no error: the command's report of itself
/// outranks a status-less success echo — even for output that carries the
/// title-and-body separator.
#[test]
fn a_failing_sentinel_outranks_a_well_shaped_output() {
    let mut messages = vec![ChatMessage::user(solve_prompt(ISSUE))];
    let shaped_but_failed =
        format!("Implement Hello World in Rust\n\n{GH_AUTH_BANNER}\n__formal_ai_exit=4\n");
    let (planned, _) = drive(
        &AGENT_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "bash" && command.starts_with("gh issue view") {
                shaped_but_failed.clone()
            } else {
                String::new()
            }
        },
        2,
    );
    assert!(
        planned.len() >= 2,
        "the exit status outranks the shape: {planned:?}"
    );
    assert_eq!(planned[1].tool, "webfetch", "{planned:?}");
}

/// Requirement 3's order after the fetch: the credential-free REST read, then
/// `gh api`. The curl command needs no `gh` and no token — exactly the case
/// an unauthenticated `gh` used to end a session for.
#[test]
fn the_rest_routes_follow_the_fetch_fallback_in_order() {
    let mut messages = vec![ChatMessage::user(solve_prompt(ISSUE))];
    let (planned, _) = drive(
        &CODEX_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "shell" && command.starts_with("gh issue view") {
                format!("{GH_AUTH_BANNER}\n__formal_ai_exit=1\n")
            } else if tool == "shell" && command.starts_with("curl ") {
                "{\"message\":\"Not Found\",\"documentation_url\":\"https://docs.github.com/rest\"}\n__formal_ai_exit=22\n".to_owned()
            } else if tool == "shell" && command.starts_with("gh api ") {
                format!("{GH_AUTH_BANNER}\n__formal_ai_exit=1\n")
            } else {
                String::new()
            }
        },
        4,
    );
    assert!(planned.len() >= 4, "every route is walked: {planned:?}");
    assert_eq!(
        planned[1].tool, "mcp__codex_apps__github_fetch",
        "{planned:?}"
    );
    let rest = command_of(&planned[2]);
    assert!(
        rest.starts_with(
            "curl -fsSL -H 'Accept: application/vnd.github.raw+json' https://api.github.com/repos/"
        ),
        "the credential-free REST read follows the fetch, got `{rest}`"
    );
    assert!(
        rest.contains("/issues/1;"),
        "the REST path names the same work item, got `{rest}`"
    );
    let api = command_of(&planned[3]);
    assert!(
        api.starts_with("gh api repos/") && api.contains("/issues/1;"),
        "the authenticated REST read follows, got `{api}`"
    );
}

/// The REST fallback of a pull-request work item reads the `pulls` endpoint.
#[test]
fn the_rest_fallback_of_a_pull_request_reads_the_pulls_endpoint() {
    let mut messages = vec![ChatMessage::user(solve_prompt(PR))];
    let (planned, _) = drive(
        &CODEX_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "shell" && command.starts_with("gh pr view") {
                format!("{GH_AUTH_BANNER}\n__formal_ai_exit=1\n")
            } else {
                String::new()
            }
        },
        3,
    );
    assert!(
        planned.len() >= 3,
        "the read failure must not end the run: {planned:?}"
    );
    let rest = command_of(&planned[2]);
    assert!(
        rest.contains("/pulls/2;"),
        "a pull-request work item reads its REST `pulls` endpoint, got `{rest}`"
    );
}

/// Requirement 3's last fallback: the prepared pull request named in the
/// prompt, whose title and body restate the issue it resolves.
#[test]
fn the_prepared_pull_request_is_the_last_read_fallback() {
    let mut messages = vec![ChatMessage::user(solve_prompt_with_pr(ISSUE, PR))];
    let (planned, _) = drive(
        &AGENT_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "bash" && command.starts_with("gh ") {
                format!("{GH_AUTH_BANNER}\n__formal_ai_exit=1\n")
            } else if tool == "bash" && command.starts_with("curl ") {
                "{\"message\":\"Not Found\"}\n__formal_ai_exit=22\n".to_owned()
            } else {
                String::new()
            }
        },
        5,
    );
    assert!(
        planned.len() >= 5,
        "the prepared PR is tried last: {planned:?}"
    );
    assert_eq!(planned[4].tool, "webfetch", "{planned:?}");
    assert!(
        planned[4].arguments.contains(PR) && !planned[4].arguments.contains(ISSUE),
        "the last fallback reads the prepared PR's own page: {}",
        planned[4].arguments
    );
}

/// Requirement 4: once every route has been tried and failed, the run closes
/// by reporting each read with its result — and never spends its only write
/// on a plan record for an issue it never read.
#[test]
fn an_exhausted_retrieval_reports_every_read_and_writes_no_plan_record() {
    let mut messages = vec![ChatMessage::user(solve_prompt(ISSUE))];
    let (planned, answer) = drive(
        &CODEX_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "shell" && command.starts_with("gh issue view") {
                format!("{GH_AUTH_BANNER}\n__formal_ai_exit=1\n")
            } else if tool == "shell" && command.starts_with("curl ") {
                // `curl -f` answers a 404 with its error JSON and exit 22.
                "{\"message\":\"Not Found\",\"documentation_url\":\"https://docs.github.com/rest\"}\n__formal_ai_exit=22\n".to_owned()
            } else if tool == "shell" && command.starts_with("gh api ") {
                format!("{GH_AUTH_BANNER}\n__formal_ai_exit=1\n")
            } else {
                // The connector answers nothing usable for the page.
                String::new()
            }
        },
        10,
    );
    let answer = answer.expect("the exhausted retrieval closes with a report");
    assert!(
        answer.contains("could not be read"),
        "the report says the work item was not read: {answer}"
    );
    assert!(
        answer.contains("gh issue view"),
        "the report lists the gh read: {answer}"
    );
    assert!(
        answer.contains("curl -fsSL"),
        "the report lists the REST read: {answer}"
    );
    assert!(
        answer.contains("gh api "),
        "the report lists the authenticated REST read: {answer}"
    );
    assert!(
        answer.contains("exit status 1"),
        "each read carries its result: {answer}"
    );
    assert_eq!(
        planned.iter().filter(|call| call.tool == "shell").count(),
        3,
        "exactly the three shell reads were planned: {planned:?}"
    );
    for call in &planned {
        assert!(
            !call
                .arguments
                .contains(".formal-ai/general-change-plan.lino"),
            "the only write was not spent on a plan record: {}",
            call.arguments
        );
    }
}

/// A successful REST read delivers the issue text and drives execution — the
/// sentinel line it prints about itself is not part of that text.
#[test]
fn a_successful_curl_read_drives_execution_and_the_sentinel_stays_out_of_the_text() {
    let mut messages = vec![ChatMessage::user(solve_prompt(ISSUE))];
    let body_with_sentinel = format!("{}\n__formal_ai_exit=0\n", issue_body());
    let (planned, _) = drive(
        &CODEX_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "shell" && command.starts_with("gh issue view") {
                GH_AUTH_BANNER.to_owned()
            } else if tool == "shell" && command.starts_with("curl ") {
                body_with_sentinel.clone()
            } else if tool == "shell" {
                "Hello, World!\n".to_owned()
            } else {
                String::new()
            }
        },
        12,
    );
    assert!(
        planned.iter().any(|call| call.tool == "apply_patch"),
        "the read body must drive the work-item execution, got {planned:?}"
    );
    for call in &planned {
        if call.tool == "apply_patch" {
            assert!(
                !call.arguments.contains("__formal_ai_exit"),
                "the sentinel is plumbing, not issue text: {}",
                call.arguments
            );
        }
    }
}

/// A read that succeeds through the sentinel closes the retrieval loop: the
/// next plan after the successful `gh` read is execution, never another read.
#[test]
fn a_sentinel_success_is_page_evidence_and_never_replanned() {
    let mut messages = vec![ChatMessage::user(solve_prompt(ISSUE))];
    let body_with_sentinel = format!("{}\n__formal_ai_exit=0\n", issue_body());
    let (planned, _) = drive(
        &CODEX_TOOLS,
        &mut messages,
        |tool, command| {
            if tool == "shell" && command.starts_with("gh issue view") {
                body_with_sentinel.clone()
            } else if tool == "shell" {
                "Hello, World!\n".to_owned()
            } else {
                String::new()
            }
        },
        12,
    );
    assert!(
        planned.iter().any(|call| call.tool == "apply_patch"),
        "the sentinel-carrying success drives execution: {planned:?}"
    );
    assert_eq!(
        planned
            .iter()
            .filter(|call| command_of(call).starts_with("gh issue view"))
            .count(),
        1,
        "the successful read is planned exactly once: {planned:?}"
    );
}
