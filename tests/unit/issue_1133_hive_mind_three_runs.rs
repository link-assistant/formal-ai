//! Issue #1133: the three 2026-09-13 Hive Mind Hello World runs, replayed.
//!
//! Each test drives `plan_chat_step` turn by turn with the tool set one Hive
//! Mind tool advertised and the results its harness returned, and pins what the
//! planner does now where it went wrong then. The full account is
//! `docs/case-studies/hive-mind-hello-world/2026-09-13-three-runs.md`.

use formal_ai::agentic_coding::general_planner::compose_edit_request;
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

/// Claude Code's tool set in the Kotlin run: every playwright tool attached.
const CLAUDE_TOOLS: [&str; 20] = [
    "Task",
    "Bash",
    "Edit",
    "Read",
    "WebFetch",
    "WebSearch",
    "Write",
    "mcp__playwright__browser_click",
    "mcp__playwright__browser_close",
    "mcp__playwright__browser_drag",
    "mcp__playwright__browser_evaluate",
    "mcp__playwright__browser_fill_form",
    "mcp__playwright__browser_hover",
    "mcp__playwright__browser_navigate",
    "mcp__playwright__browser_press_key",
    "mcp__playwright__browser_select_option",
    "mcp__playwright__browser_snapshot",
    "mcp__playwright__browser_take_screenshot",
    "mcp__playwright__browser_type",
    "mcp__playwright__browser_wait_for",
];
/// The Agent CLI's tool set in the Scala run.
const AGENT_TOOLS: [&str; 14] = [
    "bash",
    "batch",
    "codesearch",
    "edit",
    "glob",
    "grep",
    "list",
    "read",
    "task",
    "todoread",
    "todowrite",
    "webfetch",
    "websearch",
    "write",
];
/// Codex's tool set in the Rust run: its own tools plus the operator's `ChatGPT`
/// GitHub connector.
const CODEX_TOOLS: [&str; 5] = [
    "shell",
    "apply_patch",
    "update_plan",
    "mcp__codex_apps__github_fetch",
    "mcp__codex_apps__github_search",
];

const ISSUE: &str =
    "https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/issues/1";
const PR: &str =
    "https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2";
const BRANCH: &str = "issue-1-1f3e3886bcb8";

/// Hive Mind's formal-ai prompt, verbatim.
fn prompt() -> String {
    format!(
        "Resolve the GitHub issue at {ISSUE} in this repository.\nKeep the solution on branch {BRANCH}.\nUpdate the pull request at {PR}.\n\nImplement and verify the solution before reporting completion.\nProceed.\n"
    )
}

/// What `webfetch` returned for the issue page: the GitHub title line, some
/// page chrome, then the issue body.
fn page(language: &str) -> String {
    format!(
        "Implement Hello World in {language} · Issue #1 · konard/test-hello-world · GitHub\n\n  Skip to content\n\nNavigation MenuSign in\n\n## Task\nPlease implement a \"Hello World\" program in {language}.\n\n## Requirements\n1. Create a file with the appropriate extension for {language}\n2. The program should print exactly: `Hello, World!`\n3. Add clear comments explaining the code\n6. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n"
    )
}

fn calls(plan: Option<AgenticPlan>) -> Vec<PlannedToolCall> {
    match plan {
        Some(AgenticPlan::ToolCalls(calls)) => calls,
        other => panic!("expected tool calls, got {other:?}"),
    }
}

fn final_text(plan: Option<AgenticPlan>) -> String {
    match plan {
        Some(AgenticPlan::Final(text)) => text,
        other => panic!("expected a final answer, got {other:?}"),
    }
}

/// Record one planned call and the harness's result for it, as the harness
/// would echo it back: `echoed` is the call's arguments after projection.
fn record(
    messages: &mut Vec<ChatMessage>,
    id: &str,
    call: &PlannedToolCall,
    echoed: &str,
    result: &str,
) {
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.to_owned(),
        call.tool.clone(),
        echoed.to_owned(),
    )]));
    messages.push(ChatMessage::tool_result(id.to_owned(), &call.tool, result));
}

/// Drive a session: `results(tool, arguments)` says what the harness answers.
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
                let result = results(&call.tool, &call.arguments);
                record(
                    messages,
                    &format!("c{step}"),
                    &call,
                    &call.arguments,
                    &result,
                );
                planned.push(call);
            }
            Some(AgenticPlan::Final(text)) => return (planned, Some(text)),
            None => return (planned, None),
        }
    }
    (planned, None)
}

