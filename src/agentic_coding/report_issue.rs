//! Agentic issue reporting through the client CLI's advertised shell tool.

use serde_json::json;

use super::planner::{AgenticPlan, Capability, PlannedToolCall};
use crate::protocol::ChatMessage;

const REPOSITORY: &str = "link-assistant/formal-ai";
const TITLE: &str = "Formal AI agentic-mode report";
const BODY_LIMIT: usize = 8_000;

pub(super) fn plan_report_issue_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let report_index = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .unwrap_or(0);

    if let Some(result) = messages[report_index + 1..]
        .iter()
        .enumerate()
        .find(|(offset, message)| {
            message.role.eq_ignore_ascii_case("tool")
                && super::planner::result_capability(messages, report_index + 1 + offset)
                    == Some(Capability::Run)
        })
        .map(|(_, message)| message.content.plain_text())
    {
        return AgenticPlan::Final(format!(
            "GitHub issue reporting finished. The `gh` command returned:\n\n{}",
            result.trim()
        ));
    }

    let Some(tool) = tool_names
        .iter()
        .copied()
        .find(|name| super::planner::tool_capability(name) == Some(Capability::Run))
    else {
        return AgenticPlan::Final(String::from(
            "I can report this as a real GitHub issue when the agentic client advertises a shell tool (for example `bash` or `run_command`).",
        ));
    };

    let body = report_body(messages, report_index);
    let command = format!(
        "gh issue create --repo {REPOSITORY} --title {} --body {}",
        shell_quote(TITLE),
        shell_quote(&body)
    );
    AgenticPlan::ToolCalls(vec![PlannedToolCall {
        tool: tool.to_owned(),
        arguments: json!({ "command": command }).to_string(),
    }])
}

fn report_body(messages: &[ChatMessage], report_index: usize) -> String {
    let mut body = String::from("Reported from Formal AI agentic mode.\n\n## Conversation\n\n");
    for message in messages[..=report_index].iter().rev().take(20).rev() {
        if !matches!(message.role.as_str(), "user" | "assistant") {
            continue;
        }
        let content = message.content.plain_text();
        if content.trim().is_empty() {
            continue;
        }
        body.push_str("**");
        body.push_str(&message.role);
        body.push_str(":**\n\n");
        body.push_str(content.trim());
        body.push_str("\n\n");
    }
    truncate_chars(body, BODY_LIMIT)
}

fn truncate_chars(value: String, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value;
    }
    let mut truncated = value
        .chars()
        .take(limit.saturating_sub(32))
        .collect::<String>();
    truncated.push_str("\n\n[conversation truncated]\n");
    truncated
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
