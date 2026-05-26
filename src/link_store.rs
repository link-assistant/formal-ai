//! Swappable Links Notation and doublet-links storage boundary.
//!
//! The current durable store is still the human-reviewable `.lino`
//! projection in [`crate::memory::MemoryStore`]. This module defines the
//! backend trait that lets solver traces and memory events be reduced to
//! doublets without forcing every surface to share the same physical store
//! yet. Native builds can opt into the `doublets-native` feature to mirror
//! writes into the `doublets` crate; browser builds expose the same shape via
//! the `IndexedDB` mirror in `src/web/memory.js`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;

use lino_objects_codec::format::parse_indented;

use crate::engine::{stable_id, KNOWLEDGE_SCHEMA_VERSION};
use crate::memory::{import_full_memory, MemoryEvent, MemoryStore, BUNDLE_HEADER, ROOT_HEADER};

/// A single doublet edge in the canonical `from -> to` projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubletLink {
    pub index: String,
    pub from: String,
    pub to: String,
}

/// One content-addressed record and its reducible doublet projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRecord {
    pub stable_id: String,
    pub schema_version: String,
    pub record_type: String,
    pub source_id: String,
    pub links: Vec<DoubletLink>,
}

/// Physical backend selected for a build or surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStoreBackend {
    LinoProjection,
    DoubletsRs,
    DoubletsWeb,
}

/// Import or backend failure for a link store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkStoreError {
    IllFormedLinksNotation(String),
    Backend(String),
}

impl fmt::Display for LinkStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllFormedLinksNotation(message) => {
                write!(formatter, "ill-formed Links Notation: {message}")
            }
            Self::Backend(message) => write!(formatter, "link-store backend error: {message}"),
        }
    }
}

impl Error for LinkStoreError {}

/// Store abstraction used by memory and event-log projections.
pub trait LinkStore {
    /// Returns the active physical backend.
    fn backend(&self) -> LinkStoreBackend;

    /// Append a memory event and return the stable record id assigned to it.
    fn append_memory_event(&mut self, event: MemoryEvent) -> Result<String, LinkStoreError>;

    /// Strictly import a `.lino` memory or bundle document.
    fn import_memory_links_notation(&mut self, text: &str) -> Result<usize, LinkStoreError>;

    /// Export the current memory projection as Links Notation.
    fn export_memory_links_notation(&self) -> String;

    /// Return every stored record as doublet-reducible metadata.
    fn records(&self) -> Vec<LinkRecord>;
}

/// Select the backend implied by this build.
#[must_use]
pub const fn selected_link_store_backend() -> LinkStoreBackend {
    if cfg!(target_arch = "wasm32") {
        LinkStoreBackend::DoubletsWeb
    } else if cfg!(feature = "doublets-native") {
        LinkStoreBackend::DoubletsRs
    } else {
        LinkStoreBackend::LinoProjection
    }
}

/// Validate that a memory import is a syntactically valid supported `.lino`
/// document before mutating the store.
pub fn validate_memory_links_notation(text: &str) -> Result<(), LinkStoreError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(LinkStoreError::IllFormedLinksNotation(String::from(
            "document is empty",
        )));
    }
    parse_indented(trimmed)
        .map_err(|error| LinkStoreError::IllFormedLinksNotation(format!("{error:?}")))?;
    let header = trimmed.lines().find(|line| !line.trim().is_empty());
    match header.map(str::trim) {
        Some(ROOT_HEADER) => validate_demo_memory_document(trimmed),
        Some(BUNDLE_HEADER) => Ok(()),
        Some(other) => Err(LinkStoreError::IllFormedLinksNotation(format!(
            "expected {ROOT_HEADER} or {BUNDLE_HEADER}, got {other}"
        ))),
        None => Err(LinkStoreError::IllFormedLinksNotation(String::from(
            "document is empty",
        ))),
    }
}

