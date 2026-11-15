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

use crate::dreaming::bounds::{is_answer_form_cue, is_bounded_rule, is_valid_topic};
use crate::engine::{SymbolicAnswer, ThinkingStep};
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
/// for existing stores. Only valid amendments are read: an amendment learned
/// before the bounds existed — a pronoun topic, a whole prompt as its rule
/// (issue #1559) — is never applied.
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
            (is_valid_topic(topic) && is_bounded_rule(rule)).then(|| RetainedAmendment {
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
    let answer =
        solve_with_amendment_records(solver, prompt, history, &retained_amendments(events));
    if answer.intent == "unknown" {
        crate::anticipation::answer_from_prelearned_cache(prompt, events).unwrap_or(answer)
    } else {
        answer
    }
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
    let matching = matching_amendments(prompt, amendments);
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
    append_amendments(answer, &matching_amendments(prompt, &amendments));
}

/// The retained amendments an agentic final answer was given under, as trace
/// steps with an evidence link each (issue #1559 §5).
///
/// An agent publishes its final answer outside the conversation (a pull
/// request description, an issue comment), so the amendment text is never
/// appended to it; the trace lists which learned requirement applied and
/// where it came from.
#[must_use]
pub fn applied_amendment_steps(prompt: &str, events: &[MemoryEvent]) -> Vec<ThinkingStep> {
    let amendments = retained_amendments(events);
    matching_amendments(prompt, &amendments)
        .into_iter()
        .enumerate()
        .map(|(order, amendment)| {
            ThinkingStep::new(
                u32::try_from(order).unwrap_or(u32::MAX),
                "match_rule",
                amendment_trace(amendment),
                "high",
                format!("meta_algorithm_amendment:{}", amendment.id),
            )
        })
        .collect()
}

/// The trace detail of one applied amendment.
fn amendment_trace(amendment: &RetainedAmendment) -> String {
    format!("learned standing requirement ({})", amendment.topic)
}

/// Amendments whose topic matches the task — the single selection rule shared by
/// every application path, so no surface can drift into its own notion of
/// "covered". A requirement about the form of every answer ("Always answer in
/// Russian", topic `answer`) matches every prompt in its scope.
fn matching_amendments<'a>(
    prompt: &str,
    amendments: &'a [RetainedAmendment],
) -> Vec<&'a RetainedAmendment> {
    amendments
        .iter()
        .filter(|amendment| {
            is_answer_form_cue(&amendment.topic.to_lowercase())
                || topic_matches(prompt, &amendment.topic)
        })
        .collect()
}

fn amendment_lines(matching: &[&RetainedAmendment]) -> String {
    matching
        .iter()
        .map(|amendment| amendment_line(amendment))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The visible per-amendment compliance projection appended to answers.
#[must_use]
pub fn amendment_line(amendment: &RetainedAmendment) -> String {
    format!(
        "Learned standing requirement ({}): {}",
        amendment.topic, amendment.rule
    )
}

/// The intent an answer carries once a retained standing requirement applies to
/// it but the solver itself found no route.
pub const STANDING_REQUIREMENT_INTENT: &str = "standing_requirement";

fn append_amendments(answer: &mut SymbolicAnswer, matching: &[&RetainedAmendment]) {
    if matching.is_empty() {
        return;
    }
    // Issue #701 §4: a retained amendment must change *solving*, not only the
    // prose. When the solver found no route at all, the retained rule is the
    // applicable knowledge for this task, so the answer stops being classified
    // as unresolved. That reclassification is the machine-checkable delta a
    // decoration-only append could never produce — and it is confined to tasks
    // whose topic a retained amendment actually matches.
    if answer.intent == "unknown" {
        answer.intent = String::from(STANDING_REQUIREMENT_INTENT);
    }
    answer.answer.push_str("\n\n");
    answer.answer.push_str(&amendment_lines(matching));
    for amendment in matching {
        answer
            .evidence_links
            .push(format!("meta_algorithm_amendment:{}", amendment.id));
    }
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
