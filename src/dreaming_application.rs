//! Runtime application of retained dreaming amendments.
//!
//! Dreaming is useful only when retained learning changes later behaviour. This
//! module is the shared bridge used by every OpenAI-compatible protocol surface:
//! it reads structured `meta_algorithm_amendment` links, matches their topic to
//! a new task, and projects the standing rule into the produced answer.

use crate::engine::SymbolicAnswer;
use crate::memory::MemoryEvent;

/// A structured amendment recovered from the links store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedAmendment {
    pub id: String,
    pub topic: String,
    pub rule: String,
}

/// Read retained amendments from memory.
///
/// New records use `inputs` and `outputs` as the machine-readable topic/rule
/// fields; `demo_label` and `content` remain backwards-compatible projections
/// for existing stores.
#[must_use]
pub fn retained_amendments(events: &[MemoryEvent]) -> Vec<RetainedAmendment> {
    let mut amendments = events
        .iter()
        .filter(|event| event.kind.as_deref() == Some("meta_algorithm_amendment"))
        .filter_map(|event| {
            let topic = structured_value(event.inputs.as_deref(), "topic")
                .or(event.demo_label.as_deref())?
                .trim();
            let rule = structured_value(event.outputs.as_deref(), "rule")
                .or(event.content.as_deref())?
                .trim();
            (!topic.is_empty() && !rule.is_empty()).then(|| RetainedAmendment {
                id: event.id.clone(),
                topic: topic.to_owned(),
                rule: rule.to_owned(),
            })
        })
        .collect::<Vec<_>>();
    amendments.sort_by(|left, right| left.topic.cmp(&right.topic).then(left.id.cmp(&right.id)));
    amendments.dedup_by(|left, right| left.topic == right.topic && left.rule == right.rule);
    amendments
}

/// Amend an already-produced symbolic answer with all standing requirements
/// whose topic occurs as a complete, case-insensitive task token.
pub fn apply_retained_amendments(
    prompt: &str,
    answer: &mut SymbolicAnswer,
    events: &[MemoryEvent],
) {
    let additions = matching_amendment_lines(prompt, events);
    if additions.is_empty() {
        return;
    }
    answer.answer.push_str("\n\n");
    answer.answer.push_str(&additions.join("\n"));
    for amendment in retained_amendments(events)
        .into_iter()
        .filter(|amendment| topic_matches(prompt, &amendment.topic))
    {
        answer
            .evidence_links
            .push(format!("meta_algorithm_amendment:{}", amendment.id));
    }
}

/// Apply retained requirements to a plain final answer, including agentic
/// final responses that do not pass through [`SymbolicAnswer`].
#[must_use]
pub fn amended_answer(prompt: &str, answer: &str, events: &[MemoryEvent]) -> String {
    let additions = matching_amendment_lines(prompt, events);
    if additions.is_empty() {
        answer.to_owned()
    } else {
        format!("{answer}\n\n{}", additions.join("\n"))
    }
}

fn matching_amendment_lines(prompt: &str, events: &[MemoryEvent]) -> Vec<String> {
    retained_amendments(events)
        .into_iter()
        .filter(|amendment| topic_matches(prompt, &amendment.topic))
        .map(|amendment| {
            format!(
                "Learned standing requirement ({}): {}",
                amendment.topic, amendment.rule
            )
        })
        .collect()
}

fn structured_value<'a>(value: Option<&'a str>, key: &str) -> Option<&'a str> {
    let value = value?;
    value
        .strip_prefix(key)
        .and_then(|tail| tail.strip_prefix('='))
        .or(Some(value))
}

fn topic_matches(prompt: &str, topic: &str) -> bool {
    let prompt = prompt.to_lowercase();
    let topic = topic.to_lowercase();
    if topic.is_empty() {
        return false;
    }
    prompt.match_indices(&topic).any(|(start, matched)| {
        let end = start + matched.len();
        let left_ok = start == 0
            || prompt[..start]
                .chars()
                .next_back()
                .is_none_or(|character| !character.is_alphanumeric());
        let right_ok = end == prompt.len()
            || prompt[end..]
                .chars()
                .next()
                .is_none_or(|character| !character.is_alphanumeric());
        left_ok && right_ok
    })
}
