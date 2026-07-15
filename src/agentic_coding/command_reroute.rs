//! Route command-bearing symbolic answers through an agentic CLI's real tools.
//!
//! The ordinary solver can return a code artifact together with `Check command:`
//! and `Run command:` metadata. On an API request from an agentic harness, prose
//! claiming those commands ran in an embedded fixture is the wrong boundary: the
//! client owns its workspace, permission prompts, sandbox, and audit trail. This
//! adapter turns that already-derived answer into a write -> command(s) -> final
//! tool loop. It is generic over language, command, file name, and client tool
//! names; the symbolic answer remains the source of the recipe.

use serde_json::json;
use std::fmt::Write as _;

use crate::protocol::ChatMessage;

use super::planner::{
    tool_capability, tool_for, write_arguments, AgenticPlan, Capability, PlannedToolCall,
};

/// Plan the next client-side step for an answer containing source and commands.
///
/// Both a file-write and command-execution tool must be advertised. This
/// preserves ordinary text behavior for non-agentic clients and never invents a
/// tool that the harness cannot execute.
pub fn plan_symbolic_command_reroute(
    messages: &[ChatMessage],
    tool_names: &[&str],
    symbolic_answer: &str,
) -> Option<AgenticPlan> {
    let recipe = CommandRecipe::from_answer(symbolic_answer)?;
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

struct CommandRecipe {
    language: String,
    source: String,
    path: String,
    commands: Vec<String>,
}

impl CommandRecipe {
    fn from_answer(answer: &str) -> Option<Self> {
        let (language, source) = first_fenced_source(answer)?;
        let commands = command_lines(answer);
        if commands.is_empty() {
            return None;
        }
        let path = source_path(answer, &commands)?;
        Some(Self {
            language,
            source,
            path,
            commands,
        })
    }

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

fn first_fenced_source(answer: &str) -> Option<(String, String)> {
    let opening = answer.find("```")?;
    let header_end = answer[opening + 3..].find('\n')? + opening + 3;
    let language = answer[opening + 3..header_end].trim();
    if language.is_empty() || language.eq_ignore_ascii_case("text") {
        return None;
    }
    let source_start = header_end + 1;
    let source_end = answer[source_start..].find("```")? + source_start;
    let source = answer[source_start..source_end].trim_end();
    (!source.is_empty()).then(|| (language.to_owned(), source.to_owned()))
}

fn command_lines(answer: &str) -> Vec<String> {
    answer
        .lines()
        .filter_map(|line| {
            let (_, value) = line
                .split_once("Check command:")
                .or_else(|| line.split_once("Run command:"))?;
            let command = value.trim().trim_matches('`').trim();
            (!command.is_empty()).then(|| command.to_owned())
        })
        .collect()
}

fn source_path<'a>(answer: &'a str, commands: &'a [String]) -> Option<String> {
    commands
        .iter()
        .flat_map(|command| command.split_whitespace())
        .chain(answer.split_whitespace())
        .map(clean_token)
        .find(|token| looks_like_source_path(token))
        .map(str::to_owned)
}

fn clean_token(token: &str) -> &str {
    token.trim_matches(|character: char| {
        matches!(
            character,
            '`' | '"' | '\'' | ',' | ';' | ':' | '(' | ')' | '[' | ']'
        )
    })
}

fn looks_like_source_path(token: &str) -> bool {
    const SOURCE_EXTENSIONS: &[&str] = &[
        "rs", "py", "js", "ts", "go", "c", "cc", "cpp", "cxx", "java", "cs", "rb", "php", "kt",
        "kts", "swift", "scala", "sh",
    ];
    if token.is_empty()
        || token.contains("://")
        || token.starts_with('/')
        || token.starts_with('-')
        || token.split('/').any(|part| part == ".." || part.is_empty())
    {
        return false;
    }
    let Some((stem, extension)) = token.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty()
        && SOURCE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        && token.chars().all(|character| {
            character.is_alphanumeric() || matches!(character, '/' | '.' | '_' | '-')
        })
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
