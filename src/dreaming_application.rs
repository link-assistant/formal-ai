//! Runtime application of retained dreaming amendments.
//!
//! Dreaming is useful only when retained learning changes later behaviour. This
//! module is the shared bridge used by every OpenAI-compatible protocol surface
//! **and** by dreaming's own replay verification: it reads structured
//! `meta_algorithm_amendment` links, matches their topic to a new task, and
//! *injects each standing rule into the solving context itself* — exactly as if
//! the user had repeated the requirement at the start of the conversation —
//! before the solver runs. The rule is additionally projected into the produced
//! answer (with an evidence link back to the amendment record) so compliance
//! stays visible and verifiable.
//!
//! Production answering ([`solve_with_standing_requirements`]) and dreaming
//! replay ([`replay_answer_with_amendments`]) share the same
//! [`solve_with_amendment_records`] core, so "covered by amendment" literally
//! means "the production path re-derives the stored output".

use crate::engine::SymbolicAnswer;
use crate::memory::MemoryEvent;
use crate::solver::{ConversationTurn, UniversalSolver};

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

/// Solve `prompt` with every matching standing requirement injected.
///
/// This is the production entry point shared by the chat completions and
/// responses surfaces (and, via [`replay_answer_with_amendments`], by
/// dreaming's replay verification).
#[must_use]
pub fn solve_with_standing_requirements(
    solver: &UniversalSolver,
    prompt: &str,
    history: &[ConversationTurn],
    events: &[MemoryEvent],
) -> SymbolicAnswer {
    solve_with_amendment_records(solver, prompt, history, &retained_amendments(events))
}

/// The shared amendment-application core.
///
/// Select amendments whose topic matches the task, prepend each as a
/// user-stated standing-requirement turn (changing how the solver sees the
/// task), solve, then append the visible compliance projection and evidence.
#[must_use]
pub fn solve_with_amendment_records(
    solver: &UniversalSolver,
    prompt: &str,
    history: &[ConversationTurn],
    amendments: &[RetainedAmendment],
) -> SymbolicAnswer {
    let matching = amendments
        .iter()
        .filter(|amendment| topic_matches(prompt, &amendment.topic))
        .collect::<Vec<_>>();
    let mut turns = matching
        .iter()
        .map(|amendment| {
            ConversationTurn::user(format!(
                "Standing requirement ({}): {}",
                amendment.topic, amendment.rule
            ))
        })
        .collect::<Vec<_>>();
    turns.extend_from_slice(history);
    let mut answer = solver.solve_with_history(prompt, &turns);
    append_amendments(&mut answer, &matching);
    answer
}

/// Replay a stored task input through the production application path.
///
/// Dreaming's coverage verification calls this, so "the amendment reproduces
/// the specific" is checked against the same code that answers live requests.
#[must_use]
pub fn replay_answer_with_amendments(input: &str, amendments: &[RetainedAmendment]) -> String {
    solve_with_amendment_records(&UniversalSolver::default(), input, &[], amendments).answer
}

/// Amend an already-produced symbolic answer with matching requirements.
///
/// Used for answers that did not go through the solver (memory-recall
/// answers); solver-produced answers should use
/// [`solve_with_standing_requirements`] so the rules shape solving itself.
pub fn apply_retained_amendments(
    prompt: &str,
    answer: &mut SymbolicAnswer,
    events: &[MemoryEvent],
) {
    let amendments = retained_amendments(events);
    let matching = amendments
        .iter()
        .filter(|amendment| topic_matches(prompt, &amendment.topic))
        .collect::<Vec<_>>();
    append_amendments(answer, &matching);
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

/// The visible per-amendment compliance projection appended to answers.
#[must_use]
pub fn amendment_line(amendment: &RetainedAmendment) -> String {
    format!(
        "Learned standing requirement ({}): {}",
        amendment.topic, amendment.rule
    )
}

fn append_amendments(answer: &mut SymbolicAnswer, matching: &[&RetainedAmendment]) {
    if matching.is_empty() {
        return;
    }
    answer.answer.push_str("\n\n");
    answer.answer.push_str(
        &matching
            .iter()
            .map(|amendment| amendment_line(amendment))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    for amendment in matching {
        answer
            .evidence_links
            .push(format!("meta_algorithm_amendment:{}", amendment.id));
    }
}

fn matching_amendment_lines(prompt: &str, events: &[MemoryEvent]) -> Vec<String> {
    retained_amendments(events)
        .into_iter()
        .filter(|amendment| topic_matches(prompt, &amendment.topic))
        .map(|amendment| amendment_line(&amendment))
        .collect()
}

fn structured_value<'a>(value: Option<&'a str>, key: &str) -> Option<&'a str> {
    let value = value?;
    value
        .strip_prefix(key)
        .and_then(|tail| tail.strip_prefix('='))
        .or(Some(value))
}

/// Match a stored topic against a task prompt.
///
/// Single-word topics must occur as a complete, case-insensitive token.
/// Multi-word topics match when the whole phrase occurs, or when **every**
/// significant word of the topic occurs as a complete token — so a topic like
/// `latex formatting` still matches "format this latex table with proper
/// formatting" even though the words are not adjacent.
#[must_use]
pub fn topic_matches(prompt: &str, topic: &str) -> bool {
    let prompt = prompt.to_lowercase();
    let topic = topic.to_lowercase();
    if topic.is_empty() {
        return false;
    }
    if contains_token(&prompt, &topic) {
        return true;
    }
    let words = topic.split_whitespace().collect::<Vec<_>>();
    words.len() > 1 && words.iter().all(|word| contains_token(&prompt, word))
}

fn contains_token(prompt: &str, needle: &str) -> bool {
    prompt.match_indices(needle).any(|(start, matched)| {
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
