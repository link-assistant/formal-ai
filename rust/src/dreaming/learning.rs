//! Candidate-task replay, failure-driven refinement, and language-independent
//! recurring-structure mining.
//!
//! Replay verification goes through the **production** amendment application
//! path ([`crate::dreaming_application::replay_answer_with_amendments`]), so a
//! specific counts as "covered" only when the same code that answers live
//! requests re-derives its stored output. Failed replays are not discarded:
//! they drive one refinement pass that extends the topic's amendment with the
//! requirement statements the stored output proves were once in force, then
//! re-verifies. Task/topic detection reads the multilingual data lexicon
//! (`data/meta/dreaming-lexicon.lino`) instead of hardcoded English keywords.

use std::collections::{BTreeMap, BTreeSet};

use super::bounds::{
    bounded_statement, is_bounded_rule, is_valid_topic, rule_max_chars, statement_topic,
};
use super::lexicon::lexicon;
use super::{
    DreamingCandidateTask, DreamingEventObservation, DreamingPattern, DreamingSynthesizedTask,
    LearnedRequirement, MetaAlgorithmAmendment, TopicFrequency,
};
use crate::dreaming_application::{RetainedAmendment, replay_answer_with_amendments};
use crate::memory::MemoryEvent;
use crate::standing_scope::{ScopePair, scope_key, scope_pairs};

pub(super) struct LearningResult {
    pub topics: Vec<TopicFrequency>,
    pub requirements: Vec<LearnedRequirement>,
    pub amendments: Vec<MetaAlgorithmAmendment>,
    pub candidate_tasks: Vec<DreamingCandidateTask>,
    pub patterns: Vec<DreamingPattern>,
    pub synthesized_tasks: Vec<DreamingSynthesizedTask>,
}

pub(super) fn learn_from_memory(
    events: &[MemoryEvent],
    observations: &[DreamingEventObservation],
    deleted_conversations: &BTreeSet<String>,
) -> LearningResult {
    let mut interactions: BTreeMap<String, usize> = BTreeMap::new();
    let mut task_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut requirement_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut requirements: BTreeMap<(String, String, String), LearnedRequirement> = BTreeMap::new();
    let mut specific_indices: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, event) in events.iter().enumerate() {
        if event
            .conversation_id
            .as_deref()
            .is_some_and(|id| deleted_conversations.contains(id))
        {
            continue;
        }
        let Some(topic) = observations[index].topic.clone() else {
            continue;
        };
        *interactions.entry(topic.clone()).or_default() += 1;
        if is_task_event(event) {
            *task_counts.entry(topic.clone()).or_default() += 1;
            // Learning may reuse original observations without making them
            // disposable. Retention is decided separately by the planner.
            if !event.id.is_empty() {
                specific_indices
                    .entry(topic.clone())
                    .or_default()
                    .push(index);
            }
        }
        let learned = requirement_statement(event).and_then(|statement| {
            requirement_topic(event, &statement).map(|learned_topic| (learned_topic, statement))
        });
        if let Some((learned_topic, statement)) = learned {
            *requirement_counts.entry(topic.clone()).or_default() += 1;
            let scope = scope_pairs(&event.evidence);
            let entry = requirements
                .entry((
                    learned_topic.clone(),
                    scope_key(&scope),
                    statement.to_lowercase(),
                ))
                .or_insert_with(|| LearnedRequirement {
                    topic: learned_topic,
                    statement: statement.clone(),
                    source_event_ids: Vec::new(),
                    occurrences: 0,
                    scope,
                });
            entry.occurrences += 1;
            if !event.id.is_empty() {
                entry.source_event_ids.push(event.id.clone());
            }
        }
    }
    let mut topics = interactions
        .iter()
        .map(|(topic, interactions)| TopicFrequency {
            topic: topic.clone(),
            interactions: *interactions,
            task_events: task_counts.get(topic).copied().unwrap_or(0),
            requirement_events: requirement_counts.get(topic).copied().unwrap_or(0),
        })
        .collect::<Vec<_>>();
    topics.sort_by(|left, right| {
        right
            .interactions
            .cmp(&left.interactions)
            .then_with(|| left.topic.cmp(&right.topic))
    });
    let mut requirements = requirements.into_values().collect::<Vec<_>>();
    let mut amendments = generalize_amendments(events, &requirements, &specific_indices);
    let mut candidate_tasks =
        replay_candidate_tasks(events, observations, &amendments, &interactions);
    refine_amendments_from_failures(
        events,
        &specific_indices,
        &mut requirements,
        &mut amendments,
        &mut candidate_tasks,
    );
    let patterns = mine_patterns(events, observations);
    // A synthesized trial belongs to no one request, so it sees only amendments
    // without scope.
    let unscoped = amendments
        .iter()
        .filter(|amendment| amendment.scope.is_empty())
        .cloned()
        .collect::<Vec<_>>();
    let synthesized_tasks = synthesize_trials(&topics, &patterns, events, &unscoped);
    LearningResult {
        topics,
        requirements,
        amendments,
        candidate_tasks,
        patterns,
        synthesized_tasks,
    }
}

