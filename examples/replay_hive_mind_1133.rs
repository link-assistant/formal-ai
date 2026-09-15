//! Replay the three 2026-09-13 Hive Mind runs against the local planner.
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

const CLAUDE: &[&str] = &[
    "Task",
    "Bash",
    "DesignSync",
    "Edit",
    "ListAgents",
    "Read",
    "ReportFindings",
    "SendMessage",
    "Skill",
    "TaskOutput",
    "TaskStop",
    "WebFetch",
    "WebSearch",
    "Workflow",
    "Write",
    "mcp__playwright__browser_click",
    "mcp__playwright__browser_close",
    "mcp__playwright__browser_console_messages",
    "mcp__playwright__browser_drag",
    "mcp__playwright__browser_drop",
    "mcp__playwright__browser_evaluate",
    "mcp__playwright__browser_file_upload",
    "mcp__playwright__browser_fill_form",
    "mcp__playwright__browser_find",
    "mcp__playwright__browser_handle_dialog",
    "mcp__playwright__browser_hover",
    "mcp__playwright__browser_navigate",
    "mcp__playwright__browser_navigate_back",
    "mcp__playwright__browser_network_request",
    "mcp__playwright__browser_network_requests",
    "mcp__playwright__browser_press_key",
    "mcp__playwright__browser_resize",
    "mcp__playwright__browser_run_code_unsafe",
    "mcp__playwright__browser_select_option",
    "mcp__playwright__browser_snapshot",
    "mcp__playwright__browser_tabs",
    "mcp__playwright__browser_take_screenshot",
    "mcp__playwright__browser_type",
    "mcp__playwright__browser_wait_for",
];
const AGENT: &[&str] = &[
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
const CODEX: &[&str] = &[
    "shell",
    "apply_patch",
    "update_plan",
    "view_image",
    "mcp__codex_apps__github_fetch",
    "mcp__codex_apps__github_search",
];

const ISSUE: &str =
    "https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/issues/1";
const PR: &str =
    "https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2";
const BODY: &str = "## Task\nPlease implement a \"Hello World\" program in Scala.\n\n## Requirements\n1. Create a file with the appropriate extension for Scala\n2. The program should print exactly: `Hello, World!`\n3. Add clear comments explaining the code\n4. Ensure the code follows Scala best practices and idioms\n5. If applicable, include build/run instructions in a comment at the top of the file\n6. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n";

fn prompt() -> String {
    format!(
        "Resolve the GitHub issue at {ISSUE} in this repository.\nKeep the solution on branch issue-1-1f3e3886bcb8.\nUpdate the pull request at {PR}.\n\nImplement and verify the solution before reporting completion.\nProceed.\n"
    )
}
fn page() -> String {
    format!(
        "Implement Hello World in Scala · Issue #1 · konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b · GitHub\n\n\n  Skip to content\n\nNavigation MenuSign in\n\n{BODY}"
    )
}

fn show(p: Option<&AgenticPlan>) -> String {
    match p {
        Some(AgenticPlan::ToolCalls(c)) => c
            .iter()
            .map(|x| {
                format!(
                    "{}({})",
                    x.tool,
                    x.arguments.chars().take(160).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join(" | "),
        Some(AgenticPlan::Final(t)) => format!(
            "FINAL: {}",
            t.chars().take(400).collect::<String>().replace('\n', "⏎")
        ),
        None => "None".into(),
    }
}

fn drive(
    label: &str,
    tools: &[&str],
    msgs: &mut Vec<ChatMessage>,
    results: &[(&str, &str)],
    max: usize,
) {
    println!("\n=== {label}");
    let mut n = 0;
    loop {
        let planned = plan_chat_step(msgs, tools);
        println!("  step {n}: {}", show(planned.as_ref()));
        let Some(AgenticPlan::ToolCalls(calls)) = planned else {
            break;
        };
        let call = calls[0].clone();
        let id = format!("c{n}");
        // Record what the harness would echo back for this tool.
        let (echo_args, result) = results
            .iter()
            .find(|(t, _)| *t == call.tool.as_str())
            .map_or_else(
                || (call.arguments.clone(), "ok".into()),
                |(_, r)| (call.arguments.clone(), r.to_string()),
            );
        let echo_args = if call.tool == "mcp__playwright__browser_click" {
            "{\"target\":\"\"}".to_string()
        } else {
            echo_args
        };
        msgs.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            echo_args,
        )]));
        msgs.push(ChatMessage::tool_result(id, &call.tool, &result));
        n += 1;
        if n >= max {
            println!("  ... stopped after {max} steps (loop)");
            break;
        }
    }
}