/// Project memory events into content-addressed records.
#[must_use]
pub fn memory_events_to_link_records(events: &[MemoryEvent]) -> Vec<LinkRecord> {
    events
        .iter()
        .enumerate()
        .map(|(index, event)| memory_event_to_link_record(event, index))
        .collect()
}

/// Project one memory event into a `Type -> SubType -> Value` doublet graph.
#[must_use]
pub fn memory_event_to_link_record(event: &MemoryEvent, sequence: usize) -> LinkRecord {
    let canonical = canonical_memory_event(event);
    let source_id = event_source_id(event, sequence, &canonical);
    let record_id = stable_id(
        "memory_event",
        &format!("{sequence}:{}:{canonical}", source_id.as_str()),
    );
    let subtype = event
        .kind
        .as_deref()
        .or(event.role.as_deref())
        .or(event.intent.as_deref())
        .unwrap_or("memory_event");

    let mut links = Vec::new();
    push_doublet(&mut links, &record_id, "Type");
    push_doublet(&mut links, "Type", "MemoryEvent");
    push_doublet(&mut links, "MemoryEvent", "SubType");
    push_doublet(&mut links, "SubType", subtype);
    push_doublet(&mut links, subtype, "Value");
    push_doublet(&mut links, &record_id, &source_id);
    push_doublet(
        &mut links,
        &record_id,
        &format!("schema_version:{KNOWLEDGE_SCHEMA_VERSION}"),
    );
    push_optional_field(&mut links, &record_id, "id", Some(source_id.as_str()));
    push_optional_field(&mut links, &record_id, "kind", event.kind.as_deref());
    push_optional_field(&mut links, &record_id, "role", event.role.as_deref());
    push_optional_field(&mut links, &record_id, "intent", event.intent.as_deref());
    push_optional_field(&mut links, &record_id, "tool", event.tool.as_deref());
    push_optional_field(&mut links, &record_id, "inputs", event.inputs.as_deref());
    push_optional_field(&mut links, &record_id, "outputs", event.outputs.as_deref());
    push_optional_field(&mut links, &record_id, "content", event.content.as_deref());
    push_optional_field(&mut links, &record_id, "sentAt", event.sent_at.as_deref());
    push_optional_field(
        &mut links,
        &record_id,
        "demoLabel",
        event.demo_label.as_deref(),
    );
    push_optional_field(
        &mut links,
        &record_id,
        "conversationId",
        event.conversation_id.as_deref(),
    );
    push_optional_field(
        &mut links,
        &record_id,
        "conversationTitle",
        event.conversation_title.as_deref(),
    );
    for evidence in &event.evidence {
        push_optional_field(&mut links, &record_id, "evidence", Some(evidence));
    }

    LinkRecord {
        stable_id: record_id,
        schema_version: String::from(KNOWLEDGE_SCHEMA_VERSION),
        record_type: String::from("MemoryEvent"),
        source_id,
        links,
    }
}

impl LinkStore for MemoryStore {
    fn backend(&self) -> LinkStoreBackend {
        LinkStoreBackend::LinoProjection
    }

    fn append_memory_event(&mut self, mut event: MemoryEvent) -> Result<String, LinkStoreError> {
        ensure_event_id(&mut event, self.len());
        let id = event.id.clone();
        self.append(event);
        Ok(id)
    }

    fn import_memory_links_notation(&mut self, text: &str) -> Result<usize, LinkStoreError> {
        validate_memory_links_notation(text)?;
        let parsed = import_full_memory(text);
        let count = parsed.events.len();
        for event in parsed.events {
            self.append_memory_event(event)?;
        }
        Ok(count)
    }

    fn export_memory_links_notation(&self) -> String {
        Self::export_links_notation(self)
    }

    fn records(&self) -> Vec<LinkRecord> {
        memory_events_to_link_records(self.events())
    }
}