/// Derive the topic an event belongs to.
///
/// Metadata (conversation title, demo label, intent, tool) is preferred, but
/// events recorded from live usage often carry none of it — so the topic falls
/// back to the first significant word of the event's own text (`content`, then
/// `inputs`). Significance is judged against the multilingual stopword list in
/// the data lexicon, not an English-only constant.
pub(super) fn event_topic(event: &MemoryEvent) -> Option<String> {
    if let Some(topic) = metadata_topic(event) {
        return Some(topic);
    }
    for raw in [event.content.as_deref(), event.inputs.as_deref()]
        .into_iter()
        .flatten()
    {
        if let Some(topic) = first_significant_word(raw) {
            return Some(topic);
        }
    }
    None
}

/// An explicit topic label of the event (conversation title, demo label,
/// intent, tool).
fn metadata_topic(event: &MemoryEvent) -> Option<String> {
    for raw in [
        event.conversation_title.as_deref(),
        event.demo_label.as_deref(),
        event.intent.as_deref(),
        event.tool.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let normalized = raw.trim().to_lowercase();
        if !normalized.is_empty() && !is_topic_stopword(&normalized) {
            return Some(normalized);
        }
    }
    None
}

fn is_topic_stopword(word: &str) -> bool {
    lexicon()
        .topic_stopwords
        .iter()
        .any(|stopword| stopword == word)
}

fn first_significant_word(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|character: char| !character.is_alphanumeric())
                .to_lowercase()
        })
        .find(|token| {
            token.chars().count() >= 3
                && !token.chars().all(|character| character.is_ascii_digit())
                && !is_topic_stopword(token)
        })
}

/// Roles whose text is never a user's standing requirement (issue #1559 §1):
/// the assistant's own words, and system, developer or session instructions —
/// including one a client delivered as the first user message.
const NON_REQUIREMENT_ROLES: [&str; 4] = ["assistant", "system", "developer", "instruction"];

/// The bounded requirement statement of a user's message: one sentence that
/// carries a requirement cue, inside the size limit, naming no single task.
fn requirement_statement(event: &MemoryEvent) -> Option<String> {
    if event.role.as_deref().is_some_and(|role| {
        NON_REQUIREMENT_ROLES
            .iter()
            .any(|excluded| role.eq_ignore_ascii_case(excluded))
    }) {
        return None;
    }
    bounded_statement(event.content.as_deref()?)
}

/// The explicit topic label, else the subject of the statement itself; never a
/// pronoun or function word.
fn requirement_topic(event: &MemoryEvent, statement: &str) -> Option<String> {
    metadata_topic(event)
        .or_else(|| statement_topic(statement))
        .filter(|topic| is_valid_topic(topic))
}

