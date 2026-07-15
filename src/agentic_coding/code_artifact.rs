//! Workspace-backed program generation and contextual source changes (#715).
//!
//! The latest user turn does not have to repeat the filename or original text:
//! the prior tool-call history is the authoritative mutable-artifact context.

use std::fmt::Write as _;

use serde_json::{json, Value};

use super::planner::{
    plan_one, tool_capability, tool_for, write_arguments, AgenticPlan, Capability,
};
use crate::coding::{
    program_language_by_alias, program_task_by_alias, program_template, PROGRAM_LANGUAGES,
};
use crate::protocol::ChatMessage;

#[derive(Debug, Clone)]
struct CodeArtifact {
    path: String,
    content: String,
}

/// A small mutable instruction graph. Conditional jumps and backward edges make
/// it a general string-rewriting representation; execution is bounded so an
/// agent request cannot loop forever.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MutableInstruction {
    ReplaceFirst {
        old: String,
        new: String,
    },
    JumpIfContains {
        needle: String,
        then_step: usize,
        else_step: usize,
    },
    Jump(usize),
    Halt,
}

#[derive(Debug, Clone)]
struct MutableCodeProgram {
    target: String,
    instructions: Vec<MutableInstruction>,
    max_steps: usize,
}

impl MutableCodeProgram {
    fn apply(&self, input: &str) -> Option<String> {
        let mut value = input.to_owned();
        let mut cursor = 0;
        for _ in 0..self.max_steps {
            match self.instructions.get(cursor)? {
                MutableInstruction::ReplaceFirst { old, new } => {
                    value = value.replacen(old, new, 1);
                    cursor += 1;
                }
                MutableInstruction::JumpIfContains {
                    needle,
                    then_step,
                    else_step,
                } => {
                    cursor = if value.contains(needle) {
                        *then_step
                    } else {
                        *else_step
                    }
                }
                MutableInstruction::Jump(next) => cursor = *next,
                MutableInstruction::Halt => return Some(value),
            }
        }
        None
    }

    fn links_notation(&self) -> String {
        let mut links = format!("mutation:target:'{}'\n", self.target.replace('\'', "\\'"));
        for (index, instruction) in self.instructions.iter().enumerate() {
            writeln!(links, "mutation:step:{index}:{instruction:?}")
                .expect("writing to a String cannot fail");
        }
        links
    }
}

pub(super) fn plan_code_artifact_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    // OpenCode's positional `run` interface currently preserves one pair of
    // transport quotes in the user content. They are framing, not request data.
    let task = unwrap_transport_quotes(task);
    let write_tool = tool_for(tool_names, Capability::Write)?;
    let latest_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;

    if let Some(artifact) = latest_code_artifact(&messages[..latest_user]) {
        let mutation = requested_mutation(task, &artifact)?;
        let current_messages = &messages[latest_user + 1..];
        if let Some(result) = latest_result(current_messages, Capability::Write) {
            return Some(AgenticPlan::Final(format!(
                "Updated `{}` through the workspace file tools.\n\n{}\n{}",
                artifact.path,
                mutation.links_notation(),
                result.trim()
            )));
        }
        if let Some(read_source) = latest_result(current_messages, Capability::Read) {
            let current_source = source_from_read_result(&read_source);
            let updated = mutation.apply(&current_source)?;
            if updated == current_source {
                return Some(AgenticPlan::Final(format!(
                    "No matching source fragment was found in `{}`.",
                    artifact.path
                )));
            }
            return Some(plan_one(
                write_tool,
                write_arguments(&artifact.path, &updated),
            ));
        }
        let read_tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(read_tool, read_arguments(&artifact.path)));
    }

    let artifact = generated_artifact(task)?;
    if latest_result(&messages[latest_user + 1..], Capability::Write).is_some() {
        return Some(AgenticPlan::Final(format!(
            "Created `{}` through the workspace file tools.",
            artifact.path
        )));
    }
    Some(plan_one(
        write_tool,
        write_arguments(&artifact.path, &artifact.content),
    ))
}

fn generated_artifact(task: &str) -> Option<CodeArtifact> {
    let normalized = task.to_lowercase();
    let language = program_language_by_alias(&normalized)?;
    let program_task = program_task_by_alias(&normalized)?;
    let template = program_template(program_task.slug, language.slug)?;
    Some(CodeArtifact {
        path: language.save_as.to_owned(),
        content: format!("{}\n", template.code.trim_end()),
    })
}

