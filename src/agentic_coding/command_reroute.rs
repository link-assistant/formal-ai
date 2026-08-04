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
    let progress = RecipeProgress::after_latest_user(messages, write_tool);

    if let Some(failure) = &progress.failure {
        let step = failure
            .from_run
            .then(|| recipe.commands.get(progress.commands_done))
            .flatten()
            .map_or(write_tool, String::as_str);
        return Some(AgenticPlan::Final(failure.report(
            messages,
            &recipe.path,
            step,
        )));
    }
    if !progress.write_done {
        return Some(one_call(
            write_tool,
            write_arguments(&recipe.path, &recipe.source),
        ));
    }
    if let Some(command) = recipe.commands.get(progress.commands_done) {
        return Some(one_call(
            run_tool,
            json!({ "command": command }).to_string(),
        ));
    }

    Some(AgenticPlan::Final(
        recipe.final_answer(&progress.command_outputs),
    ))
}

fn one_call(tool: &str, arguments: String) -> AgenticPlan {
    AgenticPlan::ToolCalls(vec![PlannedToolCall {
        tool: tool.to_owned(),
        arguments,
    }])
}

impl ExecutionRecipe {
    fn final_answer(&self, outputs: &[String]) -> String {
        let mut answer = format!(
            "Created and verified `{}` through the agentic CLI harness.\n\n```{}\n{}\n```\n\nCommands executed by the harness:\n",
            self.path, self.language, self.source
        );
        for command in &self.commands {
            let _ = writeln!(answer, "- `{command}`");
        }
        let actual = outputs
            .iter()
            .rev()
            .find(|output| !output.trim().is_empty())
            .map_or("(command completed without output)", |output| output.trim());
        let _ = write!(answer, "\nActual tool output:\n\n```text\n{actual}\n```");
        answer
    }
}

#[derive(Default)]
struct RecipeProgress {
    write_done: bool,
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

        let language = crate::language::detect(&latest_user_text(messages)).slug();
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

fn latest_user_text(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| message.content.plain_text())
        .unwrap_or_default()
}

impl RecipeProgress {
    fn after_latest_user(messages: &[ChatMessage], write_tool: &str) -> Self {
        let start = messages
            .iter()
            .rposition(|message| message.role == "user")
            .map_or(0, |index| index + 1);
        let mut progress = Self::default();
        for message in &messages[start..] {
            if message.role != "tool" {
                continue;
            }
            let result_tool = message
                .name
                .as_deref()
                .or_else(|| tool_from_call_id(messages, message.tool_call_id.as_deref()));
            let capability = result_tool.and_then(tool_capability);
            let output = message.content.plain_text();
            if let Some(failure) =
                StepFailure::from_result(output.clone(), capability == Some(Capability::Run))
            {
                progress.failure = Some(failure);
                break;
            }
            match capability {
                _ if result_tool.is_some_and(|name| name.eq_ignore_ascii_case(write_tool)) => {
                    progress.write_done = true;
                }
                Some(Capability::Run) => {
                    progress.commands_done += 1;
                    progress.command_outputs.push(output);
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

fn tool_from_call_id<'a>(messages: &'a [ChatMessage], call_id: Option<&str>) -> Option<&'a str> {
    let call_id = call_id?;
    messages
        .iter()
        .flat_map(|message| &message.tool_calls)
        .find(|call| call.id == call_id)
        .map(|call| call.function.name.as_str())
}