/// The amendment id of `topic` in `scope`; a global amendment keeps the id it
/// always had.
fn amendment_id(topic: &str, scope: &[ScopePair]) -> String {
    let key = scope_key(scope);
    if key.is_empty() {
        crate::engine::stable_id("amendment", topic)
    } else {
        crate::engine::stable_id("amendment", &format!("{topic}\u{0}{key}"))
    }
}

/// Statements joined while the rule stays inside the bounds.
fn bounded_rule<'a>(statements: impl IntoIterator<Item = &'a str>) -> String {
    let limit = rule_max_chars();
    let mut rule = String::new();
    for statement in statements {
        let next = if rule.is_empty() {
            statement.to_owned()
        } else {
            format!("{rule}; {statement}")
        };
        if next.chars().count() <= limit {
            rule = next;
        }
    }
    rule
}

/// The amendment applies where `event` was made: every pair it carries equals
/// the event's own.
fn applies_to(amendment: &MetaAlgorithmAmendment, event: &MemoryEvent) -> bool {
    let pairs = scope_pairs(&event.evidence);
    amendment.scope.iter().all(|pair| pairs.contains(pair))
}

/// The amendments one event's replay sees.
fn records_for(
    amendments: &[MetaAlgorithmAmendment],
    event: &MemoryEvent,
) -> Vec<RetainedAmendment> {
    amendments
        .iter()
        .filter(|amendment| applies_to(amendment, event))
        .map(retained_record)
        .collect()
}

/// The specifics of the amendment's topic, in its scope, that its replay
/// re-derives.
fn covered_events(
    amendment: &MetaAlgorithmAmendment,
    events: &[MemoryEvent],
    specific_indices: &BTreeMap<String, Vec<usize>>,
) -> Vec<String> {
    specific_indices
        .get(&amendment.topic)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|index| applies_to(amendment, &events[**index]))
        .filter(|index| amendment_reproduces_specific(amendment, &events[**index]))
        .map(|index| events[*index].id.clone())
        .collect()
}

fn build_amendment(
    topic: &str,
    scope: &[ScopePair],
    grouped: &[&LearnedRequirement],
    generalized: bool,
) -> MetaAlgorithmAmendment {
    let mut sources: Vec<String> = Vec::new();
    for requirement in grouped {
        for id in &requirement.source_event_ids {
            if !sources.contains(id) {
                sources.push(id.clone());
            }
        }
    }
    MetaAlgorithmAmendment {
        id: amendment_id(topic, scope),
        topic: topic.to_owned(),
        rule: bounded_rule(
            grouped
                .iter()
                .map(|requirement| requirement.statement.as_str()),
        ),
        source_requirement_ids: sources,
        covered_event_ids: Vec::new(),
        scope: scope.to_vec(),
        generalized,
    }
}

/// The statements of one topic made in two or more distinct scopes: the
/// candidates for one amendment that applies everywhere (issue #1559 §4).
fn generalized_statements(grouped: &[&LearnedRequirement]) -> BTreeSet<String> {
    let mut scopes_by_statement: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for requirement in grouped {
        let key = scope_key(&requirement.scope);
        if key.is_empty() {
            continue;
        }
        scopes_by_statement
            .entry(requirement.statement.to_lowercase())
            .or_default()
            .insert(key);
    }
    scopes_by_statement
        .into_iter()
        .filter(|(_, scopes)| scopes.len() >= 2)
        .map(|(statement, _)| statement)
        .collect()
}

