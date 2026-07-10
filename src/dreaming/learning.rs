//! Candidate-task replay and language-independent recurring-structure mining.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use super::{
    DreamingCandidateTask, DreamingEventObservation, DreamingPattern, LearnedRequirement,
    MetaAlgorithmAmendment, TopicFrequency,
};
use crate::memory::MemoryEvent;
use crate::solver::UniversalSolver;

pub(super) struct LearningResult {
    pub topics: Vec<TopicFrequency>,
    pub requirements: Vec<LearnedRequirement>,
    pub amendments: Vec<MetaAlgorithmAmendment>,
    pub candidate_tasks: Vec<DreamingCandidateTask>,
    pub patterns: Vec<DreamingPattern>,
}

pub(super) fn learn_from_memory(
    events: &[MemoryEvent],
    observations: &[DreamingEventObservation],
    deleted_conversations: &BTreeSet<String>,
) -> LearningResult {
    let mut interactions: BTreeMap<String, usize> = BTreeMap::new();
    let mut task_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut requirement_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut requirements: BTreeMap<(String, String), LearnedRequirement> = BTreeMap::new();
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
            if observations[index].durability.is_reclaimable() && !event.id.is_empty() {
                specific_indices
                    .entry(topic.clone())
                    .or_default()
                    .push(index);
            }
        }
        if let Some(statement) = requirement_statement(event) {
            *requirement_counts.entry(topic.clone()).or_default() += 1;
            let entry = requirements
                .entry((topic.clone(), statement.to_lowercase()))
                .or_insert_with(|| LearnedRequirement {
                    topic: topic.clone(),
                    statement: statement.clone(),
                    source_event_ids: Vec::new(),
                    occurrences: 0,
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
    let requirements = requirements.into_values().collect::<Vec<_>>();
    let amendments = generalize_amendments(events, &requirements, &specific_indices);
    let candidate_tasks = replay_candidate_tasks(events, observations, &amendments);
    let patterns = mine_patterns(events, observations);
    LearningResult {
        topics,
        requirements,
        amendments,
        candidate_tasks,
        patterns,
    }
}

pub(super) fn event_topic(event: &MemoryEvent) -> Option<String> {
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
        if !matches!(
            normalized.as_str(),
            "" | "message"
                | "ask"
                | "reply"
                | "chat"
                | "note"
                | "generalize"
                | "system"
                | "user"
                | "assistant"
        ) {
            return Some(normalized);
        }
    }
    None
}

fn requirement_statement(event: &MemoryEvent) -> Option<String> {
    if event.role.as_deref().is_some_and(|role| {
        role.eq_ignore_ascii_case("assistant") || role.eq_ignore_ascii_case("system")
    }) {
        return None;
    }
    let content = event.content.as_deref()?.trim();
    let lowered = content.to_lowercase();
    requirement_cues()
        .iter()
        .any(|cue| lowered.contains(cue))
        .then(|| content.to_owned())
}

fn requirement_cues() -> Vec<String> {
    include_str!("../../data/meta/dreaming-cues.lino")
        .lines()
        .filter_map(|line| line.trim().strip_prefix("cue \"")?.strip_suffix('"'))
        .map(str::to_lowercase)
        .collect()
}

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
    by_topic
        .into_iter()
        .filter_map(|(topic, requirements)| {
            let indices = specific_indices.get(&topic)?;
            let rule = requirements
                .iter()
                .map(|requirement| requirement.statement.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            let mut amendment = MetaAlgorithmAmendment {
                id: crate::engine::stable_id("amendment", &topic),
                topic,
                rule,
                source_requirement_ids: requirements
                    .iter()
                    .flat_map(|requirement| requirement.source_event_ids.iter().cloned())
                    .collect(),
                covered_event_ids: Vec::new(),
            };
            amendment.covered_event_ids = indices
                .iter()
                .filter(|index| amendment_reproduces_specific(&amendment, &events[**index]))
                .map(|index| events[*index].id.clone())
                .collect();
            Some(amendment)
        })
        .collect()
}

fn amendment_reproduces_specific(amendment: &MetaAlgorithmAmendment, event: &MemoryEvent) -> bool {
    let (Some(input), Some(expected)) = (event.inputs.as_deref(), event.outputs.as_deref()) else {
        return false;
    };
    let simulated = simulate_candidate_output(input, &amendment.topic, [amendment]);
    normalize(&simulated) == normalize(expected)
}

fn replay_candidate_tasks(
    events: &[MemoryEvent],
    observations: &[DreamingEventObservation],
    amendments: &[MetaAlgorithmAmendment],
) -> Vec<DreamingCandidateTask> {
    events
        .iter()
        .enumerate()
        .filter(|(_, event)| is_task_event(event))
        .filter_map(|(index, event)| {
            let topic = observations[index].topic.as_deref()?;
            let input = event.inputs.as_deref()?;
            let expected = event.outputs.as_deref()?;
            let matching = amendments
                .iter()
                .filter(|amendment| amendment.topic == topic)
                .collect::<Vec<_>>();
            let simulated = simulate_candidate_output(input, topic, matching);
            Some(DreamingCandidateTask {
                topic: topic.to_owned(),
                source_event_id: event.id.clone(),
                input: input.to_owned(),
                expected_output: expected.to_owned(),
                passed: normalize(&simulated) == normalize(expected),
                simulated_output: simulated,
            })
        })
        .collect()
}

fn simulate_candidate_output<'a>(
    input: &str,
    topic: &str,
    amendments: impl IntoIterator<Item = &'a MetaAlgorithmAmendment>,
) -> String {
    let mut simulated = UniversalSolver::default().solve(input).answer;
    for amendment in amendments {
        let _ = write!(
            simulated,
            "\n\nLearned standing requirement ({topic}): {}",
            amendment.rule
        );
    }
    simulated
}

fn mine_patterns(
    events: &[MemoryEvent],
    observations: &[DreamingEventObservation],
) -> Vec<DreamingPattern> {
    let mut groups: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
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
        let structure = format!("{} *", head.to_lowercase());
        groups
            .entry((topic.to_owned(), structure))
            .or_default()
            .push(event.id.clone());
    }
    groups
        .into_iter()
        .filter(|(_, ids)| ids.len() >= 2)
        .map(|((topic, structure), source_event_ids)| DreamingPattern {
            topic,
            structure,
            occurrences: source_event_ids.len(),
            source_event_ids,
        })
        .collect()
}

fn is_task_event(event: &MemoryEvent) -> bool {
    let kind = event.kind.as_deref().unwrap_or_default().to_lowercase();
    let intent = event.intent.as_deref().unwrap_or_default().to_lowercase();
    ["task", "test_run", "test-run", "trial", "run_log"]
        .iter()
        .any(|cue| kind.contains(cue))
        || ["task", "solve", "test"]
            .iter()
            .any(|cue| intent.contains(cue))
}

fn normalize(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
