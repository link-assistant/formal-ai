//! Portable Links Notation memory log shared across every interface.
//!
//! The browser demo persists its conversation memory under `IndexedDB` as a
//! `demo_memory` Links Notation document (see `src/web/memory.js`). The CLI
//! and the HTTP server reuse the **exact same wire format** so a user can
//! migrate their agent's memory between surfaces with a single `.lino`
//! file:
//!
//! ```text
//! demo_memory
//!   event "id1"
//!     role "user"
//!     content "Hi"
//!     sentAt "2026-05-15T12:00:00.000Z"
//!   event "id2"
//!     role "assistant"
//!     intent "greeting"
//!     content "Hi, how may I help you?"
//!     sentAt "2026-05-15T12:00:01.000Z"
//! ```
//!
//! The store is append-only for normal writes. Destructive paths are explicit
//! user-initiated maintenance operations: purge already-deleted conversations
//! or reset the dynamic event log after the caller has handled confirmation /
//! backup. Older logs without the optional `kind`/`tool`/`inputs`/`outputs`
//! fields still parse as plain user/assistant turns, so the format is
//! forward-compatible.
//!
//! Full-memory bundles (`formal_ai_bundle`) — seed files + UI preferences +
//! environment metadata + the entire event log in a single document — live in
//! the [`bundle`] submodule. They are the default shape every "export memory"
//! surface now writes (see issue #18 / R109).
//!
//! See [`super::seed`] for the static knowledge surface that pairs with this
//! dynamic memory log, and `VISION.md` (Single-File Reproducibility) for the
//! reasoning behind the unified format.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

pub mod bundle;

pub use bundle::{
    export_bundle, export_full_memory, extract_memory_from_bundle, import_full_memory,
    suggest_migrations, BundleInfo, ParsedBundle,
};

pub(crate) const ROOT_HEADER: &str = "demo_memory";
pub(crate) const BUNDLE_HEADER: &str = "formal_ai_bundle";

/// One recorded turn / step / tool invocation.
///
/// All fields are optional so the same record shape covers user/assistant
/// messages, internal reasoning steps, and tool invocations without
/// branching the schema.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MemoryEvent {
    pub id: String,
    pub kind: Option<String>,
    pub role: Option<String>,
    pub intent: Option<String>,
    pub tool: Option<String>,
    pub inputs: Option<String>,
    pub outputs: Option<String>,
    pub content: Option<String>,
    pub sent_at: Option<String>,
    pub demo_label: Option<String>,
    pub conversation_id: Option<String>,
    pub conversation_title: Option<String>,
    pub evidence: Vec<String>,
}

impl MemoryEvent {
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Some(String::from("user")),
            content: Some(content.into()),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Some(String::from("assistant")),
            content: Some(content.into()),
            ..Self::default()
        }
    }
}

/// Memory log for dynamic events. Normal writes append records; explicit purge
/// and reset methods exist for irreversible user-requested cleanup.
#[derive(Debug, Default, Clone)]
pub struct MemoryStore {
    events: Vec<MemoryEvent>,
}

impl MemoryStore {
    #[must_use]
    pub const fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Build a store from an existing list. Useful for tests and for the
    /// `from_links_notation` factory below.
    #[must_use]
    pub const fn from_events(events: Vec<MemoryEvent>) -> Self {
        Self { events }
    }

    pub fn append(&mut self, event: MemoryEvent) {
        self.events.push(event);
    }

    /// Append every event from `other` to this store. Returns the number of
    /// events appended.
    pub fn import(&mut self, other: &[MemoryEvent]) -> usize {
        let initial = self.events.len();
        self.events.extend_from_slice(other);
        self.events.len() - initial
    }

    /// Permanently remove all events that belong to conversations already
    /// marked with a `conversation_deleted` event.
    pub fn purge_deleted_conversations(&mut self) -> usize {
        let deleted_ids: BTreeSet<String> = self
            .events
            .iter()
            .filter(|event| event.kind.as_deref() == Some("conversation_deleted"))
            .filter_map(|event| event.conversation_id.as_deref())
            .map(ToOwned::to_owned)
            .collect();
        if deleted_ids.is_empty() {
            return 0;
        }
        let initial = self.events.len();
        self.events.retain(|event| {
            event
                .conversation_id
                .as_deref()
                .map_or(true, |id| !deleted_ids.contains(id))
        });
        initial - self.events.len()
    }

    /// Permanently remove all events attributed to a single conversation id.
    pub fn purge_conversation(&mut self, conversation_id: &str) -> usize {
        if conversation_id.is_empty() {
            return 0;
        }
        let initial = self.events.len();
        self.events
            .retain(|event| event.conversation_id.as_deref() != Some(conversation_id));
        initial - self.events.len()
    }

    /// Clear every dynamic memory event while keeping the static seed intact.
    pub fn reset(&mut self) -> usize {
        let initial = self.events.len();
        self.events.clear();
        initial
    }

    #[must_use]
    pub fn events(&self) -> &[MemoryEvent] {
        &self.events
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Render the entire memory log as a portable `demo_memory` Links
    /// Notation document.
    #[must_use]
    pub fn export_links_notation(&self) -> String {
        export_links_notation(&self.events)
    }

    /// Parse a `demo_memory` document and replace the store's contents.
    pub fn replace_from_links_notation(&mut self, text: &str) {
        self.events = parse_links_notation(text);
    }

    /// Append every event parsed from a `demo_memory` document. Returns the
    /// number of events appended.
    pub fn import_links_notation(&mut self, text: &str) -> usize {
        let parsed = parse_links_notation(text);
        self.import(&parsed)
    }

    /// Load events from a file on disk. Missing file yields an empty store.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::new());
        }
        let text = fs::read_to_string(path)?;
        Ok(Self::from_events(import_full_memory(&text).events))
    }

    /// Persist the full store back to a file on disk. Creates parent
    /// directories as needed.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, self.export_links_notation())
    }
}