fn command_of(call: &PlannedToolCall) -> String {
    serde_json::from_str::<serde_json::Value>(&call.arguments)
        .ok()
        .and_then(|value| value.get("command")?.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn path_of(call: &PlannedToolCall) -> String {
    serde_json::from_str::<serde_json::Value>(&call.arguments)
        .ok()
        .and_then(|value| value.get("path")?.as_str().map(str::to_owned))
        .unwrap_or_default()
}

// ---- F1: the Kotlin run ---------------------------------------------------

#[test]
fn the_claude_code_tool_set_fetches_the_issue_with_webfetch_not_a_browser_click() {
    let messages = vec![ChatMessage::user(prompt())];
    let calls = calls(plan_chat_step(&messages, CLAUDE_TOOLS.as_slice()));
    assert_eq!(calls[0].tool, "WebFetch", "{calls:?}");
    assert!(calls[0].arguments.contains(ISSUE), "{calls:?}");
}

#[test]
fn a_fetch_the_harness_echoed_without_its_url_is_not_planned_again() {
    // Before the fix the harness echoed `{"target":""}` for the click and the
    // planner, finding no `url` in it, planned the same fetch forever.
    let mut messages = vec![ChatMessage::user(prompt())];
    let click = PlannedToolCall {
        tool: "mcp__playwright__browser_click".to_owned(),
        arguments: String::new(),
    };
    let error =
        "Error: browserBackend.callTool: Unexpected token \"\" while parsing css selector \"\".";
    record(&mut messages, "c0", &click, "{\"target\":\"\"}", error);
    record(&mut messages, "c1", &click, "{\"target\":\"\"}", error);
    let tools = [
        "Bash",
        "Write",
        "mcp__playwright__browser_click",
        "mcp__playwright__browser_navigate",
    ];
    match plan_chat_step(&messages, tools.as_slice()) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert!(
                calls
                    .iter()
                    .all(|call| !call.tool.starts_with("mcp__playwright__")),
                "a browser tool is not a fetch tool: {calls:?}"
            );
        }
        Some(AgenticPlan::Final(_)) | None => {}
    }
}

#[test]
fn a_tool_that_failed_twice_with_the_same_report_is_not_called_a_third_time() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let fetch = PlannedToolCall {
        tool: "WebFetch".to_owned(),
        arguments: format!("{{\"url\":\"{ISSUE}\",\"format\":\"text\"}}"),
    };
    let error = "Error: fetch failed: connect ECONNREFUSED";
    record(&mut messages, "c0", &fetch, &fetch.arguments, error);
    record(&mut messages, "c1", &fetch, &fetch.arguments, error);
    match plan_chat_step(&messages, CLAUDE_TOOLS.as_slice()) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert!(
                calls.iter().all(|call| call.tool != "WebFetch"),
                "{calls:?}"
            );
        }
        Some(AgenticPlan::Final(answer)) => {
            assert!(!answer.contains("Created and verified"), "{answer}");
        }
        None => panic!("the work item still has a plan after a failed read"),
    }
}

// ---- F2 + F3: the Scala run ----------------------------------------------

#[test]
fn a_failing_verification_command_is_reported_as_the_failed_step_not_as_verified() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let scala = page("Scala");
    let (planned, answer) = drive(
        AGENT_TOOLS.as_slice(),
        &mut messages,
        |tool, _| match tool {
            "webfetch" => scala.clone(),
            // The Agent CLI hands the model the process text only; `exit: 127`
            // stays in metadata it never sends.
            "bash" => "/bin/sh: 1: scalac: not found\n".to_owned(),
            _ => String::new(),
        },
        8,
    );
    let answer = answer.expect("the run must end in a final answer");
    assert!(
        planned
            .iter()
            .any(|call| call.tool == "write" && path_of(call) == "Main.scala"),
        "{planned:?}"
    );
    assert!(!answer.contains("Created and verified"), "{answer}");
    assert!(
        answer.contains("scalac Main.scala") && answer.contains("not found"),
        "{answer}"
    );
}

