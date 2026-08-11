//! Confirm and execute agentic report requests with complete context (#822).

use std::fmt::Write as _;

use serde_json::json;

use super::planner::{plan_one, tool_for, AgenticPlan, Capability, Progress};
use super::report_script::{shell_quote, ReportScript};
use crate::engine::normalize_prompt;
use crate::issue_report::{issue_title, ReportTurn, TitleSettings};
use crate::protocol::ChatMessage;
use crate::{language, seed};

const REPORT_BODY_COMMAND: &str = "formal-ai report body";
const CONTEXT_EXPORT_COMMAND: &str = "formal-ai context export";
const CONTEXT_LEARN_COMMAND: &str = "formal-ai context learn";
const ISSUE_CREATE_COMMAND: &str = "gh issue create --repo ";
const CONTEXT_SESSION_FLAG: &str = " --session ";
const SOURCE_FLAG: &str = " --source ";
const OUTPUT_FLAG: &str = " --output ";
const CONTEXT_OUTPUT_FLAG: &str = " --context-output ";
const SURFACE_FLAG: &str = " --surface ";
const TITLE_FLAG: &str = " --title ";
const BODY_FILE_FLAG: &str = " --body-file ";
/// Programs the generated script checks for before it does anything.
const FORMAL_AI_PROGRAM: &str = "formal-ai";
const GH_PROGRAM: &str = "gh";
const PRINTF_PROGRAM: &str = "printf";
/// One `%s` line, so the script's last output is the exported artifact's path.
const PRINTF_LINE_FORMAT: &str = " '%s\\n' ";
/// Surface recorded in the report body's User Context section.
const AGENTIC_SURFACE: &str = "agentic-cli";
const BODY_FILE: &str = "body.md";
const CONTEXT_FILE: &str = "context.lino";
/// Session argument that makes the CLI resolve the live harness session.
const LATEST_SESSION: &str = "latest";

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
            && message.tool_calls.is_empty()
            && !message.content.plain_text().trim().is_empty()
            && !is_report_question(&message.content.plain_text())
    }) {
        return None;
    }
    let report_prompt = messages[report_index].content.user_request_text();
    let language = language::detect(&report_prompt).slug();
    let choices = answer_texts(messages, report_index + 1);
    let targets = choices.iter().find_map(|(index, text)| {
        let targets = parse_targets(text);
        (!targets.is_empty()).then_some((targets, *index))
    });

    let Some((targets, target_index)) = targets else {
        return Some(ask_or_render(
            tool_names,
            language,
            "report_target",
            "agentic_report_target_question",
            "agentic_report_target_options",
            true,
        ));
    };

    let explicit_contents = choices
        .iter()
        .filter(|(index, _)| *index > target_index)
        .find_map(|(_, text)| parse_contents(text));
    let selected_harness = targets.contains(&ReportTarget::HarnessLog);
    let selected_server = targets.contains(&ReportTarget::ServerLog);
    let inferred_contents = match (selected_harness, selected_server) {
        (true, true) => Some(ReportContents::Both),
        (true, false) => Some(ReportContents::Harness),
        (false, true) => Some(ReportContents::Server),
        (false, false) => None,
    };
    let contents = explicit_contents.or(inferred_contents);
    if targets.contains(&ReportTarget::GithubIssue) && contents.is_none() {
        return Some(ask_or_render(
            tool_names,
            language,
            "report_contents",
            "agentic_report_contents_question",
            "agentic_report_contents_options",
            false,
        ));
    }

    // Earlier harness commands are part of the context being reported, not the
    // execution of this confirmation flow. Only inspect turns after "Report".
    let progress = Progress::scan(&messages[report_index + 1..]);
    let dialog_id = dialog_id();
    let commands = commands_for_targets(
        &targets,
        contents.unwrap_or(ReportContents::Both),
        messages,
        report_index,
        &dialog_id,
    );
    // One destination per command, so a failed export cannot be hidden behind a
    // filed issue. The turn is only finished once every command has answered.
    let Some(command) = commands.get(progress.run_outputs.len()) else {
        return Some(AgenticPlan::Final(report_finished(
            &targets,
            &progress.run_outputs,
            language,
        )));
    };
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
    multiple: bool,
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
                    "multiple": multiple,
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
    seed::localized_response(intent, language).unwrap_or_default()
}