fn main() {
    let page = page();
    // 1. Kotlin / Claude Code tool set
    let mut m = vec![ChatMessage::user(prompt())];
    drive(
        "CLAUDE (Kotlin run) tool set",
        CLAUDE,
        &mut m,
        &[
            (
                "mcp__playwright__browser_click",
                "Error: ### Error\nError: browserBackend.callTool: Unexpected token \"\" while parsing css selector \"\". Did you mean to CSS.escape it?",
            ),
            ("WebFetch", &page.replace("Scala", "Kotlin")),
            ("Bash", "Hello, World!\n"),
        ],
        8,
    );
    // 1b. the old loop, if a harness still echoes the click: must stop, not repeat
    let mut m = vec![ChatMessage::user(prompt())];
    drive(
        "CLAUDE with only playwright (no WebFetch)",
        &[
            "Bash",
            "Write",
            "mcp__playwright__browser_click",
            "mcp__playwright__browser_navigate",
        ],
        &mut m,
        &[],
        6,
    );
    // 2. Scala / Agent CLI tool set
    let mut m = vec![ChatMessage::user(prompt())];
    drive(
        "AGENT (Scala run) tool set",
        AGENT,
        &mut m,
        &[
            ("webfetch", &page),
            ("bash", "/bin/sh: 1: scala: not found\n"),
        ],
        8,
    );
    // 2b. restart prompt in the same session
    m.push(ChatMessage::user("🔄 Auto-restart: resume the previous session and handle its uncommitted changes.\n\nUncommitted files (1):\n?? Main.scala\n\nChanges summary:\nNo tracked-file diff summary available.\n\nPlease review these changes and commit them with an appropriate commit message.\nFollow the repository's commit message conventions from previous commits."));
    drive(
        "AGENT restart: 'commit them' in the resumed session",
        AGENT,
        &mut m,
        &[
            ("webfetch", &page),
            ("bash", "/bin/sh: 1: scala: not found\n"),
        ],
        8,
    );
    // 2c. the Agent CLI summarize call
    let mut s = vec![ChatMessage::user(format!(
        "\n              The following is the text to summarize:\n              <text>\n              {}              </text>\n            ",
        prompt()
    ))];
    println!(
        "\n=== AGENT summarize call, no tools: {}",
        show(plan_chat_step(&s, &[]).as_ref())
    );
    println!(
        "=== AGENT summarize call, agent tools: {}",
        show(plan_chat_step(&s, AGENT).as_ref())
    );
    drive(
        "AGENT summarize call driven",
        AGENT,
        &mut s,
        &[
            ("webfetch", &page),
            ("bash", "/bin/sh: 1: scala: not found\n"),
        ],
        6,
    );
    // 3. Rust / Codex tool set with the ChatGPT GitHub connector
    let mut m = vec![ChatMessage::user(
        prompt()
            .replace("00e1-73b9-955e-f357a1600d5b", "c107-78c7-8ff6-9f127a3c593c")
            .replace("Scala", "Rust"),
    )];
    let json = "{\"content\":[{\"type\":\"text\",\"text\":\"Action completed.\"}],\"structuredContent\":{\"content\":\"{\\\"url\\\":\\\"https://api.github.com/repos/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/issues/1\\\",\\\"number\\\":1,\\\"title\\\":\\\"Implement Hello World in Rust\\\",\\\"body\\\":\\\"## Task\\\\nPlease implement a \\\\\\\"Hello World\\\\\\\" program in Rust.\\\"}\"}}";
    drive(
        "CODEX (Rust run) tool set, GitHub connector JSON",
        CODEX,
        &mut m,
        &[
            ("mcp__codex_apps__github_fetch", json),
            (
                "shell",
                "Implement Hello World in Rust\n\n## Task\nPlease implement a \"Hello World\" program in Rust.\n\n6. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n",
            ),
        ],
        8,
    );
    // 3b. the connector envelope itself now reads as the issue
    let mut m = vec![ChatMessage::user(
        prompt()
            .replace("00e1-73b9-955e-f357a1600d5b", "c107-78c7-8ff6-9f127a3c593c")
            .replace("Scala", "Rust"),
    )];
    drive(
        "CODEX connector only (no shell)",
        &["apply_patch", "mcp__codex_apps__github_fetch"],
        &mut m,
        &[("mcp__codex_apps__github_fetch", json)],
        4,
    );
}