/// One amendment per topic and scope. A statement made in two or more scopes
/// is generalized into the amendment without scope, which is kept only when
/// its replay re-derives at least one stored task; otherwise the scoped
/// amendments stay.
fn generalize_amendments(
    events: &[MemoryEvent],
    requirements: &[LearnedRequirement],
    specific_indices: &BTreeMap<String, Vec<usize>>,
) -> Vec<MetaAlgorithmAmendment> {
    let mut by_topic: BTreeMap<String, Vec<&LearnedRequirement>> = BTreeMap::new();
    for requirement in requirements {
        by_topic
            .entry(requirement.topic.clone())
            .or_default()
            .push(requirement);
    }
    let mut amendments = Vec::new();
    for (topic, grouped) in by_topic {
        // A requirement stated in live chat must be learnable even when the
        // topic has no reclaimable specifics yet (issue #540 §1/§2): organic
        // stores start as raw messages plus durable `task` events, and the
        // amendment simply covers nothing until reproducible specifics appear.
        let general = generalized_statements(&grouped);
        let unscoped = grouped
            .iter()
            .copied()
            .filter(|requirement| requirement.scope.is_empty())
            .collect::<Vec<_>>();
        let mut generalized_away = BTreeSet::new();
        if !general.is_empty() {
            let mut lifted = unscoped.clone();
            for requirement in &grouped {
                let lowered = requirement.statement.to_lowercase();
                if !requirement.scope.is_empty()
                    && general.contains(&lowered)
                    && !lifted
                        .iter()
                        .any(|entry| entry.statement.to_lowercase() == lowered)
                {
                    lifted.push(*requirement);
                }
            }
            let mut candidate = build_amendment(&topic, &[], &lifted, true);
            for requirement in &grouped {
                if !requirement.scope.is_empty()
                    && general.contains(&requirement.statement.to_lowercase())
                {
                    for id in &requirement.source_event_ids {
                        if !candidate.source_requirement_ids.contains(id) {
                            candidate.source_requirement_ids.push(id.clone());
                        }
                    }
                }
            }
            candidate.covered_event_ids = covered_events(&candidate, events, specific_indices);
            if !candidate.covered_event_ids.is_empty() {
                amendments.push(candidate);
                generalized_away = general;
            }
        }
        if generalized_away.is_empty() && !unscoped.is_empty() {
            amendments.push(build_amendment(&topic, &[], &unscoped, false));
        }
        let mut by_scope: BTreeMap<String, Vec<&LearnedRequirement>> = BTreeMap::new();
        for requirement in &grouped {
            let key = scope_key(&requirement.scope);
            if key.is_empty() || generalized_away.contains(&requirement.statement.to_lowercase()) {
                continue;
            }
            by_scope.entry(key).or_default().push(*requirement);
        }
        for scoped in by_scope.into_values() {
            amendments.push(build_amendment(&topic, &scoped[0].scope, &scoped, false));
        }
    }
    for amendment in &mut amendments {
        if !amendment.generalized {
            amendment.covered_event_ids = covered_events(amendment, events, specific_indices);
        }
    }
    amendments.retain(|amendment| !amendment.rule.is_empty());
    amendments
}

/// Coverage check via the production application path: the amendment covers a
/// specific only when replaying its input through the same code that answers
/// live requests re-derives the stored output. Changing the retained rule set
/// therefore *revokes* coverage of specifics recorded under the old rules —
/// they are no longer reproducible and must not be forgotten.
fn amendment_reproduces_specific(amendment: &MetaAlgorithmAmendment, event: &MemoryEvent) -> bool {
    let (Some(input), Some(expected)) = (event.inputs.as_deref(), event.outputs.as_deref()) else {
        return false;
    };
    let simulated = replay_answer_with_amendments(input, &[retained_record(amendment)]);
    normalize(&simulated) == normalize(expected)
}

fn retained_record(amendment: &MetaAlgorithmAmendment) -> RetainedAmendment {
    RetainedAmendment {
        id: amendment.id.clone(),
        topic: amendment.topic.clone(),
        rule: amendment.rule.clone(),
    }
}

