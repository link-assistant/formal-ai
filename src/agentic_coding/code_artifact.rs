//! Workspace-backed artifact generation and contextual changes (#715).
//!
//! The latest user turn does not have to repeat the filename or original text:
//! the prior tool-call history is the authoritative mutable-artifact context.

use std::fmt::Write as _;

use serde_json::{json, Value};

use super::planner::{
    plan_one, tool_capability, tool_for, write_arguments, AgenticPlan, Capability,
};
use crate::coding::{program_language_by_alias, program_task_by_alias, program_template};
use crate::normal_markov::{
    quoted_segments, unwrap_transport_quotes, RewriteHalt, RewriteOutcome, RewriteProgram,
    RewriteRule,
};
use crate::protocol::ChatMessage;

const MAX_REWRITE_STEPS: usize = 100_000;
const RENDERED_TRACE_EDGE_STEPS: usize = 32;

#[derive(Debug, Clone)]
struct WorkspaceArtifact {
    path: String,
    content: String,
}

#[derive(Debug, Clone)]
struct WorkspaceRewrite {
    target: String,
    program: RewriteProgram,
}

impl WorkspaceRewrite {
    fn links_notation(&self, outcome: Option<&RewriteOutcome>) -> String {
        let mut links = String::from("normal_markov_program\n");
        link_field(&mut links, 2, "target", &self.target);
        link_field(
            &mut links,
            2,
            "max_steps",
            &self.program.max_steps.to_string(),
        );
        for (index, rule) in self.program.rules.iter().enumerate() {
            let _ = writeln!(links, "  rewrite_rule \"{index}\"");
            link_field(&mut links, 4, "pattern", &rule.pattern);
            link_field(&mut links, 4, "replacement", &rule.replacement);
            link_field(&mut links, 4, "terminal", &rule.terminal.to_string());
        }
        if let Some(outcome) = outcome {
            let _ = writeln!(links, "  execution");
            link_field(&mut links, 4, "halt", &format!("{:?}", outcome.halt));
            link_field(&mut links, 4, "steps", &outcome.trace.len().to_string());
            let omitted = outcome
                .trace
                .len()
                .saturating_sub(RENDERED_TRACE_EDGE_STEPS * 2);
            for (index, step) in outcome.trace.iter().enumerate() {
                if omitted > 0 && index == RENDERED_TRACE_EDGE_STEPS {
                    link_field(&mut links, 4, "omitted_steps", &omitted.to_string());
                }
                if omitted > 0
                    && (RENDERED_TRACE_EDGE_STEPS..outcome.trace.len() - RENDERED_TRACE_EDGE_STEPS)
                        .contains(&index)
                {
                    continue;
                }
                let _ = writeln!(
                    links,
                    "    applied rule={} byte_offset={}",
                    step.rule_index, step.byte_offset
                );
            }
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

    if let Some(artifact) = latest_workspace_artifact(&messages[..latest_user]) {
        let rewrite = requested_rewrite(task, &artifact)?;
        let current_messages = &messages[latest_user + 1..];
        if let Some(result) = latest_result(current_messages, Capability::Write) {
            let outcome = latest_result(current_messages, Capability::Read)
                .map(|read| rewrite.program.execute(&source_from_read_result(&read)));
            return Some(AgenticPlan::Final(format!(
                "Updated `{}` through the workspace file tools.\n\n{}\n{}",
                artifact.path,
                rewrite.links_notation(outcome.as_ref()),
                result.trim()
            )));
        }
        if let Some(read_source) = latest_result(current_messages, Capability::Read) {
            let current_source = source_from_read_result(&read_source);
            let outcome = rewrite.program.execute(&current_source);
            if outcome.halt == RewriteHalt::StepLimit {
                return Some(AgenticPlan::Final(format!(
                    "Rewrite of `{}` reached its {MAX_REWRITE_STEPS}-step safety bound; no partial bytes were written.\n\n{}",
                    artifact.path,
                    rewrite.links_notation(Some(&outcome))
                )));
            }
            if outcome.output == current_source {
                return Some(AgenticPlan::Final(format!(
                    "No matching source fragment was found in `{}`.",
                    artifact.path
                )));
            }
            return Some(plan_one(
                write_tool,
                write_arguments(&artifact.path, &outcome.output),
            ));
        }
        let read_tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(read_tool, read_arguments(&artifact.path)));
    }

    // With a command capability, let the typed `ExecutionRecipe` path own code
    // creation and verification. Write-only harnesses still receive the source.
    if tool_for(tool_names, Capability::Run).is_some() {
        return None;
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

fn generated_artifact(task: &str) -> Option<WorkspaceArtifact> {
    let normalized = task.to_lowercase();
    let language = program_language_by_alias(&normalized)?;
    let program_task = program_task_by_alias(&normalized)?;
    let template = program_template(program_task.slug, language.slug)?;
    Some(WorkspaceArtifact {
        path: language.save_as.to_owned(),
        content: format!("{}\n", template.code.trim_end()),
    })
}

fn requested_rewrite(task: &str, artifact: &WorkspaceArtifact) -> Option<WorkspaceRewrite> {
    if task.contains('?') || task.contains('？') {
        return None;
    }
    let quoted = quoted_segments(task);
    let pairs = match quoted.as_slice() {
        [new] => vec![(last_string_literal(&artifact.content)?, new.clone())],
        values if !values.is_empty() && values.len() % 2 == 0 => values
            .chunks_exact(2)
            .map(|pair| (pair[0].clone(), pair[1].clone()))
            .collect(),
        _ => return None,
    };
    if pairs.iter().any(|(old, new)| old == new) {
        return None;
    }
    let single_substitution = pairs.len() == 1;
    let rules = pairs
        .into_iter()
        .map(|(old, new)| {
            let rule = RewriteRule::new(old.clone(), new);
            if single_substitution || old.is_empty() {
                rule.terminal()
            } else {
                rule
            }
        })
        .collect();
    Some(WorkspaceRewrite {
        target: artifact.path.clone(),
        program: RewriteProgram::new(rules, MAX_REWRITE_STEPS),
    })
}

fn latest_workspace_artifact(messages: &[ChatMessage]) -> Option<WorkspaceArtifact> {
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
            Some(WorkspaceArtifact {
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

fn last_string_literal(source: &str) -> Option<String> {
    quoted_segments(source).pop()
}

fn link_field(out: &mut String, indent: usize, name: &str, value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let _ = writeln!(out, "{}{name} \"{escaped}\"", " ".repeat(indent));
}
