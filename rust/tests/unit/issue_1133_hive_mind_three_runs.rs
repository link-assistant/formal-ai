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

#[cfg(unix)]
#[test]
fn generated_output_verifier_rejects_wrong_bytes_and_nonzero_processes() {
    use std::{fs, process::Command};
    let output = "Orbit $HOME {} 'quoted' 中文";
    let request = format!("Create a program in Python that prints exactly `{output}`.");
    let answer = formal_ai::UniversalSolver::default().solve(&request);
    let recipe = answer.execution_recipe.expect("composed process contract");
    let root =
        std::env::temp_dir().join(format!("formal-ai-output-contract-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join(&recipe.path), &recipe.source).unwrap();
    for file in &recipe.supporting_files {
        let path = root.join(&file.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, &file.source).unwrap();
    }
    let verify = || {
        Command::new("sh")
            .arg("-c")
            .arg(recipe.commands.last().unwrap())
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let passed = verify();
    assert!(
        passed.status.success(),
        "{}",
        String::from_utf8_lossy(&passed.stderr)
    );
    assert_eq!(
        String::from_utf8(passed.stdout).unwrap(),
        format!("{output}\n")
    );
    fs::write(
        root.join(&recipe.path),
        recipe.source.replace("Orbit", "orbit"),
    )
    .unwrap();
    assert!(
        !verify().status.success(),
        "output case is part of the contract"
    );
    fs::write(
        root.join(&recipe.path),
        format!("{}\nprint()\n", recipe.source),
    )
    .unwrap();
    assert!(!verify().status.success(), "additional newline must fail");
    fs::write(
        root.join(&recipe.path),
        format!("{}\nraise SystemExit(7)\n", recipe.source),
    )
    .unwrap();
    assert!(
        !verify().status.success(),
        "correct stdout cannot hide exit failure"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn repository_workflows_include_runtime_setup_and_output_verification() {
    for (language, setup) in [
        ("Kotlin", "kotlin"),
        ("Scala", "coursier"),
        ("Rust", "rustup"),
        ("Python", "setup-python"),
    ] {
        let mut messages = vec![ChatMessage::user(prompt())];
        let objective = page(language);
        let (planned, _) = drive(
            &AGENT_TOOLS,
            &mut messages,
            |tool, arguments| {
                if is_work_item_read(tool, arguments) {
                    objective.clone()
                } else {
                    String::new()
                }
            },
            24,
        );
        let workflow = planned
            .iter()
            .find(|call| path_of(call) == ".github/workflows/run.yml")
            .unwrap_or_else(|| panic!("missing workflow for {language}"));
        assert!(
            workflow.arguments.to_lowercase().contains(setup),
            "{language}: {workflow:?}"
        );
        let verifier = planned
            .iter()
            .find(|call| path_of(call) == "tests/verify-output.sh")
            .unwrap_or_else(|| panic!("missing output assertion for {language}"));
        assert!(verifier.arguments.contains("Hello, World!"), "{verifier:?}");
        assert!(
            workflow.arguments.contains("tests/verify-output.sh"),
            "{workflow:?}"
        );
        let commit = planned
            .iter()
            .map(command_of)
            .find(|command| command.contains("git commit"))
            .expect("commit after verification");
        assert!(
            !commit.contains("git add -A"),
            "compiler outputs and unrelated files must not be staged: {commit}"
        );
    }
}

#[test]
fn repository_requirements_bind_language_and_output_independently_of_page_chrome() {
    for (language, path) in [
        ("Kotlin", "Main.kt"),
        ("Scala", "Main.scala"),
        ("Rust", "main.rs"),
        ("Python", "main.py"),
    ] {
        let objective = format!(
            "Repository navigation: Java JavaScript\n\n## Task\nImplement a program in {language}.\n\n## Requirements\n- Print exactly: `Aster 73!`\n- Add comments and run instructions.\n- Create a GitHub Actions workflow that tests the output.\n"
        );
        let mut messages = vec![ChatMessage::user(prompt())];
        let (planned, _) = drive(
            &AGENT_TOOLS,
            &mut messages,
            |tool, arguments| {
                if is_work_item_read(tool, arguments) {
                    objective.clone()
                } else {
                    String::new()
                }
            },
            20,
        );
        let artifact = planned
            .iter()
            .find_map(|call| {
                let value: serde_json::Value = serde_json::from_str(&call.arguments).ok()?;
                (call.tool == "write" && !value["path"].as_str()?.starts_with('.')).then_some(value)
            })
            .unwrap_or_else(|| panic!("no program for {language}: {planned:?}"));
        assert_eq!(artifact["path"], path, "{language}: {artifact}");
        assert!(
            artifact["content"].as_str().unwrap().contains("Aster 73!"),
            "{artifact}"
        );
    }
}

#[test]
fn program_output_constraints_do_not_consume_filenames_or_change_case() {
    for (language, path, source_url) in [
        (
            "Rust",
            "main.rs",
            "https://doc.rust-lang.org/std/macro.println.html",
        ),
        (
            "Scala",
            "Main.scala",
            "https://docs.scala-lang.org/tour/basics.html",
        ),
        (
            "Kotlin",
            "Main.kt",
            "https://kotlinlang.org/docs/command-line.html",
        ),
    ] {
        for output in ["Hello, World!", "release.notes", "Orbit 29"] {
            let objective = format!(
                "Implement a Hello World program in {language}.\nCreate `{path}`.\nThe program should print exactly: `{output}`.\nUse `.github/workflows/check.yml` for CI."
            );
            let answer = formal_ai::UniversalSolver::default().solve(&objective);
            let recipe = answer
                .execution_recipe
                .unwrap_or_else(|| panic!("{language}: {}", answer.answer));
            assert_eq!(recipe.path, path);
            assert_eq!(
                answer.answer,
                format!(
                    "```{}\n{}```\n\nSource: {source_url}\nExecution status: not run; the client must execute and verify the program.",
                    language.to_lowercase(),
                    recipe.source
                )
            );
            assert!(
                recipe.source.contains(output),
                "{output}: {}",
                recipe.source
            );
            assert!(
                !recipe.source.contains(".github/workflows"),
                "{}",
                recipe.source
            );
        }
    }
}

#[test]
fn whole_program_request_cannot_turn_its_source_path_into_a_report_destination() {
    let messages = vec![ChatMessage::user(
        "Implement a Hello World program in Kotlin. Create Main.kt. The program must print exactly `Hello, World!`. Add clear comments and build/run instructions. Create a GitHub Actions workflow under .github/workflows/ that sets up the compiler, runs the program on push and pull requests, and asserts the exact output.",
    )];
    let step = calls(plan_chat_step(&messages, &AGENT_TOOLS));
    let source: serde_json::Value = serde_json::from_str(&step[0].arguments).unwrap();
    assert_eq!(step[0].tool, "write", "{step:?}");
    assert_eq!(source["path"], "Main.kt");
    assert!(
        source["content"].as_str().unwrap().contains("fun main("),
        "{source}"
    );
    assert!(
        source["content"]
            .as_str()
            .unwrap()
            .contains("Hello, World!"),
        "{source}"
    );
}

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

fn is_work_item_read(tool: &str, arguments: &str) -> bool {
    tool == "webfetch" || arguments.contains("gh issue view")
}

// ---- F1 and the 2026-09-16 Kotlin rerun ----------------------------------

mod extended;
