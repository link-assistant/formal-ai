//! Turn a natural-language "report this on GitHub" request into a real
//! `gh issue create` shell tool call (issue #687).
//!
//! In agentic mode Formal AI has no web UI, so the top-bar "Report issue" button
//! is unreachable: the *only* way a user can file a report is by asking for it in
//! natural language and having the planner drive the client's shell tool. This
//! module recognises that intent from the conversation and composes the concrete
//! `gh issue create` command (repository, title, body) the agentic loop runs.
//!
//! Split out of [`super::planner`] like [`super::shell_command`]: the detection
//! and the command/answer composition live here; the capability-aware step
//! sequencing (`run → final`) stays in the planner. Keeping the report vocabulary
//! in one module also keeps the planner file under the repository line budget.

use serde_json::json;

use super::planner::{plan_one, tool_for, AgenticPlan, Capability, Progress};
use crate::engine::normalize_prompt;
use crate::protocol::ChatMessage;
use crate::seed;

/// The Formal AI repository issues are filed against. Mirrors the `Tv` constant
/// the web UI's "Report issue" button targets in `src/web/app.js`.
fn formal_ai_repo() -> String {
    config("repository")
}

/// A recognised request to file a GitHub issue against the Formal AI repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReportRequest {
    /// The issue title, derived from the conversation.
    pub(super) title: String,
    /// The issue body, a deterministic transcript of the conversation.
    pub(super) body: String,
}

impl ReportRequest {
    /// The `gh issue create` command the agentic loop's shell tool runs. Title and
    /// body are single-quote escaped so arbitrary conversation text survives the
    /// shell intact.
    pub(super) fn gh_command(&self) -> String {
        format!(
            "gh issue create --repo {} --title {} --body {}",
            formal_ai_repo(),
            shell_quote(&self.title),
            shell_quote(&self.body),
        )
    }
}

/// Recognise a request to report/file an issue and compose it from the
/// conversation, or [`None`] when the latest turn is not a report request.
///
/// The intent fires on the universal shape of an issue-filing request: a bare
/// "report", or a report/file/open/submit verb paired with an issue noun
/// (issue/bug/problem/…) or a repository reference (GitHub/repo). The title and
/// body are then derived from the conversation so the filed issue is grounded in
/// what the user was actually doing.
pub(super) fn report_issue_request_for(
    task: &str,
    messages: &[ChatMessage],
) -> Option<ReportRequest> {
    if !is_report_intent(task) {
        return None;
    }
    Some(compose_report(messages))
}

/// Whether `task` asks to report/file an issue.
pub(super) fn is_report_intent(task: &str) -> bool {
    let normalized = normalize_prompt(task);
    let lexicon = seed::lexicon();
    let action = lexicon.mentions_role(seed::ROLE_AGENT_ACTION_REPORT_VERB, &normalized);
    let bare_action = lexicon
        .words_for_role(seed::ROLE_AGENT_ACTION_REPORT_VERB)
        .iter()
        .any(|word| normalize_prompt(word) == normalized);
    bare_action
        || (action && lexicon.mentions_role(seed::ROLE_AGENT_ACTION_REPORT_SUBJECT, &normalized))
}

/// Compose the issue from the conversation: a title from the most recent
/// non-report user turn, a body that transcribes the exchange deterministically.
fn compose_report(messages: &[ChatMessage]) -> ReportRequest {
    let turns: Vec<(String, String)> = messages
        .iter()
        .filter(|m| m.role.eq_ignore_ascii_case("user") || m.role.eq_ignore_ascii_case("assistant"))
        .map(|m| (m.role.to_lowercase(), m.content.plain_text()))
        .filter(|(_, text)| !text.trim().is_empty())
        .collect();

    // The subject is the most recent user turn that is not the report request
    // itself; fall back to a generic title when the report stands alone.
    let subject = turns
        .iter()
        .rev()
        .skip(1)
        .find(|(role, _)| role == "user")
        .map(|(_, text)| text.trim().to_owned());
    let title = match subject.as_deref() {
        Some(text) if !text.is_empty() => {
            format!(
                "{}{}",
                config("issue_report_title_prefix"),
                truncate(text, 72)
            )
        }
        _ => config("issue_report_default_title"),
    };

    let mut body = format!("{}\n\n", config("issue_report_body_intro"));
    if turns.is_empty() {
        body.push_str(&format!("_{}_\n", config("issue_report_empty_history")));
    } else {
        body.push_str(&format!(
            "### {}\n\n",
            config("issue_report_conversation_heading")
        ));
        for (role, text) in &turns {
            body.push_str("- **");
            body.push_str(role);
            body.push_str(":** ");
            body.push_str(text.trim());
            body.push('\n');
        }
    }
    body.push_str(&format!("\n{}", config("issue_report_body_footer")));

    ReportRequest { title, body }
}

/// The issue-#687 report-issue recipe step: turn a recognised report request into
/// a real `gh issue create` shell tool call, then surface the created issue URL.
/// Agentic mode has no Formal AI web UI, so the "Report issue" button is
/// unreachable; this makes the same action available in natural language.
pub(super) fn plan_report_issue_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    request: &ReportRequest,
) -> AgenticPlan {
    let progress = Progress::scan(messages);
    let command = request.gh_command();
    if progress.done(Capability::Run) {
        return AgenticPlan::Final(final_answer(
            &command,
            progress.run_output.as_deref().unwrap_or_default(),
        ));
    }
    if let Some(tool) = tool_for(tool_names, Capability::Run) {
        return plan_one(tool, json!({ "command": command }).to_string());
    }
    AgenticPlan::Final(format!(
        "{}",
        render(
            "issue_report_tool_missing",
            &[("repository", &formal_ai_repo())]
        ),
    ))
}

/// The confirmation shown once the client's shell tool reports back. Surfaces the
/// created issue URL from the tool output when present.
pub(super) fn final_answer(command: &str, run_output: &str) -> String {
    let trimmed = run_output.trim();
    trimmed
        .split_whitespace()
        .find(|token| token.starts_with("https://") && token.contains("/issues/"))
        .map_or_else(
            || {
                if trimmed.is_empty() {
                    render("issue_report_ran_command", &[("command", command)])
                } else {
                    format!(
                        "{}\n\n```text\n{trimmed}\n```",
                        config("issue_report_created")
                    )
                }
            },
            |url| render("issue_report_created_with_url", &[("url", url)]),
        )
}

fn config(key: &str) -> String {
    seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned())
}

fn render(key: &str, values: &[(&str, &str)]) -> String {
    values.iter().fold(config(key), |text, (name, value)| {
        text.replace(&format!("{{{name}}}"), value)
    })
}

/// Single-quote escape a value for a POSIX shell command line.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Truncate to at most `max` characters on a char boundary, appending an ellipsis
/// when shortened. Deterministic and Unicode-safe.
fn truncate(value: &str, max: usize) -> String {
    let value = value.trim();
    if value.chars().count() <= max {
        return value.to_owned();
    }
    let head: String = value.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", head.trim_end())
}
