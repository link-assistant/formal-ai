//! Local database sync — keep the desktop (browser IndexedDB) memory store and
//! the CLI/native store in step without a manual export/import.
//!
//! Issue #347 / R5c asks that a conversation started in one surface continue in
//! another. The surfaces already *interoperate* through the portable
//! `formal_ai_bundle` / `demo_memory` Links-Notation files; this module adds the
//! conflict-aware **sync** layer on top:
//!
//! * [`events_since`] computes the delta a puller is missing (the change-feed).
//! * [`merge_union_by_id`] is the merge policy: events are append-only and
//!   content-addressed, so union-by-id is conflict-free; later writes for an
//!   existing id win the tie-break (documented, deterministic).
//! * [`SyncStore`] is a thin file-backed store the HTTP server uses so the sync
//!   endpoints are stateless across requests yet share one log on disk
//!   (`FORMAL_AI_MEMORY_PATH`).
//!
//! Per R7 the payloads on the wire stay **Links Notation** (`demo_memory`); only
//! the transport is REST. Nothing here introduces a non-OpenAI *external* REST
//! surface — these are internal `/v1/memory*` sync routes.

use std::path::{Path, PathBuf};

use crate::memory::{export_links_notation, parse_links_notation, MemoryEvent};

/// Return every event that appears strictly **after** the event `last_seen`.
///
/// Order is preserved. When `last_seen` is `None` (or empty), the full log is
/// returned — a first-time puller wants everything.
///
/// If `last_seen` is not found in `events` (the puller saw an event this log
/// never had — e.g. it synced from a different branch), the full log is
/// returned so no event is silently skipped.
#[must_use]
pub fn events_since(events: &[MemoryEvent], last_seen: Option<&str>) -> Vec<MemoryEvent> {
    let Some(last_seen) = last_seen.filter(|id| !id.is_empty()) else {
        return events.to_vec();
    };
    events
        .iter()
        .position(|event| event.id == last_seen)
        .map_or_else(|| events.to_vec(), |index| events[index + 1..].to_vec())
}

/// Merge two append-only logs by id.
///
/// `base` is kept in order; every event from `incoming` whose id is not already
/// present is appended in order. Events that share an id are reconciled by
/// [`merge_event`] (incoming non-empty fields win), so an edited event
/// propagates without duplicating the record.
#[must_use]
pub fn merge_union_by_id(base: &[MemoryEvent], incoming: &[MemoryEvent]) -> Vec<MemoryEvent> {
    let mut merged: Vec<MemoryEvent> = base.to_vec();
    for event in incoming {
        match merged.iter_mut().find(|existing| existing.id == event.id) {
            Some(existing) => *existing = merge_event(existing, event),
            None => merged.push(event.clone()),
        }
    }
    merged
}

/// Tie-break for two events that share an id.
///
/// Keep `base` but let any non-empty field from `incoming` overwrite it. This
/// makes "edited event" sync last-writer-wins per field while never dropping
/// data that only one side has.
#[must_use]
pub fn merge_event(base: &MemoryEvent, incoming: &MemoryEvent) -> MemoryEvent {
    fn pick(base: Option<&String>, incoming: Option<&String>) -> Option<String> {
        match incoming {
            Some(value) if !value.is_empty() => Some(value.clone()),
            _ => base.cloned(),
        }
    }
    let evidence = if incoming.evidence.is_empty() {
        base.evidence.clone()
    } else {
        incoming.evidence.clone()
    };
    MemoryEvent {
        id: base.id.clone(),
        kind: pick(base.kind.as_ref(), incoming.kind.as_ref()),
        role: pick(base.role.as_ref(), incoming.role.as_ref()),
        intent: pick(base.intent.as_ref(), incoming.intent.as_ref()),
        tool: pick(base.tool.as_ref(), incoming.tool.as_ref()),
        inputs: pick(base.inputs.as_ref(), incoming.inputs.as_ref()),
        outputs: pick(base.outputs.as_ref(), incoming.outputs.as_ref()),
        content: pick(base.content.as_ref(), incoming.content.as_ref()),
        sent_at: pick(base.sent_at.as_ref(), incoming.sent_at.as_ref()),
        demo_label: pick(base.demo_label.as_ref(), incoming.demo_label.as_ref()),
        conversation_id: pick(
            base.conversation_id.as_ref(),
            incoming.conversation_id.as_ref(),
        ),
        conversation_title: pick(
            base.conversation_title.as_ref(),
            incoming.conversation_title.as_ref(),
        ),
        evidence,
    }
}

/// Resolve the shared memory log path the server reads/writes for sync.
///
/// Honours `FORMAL_AI_MEMORY_PATH`; returns `None` when sync is not configured
/// (the endpoints then operate on an empty, in-memory log so they never panic).
#[must_use]
pub fn configured_memory_path() -> Option<PathBuf> {
    std::env::var("FORMAL_AI_MEMORY_PATH")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// A small file-backed event log used by the HTTP sync endpoints.
///
/// Each request loads the current log, applies its operation, and (for writes)
/// saves it back, so the stateless server still shares one log across requests.
#[derive(Debug, Clone, Default)]
pub struct SyncStore {
    path: Option<PathBuf>,
    events: Vec<MemoryEvent>,
}

impl SyncStore {
    /// Open the configured store, loading any existing events from disk.
    #[must_use]
    pub fn open() -> Self {
        configured_memory_path().map_or_else(Self::default, |path| Self::open_at(&path))
    }

    /// Open a store at an explicit path (used by tests).
    #[must_use]
    pub fn open_at(path: &Path) -> Self {
        let events = std::fs::read_to_string(path)
            .map(|text| parse_links_notation(&text))
            .unwrap_or_default();
        Self {
            path: Some(path.to_path_buf()),
            events,
        }
    }

    /// The events currently held.
    #[must_use]
    pub fn events(&self) -> &[MemoryEvent] {
        &self.events
    }

    /// Render the log as a `demo_memory` Links-Notation document.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        export_links_notation(&self.events)
    }

    /// Render only the events after `last_seen` as Links Notation (the delta a
    /// puller applies).
    #[must_use]
    pub fn delta_links_notation(&self, last_seen: Option<&str>) -> String {
        export_links_notation(&events_since(&self.events, last_seen))
    }

    /// Import a `demo_memory` document, merging by id, and persist the result.
    /// Returns the number of events added.
    ///
    /// # Errors
    /// Returns an [`std::io::Error`] when the backing file cannot be written.
    pub fn import_links_notation(&mut self, text: &str) -> std::io::Result<usize> {
        let incoming = parse_links_notation(text);
        let before = self.events.len();
        self.events = merge_union_by_id(&self.events, &incoming);
        let added = self.events.len() - before;
        self.persist()?;
        Ok(added)
    }

    fn persist(&self) -> std::io::Result<()> {
        let Some(path) = self.path.as_ref() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, self.to_links_notation())
    }
}

#[path = "source_tests/memory_sync/tests.rs"]
mod tests;
