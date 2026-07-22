//! Confirm and execute agentic report requests with complete context (#822).

use std::fmt::Write as _;

use serde_json::json;

use super::planner::{plan_one, tool_for, AgenticPlan, Capability, Progress};
use crate::engine::{normalize_prompt, stable_id};
use crate::protocol::ChatMessage;
use crate::{language, seed};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportTarget {
    HarnessLog,
    ServerLog,
    GithubIssue,
    FormalAi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportContents {
    Both,
    Harness,
    Server,
}

/// Continue a report confirmation/execution flow, or return `None` when the
/// conversation has no active report request.
pub(super) fn plan_report_flow(
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let report_index = messages.iter().rposition(|message| {
        message.role.eq_ignore_ascii_case("user")
            && is_report_intent(&message.content.user_request_text())
    })?;
    // A completed report remains in long-running agentic histories. Do not let
    // it hijack later unrelated requests after the assistant acknowledged it.
    if messages[report_index + 1..].iter().any(|message| {
        message.role.eq_ignore_ascii_case("assistant")
            && !message.content.plain_text().trim().is_empty()
            && !is_report_question(&message.content.plain_text())
    }) {
        return None;
    }
    let report_prompt = messages[report_index].content.user_request_text();
    let language = language::detect(&report_prompt).slug();
    let choices = answer_texts(messages, report_index + 1);
    let target = choices
        .iter()
        .find_map(|(index, text)| parse_target(text).map(|target| (target, *index)));

    let Some((target, target_index)) = target else {
        return Some(ask_or_render(
            tool_names,
            language,
            "report_target",
            "agentic_report_target_question",
            "agentic_report_target_options",
        ));
    };

    let contents = choices
        .iter()
        .filter(|(index, _)| *index >= target_index)
        .find_map(|(_, text)| parse_contents(text));
    if target == ReportTarget::GithubIssue && contents.is_none() {
        return Some(ask_or_render(
            tool_names,
            language,
            "report_contents",
            "agentic_report_contents_question",
            "agentic_report_contents_options",
        ));
    }

    // Earlier harness commands are part of the context being reported, not the
    // execution of this confirmation flow. Only inspect turns after "Report".
    let progress = Progress::scan(&messages[report_index + 1..]);
    if progress.done(Capability::Run) {
        return Some(AgenticPlan::Final(report_finished(
            target,
            progress.run_output.as_deref().unwrap_or_default(),
            language,
        )));
    }

    let dialog_id = dialog_id(messages);
    let command = command_for(
        target,
        contents.unwrap_or(ReportContents::Both),
        messages,
        report_index,
        &dialog_id,
    );
    if let Some(tool) = tool_for(tool_names, Capability::Run) {
        return Some(plan_one(tool, json!({"command": command}).to_string()));
    }
    Some(AgenticPlan::Final(render(
        "issue_report_tool_missing",
        &[("repository", &formal_ai_repo())],
    )))
}

fn ask_or_render(
    tool_names: &[&str],
    language: &str,
    id: &str,
    question_intent: &str,
    options_intent: &str,
) -> AgenticPlan {
    let question = localized(question_intent, language);
    let options = localized_options(options_intent, language);
    if let Some(tool) = tool_for(tool_names, Capability::AskUser) {
        let options = options
            .iter()
            .map(|(label, description)| {
                json!({
                    "label": label,
                    "description": description,
                })
            })
            .collect::<Vec<_>>();
        return plan_one(
            tool,
            json!({
                "questions": [{
                    "header": "Report",
                    "id": id,
                    "question": question,
                    "options": options,
                }]
            })
            .to_string(),
        );
    }

    let mut text = question;
    for (index, (label, description)) in options.iter().enumerate() {
        let _ = write!(text, "\n{}. {label} — {description}", index + 1);
    }
    AgenticPlan::Final(text)
}

fn localized(intent: &str, language: &str) -> String {
    seed::response_for(intent, language)
        .or_else(|| seed::response_for(intent, "en"))
        .unwrap_or_default()
}

fn localized_options(intent: &str, language: &str) -> Vec<(String, String)> {
    localized(intent, language)
        .split("||")
        .filter_map(|entry| {
            let (label, description) = entry.split_once('|')?;
            Some((label.to_owned(), description.to_owned()))
        })
        .collect()
}

fn is_report_question(text: &str) -> bool {
    ["en", "ru", "hi", "zh"].into_iter().any(|language| {
        [
            "agentic_report_target_question",
            "agentic_report_contents_question",
        ]
        .into_iter()
        .any(|intent| text.contains(&localized(intent, language)))
    })
}

fn answer_texts(messages: &[ChatMessage], start: usize) -> Vec<(usize, String)> {
    messages
        .iter()
        .enumerate()
        .skip(start)
        .filter(|(_, message)| {
            message.role.eq_ignore_ascii_case("user") || message.role.eq_ignore_ascii_case("tool")
        })
        .map(|(index, message)| (index, message.content.plain_text()))
        .collect()
}

fn parse_target(text: &str) -> Option<ReportTarget> {
    if matches_option(text, "agentic_report_target_options", 2, "github_issue") {
        return Some(ReportTarget::GithubIssue);
    }
    if matches_option(text, "agentic_report_target_options", 3, "formal_ai") {
        return Some(ReportTarget::FormalAi);
    }
    if matches_option(text, "agentic_report_target_options", 0, "harness_log") {
        return Some(ReportTarget::HarnessLog);
    }
    matches_option(text, "agentic_report_target_options", 1, "server_log")
        .then_some(ReportTarget::ServerLog)
}

fn parse_contents(text: &str) -> Option<ReportContents> {
    if matches_option(text, "agentic_report_contents_options", 0, "both_logs") {
        return Some(ReportContents::Both);
    }
    if matches_option(text, "agentic_report_contents_options", 1, "harness_log") {
        return Some(ReportContents::Harness);
    }
    matches_option(text, "agentic_report_contents_options", 2, "server_log")
        .then_some(ReportContents::Server)
}

fn matches_option(text: &str, intent: &str, option_index: usize, machine_value: &str) -> bool {
    let normalized = normalize_prompt(text);
    normalized.contains(machine_value)
        || ["en", "ru", "hi", "zh"].into_iter().any(|language| {
            localized_options(intent, language)
                .get(option_index)
                .is_some_and(|(label, _)| normalized.contains(&normalize_prompt(label)))
        })
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
        || (action
            && lexicon.mentions_role(seed::ROLE_AGENT_ACTION_REPORT_SUBJECT, &normalized)
            && report_action_governs_subject(lexicon, &normalized))
}

fn report_action_governs_subject(lexicon: &seed::Lexicon, normalized: &str) -> bool {
    let padded = format!(" {normalized} ");
    let matches_for = |role| {
        lexicon
            .words_for_role(role)
            .iter()
            .filter_map(|word| {
                let word = normalize_prompt(word);
                padded
                    .find(&format!(" {word} "))
                    .or_else(|| {
                        (!normalized.contains(char::is_whitespace))
                            .then(|| normalized.find(&word))
                            .flatten()
                    })
                    .map(|position| (position, word))
            })
            .collect::<Vec<_>>()
    };
    let actions = matches_for(seed::ROLE_AGENT_ACTION_REPORT_VERB);
    let subjects = matches_for(seed::ROLE_AGENT_ACTION_REPORT_SUBJECT);
    let ambiguous_actions = [
        seed::ROLE_FILE_WRITE_ACTION_CUE,
        seed::ROLE_FILE_WRITE_TARGET_CUE,
    ]
    .into_iter()
    .flat_map(|role| lexicon.words_for_role(role))
    .map(|word| normalize_prompt(&word))
    .collect::<Vec<_>>();

    actions.iter().any(|(action_position, action)| {
        subjects.iter().any(|(subject_position, _)| {
            let distance = action_position.abs_diff(*subject_position);
            let natural_order = action_position < subject_position;
            if ambiguous_actions
                .iter()
                .any(|candidate| candidate == action)
            {
                natural_order && distance <= 16
            } else {
                distance <= 32
            }
        })
    })
}

fn command_for(
    target: ReportTarget,
    contents: ReportContents,
    messages: &[ChatMessage],
    report_index: usize,
    dialog_id: &str,
) -> String {
    match target {
        ReportTarget::GithubIssue => github_command(contents, messages, report_index, dialog_id),
        ReportTarget::HarnessLog => format!(
            "formal-ai context export --session {} --source harness",
            shell_quote(dialog_id)
        ),
        ReportTarget::ServerLog => format!(
            "base=${{FORMAL_AI_BASE_URL:-http://127.0.0.1:3000}}; base=${{base%/v1}}; \
             curl -fsS \"$base/api/formal-ai/v1/conversations/{dialog_id}?include=server\""
        ),
        ReportTarget::FormalAi => format!(
            "base=${{FORMAL_AI_BASE_URL:-http://127.0.0.1:3000}}; base=${{base%/v1}}; \
             curl -fsS -X POST \"$base/api/formal-ai/v1/conversations/{dialog_id}/learn\""
        ),
    }
}

fn github_command(
    contents: ReportContents,
    messages: &[ChatMessage],
    report_index: usize,
    dialog_id: &str,
) -> String {
    let include = match contents {
        ReportContents::Both => "both",
        ReportContents::Harness => "harness",
        ReportContents::Server => "server",
    };
    let title = issue_title(messages, report_index);
    let intro = config("issue_report_body_intro");
    format!(
        "set -eu; context_file=$(mktemp \"${{TMPDIR:-/tmp}}/formal-ai-report.XXXXXX.lino\"); \
         body_file=$(mktemp \"${{TMPDIR:-/tmp}}/formal-ai-report.XXXXXX.md\"); \
         trap 'rm -f \"$context_file\" \"$body_file\"' EXIT; \
         base=${{FORMAL_AI_BASE_URL:-http://127.0.0.1:3000}}; base=${{base%/v1}}; \
         curl -fsS \"$base/api/formal-ai/v1/conversations/{dialog_id}?include={include}\" -o \"$context_file\"; \
         printf '%s\\n' {} > \"$body_file\"; \
         if [ \"$(wc -c < \"$context_file\")\" -le 50000 ]; then \
           printf '\\n### Complete agentic context\\n\\n```lino\\n' >> \"$body_file\"; \
           cat \"$context_file\" >> \"$body_file\"; printf '\\n```\\n' >> \"$body_file\"; \
         else \
           context_url=$(gh gist create --filename formal-ai-context.lino \"$context_file\"); \
           printf '\\n### Agentic context\\n\\nThe complete Links Notation context is available at %s.\\n\\n```lino\\n' \"$context_url\" >> \"$body_file\"; \
           head -c 12000 \"$context_file\" >> \"$body_file\"; printf '\\n```\\n' >> \"$body_file\"; \
         fi; \
         gh issue create --repo {} --title {} --body-file \"$body_file\"",
        shell_quote(&intro),
        formal_ai_repo(),
        shell_quote(&title),
    )
}

fn dialog_id(messages: &[ChatMessage]) -> String {
    let basis = messages
        .iter()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.user_request_text())
        .unwrap_or_default();
    stable_id("dialog", &basis)
}