#[test]
fn a_verified_work_item_is_committed_and_pushed_to_the_named_branch() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let scala = page("Scala");
    let (planned, answer) = drive(
        AGENT_TOOLS.as_slice(),
        &mut messages,
        |tool, arguments| match tool {
            "webfetch" => scala.clone(),
            "bash" if arguments.contains("git push") => "abc123def\n".to_owned(),
            "bash" if arguments.contains("scala Main") => "Hello, World!\n".to_owned(),
            _ => String::new(),
        },
        10,
    );
    let commit = planned
        .iter()
        .map(command_of)
        .find(|command| command.starts_with("git add -A && git commit"))
        .expect("the recipe ends with a commit step");
    assert!(
        commit.contains(&format!("git push -q origin {BRANCH}")),
        "{commit}"
    );
    assert!(
        commit.contains("feat: add Main.scala") && commit.contains(ISSUE),
        "{commit}"
    );
    let answer = answer.expect("final answer");
    assert!(
        answer.contains("Committed and pushed") && answer.contains("abc123def"),
        "{answer}"
    );
    // The verification commands come before the commit, never after it.
    let commands: Vec<String> = planned
        .iter()
        .map(command_of)
        .filter(|c| !c.is_empty())
        .collect();
    assert_eq!(
        commands
            .last()
            .map(String::as_str)
            .map(|c| c.starts_with("git add")),
        Some(true),
        "{commands:?}"
    );
}

#[test]
fn a_work_item_that_asks_for_a_workflow_writes_one_with_the_verified_commands() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let scala = page("Scala");
    let (planned, _) = drive(
        AGENT_TOOLS.as_slice(),
        &mut messages,
        |tool, _| {
            if tool == "webfetch" {
                scala.clone()
            } else {
                "Hello, World!\n".to_owned()
            }
        },
        10,
    );
    let workflow = planned
        .iter()
        .find(|call| call.tool == "write" && path_of(call) == ".github/workflows/run.yml")
        .expect("the workflow is written beside the program");
    assert!(
        workflow.arguments.contains("pull_request")
            && workflow.arguments.contains("scalac Main.scala"),
        "{workflow:?}"
    );
}

#[test]
fn a_request_to_commit_the_tree_is_a_git_step_not_a_web_search() {
    let restart = "🔄 Auto-restart: resume the previous session and handle its uncommitted changes.\n\nUncommitted files (1):\n?? Main.scala\n\nChanges summary:\nNo tracked-file diff summary available.\n\nPlease review these changes and commit them with an appropriate commit message.\nFollow the repository's commit message conventions from previous commits.";
    let messages = vec![ChatMessage::user(restart)];
    let calls = calls(plan_chat_step(&messages, AGENT_TOOLS.as_slice()));
    assert_eq!(calls[0].tool, "bash", "{calls:?}");
    let command = command_of(&calls[0]);
    assert!(
        command.starts_with("git add -A && git commit -q -m 'feat: add Main.scala'"),
        "{command}"
    );
    assert!(command.contains("git push -q origin HEAD"), "{command}");
}

// ---- F4: the Rust run -----------------------------------------------------

#[test]
fn codex_reads_the_issue_through_gh_when_the_only_fetch_tool_is_a_remote_connector() {
    let messages = vec![ChatMessage::user(prompt())];
    let calls = calls(plan_chat_step(&messages, CODEX_TOOLS.as_slice()));
    assert_eq!(calls[0].tool, "shell", "{calls:?}");
    let command = command_of(&calls[0]);
    assert!(
        command.starts_with(&format!("gh issue view {ISSUE} --json title")),
        "{command}"
    );
}

#[test]
fn codex_plans_the_work_item_and_never_narrates_the_connector_placeholder() {
    let mut messages = vec![ChatMessage::user(prompt().replace("Scala", "Rust"))];
    let rust = page("Rust");
    let (planned, answer) = drive(
        CODEX_TOOLS.as_slice(),
        &mut messages,
        |tool, arguments| match tool {
            "shell" if arguments.contains("gh issue view") => rust.clone(),
            "shell" if arguments.contains("git push") => "0123abcd\n".to_owned(),
            "shell" => "Hello, World!\n".to_owned(),
            _ => String::new(),
        },
        10,
    );
    assert!(
        planned.iter().any(|call| call.tool == "apply_patch"),
        "{planned:?}"
    );
    let answer = answer.expect("final answer");
    assert!(!answer.contains("command completed. Output"), "{answer}");
    assert!(answer.contains("Committed and pushed"), "{answer}");
}