fn localized_options(intent: &str, language: &str) -> Vec<(String, String)> {
    seed::response_values_for(intent, language)
        .or_else(|| seed::response_values_for(intent, "en"))
        .unwrap_or_default()
        .chunks_exact(2)
        .map(|pair| (pair[0].clone(), pair[1].clone()))
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

fn parse_targets(text: &str) -> Vec<ReportTarget> {
    [
        (ReportTarget::HarnessLog, 0, "harness_log"),
        (ReportTarget::ServerLog, 1, "server_log"),
        (ReportTarget::GithubIssue, 2, "github_issue"),
        (ReportTarget::FormalAi, 3, "formal_ai"),
    ]
    .into_iter()
    .filter_map(|(target, option_index, machine_value)| {
        matches_option(
            text,
            "agentic_report_target_options",
            option_index,
            machine_value,
        )
        .then_some(target)
    })
    .collect()
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
    // Normalization turns `_` into a space, so the machine value must be
    // normalized too: a raw `contains("formal_ai")` can never match and the
    // learning target silently vanished from harness answers (#996).
    let normalized = normalize_prompt(text);
    normalized.contains(&normalize_prompt(machine_value))
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

/// One shell script per destination, GitHub last.
///
/// #838 packed every destination into one `set -eu` line, so the exports and
/// the filing shared a single exit status and a single tool result: whichever
/// step failed, the narration reported what the *last* one printed. Separate
/// commands keep each destination's output attributable, and filing last means
/// the issue is only created once the exports it describes have succeeded.
fn commands_for_targets(
    targets: &[ReportTarget],
    contents: ReportContents,
    messages: &[ChatMessage],
    report_index: usize,
    dialog_id: &str,
) -> Vec<String> {
    let mut ordered = targets.to_vec();
    ordered.sort_by_key(|target| usize::from(*target == ReportTarget::GithubIssue));
    ordered
        .iter()
        .map(|target| command_for(*target, contents, messages, report_index, dialog_id))
        .collect()
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
        ReportTarget::HarnessLog => export_command(
            dialog_id,
            ReportContents::Harness,
            &format!("formal-ai-harness-{dialog_id}.lino"),
        ),
        ReportTarget::ServerLog => export_command(
            dialog_id,
            ReportContents::Server,
            &format!("formal-ai-server-{dialog_id}.lino"),
        ),
        ReportTarget::FormalAi => learning_command(dialog_id),
    }
}

/// Export a session log into a surviving directory outside the CWD (#945).
///
/// A bare relative `--output` made runs from a repository checkout drop
/// session dumps at the repo root. The export directory keeps the artifact
/// usable, and the final `printf` tells the caller where it went.
fn export_command(dialog_id: &str, contents: ReportContents, output: &str) -> String {
    let mut script = ReportScript::new();
    let output_path = script.export(output);
    let mut command = CONTEXT_EXPORT_COMMAND.to_owned();
    command.push_str(CONTEXT_SESSION_FLAG);
    command.push_str(&shell_quote(dialog_id));
    command.push_str(SOURCE_FLAG);
    command.push_str(source_name(contents));
    command.push_str(OUTPUT_FLAG);
    command.push_str(&output_path);
    script.step(FORMAL_AI_PROGRAM, command);
    let mut echo = PRINTF_PROGRAM.to_owned();
    echo.push_str(PRINTF_LINE_FORMAT);
    echo.push_str(&output_path);
    script.step(PRINTF_PROGRAM, echo);
    script.render()
}

fn learning_command(dialog_id: &str) -> String {
    let mut command = CONTEXT_LEARN_COMMAND.to_owned();
    command.push_str(CONTEXT_SESSION_FLAG);
    command.push_str(&shell_quote(dialog_id));
    let mut script = ReportScript::new();
    script.step(FORMAL_AI_PROGRAM, command);
    script.render()
}

/// Render the body with `formal-ai report body`, then file it.
///
/// The body is a file, not a shell-assembled string: `formal-ai report body`
/// fails when the export is empty, and `set -eu` then stops the script before
/// `gh issue create` can file an issue that claims context it does not have.
fn github_command(
    contents: ReportContents,
    messages: &[ChatMessage],
    report_index: usize,
    dialog_id: &str,
) -> String {
    let mut script = ReportScript::new();
    let body_file = script.scratch(BODY_FILE);
    let context_file = script.scratch(CONTEXT_FILE);

    let mut body = REPORT_BODY_COMMAND.to_owned();
    body.push_str(CONTEXT_SESSION_FLAG);
    body.push_str(&shell_quote(dialog_id));
    body.push_str(SOURCE_FLAG);
    body.push_str(source_name(contents));
    body.push_str(SURFACE_FLAG);
    body.push_str(AGENTIC_SURFACE);
    body.push_str(OUTPUT_FLAG);
    body.push_str(&body_file);
    body.push_str(CONTEXT_OUTPUT_FLAG);
    body.push_str(&context_file);
    script.step(FORMAL_AI_PROGRAM, body);

    let mut create = ISSUE_CREATE_COMMAND.to_owned();
    create.push_str(&formal_ai_repo());
    create.push_str(TITLE_FLAG);
    create.push_str(&shell_quote(&report_title(messages, report_index)));
    create.push_str(BODY_FILE_FLAG);
    create.push_str(&body_file);
    script.step(GH_PROGRAM, create);

    script.render()
}

const fn source_name(contents: ReportContents) -> &'static str {
    match contents {
        ReportContents::Both => "both",
        ReportContents::Harness => "harness",
        ReportContents::Server => "server",
    }
}

