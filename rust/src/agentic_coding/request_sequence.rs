//! A file operation followed by edits of the file it makes (PR #1188 G82).
//!
//! `Copy a.lino to b.lino. In b.lino replace 'x' with 'y', replace 'p' with
//! 'q'.` was written as one literal file. When the first sentence is a seeded
//! shell intent (copy, move) and every later sentence is an edit request whose
//! file that sentence names, the sentences are planned one after another, each
//! as the request it would be alone, over the same tool history; the answer
//! states each in turn. The Rust original of `js/agentic/request_sequence.mjs`.

use super::planner::AgenticPlan;
use crate::protocol::{ChatMessage, MessageContent};

/// The request's sentences when the first is a seeded shell intent and every
/// later one edits a file the first names (mirrors `requestSequence`).
fn request_sequence(task: &str) -> Option<Vec<String>> {
    let parts: Vec<String> = super::shell_command_policy::sentences(task)
        .iter()
        .map(|sentence| sentence.text.to_owned())
        .collect();
    let (first, rest) = parts.split_first()?;
    if rest.is_empty() || super::shell_command::semantic_shell_command_for_task(first).is_none() {
        return None;
    }
    let named = super::module_function::paths_in(first);
    for part in rest {
        let target = super::replace_list::replace_list(part)
            .map(|(target, _)| target)
            .or_else(|| {
                super::write_request::compose_edit_request(part).map(|(target, _, _)| target)
            })?;
        if !named.contains(&target) {
            return None;
        }
    }
    Some(parts)
}

/// The conversation with its latest user turn asking `part` alone.
fn with_request(messages: &[ChatMessage], part: &str) -> Vec<ChatMessage> {
    let latest = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"));
    messages
        .iter()
        .enumerate()
        .map(|(index, message)| {
            let mut message = message.clone();
            if Some(index) == latest {
                message.content = MessageContent::Text(part.to_owned());
            }
            message
        })
        .collect()
}

/// The next step of the first sentence not yet answered, or every answer.
///
/// Each sentence is planned by `plan_for` as that sentence alone; once each is
/// answered, the answers in order (mirrors `planRequestSequenceStep`).
pub(super) fn plan_request_sequence_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    plan_for: fn(&[ChatMessage], &[&str]) -> Option<AgenticPlan>,
) -> Option<AgenticPlan> {
    let parts = request_sequence(task)?;
    let mut answers = Vec::new();
    for part in &parts {
        match plan_for(&with_request(messages, part), tool_names)? {
            AgenticPlan::Final(answer) => answers.push(answer),
            plan => return Some(plan),
        }
    }
    Some(AgenticPlan::Final(answers.join("\n\n")))
}
