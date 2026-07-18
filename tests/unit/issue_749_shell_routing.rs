//! Regression coverage for issue #749 shell-command passthrough.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::ChatMessage;

fn shell_command(prompt: &str) -> Option<String> {
    let plan = plan_chat_step(&[ChatMessage::user(prompt)], &["exec_command"])?;
    let AgenticPlan::ToolCalls(calls) = plan else {
        return None;
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    arguments["command"].as_str().map(str::to_owned)
}

#[test]
fn explicit_shell_forms_pass_the_complete_command_through() {
    for (prompt, expected) in [
        ("execute date", "date"),
        ("run bash: echo hi", "echo hi"),
        ("run the command wc -l foo.txt", "wc -l foo.txt"),
        ("execute ln -s a b", "ln -s a b"),
        ("execute sort foo.txt", "sort foo.txt"),
        (
            "execute arbitrary-tool --alpha beta",
            "arbitrary-tool --alpha beta",
        ),
        ("bash -c 'ls -la'", "bash -c 'ls -la'"),
        ("powershell Get-Location", "powershell Get-Location"),
        ("pwsh -c 'Get-ChildItem'", "pwsh -c 'Get-ChildItem'"),
    ] {
        assert_eq!(shell_command(prompt).as_deref(), Some(expected), "{prompt}");
    }
}
