//! Low-priority memory maintenance planning **and** self-generalization.
//!
//! Dreaming is deliberately split into a pure planner and an explicit apply
//! helper. The planner can run by default in background contexts because it
//! only reads memory and proposes work. Physical deletion remains a caller
//! decision guarded by the same confirmation/backup flow as other maintenance
//! commands.
//!
//! Beyond garbage collection, dreaming is where Formal AI *learns from its own
//! stored experience* (issue #540). While idle, the planner recalculates which
//! topics the user interacts with most, extracts the durable requirements the
//! user has stated on those topics, and **generalizes** them into
//! [`MetaAlgorithmAmendment`] records — changes baked into how similar future
//! tasks are solved so the user never has to repeat a requirement. Because the
//! amendment can re-derive the specific task/test-run records it subsumes, those
//! specifics become safe to forget under storage pressure while the generalized
//! amendment is retained forever. This is the issue's core rule: "as soon as
//! [a generalization] is working, [the] specific algorithm can be forgotten",
//! yet "our general meta algorithm must keep changes that allow it to solve all
//! other tasks".

use std::collections::{BTreeMap, BTreeSet};

use crate::memory::{MemoryEvent, MemoryStore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamingConfig {
    /// Background planning is on by default; disabling it returns an empty plan.
    pub daydreaming_enabled: bool,
    /// Desired free space after satisfying the next known incoming write.
    pub target_free_ratio_percent: u8,
    /// Total capacity of the storage area that holds the memory file.
    pub storage_capacity_bytes: Option<u64>,
    /// Current free bytes in the storage area.
    pub free_bytes: Option<u64>,
    /// Bytes the caller expects to need for the next write/fetch.
    pub incoming_bytes: u64,
}

impl Default for DreamingConfig {
    fn default() -> Self {
        Self {
            daydreaming_enabled: true,
            target_free_ratio_percent: 20,
            storage_capacity_bytes: None,
            free_bytes: None,
            incoming_bytes: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DreamingDurability {
    /// Raw user/assistant/system experience that cannot be reconstructed from a
    /// public source.
    IrreplaceableRaw,
    /// Promoted lessons, skill ledgers, and learned experience.
    RetainedLearning,
    /// Events already attached to a soft-deleted conversation.
    DeletedConversation,
    /// Public-source cache or tool output that can be fetched again.
    RecomputableCache,
    /// Summaries and conclusions that can be derived from retained inputs.
    RecomputableIntermediate,
}

impl DreamingDurability {
    #[must_use]
    pub const fn is_reclaimable(self) -> bool {
        matches!(
            self,
            Self::DeletedConversation | Self::RecomputableCache | Self::RecomputableIntermediate
        )
    }

    const fn pressure_priority(self) -> u8 {
        match self {
            Self::DeletedConversation => 0,
            Self::RecomputableCache => 1,
            Self::RecomputableIntermediate => 2,
            Self::RetainedLearning | Self::IrreplaceableRaw => 9,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DreamingActionKind {
    PurgeDeletedConversation,
    RemoveDuplicateRecomputable,
    EvictLowUseRecomputable,
    /// A specific task/test-run record that a retained meta-algorithm amendment
    /// can reproduce, so it is forgotten first under pressure.
    ForgetCoveredSpecific,
}

/// How often the user has interacted with one topic, recalculated from the
/// current memory graph. Topics are the unit dreaming learns about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicFrequency {
    pub topic: String,
    /// Total events attributed to the topic.
    pub interactions: usize,
    /// Events that look like concrete tasks / test runs on the topic.
    pub task_events: usize,
    /// Events that state a durable user requirement on the topic.
    pub requirement_events: usize,
}

/// A durable requirement the user stated on a topic. Dreaming remembers these so
/// the user never has to repeat them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnedRequirement {
    pub topic: String,
    pub statement: String,
    pub source_event_ids: Vec<String>,
    pub occurrences: usize,
}

/// A generalization baked into Formal AI's meta-algorithm: when solving tasks on
/// `topic`, always apply `rule`.
///
/// Materialized into memory as a retained, never-reclaimable learning record;
/// the specifics it `covered_event_ids` subsumes become safe to forget under
/// pressure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaAlgorithmAmendment {
    pub id: String,
    pub topic: String,
    pub rule: String,
    pub source_requirement_ids: Vec<String>,
    pub covered_event_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamingAction {
    pub kind: DreamingActionKind,
    pub event_id: String,
    pub conversation_id: Option<String>,
    pub estimated_bytes: u64,
    pub usage_count: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamingEventObservation {
    pub event_id: String,
    pub durability: DreamingDurability,
    pub usage_count: usize,
    pub estimated_bytes: u64,
    pub duplicate_key: Option<String>,
    /// The topic this event belongs to, if one could be recalculated.
    pub topic: Option<String>,
    /// True when a retained meta-algorithm amendment can reproduce this record,
    /// so it is safe to forget first under pressure.
    pub covered_by_amendment: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamingPlan {
    pub daydreaming_enabled: bool,
    pub target_free_ratio_percent: u8,
    pub storage_capacity_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
    pub incoming_bytes: u64,
    pub target_free_bytes: Option<u64>,
    pub required_reclaim_bytes: u64,
    pub selected_reclaim_bytes: u64,
    pub total_reclaimable_bytes: u64,
    pub requires_bigger_storage: bool,
    pub actions: Vec<DreamingAction>,
    pub observations: Vec<DreamingEventObservation>,
    /// Topics ranked by recalculated interaction frequency (most-used first).
    pub topics: Vec<TopicFrequency>,
    /// Durable user requirements recovered from memory.
    pub learned_requirements: Vec<LearnedRequirement>,
    /// Meta-algorithm generalizations baked in from the learned requirements.
    pub amendments: Vec<MetaAlgorithmAmendment>,
}

impl DreamingPlan {
    #[must_use]
    pub fn event_usage(&self, event_id: &str) -> Option<usize> {
        self.observations
            .iter()
            .find(|observation| observation.event_id == event_id)
            .map(|observation| observation.usage_count)
    }

    /// The recalculated interaction count for a topic, if it was observed.
    #[must_use]
    pub fn topic_interactions(&self, topic: &str) -> Option<usize> {
        self.topics
            .iter()
            .find(|frequency| frequency.topic == topic)
            .map(|frequency| frequency.interactions)
    }

    /// The amendment learned for a topic, if any.
    #[must_use]
    pub fn amendment_for(&self, topic: &str) -> Option<&MetaAlgorithmAmendment> {
        self.amendments
            .iter()
            .find(|amendment| amendment.topic == topic)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamingOutcome {
    pub removed_events: usize,
    pub estimated_reclaimed_bytes: u64,
    /// Meta-algorithm amendments newly materialized into the store.
    pub learned_amendments: usize,
}

#[must_use]
pub fn plan_memory_dreaming(events: &[MemoryEvent], config: &DreamingConfig) -> DreamingPlan {
    let target_free_ratio_percent = config.target_free_ratio_percent.min(100);
    let target_free_bytes = config
        .storage_capacity_bytes
        .map(|capacity| percent_ceil(capacity, target_free_ratio_percent));
    let required_reclaim_bytes =
        required_reclaim_bytes(target_free_bytes, config.free_bytes, config.incoming_bytes);

    if !config.daydreaming_enabled {
        return DreamingPlan {
            daydreaming_enabled: false,
            target_free_ratio_percent,
            storage_capacity_bytes: config.storage_capacity_bytes,
            free_bytes: config.free_bytes,
            incoming_bytes: config.incoming_bytes,
            target_free_bytes,
            required_reclaim_bytes,
            selected_reclaim_bytes: 0,
            total_reclaimable_bytes: 0,
            requires_bigger_storage: required_reclaim_bytes > 0,
            actions: Vec::new(),
            observations: Vec::new(),
            topics: Vec::new(),
            learned_requirements: Vec::new(),
            amendments: Vec::new(),
        };
    }

    let deleted_conversations = deleted_conversation_ids(events);
    let usage_counts = usage_counts(events);
    let mut observations = Vec::with_capacity(events.len());
    let mut reclaimable_candidates = Vec::new();
    let mut duplicate_groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for (index, event) in events.iter().enumerate() {
        let durability = classify_event(event, &deleted_conversations);
        let estimated_bytes = estimate_event_bytes(event);
        let duplicate_key = duplicate_key(event, durability);
        if let Some(key) = duplicate_key.clone() {
            duplicate_groups.entry(key).or_default().push(index);
        }
        if durability.is_reclaimable() && !event.id.is_empty() {
            reclaimable_candidates.push(index);
        }
        observations.push(DreamingEventObservation {
            event_id: event.id.clone(),
            durability,
            usage_count: usage_counts[index],
            estimated_bytes,
            duplicate_key,
            topic: event_topic(event),
            covered_by_amendment: false,
        });
    }

    // Self-generalization: learn topics and durable requirements from the same
    // memory graph, then bake each requirement into a meta-algorithm amendment.
    let (topics, learned_requirements, amendments) =
        learn_from_memory(events, &observations, &deleted_conversations);
    let covered_event_ids: BTreeSet<&str> = amendments
        .iter()
        .flat_map(|amendment| amendment.covered_event_ids.iter().map(String::as_str))
        .collect();
    for observation in &mut observations {
        observation.covered_by_amendment =
            covered_event_ids.contains(observation.event_id.as_str());
    }

    let total_reclaimable_bytes = reclaimable_candidates
        .iter()
        .map(|index| observations[*index].estimated_bytes)
        .sum();
    let mut actions = Vec::new();
    let mut selected_event_ids = BTreeSet::new();

    for event_index in deleted_event_indices(events, &deleted_conversations) {
        push_action_once(
            &mut actions,
            &mut selected_event_ids,
            DreamingAction {
                kind: DreamingActionKind::PurgeDeletedConversation,
                event_id: events[event_index].id.clone(),
                conversation_id: events[event_index].conversation_id.clone(),
                estimated_bytes: observations[event_index].estimated_bytes,
                usage_count: observations[event_index].usage_count,
                reason: String::from("event belongs to a conversation already marked deleted"),
            },
        );
    }

    for group in duplicate_groups.values() {
        if group.len() < 2 {
            continue;
        }
        let keep = group
            .iter()
            .copied()
            .max_by(|left, right| {
                usage_counts[*left]
                    .cmp(&usage_counts[*right])
                    .then_with(|| {
                        observations[*left]
                            .estimated_bytes
                            .cmp(&observations[*right].estimated_bytes)
                    })
                    .then_with(|| right.cmp(left))
            })
            .unwrap_or(group[0]);
        for event_index in group.iter().copied().filter(|index| *index != keep) {
            push_action_once(
                &mut actions,
                &mut selected_event_ids,
                DreamingAction {
                    kind: DreamingActionKind::RemoveDuplicateRecomputable,
                    event_id: events[event_index].id.clone(),
                    conversation_id: events[event_index].conversation_id.clone(),
                    estimated_bytes: observations[event_index].estimated_bytes,
                    usage_count: observations[event_index].usage_count,
                    reason: format!(
                        "duplicate recomputable event; retained {} with usage {}",
                        events[keep].id, usage_counts[keep]
                    ),
                },
            );
        }
    }

    let mut selected_reclaim_bytes = selected_bytes(&actions);
    if required_reclaim_bytes > selected_reclaim_bytes {
        let mut pressure_candidates = reclaimable_candidates
            .into_iter()
            .filter(|index| !selected_event_ids.contains(events[*index].id.as_str()))
            .collect::<Vec<_>>();
        pressure_candidates.sort_by(|left, right| {
            // Specifics a retained amendment can reproduce are forgotten first.
            observations[*right]
                .covered_by_amendment
                .cmp(&observations[*left].covered_by_amendment)
                .then_with(|| {
                    observations[*left]
                        .usage_count
                        .cmp(&observations[*right].usage_count)
                })
                .then_with(|| {
                    observations[*left]
                        .durability
                        .pressure_priority()
                        .cmp(&observations[*right].durability.pressure_priority())
                })
                .then_with(|| {
                    observations[*right]
                        .estimated_bytes
                        .cmp(&observations[*left].estimated_bytes)
                })
                .then_with(|| left.cmp(right))
        });

        for event_index in pressure_candidates {
            if selected_reclaim_bytes >= required_reclaim_bytes {
                break;
            }
            let (kind, reason) = if observations[event_index].covered_by_amendment {
                (
                    DreamingActionKind::ForgetCoveredSpecific,
                    String::from(
                        "specific task/test-run record reproducible from a retained meta-algorithm amendment",
                    ),
                )
            } else {
                (
                    DreamingActionKind::EvictLowUseRecomputable,
                    String::from(
                        "lowest-use recomputable event selected to satisfy the free-space target",
                    ),
                )
            };
            push_action_once(
                &mut actions,
                &mut selected_event_ids,
                DreamingAction {
                    kind,
                    event_id: events[event_index].id.clone(),
                    conversation_id: events[event_index].conversation_id.clone(),
                    estimated_bytes: observations[event_index].estimated_bytes,
                    usage_count: observations[event_index].usage_count,
                    reason,
                },
            );
            selected_reclaim_bytes = selected_bytes(&actions);
        }
    }

    selected_reclaim_bytes = selected_bytes(&actions);
    DreamingPlan {
        daydreaming_enabled: true,
        target_free_ratio_percent,
        storage_capacity_bytes: config.storage_capacity_bytes,
        free_bytes: config.free_bytes,
        incoming_bytes: config.incoming_bytes,
        target_free_bytes,
        required_reclaim_bytes,
        selected_reclaim_bytes,
        total_reclaimable_bytes,
        requires_bigger_storage: required_reclaim_bytes > selected_reclaim_bytes,
        actions,
        observations,
        topics,
        learned_requirements,
        amendments,
    }
}

#[must_use]
pub fn apply_dreaming_plan(store: &mut MemoryStore, plan: &DreamingPlan) -> DreamingOutcome {
    let selected_ids = plan
        .actions
        .iter()
        .map(|action| action.event_id.as_str())
        .collect::<BTreeSet<_>>();

    // Bake each learned generalization into memory as a retained learning record
    // *before* forgetting the specifics it covers. Applying an unchanged plan
    // twice must not duplicate amendments, so we skip ids already present.
    let existing_ids: BTreeSet<String> = store
        .events()
        .iter()
        .map(|event| event.id.clone())
        .collect();
    let new_amendments: Vec<MemoryEvent> = plan
        .amendments
        .iter()
        .filter(|amendment| !existing_ids.contains(&amendment.id))
        .map(amendment_event)
        .collect();
    let learned_amendments = new_amendments.len();

    if selected_ids.is_empty() && new_amendments.is_empty() {
        return DreamingOutcome {
            removed_events: 0,
            estimated_reclaimed_bytes: 0,
            learned_amendments: 0,
        };
    }

    let initial_len = store.len();
    let mut retained = store
        .events()
        .iter()
        .filter(|event| !selected_ids.contains(event.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let removed_events = initial_len - retained.len();
    retained.extend(new_amendments);
    *store = MemoryStore::from_events(retained);
    DreamingOutcome {
        removed_events,
        estimated_reclaimed_bytes: selected_bytes(&plan.actions),
        learned_amendments,
    }
}

/// Render a learned generalization as a durable, never-reclaimable memory event.
fn amendment_event(amendment: &MetaAlgorithmAmendment) -> MemoryEvent {
    MemoryEvent {
        id: amendment.id.clone(),
        kind: Some(String::from("meta_algorithm_amendment")),
        role: Some(String::from("system")),
        intent: Some(String::from("generalize")),
        content: Some(amendment.rule.clone()),
        demo_label: Some(amendment.topic.clone()),
        conversation_title: Some(amendment.topic.clone()),
        evidence: amendment.source_requirement_ids.clone(),
        ..MemoryEvent::default()
    }
}

#[must_use]
pub fn render_dreaming_plan(plan: &DreamingPlan) -> String {
    let mut lines = Vec::new();
    lines.push(String::from("memory_dreaming_plan"));
    lines.push(format!("  enabled: {}", plan.daydreaming_enabled));
    lines.push(format!(
        "  target_free_ratio_percent: {}",
        plan.target_free_ratio_percent
    ));
    if let Some(capacity) = plan.storage_capacity_bytes {
        lines.push(format!("  storage_capacity_bytes: {capacity}"));
    }
    if let Some(free) = plan.free_bytes {
        lines.push(format!("  free_bytes: {free}"));
    }
    if let Some(target) = plan.target_free_bytes {
        lines.push(format!("  target_free_bytes: {target}"));
    }
    lines.push(format!("  incoming_bytes: {}", plan.incoming_bytes));
    lines.push(format!(
        "  required_reclaim_bytes: {}",
        plan.required_reclaim_bytes
    ));
    lines.push(format!(
        "  selected_reclaim_bytes: {}",
        plan.selected_reclaim_bytes
    ));
    lines.push(format!(
        "  total_reclaimable_bytes: {}",
        plan.total_reclaimable_bytes
    ));
    lines.push(format!(
        "  requires_bigger_storage: {}",
        plan.requires_bigger_storage
    ));
    if plan.actions.is_empty() {
        lines.push(String::from("  action: none"));
    } else {
        for action in &plan.actions {
            lines.push(format!(
                "  action {:?} event={} bytes={} usage={} reason={}",
                action.kind,
                action.event_id,
                action.estimated_bytes,
                action.usage_count,
                action.reason
            ));
        }
    }

    if plan.topics.is_empty() {
        lines.push(String::from("  topic: none"));
    } else {
        for topic in &plan.topics {
            lines.push(format!(
                "  topic {} interactions={} tasks={} requirements={}",
                topic.topic, topic.interactions, topic.task_events, topic.requirement_events
            ));
        }
    }

    for requirement in &plan.learned_requirements {
        lines.push(format!(
            "  learned_requirement topic={} occurrences={} statement={}",
            requirement.topic, requirement.occurrences, requirement.statement
        ));
    }

    for amendment in &plan.amendments {
        lines.push(format!(
            "  meta_algorithm_amendment id={} topic={} covers={} rule={}",
            amendment.id,
            amendment.topic,
            amendment.covered_event_ids.len(),
            amendment.rule
        ));
    }

    lines.join("\n")
}

fn push_action_once(
    actions: &mut Vec<DreamingAction>,
    selected_event_ids: &mut BTreeSet<String>,
    action: DreamingAction,
) {
    if action.event_id.is_empty() || !selected_event_ids.insert(action.event_id.clone()) {
        return;
    }
    actions.push(action);
}

fn deleted_conversation_ids(events: &[MemoryEvent]) -> BTreeSet<String> {
    events
        .iter()
        .filter(|event| event.kind.as_deref() == Some("conversation_deleted"))
        .filter_map(|event| event.conversation_id.as_deref())
        .map(ToOwned::to_owned)
        .collect()
}

fn deleted_event_indices(
    events: &[MemoryEvent],
    deleted_conversations: &BTreeSet<String>,
) -> Vec<usize> {
    events
        .iter()
        .enumerate()
        .filter(|(_, event)| {
            event
                .conversation_id
                .as_deref()
                .is_some_and(|id| deleted_conversations.contains(id))
        })
        .map(|(index, _)| index)
        .collect()
}

fn classify_event(
    event: &MemoryEvent,
    deleted_conversations: &BTreeSet<String>,
) -> DreamingDurability {
    if event
        .conversation_id
        .as_deref()
        .is_some_and(|id| deleted_conversations.contains(id))
    {
        return DreamingDurability::DeletedConversation;
    }

    let kind = lower_opt(event.kind.as_deref());
    let tool = lower_opt(event.tool.as_deref());
    let content = lower_opt(event.content.as_deref());
    let evidence = event.evidence.join("\n").to_ascii_lowercase();
    if contains_any(
        &kind,
        &[
            "learning",
            "ledger",
            "skill",
            "meta_algorithm",
            "amendment",
            "generalization",
        ],
    ) || contains_any(&content, &["learned", "learning ledger", "promoted lesson"])
        || evidence.contains("learning")
    {
        return DreamingDurability::RetainedLearning;
    }

    if contains_any(
        &kind,
        &["source:", "source_", "cache", "http", "web_search"],
    ) || contains_any(&tool, &["web_search", "http", "fetch", "source", "cache"])
    {
        return DreamingDurability::RecomputableCache;
    }

    if contains_any(
        &kind,
        &[
            "intermediate",
            "summary",
            "analysis",
            "conclusion",
            "derived",
            "test_run",
            "test-run",
            "trial",
            "run_log",
        ],
    ) {
        return DreamingDurability::RecomputableIntermediate;
    }

    DreamingDurability::IrreplaceableRaw
}

/// Cues that mark a piece of text as a durable user requirement rather than a
/// one-off request. Kept deliberately conservative so learning stays precise.
const REQUIREMENT_CUES: &[&str] = &[
    "always",
    "never",
    "must",
    "should",
    "requirement",
    "require",
    "prefer",
    "make sure",
    "ensure",
    "do not",
    "don't",
];

/// Intents/labels too generic to identify a topic the user actually cares about.
fn is_generic_topic(value: &str) -> bool {
    matches!(
        value,
        "" | "message"
            | "ask"
            | "reply"
            | "chat"
            | "note"
            | "generalize"
            | "system"
            | "user"
            | "assistant"
    )
}

/// Recalculate which topic an event belongs to, preferring the most specific
/// stable label available.
fn event_topic(event: &MemoryEvent) -> Option<String> {
    for raw in [
        event.conversation_title.as_deref(),
        event.demo_label.as_deref(),
        event.intent.as_deref(),
        event.tool.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let normalized = raw.trim().to_ascii_lowercase();
        if !is_generic_topic(&normalized) {
            return Some(normalized);
        }
    }
    None
}

/// A concrete task or test run on a topic, as opposed to a durable requirement.
fn is_task_event(event: &MemoryEvent) -> bool {
    let kind = lower_opt(event.kind.as_deref());
    let intent = lower_opt(event.intent.as_deref());
    contains_any(&kind, &["task", "test_run", "test-run", "trial", "run_log"])
        || contains_any(&intent, &["task", "solve", "test"])
}

/// Extract a durable requirement statement from an event, if it states one. Only
/// user-authored (or unlabeled) events count so amendments and assistant chatter
/// are never mistaken for standing requirements.
fn requirement_statement(event: &MemoryEvent) -> Option<String> {
    if event.role.as_deref().is_some_and(|role| {
        role.eq_ignore_ascii_case("assistant") || role.eq_ignore_ascii_case("system")
    }) {
        return None;
    }
    let content = event.content.as_deref()?.trim();
    if content.is_empty() {
        return None;
    }
    if !contains_any(&content.to_ascii_lowercase(), REQUIREMENT_CUES) {
        return None;
    }
    Some(content.to_string())
}

/// Turn a topic name into a stable, id-safe slug.
fn topic_slug(topic: &str) -> String {
    topic
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Learn from stored experience: recalculate topic interaction frequency, recover
/// durable user requirements, and generalize them into meta-algorithm amendments
/// whose retained rule can reproduce the specific task/test-run records it covers.
fn learn_from_memory(
    events: &[MemoryEvent],
    observations: &[DreamingEventObservation],
    deleted_conversations: &BTreeSet<String>,
) -> (
    Vec<TopicFrequency>,
    Vec<LearnedRequirement>,
    Vec<MetaAlgorithmAmendment>,
) {
    let mut interactions: BTreeMap<String, usize> = BTreeMap::new();
    let mut task_events: BTreeMap<String, usize> = BTreeMap::new();
    let mut requirement_events: BTreeMap<String, usize> = BTreeMap::new();
    let mut requirements: BTreeMap<(String, String), LearnedRequirement> = BTreeMap::new();
    let mut coverable: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (index, event) in events.iter().enumerate() {
        // Events on deleted conversations are being purged; do not learn from them.
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
            *task_events.entry(topic.clone()).or_default() += 1;
            // Only specifics that can be recomputed are safe to eventually forget.
            if observations[index].durability.is_reclaimable() && !event.id.is_empty() {
                coverable
                    .entry(topic.clone())
                    .or_default()
                    .push(event.id.clone());
            }
        }

        if let Some(statement) = requirement_statement(event) {
            *requirement_events.entry(topic.clone()).or_default() += 1;
            let key = (topic.clone(), statement.to_ascii_lowercase());
            let entry = requirements
                .entry(key)
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

    let mut topics: Vec<TopicFrequency> = interactions
        .iter()
        .map(|(topic, count)| TopicFrequency {
            topic: topic.clone(),
            interactions: *count,
            task_events: task_events.get(topic).copied().unwrap_or(0),
            requirement_events: requirement_events.get(topic).copied().unwrap_or(0),
        })
        .collect();
    topics.sort_by(|left, right| {
        right
            .interactions
            .cmp(&left.interactions)
            .then_with(|| left.topic.cmp(&right.topic))
    });

    let learned_requirements: Vec<LearnedRequirement> = requirements.into_values().collect();

    // Generalize: one amendment per topic that has both a durable requirement and
    // specific task/test-run records the amendment can reproduce.
    let mut by_topic: BTreeMap<String, Vec<&LearnedRequirement>> = BTreeMap::new();
    for requirement in &learned_requirements {
        by_topic
            .entry(requirement.topic.clone())
            .or_default()
            .push(requirement);
    }
    let mut amendments = Vec::new();
    for (topic, reqs) in &by_topic {
        let covered = coverable.get(topic).cloned().unwrap_or_default();
        if covered.is_empty() {
            continue;
        }
        let mut source_requirement_ids = Vec::new();
        let mut statements = Vec::new();
        for requirement in reqs {
            statements.push(requirement.statement.clone());
            source_requirement_ids.extend(requirement.source_event_ids.iter().cloned());
        }
        let rule = format!(
            "When solving \"{topic}\" tasks, always apply the user's standing requirement(s): {}",
            statements.join("; ")
        );
        amendments.push(MetaAlgorithmAmendment {
            id: format!("amendment:{}", topic_slug(topic)),
            topic: topic.clone(),
            rule,
            source_requirement_ids,
            covered_event_ids: covered,
        });
    }

    (topics, learned_requirements, amendments)
}

fn duplicate_key(event: &MemoryEvent, durability: DreamingDurability) -> Option<String> {
    if !matches!(
        durability,
        DreamingDurability::RecomputableCache | DreamingDurability::RecomputableIntermediate
    ) {
        return None;
    }
    Some(format!(
        "kind={}|role={}|intent={}|tool={}|inputs={}|outputs={}|content={}",
        normalized(event.kind.as_deref()),
        normalized(event.role.as_deref()),
        normalized(event.intent.as_deref()),
        normalized(event.tool.as_deref()),
        normalized(event.inputs.as_deref()),
        normalized(event.outputs.as_deref()),
        normalized(event.content.as_deref()),
    ))
}

fn usage_counts(events: &[MemoryEvent]) -> Vec<usize> {
    let searchable = events.iter().map(searchable_text).collect::<Vec<_>>();
    events
        .iter()
        .enumerate()
        .map(|(target_index, target)| {
            if target.id.is_empty() {
                return 0;
            }
            searchable
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != target_index)
                .map(|(_, text)| text.matches(&target.id).count())
                .sum()
        })
        .collect()
}

fn searchable_text(event: &MemoryEvent) -> String {
    let mut text = String::new();
    push_opt(&mut text, event.kind.as_deref());
    push_opt(&mut text, event.role.as_deref());
    push_opt(&mut text, event.intent.as_deref());
    push_opt(&mut text, event.tool.as_deref());
    push_opt(&mut text, event.inputs.as_deref());
    push_opt(&mut text, event.outputs.as_deref());
    push_opt(&mut text, event.content.as_deref());
    push_opt(&mut text, event.demo_label.as_deref());
    push_opt(&mut text, event.conversation_id.as_deref());
    push_opt(&mut text, event.conversation_title.as_deref());
    for evidence in &event.evidence {
        push_opt(&mut text, Some(evidence));
    }
    text
}

fn estimate_event_bytes(event: &MemoryEvent) -> u64 {
    64 + string_bytes(&event.id)
        + option_bytes(event.kind.as_deref())
        + option_bytes(event.role.as_deref())
        + option_bytes(event.intent.as_deref())
        + option_bytes(event.tool.as_deref())
        + option_bytes(event.inputs.as_deref())
        + option_bytes(event.outputs.as_deref())
        + option_bytes(event.content.as_deref())
        + option_bytes(event.sent_at.as_deref())
        + option_bytes(event.demo_label.as_deref())
        + option_bytes(event.conversation_id.as_deref())
        + option_bytes(event.conversation_title.as_deref())
        + event
            .evidence
            .iter()
            .map(|entry| string_bytes(entry))
            .sum::<u64>()
}

fn selected_bytes(actions: &[DreamingAction]) -> u64 {
    actions.iter().map(|action| action.estimated_bytes).sum()
}

fn required_reclaim_bytes(
    target_free_bytes: Option<u64>,
    free_bytes: Option<u64>,
    incoming_bytes: u64,
) -> u64 {
    let Some(target_free_bytes) = target_free_bytes else {
        return 0;
    };
    let free_after_incoming = free_bytes.unwrap_or(0).saturating_sub(incoming_bytes);
    target_free_bytes.saturating_sub(free_after_incoming)
}

fn percent_ceil(total: u64, percent: u8) -> u64 {
    if percent == 0 || total == 0 {
        return 0;
    }
    total.saturating_mul(u64::from(percent)).saturating_add(99) / 100
}

fn push_opt(target: &mut String, value: Option<&str>) {
    if let Some(value) = value {
        target.push('\n');
        target.push_str(value);
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn lower_opt(value: Option<&str>) -> String {
    value.unwrap_or_default().to_ascii_lowercase()
}

fn normalized(value: Option<&str>) -> String {
    value.unwrap_or_default().trim().to_ascii_lowercase()
}

fn option_bytes(value: Option<&str>) -> u64 {
    value.map_or(0, string_bytes)
}

const fn string_bytes(value: &str) -> u64 {
    value.len() as u64
}