impl MemoryStore {
    /// Strictly import a `.lino` memory document, rejecting malformed input.
    pub fn try_import_links_notation(&mut self, text: &str) -> Result<usize, LinkStoreError> {
        <Self as LinkStore>::import_memory_links_notation(self, text)
    }

    /// Strictly replace current memory from a `.lino` document.
    pub fn try_replace_from_links_notation(&mut self, text: &str) -> Result<(), LinkStoreError> {
        validate_memory_links_notation(text)?;
        let parsed = import_full_memory(text);
        let mut replacement = Self::new();
        for event in parsed.events {
            replacement.append_memory_event(event)?;
        }
        *self = replacement;
        Ok(())
    }

    /// Return the doublet-reducible projection of every memory event.
    #[must_use]
    pub fn link_records(&self) -> Vec<LinkRecord> {
        memory_events_to_link_records(self.events())
    }
}

/// Native `doublets`-backed mirror for Rust builds.
#[cfg(feature = "doublets-native")]
type NativeDoubletsStore =
    doublets::unit::Store<usize, mem::Global<doublets::parts::LinkPart<usize>>>;

/// Native `doublets`-backed mirror for Rust builds.
#[cfg(feature = "doublets-native")]
pub struct DoubletsLinkStore {
    events: Vec<MemoryEvent>,
    records: Vec<LinkRecord>,
    nodes: BTreeMap<String, usize>,
    native: NativeDoubletsStore,
}

#[cfg(feature = "doublets-native")]
impl DoubletsLinkStore {
    /// Create an empty in-memory native doublets store.
    pub fn new() -> Result<Self, LinkStoreError> {
        let native = doublets::unit::Store::<usize, _>::new(mem::Global::new())
            .map_err(|error| format_backend_error(&error))?;
        Ok(Self {
            events: Vec::new(),
            records: Vec::new(),
            nodes: BTreeMap::new(),
            native,
        })
    }

    /// Number of raw native doublets links, including point nodes.
    #[must_use]
    pub fn native_link_count(&self) -> usize {
        use doublets::Doublets as _;
        self.native.count()
    }

    fn insert_record(&mut self, record: LinkRecord) -> Result<(), LinkStoreError> {
        for link in &record.links {
            self.append_native_doublet(&link.from, &link.to)?;
        }
        self.records.push(record);
        Ok(())
    }

    fn append_native_doublet(&mut self, from: &str, to: &str) -> Result<(), LinkStoreError> {
        use doublets::Doublets as _;
        let source = self.node_id(from)?;
        let target = self.node_id(to)?;
        self.native
            .create_link(source, target)
            .map_err(|error| format_backend_error(&error))?;
        Ok(())
    }

    fn node_id(&mut self, node: &str) -> Result<usize, LinkStoreError> {
        use doublets::Doublets as _;
        if let Some(id) = self.nodes.get(node) {
            return Ok(*id);
        }
        let id = self
            .native
            .create_point()
            .map_err(|error| format_backend_error(&error))?;
        self.nodes.insert(node.to_owned(), id);
        Ok(id)
    }
}

#[cfg(feature = "doublets-native")]
impl LinkStore for DoubletsLinkStore {
    fn backend(&self) -> LinkStoreBackend {
        LinkStoreBackend::DoubletsRs
    }

    fn append_memory_event(&mut self, mut event: MemoryEvent) -> Result<String, LinkStoreError> {
        ensure_event_id(&mut event, self.events.len());
        let id = event.id.clone();
        let record = memory_event_to_link_record(&event, self.events.len());
        self.insert_record(record)?;
        self.events.push(event);
        Ok(id)
    }

    fn import_memory_links_notation(&mut self, text: &str) -> Result<usize, LinkStoreError> {
        validate_memory_links_notation(text)?;
        let parsed = import_full_memory(text);
        let count = parsed.events.len();
        for event in parsed.events {
            self.append_memory_event(event)?;
        }
        Ok(count)
    }

    fn export_memory_links_notation(&self) -> String {
        crate::memory::export_links_notation(&self.events)
    }

