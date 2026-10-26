//! A small literal edit keeps provider line numbers outside the physical source.
use super::{WorkspaceArtifact, WorkspaceRewrite};
use crate::agentic_coding::intent_router::edit_arguments;
use crate::agentic_coding::planner::{AgenticPlan, Capability, Progress, plan_one, tool_for};
use crate::agentic_coding::workspace_change::read_arguments;
fn targets(arguments: Option<&str>, path: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(arguments.unwrap_or("null")) else {
        return false;
    };
    ["path", "filePath", "file_path"]
        .iter()
        .any(|key| value.get(key).and_then(serde_json::Value::as_str) == Some(path))
}
pub(super) fn plan(
    artifact: &WorkspaceArtifact,
    rewrite: &WorkspaceRewrite,
    messages: &[crate::protocol::ChatMessage],
    tools: &[&str],
) -> Option<AgenticPlan> {
    let [rule] = rewrite.program.rules.as_slice() else {
        return None;
    };
    let edit = tool_for(tools, Capability::Edit)?;
    let read = tool_for(tools, Capability::Read)?;
    if rule.pattern.is_empty()
        || rule.pattern.contains(['\r', '\n'])
        || rule.replacement.contains(['\r', '\n'])
    {
        return None;
    }
    let progress = Progress::scan(messages);
    let attempts = progress
        .research_attempts()
        .iter()
        .filter(|attempt| targets(attempt.arguments.as_deref(), &artifact.path))
        .collect::<Vec<_>>();
    let fail = || {
        AgenticPlan::Final(format!(
            "Could not independently verify the literal update of {}; no whole-file replacement was attempted.",
            artifact.path
        ))
    };
    if let Some(index) = attempts
        .iter()
        .rposition(|attempt| attempt.capability == Capability::Edit)
    {
        if !attempts[index].succeeded {
            return Some(fail());
        }
        let before = attempts[..index]
            .iter()
            .rev()
            .find(|attempt| attempt.capability == Capability::Read)
            .and_then(|attempt| attempt.source_observation());
        let after_attempt = attempts[index + 1..]
            .iter()
            .rev()
            .find(|attempt| attempt.capability == Capability::Read);
        let Some(after_attempt) = after_attempt else {
            return Some(plan_one(read, read_arguments(&artifact.path)));
        };
        let after = after_attempt.source_observation();
        if let (Some(before), Some(after)) = (before, after)
            && after_attempt.succeeded
            && before.error.is_none()
            && after.error.is_none()
            && let (Some(before), Some(after)) = (&before.source, &after.source)
            && before.replacen(rule.pattern.as_str(), &rule.replacement, 1) == *after
        {
            return Some(AgenticPlan::Final(format!(
                "Updated {} with an exact literal edit and observed the requested result; all other content stayed outside the edit.",
                artifact.path
            )));
        }
        return Some(fail());
    }
    let observed = attempts
        .iter()
        .rev()
        .find(|attempt| attempt.capability == Capability::Read);
    let Some(observed) = observed else {
        return Some(plan_one(read, read_arguments(&artifact.path)));
    };
    let Some(source) = observed.source_observation() else {
        return Some(fail());
    };
    let Some(text) = source.source.as_deref() else {
        return Some(fail());
    };
    if !observed.succeeded
        || source.error.is_some()
        || text.matches(rule.pattern.as_str()).count() != 1
    {
        return Some(fail());
    }
    Some(plan_one(
        edit,
        edit_arguments(&artifact.path, &rule.pattern, &rule.replacement),
    ))
}
