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

use super::planner::{
    tool_capability, tool_for, write_arguments, AgenticPlan, Capability, PlannedToolCall,
};

/// Plan the next client-side step for a typed source-and-command artifact.
///
/// Both a file-write and command-execution tool must be advertised. This
/// preserves ordinary text behavior for non-agentic clients and never invents a
/// tool that the harness cannot execute.
pub fn plan_symbolic_command_reroute(
    messages: &[ChatMessage],
    tool_names: &[&str],
    symbolic_answer: &SymbolicAnswer,
) -> Option<AgenticPlan> {
    let recipe = symbolic_answer.execution_recipe.as_ref()?;
    let write_tool = tool_for(tool_names, Capability::Write)?;
    let run_tool = tool_for(tool_names, Capability::Run)?;
    let progress = RecipeProgress::after_latest_user(messages);

    if let Some(failure) = &progress.failure {
        return Some(AgenticPlan::Final(format!(
            "The agentic CLI harness could not complete `{}`. The tool reported:\n\n```text\n{}\n```",
            recipe.path,
            failure.trim()
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
    failure: Option<String>,
}

impl RecipeProgress {
    fn after_latest_user(messages: &[ChatMessage]) -> Self {
        let start = messages
            .iter()
            .rposition(|message| message.role == "user")
            .map_or(0, |index| index + 1);
        let mut progress = Self::default();
        for message in &messages[start..] {
            if message.role != "tool" {
                continue;
            }
            let capability = message
                .name
                .as_deref()
                .and_then(tool_capability)
                .or_else(|| capability_from_call_id(messages, message.tool_call_id.as_deref()));
            let output = message.content.plain_text();
            if tool_result_failed(&output) {
                progress.failure = Some(output);
                break;
            }
            match capability {
                Some(Capability::Write) => progress.write_done = true,
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

fn tool_result_failed(output: &str) -> bool {
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

fn capability_from_call_id(messages: &[ChatMessage], call_id: Option<&str>) -> Option<Capability> {
    let call_id = call_id?;
    messages
        .iter()
        .flat_map(|message| &message.tool_calls)
        .find(|call| call.id == call_id)
        .and_then(|call| tool_capability(&call.function.name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coding::PROGRAM_LANGUAGES;
    use crate::solver::{SolverConfig, UniversalSolver};

    #[test]
    fn catalog_execution_recipe_does_not_depend_on_rendered_command_labels() {
        let answer = UniversalSolver::new(SolverConfig {
            agent_mode: true,
            ..SolverConfig::default()
        })
        .solve("Please produce a Rust hello world program");
        let mut reworded = answer.clone();
        reworded.answer = answer
            .answer
            .replace("Check command:", "Compile using:")
            .replace("Run command:", "Execute using:");

        let plan = plan_symbolic_command_reroute(
            &[ChatMessage::user(
                "Please produce a Rust hello world program",
            )],
            &["write", "bash"],
            &reworded,
        );

        assert!(
            matches!(plan, Some(AgenticPlan::ToolCalls(_))),
            "execution is symbolic data and must survive presentation changes"
        );
    }

    #[test]
    fn every_catalog_language_projects_its_structured_execution_metadata() {
        let solver = UniversalSolver::new(SolverConfig {
            agent_mode: true,
            ..SolverConfig::default()
        });

        for language in PROGRAM_LANGUAGES {
            let answer = solver.solve(&format!(
                "Please produce a {} hello world program",
                language.name
            ));
            let recipe = answer
                .execution_recipe
                .unwrap_or_else(|| panic!("{} did not produce an execution recipe", language.slug));
            let expected_commands: Vec<String> = language
                .execution
                .check_command
                .into_iter()
                .chain(std::iter::once(language.execution.run_command))
                .map(str::to_owned)
                .collect();

            assert_eq!(recipe.path, language.save_as, "{}", language.slug);
            assert_eq!(recipe.language, language.code_fence, "{}", language.slug);
            assert_eq!(recipe.commands, expected_commands, "{}", language.slug);
            assert!(!recipe.source.is_empty(), "{}", language.slug);
        }
    }
}