    fn records(&self) -> Vec<LinkRecord> {
        self.records.clone()
    }
}

#[cfg(feature = "doublets-native")]
fn format_backend_error(error: &doublets::Error<usize>) -> LinkStoreError {
    LinkStoreError::Backend(format!("{error:?}"))
}

fn ensure_event_id(event: &mut MemoryEvent, sequence: usize) {
    if !event.id.is_empty() {
        return;
    }
    let canonical = canonical_memory_event(event);
    event.id = stable_id("memory_event", &format!("{sequence}:{canonical}"));
}

fn validate_demo_memory_document(text: &str) -> Result<(), LinkStoreError> {
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let indent = line.chars().take_while(|ch| *ch == ' ').count();
        let content = &line[indent..];
        match indent {
            0 if content == ROOT_HEADER => {}
            2 => validate_event_line(content)?,
            4 => validate_field_line(content)?,
            _ => {
                return Err(LinkStoreError::IllFormedLinksNotation(format!(
                    "unexpected indentation or record line: {content}"
                )));
            }
        }
    }
    Ok(())
}

fn validate_event_line(content: &str) -> Result<(), LinkStoreError> {
    let Some(rest) = content.strip_prefix("event ") else {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "expected event record, got {content}"
        )));
    };
    validate_strict_quoted(rest)
}

fn validate_field_line(content: &str) -> Result<(), LinkStoreError> {
    let Some((key, rest)) = content.split_once(' ') else {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "expected field value, got {content}"
        )));
    };
    if !key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "invalid field name {key}"
        )));
    }
    validate_strict_quoted(rest)
}

fn validate_strict_quoted(rest: &str) -> Result<(), LinkStoreError> {
    let trimmed = rest.trim_start();
    let bytes = trimmed.as_bytes();
    if bytes.first() != Some(&b'"') {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "expected quoted value, got {rest}"
        )));
    }
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => {
                if trimmed[index + 1..].trim().is_empty() {
                    return Ok(());
                }
                return Err(LinkStoreError::IllFormedLinksNotation(format!(
                    "unexpected trailing content after quoted value: {}",
                    &trimmed[index + 1..]
                )));
            }
            _ => index += 1,
        }
    }
    Err(LinkStoreError::IllFormedLinksNotation(String::from(
        "unterminated quoted value",
    )))
}

fn event_source_id(event: &MemoryEvent, sequence: usize, canonical: &str) -> String {
    if event.id.is_empty() {
        stable_id("memory_event", &format!("{sequence}:{canonical}"))
    } else {
        event.id.clone()
    }
}

fn canonical_memory_event(event: &MemoryEvent) -> String {
    let mut fields = BTreeMap::new();
    push_canonical(&mut fields, "id", Some(event.id.as_str()));
    push_canonical(&mut fields, "kind", event.kind.as_deref());
    push_canonical(&mut fields, "role", event.role.as_deref());
    push_canonical(&mut fields, "intent", event.intent.as_deref());
    push_canonical(&mut fields, "tool", event.tool.as_deref());
    push_canonical(&mut fields, "inputs", event.inputs.as_deref());
    push_canonical(&mut fields, "outputs", event.outputs.as_deref());
    push_canonical(&mut fields, "content", event.content.as_deref());
    push_canonical(&mut fields, "sentAt", event.sent_at.as_deref());
    push_canonical(&mut fields, "demoLabel", event.demo_label.as_deref());
    push_canonical(
        &mut fields,
        "conversationId",
        event.conversation_id.as_deref(),
    );
    push_canonical(
        &mut fields,
        "conversationTitle",
        event.conversation_title.as_deref(),
    );
    for (index, evidence) in event.evidence.iter().enumerate() {
        let key = format!("evidence_{index:04}");
        fields.insert(key, evidence.clone());
    }
    let mut out = String::new();
    for (key, value) in fields {
        let _ = write!(out, "{key}={}:{};", value.len(), value);
    }
    out
}