/// Serialize a slice of events as a `demo_memory` Links Notation document.
#[must_use]
pub fn export_links_notation(events: &[MemoryEvent]) -> String {
    let mut out = String::from(ROOT_HEADER);
    out.push('\n');
    for event in events {
        format_event_into(event, &mut out);
    }
    out
}

pub(crate) fn format_event_into(event: &MemoryEvent, out: &mut String) {
    out.push_str("  event \"");
    out.push_str(&escape_value(&event.id));
    out.push_str("\"\n");
    let pairs: [(&str, Option<&str>); 11] = [
        ("kind", event.kind.as_deref()),
        ("role", event.role.as_deref()),
        ("intent", event.intent.as_deref()),
        ("tool", event.tool.as_deref()),
        ("inputs", event.inputs.as_deref()),
        ("outputs", event.outputs.as_deref()),
        ("content", event.content.as_deref()),
        ("sentAt", event.sent_at.as_deref()),
        ("demoLabel", event.demo_label.as_deref()),
        ("conversationId", event.conversation_id.as_deref()),
        ("conversationTitle", event.conversation_title.as_deref()),
    ];
    for (key, value) in pairs {
        let Some(value) = value else { continue };
        if value.is_empty() {
            continue;
        }
        out.push_str("    ");
        out.push_str(key);
        out.push_str(" \"");
        out.push_str(&escape_value(value));
        out.push_str("\"\n");
    }
    if !event.evidence.is_empty() {
        let joined = event.evidence.join("|");
        out.push_str("    evidence \"");
        out.push_str(&escape_value(&joined));
        out.push_str("\"\n");
    }
}

/// Parse a `demo_memory` Links Notation document into events.
///
/// The parser is lenient: a missing or differently-named header yields an
/// empty list (no panic), and unknown field names are ignored so newer
/// browser logs can be imported into older CLI builds without breaking.
#[must_use]
pub fn parse_links_notation(text: &str) -> Vec<MemoryEvent> {
    let mut events = Vec::new();
    let mut current: Option<MemoryEvent> = None;
    let mut saw_header = false;
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let content = &line[indent..];
        if indent == 0 {
            if content == ROOT_HEADER {
                saw_header = true;
            }
            continue;
        }
        if !saw_header {
            continue;
        }
        if indent == 2 {
            if let Some(name) = content.strip_prefix("event ") {
                if let Some(existing) = current.take() {
                    events.push(existing);
                }
                let id = parse_quoted(name).unwrap_or_default();
                current = Some(MemoryEvent {
                    id,
                    ..MemoryEvent::default()
                });
            }
            continue;
        }
        if indent == 4 {
            let Some(current) = current.as_mut() else {
                continue;
            };
            let Some((key, rest)) = split_first_token(content) else {
                continue;
            };
            let Some(value) = parse_quoted(rest) else {
                continue;
            };
            match key {
                "kind" => current.kind = Some(value),
                "role" => current.role = Some(value),
                "intent" => current.intent = Some(value),
                "tool" => current.tool = Some(value),
                "inputs" => current.inputs = Some(value),
                "outputs" => current.outputs = Some(value),
                "content" => current.content = Some(value),
                "sentAt" => current.sent_at = Some(value),
                "demoLabel" => current.demo_label = Some(value),
                "conversationId" => current.conversation_id = Some(value),
                "conversationTitle" => current.conversation_title = Some(value),
                "evidence" => {
                    current.evidence = value
                        .split('|')
                        .filter(|s| !s.is_empty())
                        .map(ToOwned::to_owned)
                        .collect();
                }
                _ => {}
            }
        }
    }
    if let Some(existing) = current.take() {
        events.push(existing);
    }
    events
}

pub(crate) fn escape_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn unescape_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub(crate) fn parse_quoted(rest: &str) -> Option<String> {
    let trimmed = rest.trim_start();
    let bytes = trimmed.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut i = 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return Some(unescape_value(&trimmed[1..i])),
            _ => i += 1,
        }
    }
    None
}

pub(crate) fn split_first_token(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.trim_start();
    let mut split = trimmed.splitn(2, ' ');
    let head = split.next()?;
    let tail = split.next().unwrap_or("");
    Some((head, tail))
}

// A tiny deterministic ISO-8601 stamp that does not pull in `chrono`. The
// browser side records `new Date().toISOString()`; for the CLI we emit a
// fixed-precision UTC string built from the system clock.
#[allow(clippy::cast_possible_wrap)]
pub(crate) fn isoformat_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let millis = now.subsec_millis();
    format_iso8601(secs, millis)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn format_iso8601(secs_since_epoch: i64, millis: u32) -> String {
    // Convert seconds since epoch to UTC components without external deps.
    let days = secs_since_epoch.div_euclid(86_400);
    let time = secs_since_epoch.rem_euclid(86_400);
    let hours = (time / 3_600) as u32;
    let minutes = ((time % 3_600) / 60) as u32;
    let seconds = (time % 60) as u32;
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}.{millis:03}Z")
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
const fn days_to_date(days: i64) -> (i32, u32, u32) {
    // Algorithm adapted from civil-from-days (Howard Hinnant, public domain).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let mut y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    if m <= 2 {
        y += 1;
    }
    (y as i32, m as u32, d as u32)
}

#[path = "source_tests/memory/tests.rs"]
mod tests;