fn replay_candidate_tasks(
    events: &[MemoryEvent],
    observations: &[DreamingEventObservation],
    amendments: &[MetaAlgorithmAmendment],
    interactions: &BTreeMap<String, usize>,
) -> Vec<DreamingCandidateTask> {
    let mut candidates = events
        .iter()
        .enumerate()
        .filter(|(_, event)| is_task_event(event))
        .filter_map(|(index, event)| {
            let topic = observations[index].topic.as_deref()?;
            let input = event.inputs.as_deref()?;
            let expected = event.outputs.as_deref()?;
            // Production parity: the retained set the event's own scope sees
            // is offered and the same topic-matching gate used on live
            // requests decides what applies.
            let simulated = replay_answer_with_amendments(input, &records_for(amendments, event));
            Some(DreamingCandidateTask {
                topic: topic.to_owned(),
                source_event_id: event.id.clone(),
                input: input.to_owned(),
                expected_output: expected.to_owned(),
                passed: normalize(&simulated) == normalize(expected),
                simulated_output: simulated,
            })
        })
        .collect::<Vec<_>>();
    // Most-used topics are replayed (and later refined/synthesized) first.
    candidates.sort_by(|left, right| {
        interactions
            .get(&right.topic)
            .copied()
            .unwrap_or(0)
            .cmp(&interactions.get(&left.topic).copied().unwrap_or(0))
            .then_with(|| left.topic.cmp(&right.topic))
            .then_with(|| left.source_event_id.cmp(&right.source_event_id))
    });
    candidates
}

/// The auto-learning loop's consumer of failed replays: a failed candidate
/// whose stored output carries requirement statements missing from the topic's
/// current rule proves the meta-algorithm regressed (or never learned) those
/// statements. One refinement pass folds them back into the amendment, records
/// provenance, and re-verifies every candidate and specific on the topic.
fn refine_amendments_from_failures(
    events: &[MemoryEvent],
    specific_indices: &BTreeMap<String, Vec<usize>>,
    requirements: &mut Vec<LearnedRequirement>,
    amendments: &mut Vec<MetaAlgorithmAmendment>,
    candidates: &mut [DreamingCandidateTask],
) {
    let empty = MemoryEvent::default();
    let source_of = |id: &str| source_event(events, &empty, id);
    let mut refined: BTreeSet<String> = BTreeSet::new();
    let failures = candidates
        .iter()
        .filter(|candidate| !candidate.passed && is_valid_topic(&candidate.topic))
        .map(|candidate| {
            (
                candidate.topic.clone(),
                candidate.source_event_id.clone(),
                candidate.expected_output.clone(),
            )
        })
        .collect::<Vec<_>>();
    for (topic, source_event_id, expected_output) in failures {
        let scope = scope_pairs(&source_of(&source_event_id).evidence);
        let key = scope_key(&scope);
        let position = amendments
            .iter()
            .position(|amendment| amendment.topic == topic && scope_key(&amendment.scope) == key);
        let missing = missing_statements_for_topic(
            &expected_output,
            position.map(|position| &amendments[position]),
            &topic,
        );
        if missing.is_empty() {
            continue;
        }
        let amendment = if let Some(position) = position {
            &mut amendments[position]
        } else {
            amendments.push(MetaAlgorithmAmendment {
                id: amendment_id(&topic, &scope),
                topic: topic.clone(),
                rule: String::new(),
                source_requirement_ids: Vec::new(),
                covered_event_ids: Vec::new(),
                scope: scope.clone(),
                generalized: false,
            });
            amendments.last_mut().expect("just pushed")
        };
        for statement in missing {
            let next = if amendment.rule.is_empty() {
                statement.clone()
            } else {
                format!("{}; {statement}", amendment.rule)
            };
            if next.chars().count() > rule_max_chars() {
                continue;
            }
            amendment.rule = next;
            requirements.push(LearnedRequirement {
                topic: topic.clone(),
                statement,
                source_event_ids: vec![source_event_id.clone()],
                occurrences: 1,
                scope: scope.clone(),
            });
        }
        if !source_event_id.is_empty()
            && !amendment.source_requirement_ids.contains(&source_event_id)
        {
            amendment.source_requirement_ids.push(source_event_id);
        }
        refined.insert(amendment.id.clone());
    }
    if refined.is_empty() {
        return;
    }
    for candidate in candidates.iter_mut().filter(|candidate| !candidate.passed) {
        let records = records_for(amendments, source_of(&candidate.source_event_id));
        candidate.simulated_output = replay_answer_with_amendments(&candidate.input, &records);
        candidate.passed =
            normalize(&candidate.simulated_output) == normalize(&candidate.expected_output);
    }
    let covered = amendments
        .iter()
        .map(|amendment| {
            (refined.contains(&amendment.id) && specific_indices.contains_key(&amendment.topic))
                .then(|| covered_events(amendment, events, specific_indices))
        })
        .collect::<Vec<_>>();
    for (amendment, covered) in amendments.iter_mut().zip(covered) {
        if let Some(covered) = covered {
            amendment.covered_event_ids = covered;
        }
    }
    // A refinement whose every statement fell outside the bounds teaches
    // nothing.
    amendments.retain(|amendment| !amendment.rule.is_empty());
}