fn push_canonical(fields: &mut BTreeMap<String, String>, key: &str, value: Option<&str>) {
    let Some(value) = value else { return };
    if value.is_empty() {
        return;
    }
    fields.insert(key.to_owned(), value.to_owned());
}

fn push_optional_field(
    links: &mut Vec<DoubletLink>,
    record_id: &str,
    key: &str,
    value: Option<&str>,
) {
    let Some(value) = value else { return };
    if value.is_empty() {
        return;
    }
    let field = format!("field:{key}");
    let field_value = format!("value:{value}");
    push_doublet(links, record_id, &field);
    push_doublet(links, &field, &field_value);
}

fn push_doublet(links: &mut Vec<DoubletLink>, from: &str, to: &str) {
    links.push(DoubletLink {
        index: stable_id("doublet", &format!("{from}->{to}")),
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

#[cfg(test)]
mod tests {
    use super::{
        memory_event_to_link_record, validate_memory_links_notation, LinkStore, LinkStoreBackend,
        LinkStoreError,
    };
    use crate::memory::{export_links_notation, MemoryEvent, MemoryStore};

    #[test]
    fn memory_events_reduce_to_type_subtype_value_doublets() {
        let record = memory_event_to_link_record(&MemoryEvent::user("hello"), 0);
        assert_eq!(record.record_type, "MemoryEvent");
        assert!(record
            .links
            .iter()
            .any(|link| link.from == "Type" && link.to == "MemoryEvent"));
        assert!(record
            .links
            .iter()
            .any(|link| link.from == "SubType" && link.to == "user"));
        assert!(record
            .links
            .iter()
            .any(|link| link.from == "field:content" && link.to == "value:hello"));
    }

    #[test]
    fn memory_store_trait_assigns_stable_ids_and_exports_lino() {
        let mut store = MemoryStore::new();
        let id =
            LinkStore::append_memory_event(&mut store, MemoryEvent::user("hello")).expect("append");
        assert!(id.starts_with("memory_event_"));
        assert_eq!(store.backend(), LinkStoreBackend::LinoProjection);
        assert!(store.export_memory_links_notation().contains("demo_memory"));
        assert_eq!(store.records().len(), 1);
    }

    #[test]
    fn strict_import_rejects_ill_formed_links_notation_without_mutation() {
        let mut store = MemoryStore::new();
        let err = store
            .try_import_links_notation("demo_memory\n  event \"unterminated\n")
            .expect_err("malformed import must fail");
        assert!(matches!(err, LinkStoreError::IllFormedLinksNotation(_)));
        assert!(store.is_empty());
    }

    #[test]
    fn strict_import_accepts_legacy_memory_documents() {
        let text = export_links_notation(&[MemoryEvent {
            id: String::from("event_1"),
            role: Some(String::from("user")),
            content: Some(String::from("Hi")),
            ..MemoryEvent::default()
        }]);
        validate_memory_links_notation(&text).expect("valid memory document");
        let mut store = MemoryStore::new();
        let inserted = store.try_import_links_notation(&text).expect("import");
        assert_eq!(inserted, 1);
        assert_eq!(store.events()[0].id, "event_1");
    }

    #[cfg(feature = "doublets-native")]
    #[test]
    fn doublets_native_backend_mirrors_memory_events() {
        use super::DoubletsLinkStore;

        let mut store = DoubletsLinkStore::new().expect("native doublets store");
        let id = store
            .append_memory_event(MemoryEvent::assistant("hi back"))
            .expect("append");
        assert!(id.starts_with("memory_event_"));
        assert_eq!(store.backend(), LinkStoreBackend::DoubletsRs);
        assert_eq!(store.records().len(), 1);
        assert!(
            store.native_link_count() > store.records()[0].links.len(),
            "native store should contain point nodes plus projected doublets"
        );
    }
}