/// The session to export.
///
/// Until #839 this was an FNV hash of the conversation's first user message, so
/// #838 asked for `dialog_a57762f1eb61e809` while the session it meant was
/// `ses_06ac01b87ffeW5XnFmtYE8Amil`, and any two conversations that opened with
/// the same sentence asked for the same export. The request being served
/// carries the real id in `x-formal-ai-dialog-id`; when it does not, `latest`
/// makes the CLI resolve the session this shell is actually inside.
fn dialog_id() -> String {
    crate::dialog_log::current_dialog_id()
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| LATEST_SESSION.to_owned())
}

fn report_title(messages: &[ChatMessage], report_index: usize) -> String {
    issue_title(
        &title_turns(messages, report_index),
        &TitleSettings::from_seed(),
    )
}

/// The user turns the title convention may quote (#839, §4).
///
/// Only the turn that opened *this* report flow is marked report-invoking: an
/// earlier report request that the agent already answered is part of the story
/// being reported, which is why issue #826's title ends with `Зарепорти баг`.
fn title_turns(messages: &[ChatMessage], report_index: usize) -> Vec<ReportTurn> {
    messages[..=report_index]
        .iter()
        .enumerate()
        .filter(|(_, message)| message.role.eq_ignore_ascii_case("user"))
        .map(|(index, message)| ReportTurn {
            report_invoking: index == report_index,
            ..ReportTurn::new(message.role.as_str(), message.content.user_request_text())
        })
        .collect()
}

/// Narrate what the destinations reported, over every command that ran.
fn report_finished(targets: &[ReportTarget], run_outputs: &[String], language: &str) -> String {
    let trimmed = run_outputs
        .iter()
        .map(|output| output.trim())
        .filter(|output| !output.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if targets.contains(&ReportTarget::GithubIssue) {
        // Honesty rule (#839, §5): the URL GitHub printed is the only evidence
        // that an issue exists. Without it the report failed, whatever else the
        // exports may have printed.
        if let Some(url) = trimmed
            .split_whitespace()
            .find(|token| token.starts_with("https://") && token.contains("/issues/"))
        {
            return render("issue_report_created_with_url", &[("url", url)]);
        }
        let failed = config("issue_report_failed");
        return if trimmed.is_empty() {
            failed
        } else {
            format!("{failed}\n\n```text\n{trimmed}\n```")
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