fn issue_title(messages: &[ChatMessage], report_index: usize) -> String {
    let subject = messages[..report_index]
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.user_request_text())
        .filter(|text| !text.trim().is_empty());
    subject.map_or_else(
        || config("issue_report_default_title"),
        |text| {
            format!(
                "{}{}",
                config("issue_report_title_prefix"),
                truncate(&text, 72)
            )
        },
    )
}

fn report_finished(target: ReportTarget, run_output: &str, language: &str) -> String {
    let trimmed = run_output.trim();
    if target == ReportTarget::GithubIssue {
        if let Some(url) = trimmed
            .split_whitespace()
            .find(|token| token.starts_with("https://") && token.contains("/issues/"))
        {
            return render("issue_report_created_with_url", &[("url", url)]);
        }
        return if trimmed.is_empty() {
            config("issue_report_created")
        } else {
            format!(
                "{}\n\n```text\n{trimmed}\n```",
                config("issue_report_created")
            )
        };
    }
    let exported = localized("agentic_report_exported", language);
    if trimmed.is_empty() {
        exported
    } else {
        format!("{exported}\n\n```text\n{trimmed}\n```")
    }
}

fn formal_ai_repo() -> String {
    config("repository")
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn truncate(value: &str, max: usize) -> String {
    let value = value.trim();
    if value.chars().count() <= max {
        return value.to_owned();
    }
    let head = value
        .chars()
        .take(max.saturating_sub(1))
        .collect::<String>();
    format!("{}…", head.trim_end())
}
