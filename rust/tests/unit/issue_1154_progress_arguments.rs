//! Issue #1154: the Codex shell-tool argument key, and the repeat invariant.
//!
//! Codex's `exec_command` sends the command under `cmd`; `progress.rs` read
//! only `command`, so the work-item read was never recorded as attempted and
//! the planner re-planned the identical `gh issue view` 78 times (6.2M input
//! tokens, 2026-09-27 Rust run). The replay below uses the transcript's own
//! first two Responses items. The full account is
//! `docs/case-studies/issue-1154/`.

use formal_ai::ResponsesRequest;
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use serde_json::json;

/// Codex's tool set in the 2026-09-27 Rust run.
const CODEX_TOOLS: [&str; 5] = [
    "shell",
    "apply_patch",
    "update_plan",
    "mcp__codex_apps__github_fetch",
    "mcp__codex_apps__github_search",
];

const ISSUE: &str =
    "https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/issues/1";

/// The work-item prompt Hive Mind sent (its regular user prompt shape).
fn solve_prompt() -> String {
    format!(
        "Issue to solve: {ISSUE}\nYour prepared branch: issue-1-9f127a3c593c\n\
         Your prepared working directory: /tmp/hive-mind-workspace\n\nProceed.\n"
    )
}

/// The exact first Responses pair of the transcript: the `exec_command` call
/// Codex logged, with its `cmd` argument key, and the title-and-body output it
/// returned.
fn codex_read_pair() -> serde_json::Value {
    let command = format!(
        "gh issue view '{ISSUE}' --json title --jq .title && echo && \
         gh issue view '{ISSUE}' --json body --jq .body"
    );
    json!([
        { "type": "message", "role": "user", "content": solve_prompt() },
        { "type": "function_call", "call_id": "fc_1", "name": "exec_command", "arguments": {
            "cmd": command
        } },
        { "type": "function_call_output", "call_id": "fc_1", "output":
            "Implement Hello World in Rust\n\n## Task\nPlease implement a \"Hello World\" program in Rust.\n\n## Requirements\n1. Create a file with the appropriate extension for Rust\n2. The program should print exactly: `Hello, World!`\n3. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n"
        }
    ])
}

/// The shared `ChatMessage` list a Responses request replays into.
fn replayed_messages(input: &serde_json::Value) -> Vec<formal_ai::ChatMessage> {
    let request: ResponsesRequest = serde_json::from_value(json!({
        "model": "formal-ai",
        "input": input,
    }))
    .expect("responses envelope");
    request.to_chat_completion_request().messages
}

fn calls(plan: Option<AgenticPlan>) -> Vec<PlannedToolCall> {
    match plan {
        Some(AgenticPlan::ToolCalls(calls)) => calls,
        other => panic!("expected tool calls, got {other:?}"),
    }
}

/// Requirement 3 and the replay half of requirement 4: after the `cmd`-keyed
/// read returns the issue, the next plan is driven by the fetched text -- the
/// artifact write of the work-item execution path -- and is never another
/// `gh issue view`.
#[test]
fn codex_cmd_read_drives_execution_instead_of_repeating() {
    let messages = replayed_messages(&codex_read_pair());
    let planned = calls(plan_chat_step(&messages, &CODEX_TOOLS));
    assert!(
        !planned.is_empty(),
        "the fetched issue must drive a work-item execution step"
    );
    for call in &planned {
        let command = serde_json::from_str::<serde_json::Value>(&call.arguments)
            .ok()
            .and_then(|value| {
                ["command", "cmd", "script"].iter().find_map(|key| {
                    value
                        .get(*key)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
            })
            .unwrap_or_default();
        assert!(
            !command.starts_with("gh issue view"),
            "the read already succeeded under the `cmd` key; re-planning it is the 78-times loop, got `{command}`"
        );
    }
}

/// Requirement 4's table: every argument spelling a client uses records the
/// same work-item read, so the planner never re-plans it under another name.
#[test]
fn every_argument_spelling_records_the_same_read() {
    let spellings = [
        r#"{ "command": "gh issue view 'ISSUE' --json title --jq .title" }"#,
        r#"{ "cmd": "gh issue view 'ISSUE' --json title --jq .title" }"#,
        r#"{ "script": "gh issue view 'ISSUE' --json title --jq .title" }"#,
        r#"{ "command": ["gh", "issue", "view", "ISSUE", "--json", "title"] }"#,
    ];
    for spelling in spellings {
        let arguments = spelling.replace("ISSUE", ISSUE);
        let request: ResponsesRequest = serde_json::from_value(json!({
            "model": "formal-ai",
            "input": [
                { "type": "message", "role": "user", "content": solve_prompt() },
                { "type": "function_call", "call_id": "fc_1", "name": "exec_command",
                  "arguments": arguments },
                { "type": "function_call_output", "call_id": "fc_1",
                  "output": "Implement Hello World in Rust\n\n## Task\nA Hello World program in Rust.\n2. The program should print exactly: `Hello, World!`\n" }
            ]
        }))
        .expect("responses envelope");
        let messages = request.to_chat_completion_request().messages;
        let planned = calls(plan_chat_step(&messages, &CODEX_TOOLS));
        assert!(
            !planned.is_empty(),
            "spelling `{spelling}` must still let the fetched text drive execution"
        );
        for call in &planned {
            assert!(
                !call.arguments.to_lowercase().contains("gh issue view"),
                "spelling `{spelling}` re-planned the read: {}",
                call.arguments
            );
        }
    }
}

/// Requirement 2's invariant, replayed on the loop's own transcript shape:
/// even when a transcript already carries the same successful read twice
/// (what the 2026-09-27 run accumulated), no third identical call is planned.
#[test]
fn repeated_successful_read_is_reported_not_replanned() {
    let mut items = codex_read_pair();
    let array = items.as_array_mut().expect("items");
    let first_call = array[1].clone();
    let first_output = array[2].clone();
    array.push(first_call);
    array.push(first_output);
    let messages = replayed_messages(&items);
    match plan_chat_step(&messages, &CODEX_TOOLS) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            for call in &calls {
                assert!(
                    !call.arguments.to_lowercase().contains("gh issue view"),
                    "a third identical read was planned: {}",
                    call.arguments
                );
            }
        }
        Some(AgenticPlan::Final(text)) => {
            assert!(
                text.contains("gh issue view") || text.contains("already completed"),
                "the stuck report names the repeated step: {text}"
            );
        }
        None => panic!("the twice-read work item must still be planned"),
    }
}
