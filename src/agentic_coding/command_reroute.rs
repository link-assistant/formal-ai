//! Route command-bearing symbolic answers through an agentic CLI's real tools.
//!
//! The ordinary solver can return a code artifact with a typed execution recipe.
//! On an API request from an agentic harness, the client owns its workspace,
//! permission prompts, sandbox, and audit trail. This adapter lowers the recipe
//! into a write -> command(s) -> final tool loop. It is generic over language,
//! command, file name, and client tool names and never scrapes rendered prose.

use serde_json::json;
use std::fmt::Write as _;

use crate::engine::{ExecutionRecipe, SymbolicAnswer};
use crate::protocol::ChatMessage;

use super::capability_router::is_workspace_creation_tool;
use super::git_commit::{self, CommitTarget};
use super::planner::{
    tool_capability, tool_for, write_arguments, AgenticPlan, Capability, PlannedToolCall,
};

/// Plan the next client-side step for a typed source-and-command artifact.
///
/// Both a file-write and command-execution tool must be advertised. This
/// preserves ordinary text behavior for non-agentic clients and never invents a
/// tool that the harness cannot execute.
#[must_use]
pub fn plan_symbolic_command_reroute(
    messages: &[ChatMessage],
    tool_names: &[&str],
    symbolic_answer: &SymbolicAnswer,
) -> Option<AgenticPlan> {
    let recipe = symbolic_answer.execution_recipe.as_ref()?;
    let write_tool = tool_for(tool_names, Capability::Write).or_else(|| {
        tool_names
            .iter()
            .copied()
            .find(|name| is_workspace_creation_tool(name))
    })?;
    let run_tool = tool_for(tool_names, Capability::Run)?;
    // A request that names the pull request or issue the work is for is asking
    // for the result to land there, so the recipe ends by committing and
    // pushing it (issue #1133).
    let commit = crate::protocol::latest_user_request(messages)
        .as_deref()
        .and_then(git_commit::target_of);
    let commit_step = |target: &CommitTarget| git_commit::recipe_commit_command(recipe, target);
    // Only the recipe's own commands count as recipe steps. A shell result
    // from before the recipe -- the `gh issue view` that read the work item --
    // is neither a step done nor a step failed (issue #1133).
    let expected_commands: Vec<String> = recipe
        .commands
        .iter()
        .cloned()
        .chain(commit.as_ref().map(commit_step))
        .collect();
    let progress = RecipeProgress::after_latest_user(messages, write_tool, recipe, &expected_commands);

    let next_file = || {
        if progress.files_written == 0 {
            Some((&recipe.path, &recipe.source))
        } else {
            recipe
                .supporting_files
                .get(progress.files_written - 1)
                .map(|file| (&file.path, &file.source))
        }
    };
    if let Some(failure) = &progress.failure {
        let commit_command = commit.as_ref().map(commit_step);
        let step = failure
            .from_run
            .then(|| {
                recipe
                    .commands
                    .get(progress.commands_done)
                    .map(String::as_str)
                    .or(commit_command.as_deref())
            })
            .flatten()
            .unwrap_or(write_tool);
        let failed_path = next_file().map_or(recipe.path.as_str(), |(path, _)| path);
        return Some(AgenticPlan::Final(failure.report(
            messages,
            failed_path,
            step,
        )));
    }
    if let Some((path, source)) = next_file() {
        return Some(one_call(write_tool, write_arguments(path, source)));
    }
    if let Some(command) = recipe.commands.get(progress.commands_done) {
        return Some(one_call(
            run_tool,
            json!({ "command": command }).to_string(),
        ));
    }
    if let Some(target) = &commit
        && progress.commands_done == recipe.commands.len()
    {
        return Some(one_call(
            run_tool,
            json!({ "command": commit_step(target) }).to_string(),
        ));
    }

    Some(AgenticPlan::Final(
        recipe.final_answer(&progress.command_outputs, commit.as_ref()),
    ))
}

fn one_call(tool: &str, arguments: String) -> AgenticPlan {
    AgenticPlan::ToolCalls(vec![PlannedToolCall {
        tool: tool.to_owned(),
        arguments,
    }])
}

