//! Eviction eligibility is based on origin and verified reconstruction.
use super::support::{contains_any, lower_opt};
use super::{DreamingDurability, lexicon};
use crate::memory::MemoryEvent;
use std::collections::BTreeSet;

/// Keep the procedure and provenance after discarding an explicitly disposable
/// payload. This is not a copy of the observation: the original remains under
/// its own identity, and reacquisition may yield different bytes.
pub(super) fn reconstruction_record(event: &MemoryEvent) -> Option<MemoryEvent> {
    if event.role.as_deref() != Some("cache") {
        return None;
    }
    let mut record = MemoryEvent {
        kind: Some("cache_reconstruction".to_owned()),
        role: Some("system".to_owned()),
        inputs: Some(event.id.clone()),
        tool: event.tool.clone(),
        conversation_id: event.conversation_id.clone(),
        evidence: event.evidence.clone(),
        unknown_fields: event.unknown_fields.clone(),
        ..MemoryEvent::default()
    };
    record.id = crate::engine::stable_id("cache_reconstruction", &format!("{record:?}"));
    Some(record)
}

pub(super) fn reclaimable_bytes(event: &MemoryEvent, durability: DreamingDurability) -> u64 {
    let retained_bytes = if durability == DreamingDurability::RecomputableCache {
        reconstruction_record(event)
            .as_ref()
            .map_or(0, super::support::estimate_event_bytes)
    } else {
        0
    };
    super::support::estimate_event_bytes(event).saturating_sub(retained_bytes)
}

pub(super) fn classify_event(
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

    // Durability cues are grounded as data (multilingual) in
    // data/meta/dreaming-lexicon.lino instead of hardcoded English keywords.
    let cues = lexicon::lexicon();
    // Original dialogue and observed tool actions are historical evidence.
    // Re-fetching a page or regenerating a summary cannot recreate that
    // experience. A cache-shaped kind/tool label does not override its origin.
    if event.role.as_deref().is_some_and(|role| {
        ["user", "assistant", "tool"]
            .iter()
            .any(|original| role.eq_ignore_ascii_case(original))
    }) {
        return DreamingDurability::IrreplaceableRaw;
    }
    let kind = lower_opt(event.kind.as_deref());
    let tool = lower_opt(event.tool.as_deref());
    let content = lower_opt(event.content.as_deref());
    let evidence = event.evidence.join("\n").to_lowercase();
    if contains_any(&kind, &cues.learning_kind_cues)
        || contains_any(&content, &cues.learning_content_cues)
        || evidence.contains("learning")
    {
        return DreamingDurability::RetainedLearning;
    }

    // Unknown legacy records are retained. The producer must explicitly mark
    // a disposable copy and supply a reconstruction edge; names alone are
    // insufficient. An embedded-seed edge is revalidated byte-for-byte.
    let reconstructable = event.role.as_deref() == Some("cache")
        && (event.evidence.iter().any(|evidence| {
            evidence
                .strip_prefix("rediscover:https://")
                .is_some_and(|location| {
                    !location.is_empty()
                        && !location.chars().any(char::is_whitespace)
                        && !location.starts_with('/')
                        && !location.contains('@')
                })
        }) || (event
            .evidence
            .iter()
            .any(|evidence| evidence == "reconstruct:embedded-seed")
            && crate::seed::seed_files().iter().any(|(path, contents)| {
                event.tool.as_deref() == Some(*path) && event.content.as_deref() == Some(*contents)
            })));
    if reconstructable
        && (contains_any(&kind, &cues.cache_kind_cues)
            || contains_any(&tool, &cues.cache_tool_cues))
    {
        return DreamingDurability::RecomputableCache;
    }

    DreamingDurability::IrreplaceableRaw
}