fn unwrap_transport_quotes(text: &str) -> &str {
    let trimmed = text.trim();
    for quote in ['"', '\''] {
        if let Some(inner) = trimmed
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
        {
            return inner;
        }
    }
    trimmed
}

fn requested_mutation(task: &str, artifact: &CodeArtifact) -> Option<MutableCodeProgram> {
    if task.contains('?') || task.contains('？') {
        return None;
    }
    let quoted = quoted_segments(task);
    let (old, new) = match quoted.as_slice() {
        [new] => (last_string_literal(&artifact.content)?, new.clone()),
        [old, new, ..] => (old.clone(), new.clone()),
        _ => return None,
    };
    if old == new || !artifact.content.contains(&old) {
        return None;
    }
    Some(MutableCodeProgram {
        target: artifact.path.clone(),
        instructions: vec![
            MutableInstruction::JumpIfContains {
                needle: old.clone(),
                then_step: 1,
                else_step: 3,
            },
            MutableInstruction::ReplaceFirst { old, new },
            MutableInstruction::Jump(3),
            MutableInstruction::Halt,
        ],
        max_steps: 16,
    })
}

fn latest_code_artifact(messages: &[ChatMessage]) -> Option<CodeArtifact> {
    messages
        .iter()
        .rev()
        .flat_map(|message| message.tool_calls.iter().rev())
        .find_map(|call| {
            if tool_capability(&call.function.name) != Some(Capability::Write) {
                return None;
            }
            let arguments: Value = serde_json::from_str(&call.function.arguments).ok()?;
            let path = argument_string(&arguments, &["path", "filePath", "file_path"])?;
            let content = argument_string(&arguments, &["content"])?;
            is_code_path(path).then(|| CodeArtifact {
                path: path.to_owned(),
                content: content.to_owned(),
            })
        })
}

fn latest_result(messages: &[ChatMessage], capability: Capability) -> Option<String> {
    messages
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, message)| {
            if !message.role.eq_ignore_ascii_case("tool") {
                return None;
            }
            let call_id = message.tool_call_id.as_ref()?;
            let call = messages[..index]
                .iter()
                .flat_map(|prior| prior.tool_calls.iter())
                .find(|call| &call.id == call_id)?;
            (tool_capability(&call.function.name) == Some(capability))
                .then(|| message.content.plain_text())
        })
}

/// Decode the decorated result emitted by `OpenCode`'s `read` tool. Other CLIs
/// return raw contents, which pass through byte-for-byte.
fn source_from_read_result(result: &str) -> String {
    let Some((_, after_open)) = result.split_once("<content>\n") else {
        return result.to_owned();
    };
    let Some((content, _)) = after_open.rsplit_once("\n</content>") else {
        return result.to_owned();
    };
    let mut decoded = String::new();
    for line in content.lines() {
        if line.starts_with("(End of file - total ") {
            break;
        }
        let source_line = line
            .split_once(": ")
            .filter(|(number, _)| number.chars().all(|character| character.is_ascii_digit()))
            .map_or(line, |(_, source)| source);
        decoded.push_str(source_line);
        decoded.push('\n');
    }
    while decoded.ends_with("\n\n") {
        decoded.pop();
    }
    decoded
}

fn read_arguments(path: &str) -> String {
    json!({"path": path, "filePath": path, "file_path": path}).to_string()
}

fn argument_string<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| value.get(key)?.as_str())
}

fn is_code_path(path: &str) -> bool {
    let extension = path.rsplit_once('.').map(|(_, extension)| extension);
    extension.is_some_and(|extension| {
        PROGRAM_LANGUAGES.iter().any(|language| {
            language
                .save_as
                .rsplit_once('.')
                .is_some_and(|(_, known)| known.eq_ignore_ascii_case(extension))
        })
    })
}

fn quoted_segments(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut result = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let quote = chars[index];
        let closing = match quote {
            '\'' | '"' => quote,
            '`' => '`',
            '“' => '”',
            '‘' => '’',
            _ => {
                index += 1;
                continue;
            }
        };
        let start = index + 1;
        index = start;
        while index < chars.len() && chars[index] != closing {
            index += 1;
        }
        if index < chars.len() && index > start {
            result.push(chars[start..index].iter().collect());
        }
        index += 1;
    }
    result
}

fn last_string_literal(source: &str) -> Option<String> {
    quoted_segments(source).pop()
}