impl ExecutionRecipe {
    fn final_answer(&self, outputs: &[String], committed: Option<&CommitTarget>) -> String {
        // With a commit step the last output is the push's; the verification
        // output the claim rests on is the one before it.
        let (verification_outputs, commit_output) = match committed {
            Some(_) if outputs.len() > self.commands.len() => {
                (&outputs[..self.commands.len()], outputs.last())
            }
            _ => (outputs, None),
        };
        let outputs = verification_outputs;
        let mut answer = format!(
            "Created and verified `{}` through the agentic CLI harness.\n\n```{}\n{}\n```\n\nCommands executed by the harness:\n",
            self.path, self.language, self.source
        );
        for file in &self.supporting_files {
            let _ = write!(
                answer,
                "\n`{}`\n\n```text\n{}\n```\n",
                file.path, file.source
            );
        }
        for command in &self.commands {
            let _ = writeln!(answer, "- `{command}`");
        }
        let actual = outputs
            .iter()
            .rev()
            .find(|output| !output.trim().is_empty())
            .map_or("(command completed without output)", |output| output.trim());
        let _ = write!(answer, "\nActual tool output:\n\n```text\n{actual}\n```");
        if let (Some(target), Some(output)) = (committed, commit_output) {
            let _ = write!(
                answer,
                "\n\nCommitted and pushed to `{}` for {}:\n\n```text\n{}\n```",
                target.push_ref(),
                target.reference,
                output.trim()
            );
        }
        answer
    }
}

#[derive(Default)]
struct RecipeProgress {
    files_written: usize,
    commands_done: usize,
    command_outputs: Vec<String>,
    failure: Option<StepFailure>,
}

/// A step the harness reported as failed, kept with the evidence that makes the
/// verdict checkable: the exit code the harness stated (when it stated one) and
/// whether the step was a command run rather than the file write.
struct StepFailure {
    reported: String,
    exit_code: Option<i32>,
    from_run: bool,
}

impl StepFailure {
    /// Report the failed step to the caller.
    ///
    /// The exit code is named when the harness gave one, and the step is named
    /// by the command that produced it: a harness that ran the command exactly
    /// as asked did not itself fail, so the report never says it did
    /// (issue #908).
    fn report(&self, messages: &[ChatMessage], path: &str, step: &str) -> String {
        const STEP_PLACEHOLDER: &str = "{step}";
        const PATH_PLACEHOLDER: &str = "{path}";
        const CODE_PLACEHOLDER: &str = "{code}";
        const REPORT_PLACEHOLDER: &str = "{report}";

        let language =
            crate::language::detect(&crate::protocol::latest_user_request(messages).unwrap_or_default())
                .slug();
        let intent = if self.exit_code.is_some() {
            "agentic_step_failed_with_exit_code"
        } else {
            "agentic_step_failed"
        };
        let code = self
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_default();
        crate::seed::localized_response(intent, language)
            .unwrap_or_default()
            .replace(STEP_PLACEHOLDER, step)
            .replace(PATH_PLACEHOLDER, path)
            .replace(CODE_PLACEHOLDER, &code)
            .replace(REPORT_PLACEHOLDER, self.reported.trim())
    }
}

impl RecipeProgress {
    fn after_latest_user(
        messages: &[ChatMessage],
        write_tool: &str,
        recipe: &ExecutionRecipe,
        expected_commands: &[String],
    ) -> Self {
        let start = messages
            .iter()
            .rposition(|message| message.role == "user")
            .map_or(0, |index| index + 1);
        let mut progress = Self::default();
        let files: Vec<(&str, &str)> = std::iter::once((recipe.path.as_str(), recipe.source.as_str()))
            .chain(recipe.supporting_files.iter().map(|file| (file.path.as_str(), file.source.as_str())))
            .collect();
        let mut observed_ids = std::collections::BTreeSet::new();
        for (index, message) in messages.iter().enumerate().skip(start) {
            if message.role != "tool" {
                continue;
            }
            let Some(call_id) = message.tool_call_id.as_deref() else { continue; };
            let Some(call) = messages[start..index].iter().rev()
                .flat_map(|prior| &prior.tool_calls).find(|call| call.id == call_id)
                .filter(|_| observed_ids.insert(call_id)) else { continue; };
            let result_tool = call.function.name.as_str();
            if message.name.as_deref().is_some_and(|name| !name.eq_ignore_ascii_case(result_tool)) {
                continue;
            }
            let capability = tool_capability(result_tool);
            let matches_write = result_tool.eq_ignore_ascii_case(write_tool)
                && files.get(progress.files_written).is_some_and(|(path, source)| {
                    super::progress::write_matches(&call.function.arguments, path, source)
                });
            let matches_run = capability == Some(Capability::Run)
                && progress.files_written == files.len()
                && run_command_of(&messages[start..index], Some(call_id))
                    .is_some_and(|command| expected_commands.get(progress.commands_done) == Some(&command));
            if !matches_write && !matches_run {
                continue;
            }
            let output = message.content.plain_text();
            if message.is_error {
                progress.failure = Some(StepFailure { reported: output, exit_code: None, from_run: matches_run });
                continue;
            }
            if let Some(failure) =
                StepFailure::from_result(output.clone(), capability == Some(Capability::Run))
            {
                progress.failure = Some(failure);
                continue;
            }
            // Only this still-pending action can match above. A later bound
            // successful retry clears its failure; unrelated setup or a future
            // verification result cannot. The failed observation stays in the
            // transcript, so interrupted recovery replays without hidden state.
            progress.failure = None;
            match capability {
                _ if matches_write => {
                    progress.files_written += 1;
                }
                Some(Capability::Run) => {
                    progress.commands_done += 1;
                    // Quote what the command printed, not the harness envelope
                    // it arrived in (issues #905 and #908).
                    let text = super::tool_result::shell_step(&output)
                        .map_or(output, |step: super::tool_result::ShellStep| step.text);
                    progress.command_outputs.push(text);
                }
                _ => {}
            }
        }
        progress
    }
}

