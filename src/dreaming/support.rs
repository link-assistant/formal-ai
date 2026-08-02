//! Small text/byte utilities shared by the dreaming planner.

use super::DreamingAction;
use crate::memory::MemoryEvent;

pub(super) fn estimate_event_bytes(event: &MemoryEvent) -> u64 {
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

pub(super) fn selected_bytes(actions: &[DreamingAction]) -> u64 {
    actions.iter().map(|action| action.estimated_bytes).sum()
}

pub(super) fn required_reclaim_bytes(
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

pub(super) fn percent_ceil(total: u64, percent: u8) -> u64 {
    if percent == 0 || total == 0 {
        return 0;
    }
    total.saturating_mul(u64::from(percent)).saturating_add(99) / 100
}

pub(super) fn contains_any(haystack: &str, needles: &[String]) -> bool {
    needles
        .iter()
        .any(|needle| haystack.contains(needle.as_str()))
}

pub(super) fn lower_opt(value: Option<&str>) -> String {
    value.unwrap_or_default().to_ascii_lowercase()
}

pub(super) fn normalized(value: Option<&str>) -> String {
    value.unwrap_or_default().trim().to_ascii_lowercase()
}

pub(super) fn option_bytes(value: Option<&str>) -> u64 {
    value.map_or(0, string_bytes)
}

pub(super) const fn string_bytes(value: &str) -> u64 {
    value.len() as u64
}