/// The stored event a candidate replayed, or an empty event without scope.
fn source_event<'a>(
    events: &'a [MemoryEvent],
    empty: &'a MemoryEvent,
    id: &str,
) -> &'a MemoryEvent {
    events.iter().find(|event| event.id == id).unwrap_or(empty)
}

/// Requirement statements present in a stored output but absent from the
/// topic's current rule. Only explicit `Learned standing requirement (...)`
/// projection lines count: they are produced exclusively by the production
/// application path, so they prove the statement was once in force. Free-form
/// cue-bearing prose is deliberately excluded — an answer that merely *quotes*
/// a requirement (for example the solver's fallback text echoing the prompt)
/// must not fold that prose into the rule. A line over the bounds is never a
/// statement (issue #1559 §2).
fn missing_statements_for_topic(
    expected_output: &str,
    current: Option<&MetaAlgorithmAmendment>,
    topic: &str,
) -> Vec<String> {
    let current_rule = current
        .map(|amendment| amendment.rule.to_lowercase())
        .unwrap_or_default();
    let marker = format!("learned standing requirement ({topic}):");
    let mut statements = Vec::new();
    for line in expected_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !line.to_lowercase().starts_with(&marker) {
            continue;
        }
        let statement = line[marker.len()..].trim().to_owned();
        if !statement.is_empty()
            && is_bounded_rule(&statement)
            && !current_rule.contains(&statement.to_lowercase())
            && !statements.contains(&statement)
        {
            statements.push(statement);
        }
    }
    statements
}

/// Mine recurring structures across task inputs without natural-language cues.
///
/// Inputs on one topic are grouped by their leading token, then each group's
/// inputs are *aligned token by token*: digit runs become `#`, positions where
/// every input agrees keep their token, and positions that vary become `*`.
/// The result is a whole-input template (e.g. `refactor * safely`, `add # #`)
/// rather than the old first-word bucket.
fn mine_patterns(
    events: &[MemoryEvent],
    observations: &[DreamingEventObservation],
) -> Vec<DreamingPattern> {
    let mut groups: BTreeMap<(String, String), Vec<(String, String)>> = BTreeMap::new();
    for (index, event) in events.iter().enumerate() {
        if !is_task_event(event) {
            continue;
        }
        let (Some(topic), Some(input)) = (
            observations[index].topic.as_deref(),
            event.inputs.as_deref(),
        ) else {
            continue;
        };
        let Some(head) = input.split_whitespace().next() else {
            continue;
        };
        groups
            .entry((topic.to_owned(), head.to_lowercase()))
            .or_default()
            .push((event.id.clone(), input.to_owned()));
    }
    groups
        .into_iter()
        .filter(|(_, members)| members.len() >= 2)
        .map(|((topic, _), members)| {
            let structure = aligned_template(
                &members
                    .iter()
                    .map(|(_, input)| input.as_str())
                    .collect::<Vec<_>>(),
            );
            DreamingPattern {
                topic,
                structure,
                occurrences: members.len(),
                source_event_ids: members.into_iter().map(|(id, _)| id).collect(),
            }
        })
        .collect()
}

