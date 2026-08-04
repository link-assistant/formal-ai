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
    let progress = RecipeProgress::after_latest_user(messages, write_tool, recipe);

    if let Some(failure) = &progress.failure {
        return Some(AgenticPlan::Final(failure.report(recipe, messages)));
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

/// A step the harness reported as unsuccessful, kept with the label of whatever
/// was actually running so the report can name it (issue #908).
struct StepFailure {
    label: String,
    output: String,
}

impl StepFailure {
    fn report(&self, recipe: &ExecutionRecipe, messages: &[ChatMessage]) -> String {
        // When the harness reported an exit status, the status is the finding:
        // name the failing command and the code it exited with. Blaming the
        // harness hid both (issue #908, suggested fix 3).
        if super::tool_result::shell_step(&self.output).is_some() {
            let prompt = crate::protocol::latest_user_request(messages).unwrap_or_default();
            return super::tool_result::render(&self.label, &self.output, &prompt);
        }
        format!(
            "The agentic CLI harness could not complete `{}`. The tool reported:\n\n```text\n{}\n```",
            recipe.path,
            self.output.trim()
        )
    }
}

#[derive(Default)]
struct RecipeProgress {
    write_done: bool,
    commands_done: usize,
    command_outputs: Vec<String>,
    failure: Option<StepFailure>,
}

impl RecipeProgress {
    fn after_latest_user(
        messages: &[ChatMessage],
        write_tool: &str,
        recipe: &ExecutionRecipe,
    ) -> Self {
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
            if tool_result_failed(&output) {
                let label = match capability {
                    Some(Capability::Run) => recipe
                        .commands
                        .get(progress.commands_done)
                        .unwrap_or(&recipe.path),
                    _ => &recipe.path,
                };
                progress.failure = Some(StepFailure {
                    label: label.clone(),
                    output,
                });
                break;
            }
            match capability {
                _ if result_tool.is_some_and(|name| name.eq_ignore_ascii_case(write_tool)) => {
                    progress.write_done = true;
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

/// A step failed when the harness said so. Only when no harness reported a
/// status does the wording of the result get a vote — reading `Error: (none)`
/// as an error is exactly the defect issue #908 filed.
fn tool_result_failed(output: &str) -> bool {
    match super::tool_result::step_outcome(output) {
        super::tool_result::StepOutcome::Succeeded => return false,
        super::tool_result::StepOutcome::Failed => return true,
        super::tool_result::StepOutcome::Unreported => {}
    }
    let normalized = output.to_ascii_lowercase();
    let explicit_failure = [
        "command exited with status ",
        "command timed out",
        "command terminated without an exit status",
        "error:",
        "failed:",
        "permission denied",
        "no such file or directory",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    explicit_failure
        || normalized.lines().any(|line| {
            ["exit code:", "exit status:"]
                .iter()
                .find_map(|prefix| line.trim().strip_prefix(prefix))
                .and_then(|value| value.trim().parse::<i32>().ok())
                .is_some_and(|code| code != 0)
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
