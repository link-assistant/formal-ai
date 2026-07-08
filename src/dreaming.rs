//! Low-priority memory maintenance planning.
//!
//! Dreaming is deliberately split into a pure planner and an explicit apply
//! helper. The planner can run by default in background contexts because it
//! only reads memory and proposes work. Physical deletion remains a caller
//! decision guarded by the same confirmation/backup flow as other maintenance
//! commands.

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
}

impl DreamingPlan {
    #[must_use]
    pub fn event_usage(&self, event_id: &str) -> Option<usize> {
        self.observations
            .iter()
            .find(|observation| observation.event_id == event_id)
            .map(|observation| observation.usage_count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamingOutcome {
    pub removed_events: usize,
    pub estimated_reclaimed_bytes: u64,
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
        });
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
            observations[*left]
                .usage_count
                .cmp(&observations[*right].usage_count)
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
            push_action_once(
                &mut actions,
                &mut selected_event_ids,
                DreamingAction {
                    kind: DreamingActionKind::EvictLowUseRecomputable,
                    event_id: events[event_index].id.clone(),
                    conversation_id: events[event_index].conversation_id.clone(),
                    estimated_bytes: observations[event_index].estimated_bytes,
                    usage_count: observations[event_index].usage_count,
                    reason: String::from(
                        "lowest-use recomputable event selected to satisfy the free-space target",
                    ),
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
    }
}

#[must_use]
pub fn apply_dreaming_plan(store: &mut MemoryStore, plan: &DreamingPlan) -> DreamingOutcome {
    let selected_ids = plan
        .actions
        .iter()
        .map(|action| action.event_id.as_str())
        .collect::<BTreeSet<_>>();
    if selected_ids.is_empty() {
        return DreamingOutcome {
            removed_events: 0,
            estimated_reclaimed_bytes: 0,
        };
    }

    let initial_len = store.len();
    let retained = store
        .events()
        .iter()
        .filter(|event| !selected_ids.contains(event.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let removed_events = initial_len - retained.len();
    *store = MemoryStore::from_events(retained);
    DreamingOutcome {
        removed_events,
        estimated_reclaimed_bytes: selected_bytes(&plan.actions),
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
    if contains_any(&kind, &["learning", "ledger", "skill"])
        || contains_any(&content, &["learned", "learning ledger", "promoted lesson"])
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
        ],
    ) {
        return DreamingDurability::RecomputableIntermediate;
    }

    DreamingDurability::IrreplaceableRaw
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
        + option_bytes(&event.kind)
        + option_bytes(&event.role)
        + option_bytes(&event.intent)
        + option_bytes(&event.tool)
        + option_bytes(&event.inputs)
        + option_bytes(&event.outputs)
        + option_bytes(&event.content)
        + option_bytes(&event.sent_at)
        + option_bytes(&event.demo_label)
        + option_bytes(&event.conversation_id)
        + option_bytes(&event.conversation_title)
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

fn option_bytes(value: &Option<String>) -> u64 {
    value.as_deref().map_or(0, string_bytes)
}

fn string_bytes(value: &str) -> u64 {
    value.len() as u64
}