fn skeleton_tokens(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(|token| {
            let lowered = token.to_lowercase();
            if lowered.chars().any(|character| character.is_ascii_digit()) {
                let mut collapsed = String::new();
                let mut in_digits = false;
                for character in lowered.chars() {
                    if character.is_ascii_digit() {
                        if !in_digits {
                            collapsed.push('#');
                            in_digits = true;
                        }
                    } else {
                        collapsed.push(character);
                        in_digits = false;
                    }
                }
                collapsed
            } else {
                lowered
            }
        })
        .collect()
}

fn aligned_template(inputs: &[&str]) -> String {
    let tokenized = inputs
        .iter()
        .map(|input| skeleton_tokens(input))
        .collect::<Vec<_>>();
    let width = tokenized.iter().map(Vec::len).max().unwrap_or(0);
    let mut template = Vec::with_capacity(width);
    for position in 0..width {
        let mut shared: Option<&str> = None;
        let mut agree = true;
        for tokens in &tokenized {
            let Some(token) = tokens.get(position) else {
                agree = false;
                break;
            };
            match shared {
                None => shared = Some(token),
                Some(existing) if existing == token => {}
                Some(_) => {
                    agree = false;
                    break;
                }
            }
        }
        template.push(if agree {
            shared.unwrap_or("*").to_owned()
        } else {
            String::from("*")
        });
    }
    template.join(" ")
}

/// Dream up new trials on the most-used topics: for every mined pattern whose
/// template has numeric slots, synthesize a fresh task instance the store has
/// never seen (each number advanced by one), solve it through the production
/// amendment path, and keep the result as retained learning. This is the
/// consumer that turns `TopicFrequency` and `DreamingPattern` from write-only
/// statistics into new work dreamed while idle.
fn synthesize_trials(
    topics: &[TopicFrequency],
    patterns: &[DreamingPattern],
    events: &[MemoryEvent],
    amendments: &[MetaAlgorithmAmendment],
) -> Vec<DreamingSynthesizedTask> {
    let top_topics = topics
        .iter()
        .take(3)
        .map(|frequency| frequency.topic.as_str())
        .collect::<BTreeSet<_>>();
    let inputs_by_id: BTreeMap<&str, &str> = events
        .iter()
        .filter_map(|event| Some((event.id.as_str(), event.inputs.as_deref()?)))
        .collect();
    let records = amendments.iter().map(retained_record).collect::<Vec<_>>();
    let mut trials = Vec::new();
    for pattern in patterns {
        if !top_topics.contains(pattern.topic.as_str()) || !pattern.structure.contains('#') {
            continue;
        }
        let Some(exemplar) = pattern
            .source_event_ids
            .iter()
            .find_map(|id| inputs_by_id.get(id.as_str()))
        else {
            continue;
        };
        let synthesized = advance_numbers(exemplar);
        if synthesized == *exemplar
            || pattern
                .source_event_ids
                .iter()
                .filter_map(|id| inputs_by_id.get(id.as_str()))
                .any(|input| **input == synthesized)
        {
            continue;
        }
        let answer = replay_answer_with_amendments(&synthesized, &records);
        trials.push(DreamingSynthesizedTask {
            topic: pattern.topic.clone(),
            structure: pattern.structure.clone(),
            input: synthesized,
            answer,
        });
    }
    trials
}

fn advance_numbers(input: &str) -> String {
    input
        .split_whitespace()
        .map(|token| {
            token
                .parse::<u64>()
                .map_or_else(|_| token.to_owned(), |value| (value + 1).to_string())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_task_event(event: &MemoryEvent) -> bool {
    let kind = event.kind.as_deref().unwrap_or_default().to_lowercase();
    let intent = event.intent.as_deref().unwrap_or_default().to_lowercase();
    let lexicon = lexicon();
    lexicon
        .task_kind_cues
        .iter()
        .any(|cue| kind.contains(cue.as_str()))
        || lexicon
            .task_intent_cues
            .iter()
            .any(|cue| intent.contains(cue.as_str()))
}

fn normalize(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
