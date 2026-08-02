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
    let payload_changed = [
        (&base.kind, &incoming.kind),
        (&base.role, &incoming.role),
        (&base.intent, &incoming.intent),
        (&base.tool, &incoming.tool),
        (&base.inputs, &incoming.inputs),
        (&base.outputs, &incoming.outputs),
        (&base.content, &incoming.content),
        (&base.sent_at, &incoming.sent_at),
        (&base.demo_label, &incoming.demo_label),
        (&base.conversation_id, &incoming.conversation_id),
        (&base.conversation_title, &incoming.conversation_title),
    ]
    .iter()
    .any(|(left, right)| right.as_ref().is_some_and(|value| !value.is_empty()) && left != right)
        || (!incoming.evidence.is_empty() && base.evidence != incoming.evidence);
    let observed_writes = base.write_count.max(1).max(incoming.write_count.max(1));
    let write_count = if payload_changed && incoming.write_count <= base.write_count {
        observed_writes.saturating_add(1)
    } else {
        observed_writes
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
        // Access counts are monotone per event; the larger side has seen more
        // reads, so max is the lossless merge.
        access_count: base.access_count.max(incoming.access_count),
        // A peer can bring a newer monotone count. Legacy/uncounted edits are
        // recognized from their changed payload and become one durable write.
        write_count,
    }
}

/// Resolve the shared memory log path the server reads/writes for sync.
///
/// Honours `FORMAL_AI_MEMORY_PATH` and otherwise returns the platform's shared
/// per-user memory file.
#[must_use]
pub fn configured_memory_path() -> Option<PathBuf> {
    Some(crate::shared_memory::shared_memory_path())
}

/// Live chat exchanges are recorded into memory unless explicitly disabled.
#[must_use]
pub fn chat_recording_enabled() -> bool {
    !matches!(
        std::env::var("FORMAL_AI_RECORD_CHAT").as_deref(),
        Ok("0" | "false" | "off")
    )
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

/// One client-executed tool step recovered from an agentic API transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedToolExecution {
    pub tool: String,
    pub inputs: String,
    pub outputs: String,
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
        if let Err(error) = crate::shared_memory::ensure_shared_memory_file(path) {
            if std::env::var("FORMAL_AI_MEMORY_DEBUG").as_deref() == Ok("1") {
                eprintln!("[memory] could not initialize {}: {error}", path.display());
            }
        }
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
        if let Some(path) = self.path.as_deref() {
            let mut memory =
                crate::memory::MemoryStore::from_events(std::mem::take(&mut self.events));
            let _ = crate::storage_policy::apply_auto_free_space_for_write(
                &mut memory,
                path,
                u64::try_from(text.len()).unwrap_or(u64::MAX),
            )?;
            self.events = memory.events().to_vec();
        }
        self.persist()?;
        Ok(added)
    }

    /// Record one live chat exchange into the shared memory log (issue #540's
    /// live-usage loop): the user turn becomes a `message` event that
    /// requirement learning can lift, and the assistant turn becomes a `task`
    /// event with the exact input/output pair dreaming can replay and
    /// generalize. Ids are stable over (prompt, answer), so retries do not
    /// duplicate. Set `FORMAL_AI_RECORD_CHAT=0` to opt out.
    ///
    /// # Errors
    /// Returns an [`std::io::Error`] when the backing file cannot be written.
    pub fn record_chat_exchange(&mut self, prompt: &str, answer: &str) -> std::io::Result<usize> {
        self.record_chat_exchange_with_tools(prompt, answer, &[])
    }

    /// Record a completed exchange plus the actual tool work delegated to and
    /// returned by an agentic API client. The tool events use the same durable
    /// schema as browser-side tool traces, and the final task cites them as
    /// evidence. Stable ids omit transient protocol call ids, so a retried
    /// exchange merges instead of duplicating learned evidence.
    ///
    /// # Errors
    /// Returns an [`std::io::Error`] when the backing file cannot be written.
    pub fn record_chat_exchange_with_tools(
        &mut self,
        prompt: &str,
        answer: &str,
        tools: &[RecordedToolExecution],
    ) -> std::io::Result<usize> {
        if self.path.is_none() || !chat_recording_enabled() {
            return Ok(0);
        }
        let seed = format!("{prompt}\0{answer}");
        let user_id = crate::engine::stable_id("chat_user", &seed);
        let mut recorded = vec![MemoryEvent {
            id: user_id.clone(),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some(prompt.to_owned()),
            write_count: 1,
            ..MemoryEvent::default()
        }];
        let mut evidence = vec![user_id.clone()];
        for execution in tools {
            let tool_seed = format!(
                "{prompt}\0{}\0{}\0{}",
                execution.tool, execution.inputs, execution.outputs
            );
            let id = crate::engine::stable_id("chat_tool", &tool_seed);
            evidence.push(id.clone());
            recorded.push(MemoryEvent {
                id,
                kind: Some(String::from("tool_call")),
                role: Some(String::from("assistant")),
                intent: Some(String::from("execute_tool")),
                tool: Some(execution.tool.clone()),
                inputs: Some(execution.inputs.clone()),
                outputs: Some(execution.outputs.clone()),
                content: Some(format!("tool:{}", execution.tool)),
                evidence: vec![user_id.clone()],
                write_count: 1,
                ..MemoryEvent::default()
            });
        }
        recorded.push(MemoryEvent {
            id: crate::engine::stable_id("chat_task", &seed),
            kind: Some(String::from("task")),
            role: Some(String::from("assistant")),
            intent: Some(String::from("solve")),
            inputs: Some(prompt.to_owned()),
            outputs: Some(answer.to_owned()),
            evidence,
            write_count: 1,
            ..MemoryEvent::default()
        });
        let before = self.events.len();
        self.events = merge_union_by_id(&self.events, &recorded);
        let added = self.events.len() - before;
        if added > 0 {
            self.persist()?;
        }
        Ok(added)
    }

    fn persist(&self) -> std::io::Result<()> {
        let Some(path) = self.path.as_ref() else {
            return Ok(());
        };
        // Locked atomic write (issue #540 §6): the HTTP handlers and the
        // background dreaming thread share this log.
        crate::memory::write_locked_atomic(path, &self.to_links_notation())
    }
}