#[test]
fn a_connector_result_keeps_its_structured_content() {
    // The connector answered `Action completed.` in `content` and the issue in
    // `structuredContent`; the issue is what the planner reads.
    let mut messages = vec![ChatMessage::user(prompt().replace("Scala", "Rust"))];
    let fetch = PlannedToolCall {
        tool: "mcp__codex_apps__github_fetch".to_owned(),
        arguments: format!("{{\"url\":\"{ISSUE}\"}}"),
    };
    let envelope = "{\"content\":[{\"type\":\"text\",\"text\":\"Action completed.\"}],\"structuredContent\":{\"content\":\"{\\\"number\\\":1,\\\"title\\\":\\\"Implement Hello World in Rust\\\",\\\"body\\\":\\\"## Task\\\\nPlease implement a Hello World program in Rust.\\\"}\"}}";
    record(&mut messages, "c0", &fetch, &fetch.arguments, envelope);
    let tools = ["write", "bash", "mcp__codex_apps__github_fetch"];
    let calls = calls(plan_chat_step(&messages, tools.as_slice()));
    assert_eq!(calls[0].tool, "write", "{calls:?}");
    assert!(calls[0].arguments.contains("fn main()"), "{calls:?}");
}

// ---- F5: the harness summary envelope ------------------------------------

#[test]
fn a_text_to_summarize_envelope_is_summarized_never_executed() {
    let envelope = format!(
        "\n              The following is the text to summarize:\n              <text>\n              {}              </text>\n            ",
        prompt()
    );
    let messages = vec![ChatMessage::user(envelope)];
    let summary = final_text(plan_chat_step(&messages, AGENT_TOOLS.as_slice()));
    assert_eq!(
        summary,
        format!("Resolve the GitHub issue at {ISSUE} in this repository.")
    );
}

// ---- F6: #1115 / #1116 ------------------------------------------------------

#[test]
fn an_additive_edit_composes_to_an_edit_of_its_anchor_line() {
    let (target, old, new) = compose_edit_request(
        "Edit the tracked file self-development-status.yml: add a line \"  pull_request:\" directly after the line \"on:\"",
    )
    .expect("an additive edit composes");
    assert_eq!(target, "self-development-status.yml");
    assert_eq!(old, "on:");
    assert_eq!(new, "on:\n  pull_request:");
}

#[test]
fn an_escaped_newline_in_edit_prose_is_a_newline() {
    let (_, old, new) = compose_edit_request(
        "In the file self-development-status.yml replace \"  schedule:\" with \"  pull_request:\\n  schedule:\"",
    )
    .expect("a replacement composes");
    assert_eq!(old, "  schedule:");
    assert_eq!(new, "  pull_request:\n  schedule:");
}

#[test]
fn an_edit_instruction_is_never_routed_to_web_search() {
    let messages = vec![ChatMessage::user(
        "Edit the tracked file self-hosting-attribution.rs: add \"Gemfile.lock\" to the LOCKFILE_NAMES list.",
    )];
    if let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, AGENT_TOOLS.as_slice()) {
        assert!(
            calls
                .iter()
                .all(|call| call.tool != "websearch" && call.tool != "webfetch"),
            "{calls:?}"
        );
    }
}

// ---- The seeded words, in every registered language ------------------------

/// A request to commit, in each language: English, Russian, Hindi, Chinese,
/// Spanish. Each is a git step.
#[test]
fn a_commit_request_is_a_git_step_in_every_language() {
    let requests = [
        (
            "english",
            "Please review these changes and commit them.\n?? Main.scala",
        ),
        (
            "russian",
            "Проверь изменения и закоммить изменения.\n?? Main.scala",
        ),
        (
            "hindi",
            "इन परिवर्तनों की समीक्षा करें और बदलाव कमिट करें।\n?? Main.scala",
        ),
        ("chinese", "请检查并提交这些更改。\n?? Main.scala"),
        ("spanish", "Revisa los cambios y haz commit.\n?? Main.scala"),
    ];
    for (language, request) in requests {
        let messages = vec![ChatMessage::user(request)];
        let calls = calls(plan_chat_step(&messages, AGENT_TOOLS.as_slice()));
        assert_eq!(calls[0].tool, "bash", "{language}: {calls:?}");
        assert!(
            command_of(&calls[0]).contains("git commit"),
            "{language}: {calls:?}"
        );
    }
}