/// Labels harnesses use to state the exit status of the process they ran.
const EXIT_CODE_LABELS: [&str; 4] = [
    "exit code:",
    "exit status:",
    "command exited with status",
    "process exited with code",
];

/// Values a labelled field carries when the harness means "there was none".
const ABSENT_FIELD_VALUES: [&str; 8] = [
    "",
    "-",
    "none",
    "(none)",
    "empty",
    "(empty)",
    "no output",
    "(no output)",
];

/// The exit code the harness stated for the process, when it stated one.
///
/// Every harness in use reports it in a fixed `Exit Code: <n>` field, so this
/// is a read rather than an inference.
fn reported_exit_code(output: &str) -> Option<i32> {
    output.lines().find_map(|line| {
        let line = line.trim().to_ascii_lowercase();
        EXIT_CODE_LABELS
            .iter()
            .find_map(|label| line.strip_prefix(label))?
            .split(|character: char| !(character.is_ascii_digit() || character == '-'))
            .find(|token| !token.is_empty())
            .and_then(|token| token.parse::<i32>().ok())
    })
}

impl StepFailure {
    /// Judge one tool result.
    ///
    /// The exit code the harness reported is the primary signal (issue #908):
    /// `0` is a success even when the command printed nothing and the envelope
    /// spells out an empty `Error:` field, and a non-zero code is a failure even
    /// when the command printed output that reads like a result (#905). Prose
    /// markers decide only when the harness reported no exit code at all, and
    /// silence is never a failure by itself.
    fn from_result(reported: String, from_run: bool) -> Option<Self> {
        if let Some(exit_code) = reported_exit_code(&reported) {
            return (exit_code != 0).then_some(Self {
                reported,
                exit_code: Some(exit_code),
                from_run,
            });
        }
        // The Agent CLI's shell tool hands the model only the process text and
        // keeps the status in metadata it never sends, so `/bin/sh: 1: scala:
        // not found` arrived with no code at all and was counted as a success
        // -- "Created and verified", with the failure quoted underneath (issue
        // #1133). The seed's failure lexicon already judges such text for the
        // narration path; a verification command's text is judged the same way.
        if super::tool_result::failure_message(&reported, false, from_run).is_some() {
            return Some(Self {
                exit_code: super::tool_result::reported_exit_code(&reported)
                    .and_then(|code| i32::try_from(code).ok()),
                reported,
                from_run,
            });
        }
        reports_failure_in_prose(&reported).then_some(Self {
            reported,
            exit_code: None,
            from_run,
        })
    }
}

/// Fallback verdict for a harness that reported no exit code at all.
fn reports_failure_in_prose(output: &str) -> bool {
    let normalized = output.to_ascii_lowercase();
    if [
        "command timed out",
        "command terminated without an exit status",
        "permission denied",
        "no such file or directory",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
    {
        return true;
    }
    // `Error:` and `Failed:` head a field whose *contents* decide the verdict:
    // harnesses print the label unconditionally and fill it with a placeholder
    // when nothing went wrong (issue #908).
    normalized.lines().any(|line| {
        ["error:", "failed:"].iter().any(|marker| {
            line.split_once(marker)
                .is_some_and(|(_, rest)| !ABSENT_FIELD_VALUES.contains(&rest.trim()))
        })
    })
}

/// The `command` argument of the shell call a result answers.
fn run_command_of(messages: &[ChatMessage], call_id: Option<&str>) -> Option<String> {
    let call_id = call_id?;
    let call = messages
        .iter()
        .flat_map(|message| &message.tool_calls)
        .find(|call| call.id == call_id)?;
    let arguments: serde_json::Value = serde_json::from_str(&call.function.arguments).ok()?;
    arguments
        .get("command")
        .or_else(|| arguments.get("cmd"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}