/// The branch cue in each language names where the work item is pushed.
#[test]
fn the_named_branch_is_pushed_to_in_every_language() {
    let prompts = [
        (
            "english",
            format!(
                "Resolve the GitHub issue at {ISSUE} in this repository.\nKeep the solution on branch {BRANCH}."
            ),
        ),
        (
            "russian",
            format!(
                "Реши задачу GitHub {ISSUE} в этом репозитории.\nДержи решение в ветке {BRANCH}."
            ),
        ),
        (
            "hindi",
            format!("इस रिपॉज़िटरी में GitHub इश्यू {ISSUE} को हल करें।\nसमाधान को शाखा {BRANCH} पर रखें।"),
        ),
        (
            "chinese",
            format!("在此仓库中解决 GitHub 问题 {ISSUE}。\n将解决方案保留在分支 {BRANCH} 上。"),
        ),
        (
            "spanish",
            format!(
                "Resuelve el issue de GitHub {ISSUE} en este repositorio.\nMantén la solución en la rama {BRANCH}."
            ),
        ),
    ];
    let scala = page("Scala");
    for (language, request) in prompts {
        let mut messages = vec![ChatMessage::user(request)];
        let (planned, _) = drive(
            AGENT_TOOLS.as_slice(),
            &mut messages,
            |tool, _| {
                if tool == "webfetch" {
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
            .find(|command| command.starts_with("git add -A && git commit"))
            .unwrap_or_else(|| panic!("{language}: no commit step in {planned:?}"));
        assert!(
            commit.contains(&format!("git push -q origin {BRANCH}")),
            "{language}: {commit}"
        );
    }
}

/// A workflow requirement in each language adds the workflow file.
#[test]
fn a_workflow_requirement_adds_the_workflow_in_every_language() {
    let bodies = [
        (
            "english",
            "Please implement a Hello World program in Scala. Create a GitHub Actions workflow that runs it.",
        ),
        (
            "russian",
            "Реализуй программу Hello World на Scala. Создай рабочий процесс GitHub Actions, который её запускает.",
        ),
        (
            "hindi",
            "Scala में Hello World प्रोग्राम लागू करें। एक GitHub Actions वर्कफ़्लो बनाएँ जो उसे चलाए।",
        ),
        (
            "chinese",
            "请用 Scala 实现 Hello World 程序。创建一个运行它的 GitHub Actions 工作流。",
        ),
        (
            "spanish",
            "Implementa un programa Hello World en Scala. Crea un flujo de trabajo de GitHub Actions que lo ejecute.",
        ),
    ];
    for (language, body) in bodies {
        let mut messages = vec![ChatMessage::user(prompt())];
        let (planned, _) = drive(
            AGENT_TOOLS.as_slice(),
            &mut messages,
            |tool, _| {
                if tool == "webfetch" {
                    body.to_owned()
                } else {
                    "Hello, World!\n".to_owned()
                }
            },
            10,
        );
        assert!(
            planned
                .iter()
                .any(|call| call.tool == "write" && path_of(call) == ".github/workflows/run.yml"),
            "{language}: {planned:?}"
        );
    }
}

/// The position cues in each language place the new line after or before its
/// anchor.
#[test]
fn positional_inserts_compose_in_every_language() {
    let after = [
        (
            "english",
            "In config.yml add the line \"  pull_request:\" directly after the line \"on:\"",
        ),
        (
            "russian",
            "В файле config.yml добавь строку \"  pull_request:\" сразу после строки \"on:\"",
        ),
        (
            "hindi",
            "config.yml में पंक्ति \"on:\" के बाद \"  pull_request:\" जोड़ें",
        ),
        (
            "chinese",
            "在 config.yml 中，在 \"on:\" 行后添加 \"  pull_request:\"",
        ),
        (
            "spanish",
            "En config.yml añade la línea \"  pull_request:\" justo después de la línea \"on:\"",
        ),
    ];
    for (language, request) in after {
        let (target, old, new) =
            compose_edit_request(request).unwrap_or_else(|| panic!("{language}: {request}"));
        assert_eq!(target, "config.yml", "{language}");
        assert_eq!(old, "on:", "{language}");
        assert!(
            new == "on:\n  pull_request:" || new == "  pull_request:\non:",
            "{language}: {new:?}"
        );
    }
    let before = [
        (
            "english",
            "In config.yml insert \"# generated\" right before the line \"on:\"",
        ),
        (
            "russian",
            "В файле config.yml вставь \"# generated\" прямо перед строкой \"on:\"",
        ),
        (
            "spanish",
            "En config.yml inserta \"# generated\" justo antes de la línea \"on:\"",
        ),
    ];
    for (language, request) in before {
        let (_, old, new) =
            compose_edit_request(request).unwrap_or_else(|| panic!("{language}: {request}"));
        assert_eq!(old, "on:", "{language}");
        assert_eq!(new, "# generated\non:", "{language}");
    }
}
