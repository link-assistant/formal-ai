//! Swappable Links Notation and link-cli storage boundary.
//!
//! Default native builds embed link-cli's file-mapped `doublets-rs` store and
//! transaction-recovery log through the `doublets-native` feature. The
//! human-reviewable `.lino` memory and bundle formats remain the deterministic
//! export/import projection, and native callers can still compile with
//! `--no-default-features` to use the [`crate::memory::MemoryStore`] Links
//! Notation projection directly. Browser builds expose the same shape via the
//! `IndexedDB` mirror in `src/web/memory.js`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
use std::sync::atomic::{AtomicU64, Ordering};

use lino_objects_codec::format::parse_indented;

use crate::engine::{KNOWLEDGE_SCHEMA_VERSION, stable_id};
use crate::memory::{BUNDLE_HEADER, MemoryEvent, MemoryStore, ROOT_HEADER, import_full_memory};

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
    LinkCli,
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
        LinkStoreBackend::LinkCli
    } else {
        LinkStoreBackend::LinoProjection
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub type DefaultNativeLinkStore = LinkCliLinkStore;

#[cfg(any(target_arch = "wasm32", not(feature = "doublets-native")))]
pub type DefaultNativeLinkStore = MemoryStore;

/// Create the default Rust-side link store for this build.
///
/// Native default builds return [`LinkCliLinkStore`]. Builds compiled with
/// `--no-default-features` keep the explicit `.lino` projection fallback.
pub fn default_native_link_store() -> Result<DefaultNativeLinkStore, LinkStoreError> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
    {
        LinkCliLinkStore::new()
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "doublets-native")))]
    {
        Ok(MemoryStore::new())
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
    let access_count = event.access_count.to_string();
    if event.access_count > 0 {
        push_optional_field(&mut links, &record_id, "accessCount", Some(&access_count));
    }
    let write_count = event.write_count.max(1).to_string();
    push_optional_field(&mut links, &record_id, "writeCount", Some(&write_count));

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

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
type NativeLinkCliStorage = link_cli::DoubletsStorage<usize, link_cli::FileMappedUnitStore<usize>>;

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
type NativeLinkCliTransactions = link_cli::GenericTransactionsDecorator<
    usize,
    NativeLinkCliStorage,
    link_cli::FileTransitionLog,
>;

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
#[derive(Clone)]
struct LinkStoreSnapshot {
    events: Vec<MemoryEvent>,
    records: Vec<LinkRecord>,
    nodes: BTreeMap<String, usize>,
}

/// Persistent native link-cli store used by Rust builds and the HTTP server.
///
/// The binary database is link-cli's file-mapped `doublets-rs` store. Every
/// mutation passes through `GenericTransactionsDecorator`, whose sidecar log
/// recovers an interrupted write on the next open. String-to-address mappings
/// and the reviewable events stay in memory because `.lino` is their portable
/// source projection; [`Self::replace_memory_events_transactionally`] rebuilds
/// the complete binary graph from that source in one transaction.
#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub struct LinkCliLinkStore {
    database: PathBuf,
    database_lock: Option<link_cli::FileLock>,
    events: Vec<MemoryEvent>,
    records: Vec<LinkRecord>,
    nodes: BTreeMap<String, usize>,
    transactions: Option<NativeLinkCliTransactions>,
    transaction_snapshot: Option<LinkStoreSnapshot>,
    temporary_database: Option<PathBuf>,
}

/// Backwards-compatible name for embedders that used the pre-link-cli type.
#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub type DoubletsLinkStore = LinkCliLinkStore;

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
impl LinkCliLinkStore {
    /// Create an empty temporary link-cli store.
    pub fn new() -> Result<Self, LinkStoreError> {
        static NEXT_TEMPORARY_STORE: AtomicU64 = AtomicU64::new(0);
        let sequence = NEXT_TEMPORARY_STORE.fetch_add(1, Ordering::Relaxed);
        let database = std::env::temp_dir().join(format!(
            "formal-ai-link-cli-{}-{sequence}.links",
            std::process::id()
        ));
        let mut store = Self::open_at(&database)?;
        store.temporary_database = Some(database);
        Ok(store)
    }

    /// Open or create a file-mapped link-cli database with an exclusive lock.
    pub fn open_at(database: &Path) -> Result<Self, LinkStoreError> {
        let database_lock = link_cli::FileLock::acquire(
            link_cli::lock_file_path(database),
            link_cli::LockMode::Exclusive,
        )
        .map_err(LinkStoreError::from)?;
        let transactions = open_link_cli_transactions(database)?;
        Ok(Self {
            database: database.to_path_buf(),
            database_lock: Some(database_lock),
            events: Vec::new(),
            records: Vec::new(),
            nodes: BTreeMap::new(),
            transactions: Some(transactions),
            transaction_snapshot: None,
            temporary_database: None,
        })
    }

    /// Build a temporary native store from a `.lino` memory or bundle document.
    pub fn from_links_notation(text: &str) -> Result<Self, LinkStoreError> {
        let mut store = Self::new()?;
        store.import_memory_links_notation(text)?;
        Ok(store)
    }

    /// Return the imported or appended memory events in append order.
    #[must_use]
    pub fn events(&self) -> &[MemoryEvent] {
        &self.events
    }

    /// Number of memory events projected into link-cli.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether this native store currently has no memory events.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Number of raw native links, including point nodes.
    #[must_use]
    pub fn native_link_count(&self) -> usize {
        use link_cli::LinksStorage as _;
        self.transactions().inner().links_count()
    }

    /// Begin one explicit transaction spanning both the local projection and
    /// link-cli's binary store.
    pub fn begin_transaction(&mut self) -> Result<(), LinkStoreError> {
        if self.transaction_snapshot.is_some() {
            return Err(LinkStoreError::Backend(String::from(
                "nested link-cli transactions are not supported",
            )));
        }
        self.transactions_mut()
            .begin_transaction()
            .map_err(LinkStoreError::from)?;
        self.transaction_snapshot = Some(self.snapshot());
        Ok(())
    }

    /// Commit and `fsync` the current transaction and recovery log.
    pub fn commit_transaction(&mut self) -> Result<(), LinkStoreError> {
        self.transactions_mut()
            .commit()
            .map_err(LinkStoreError::from)?;
        self.transaction_snapshot = None;
        self.transactions_mut()
            .flush()
            .map_err(LinkStoreError::from)
    }

    /// Roll back the current transaction and restore its in-memory projection.
    pub fn rollback_transaction(&mut self) -> Result<(), LinkStoreError> {
        self.transactions_mut()
            .rollback()
            .map_err(LinkStoreError::from)?;
        if let Some(snapshot) = self.transaction_snapshot.take() {
            self.restore_snapshot(snapshot);
        }
        self.transactions_mut()
            .flush()
            .map_err(LinkStoreError::from)
    }

    /// Transactionally build and atomically publish a complete replacement.
    ///
    /// This is the server synchronization boundary. Rebuilding makes the
    /// `.lino` projection a deterministic recovery source even if a previous
    /// process stopped between writing it and opening link-cli. Building a new
    /// graph instead of deleting the old graph also avoids invalidating the
    /// usage indexes that link-cli's underlying tree decorators maintain.
    pub fn replace_memory_events_transactionally(
        &mut self,
        events: &[MemoryEvent],
    ) -> Result<(), LinkStoreError> {
        if self.transaction_snapshot.is_some() {
            return Err(LinkStoreError::Backend(String::from(
                "cannot replace memory inside an open link-cli transaction",
            )));
        }
        let replacement_database = replacement_path(&self.database, "database");
        let replacement_log = server_link_transition_log_path(&replacement_database);
        cleanup_link_cli_files(&replacement_database);

        let mut replacement = match Self::open_at(&replacement_database) {
            Ok(replacement) => replacement,
            Err(error) => {
                cleanup_link_cli_files(&replacement_database);
                return Err(error);
            }
        };
        if let Err(error) = replacement.begin_transaction() {
            drop(replacement);
            cleanup_link_cli_files(&replacement_database);
            return Err(error);
        }
        for event in events.iter().cloned() {
            if let Err(error) = replacement.append_memory_event_in_open_transaction(event) {
                let rollback = replacement.rollback_transaction();
                drop(replacement);
                cleanup_link_cli_files(&replacement_database);
                return rollback.and(Err(error));
            }
        }
        if let Err(error) = replacement.commit_transaction() {
            drop(replacement);
            cleanup_link_cli_files(&replacement_database);
            return Err(error);
        }
        let snapshot = replacement.snapshot();
        drop(replacement);

        // Close the old memory map while retaining the explicit sidecar lock.
        // Install the fully-applied recovery log first: either database is a
        // valid baseline for that log because it contains no pending replay.
        let _ = self.transactions.take();
        let replacement_result = replace_file(
            &replacement_log,
            &server_link_transition_log_path(&self.database),
        )
        .and_then(|()| replace_file(&replacement_database, &self.database))
        .map_err(|error| {
            LinkStoreError::Backend(format!(
                "failed to publish link-cli replacement {}: {error}",
                self.database.display()
            ))
        });

        let reopen_result = open_link_cli_transactions(&self.database);
        cleanup_link_cli_files(&replacement_database);
        self.transactions = Some(reopen_result?);
        replacement_result?;
        self.restore_snapshot(snapshot);
        Ok(())
    }

    fn append_memory_event_in_open_transaction(
        &mut self,
        mut event: MemoryEvent,
    ) -> Result<String, LinkStoreError> {
        ensure_event_id(&mut event, self.events.len());
        let id = event.id.clone();
        let record = memory_event_to_link_record(&event, self.events.len());
        self.insert_record(record)?;
        self.events.push(event);
        Ok(id)
    }

    fn insert_record(&mut self, record: LinkRecord) -> Result<(), LinkStoreError> {
        for link in &record.links {
            self.append_native_doublet(&link.from, &link.to)?;
        }
        self.records.push(record);
        Ok(())
    }

    fn append_native_doublet(&mut self, from: &str, to: &str) -> Result<(), LinkStoreError> {
        let source = self.node_id(from)?;
        let target = self.node_id(to)?;
        self.transactions_mut()
            .create(source, target)
            .map_err(LinkStoreError::from)?;
        Ok(())
    }

    fn node_id(&mut self, node: &str) -> Result<usize, LinkStoreError> {
        if let Some(id) = self.nodes.get(node) {
            return Ok(*id);
        }
        let id = self
            .transactions_mut()
            .create(0, 0)
            .map_err(LinkStoreError::from)?;
        self.nodes.insert(node.to_owned(), id);
        Ok(id)
    }

    const fn transactions(&self) -> &NativeLinkCliTransactions {
        self.transactions
            .as_ref()
            .expect("link-cli transactions remain present until drop")
    }

    const fn transactions_mut(&mut self) -> &mut NativeLinkCliTransactions {
        self.transactions
            .as_mut()
            .expect("link-cli transactions remain present until drop")
    }

    fn snapshot(&self) -> LinkStoreSnapshot {
        LinkStoreSnapshot {
            events: self.events.clone(),
            records: self.records.clone(),
            nodes: self.nodes.clone(),
        }
    }

    fn restore_snapshot(&mut self, snapshot: LinkStoreSnapshot) {
        self.events = snapshot.events;
        self.records = snapshot.records;
        self.nodes = snapshot.nodes;
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
impl LinkStore for LinkCliLinkStore {
    fn backend(&self) -> LinkStoreBackend {
        LinkStoreBackend::LinkCli
    }

    fn append_memory_event(&mut self, event: MemoryEvent) -> Result<String, LinkStoreError> {
        let owns_transaction = self.transaction_snapshot.is_none();
        if owns_transaction {
            self.begin_transaction()?;
        }
        let result = self.append_memory_event_in_open_transaction(event);
        if !owns_transaction {
            return result;
        }
        match result {
            Ok(id) => {
                self.commit_transaction()?;
                Ok(id)
            }
            Err(error) => {
                let rollback = self.rollback_transaction();
                rollback.and(Err(error))
            }
        }
    }

    fn import_memory_links_notation(&mut self, text: &str) -> Result<usize, LinkStoreError> {
        validate_memory_links_notation(text)?;
        let parsed = import_full_memory(text);
        let count = parsed.events.len();
        self.begin_transaction()?;
        for event in parsed.events {
            if let Err(error) = self.append_memory_event_in_open_transaction(event) {
                let rollback = self.rollback_transaction();
                return rollback.and(Err(error));
            }
        }
        self.commit_transaction()?;
        Ok(count)
    }

    fn export_memory_links_notation(&self) -> String {
        crate::memory::export_links_notation(&self.events)
    }

    fn records(&self) -> Vec<LinkRecord> {
        self.records.clone()
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
impl Drop for LinkCliLinkStore {
    fn drop(&mut self) {
        if self.transaction_snapshot.is_some() {
            let _ = self.rollback_transaction();
        }
        let _ = self.transactions.take();
        let _ = self.database_lock.take();
        let Some(database) = self.temporary_database.take() else {
            return;
        };
        let _ = std::fs::remove_file(server_link_transition_log_path(&database));
        let _ = std::fs::remove_file(link_cli::lock_file_path(&database));
        let _ = std::fs::remove_file(database);
    }
}

/// Conventional transition-log path used by the embedded link-cli store.
#[must_use]
pub fn server_link_transition_log_path(database: &Path) -> PathBuf {
    let stem = database
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let filename = format!("{stem}.transitions.links");
    database
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from(&filename), |parent| parent.join(&filename))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
impl From<link_cli::LinkError> for LinkStoreError {
    fn from(error: link_cli::LinkError) -> Self {
        Self::Backend(error.to_string())
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
fn open_link_cli_transactions(
    database: &Path,
) -> Result<NativeLinkCliTransactions, LinkStoreError> {
    let storage = NativeLinkCliStorage::open(database).map_err(LinkStoreError::from)?;
    let transition_log =
        link_cli::FileTransitionLog::open(server_link_transition_log_path(database))
            .map_err(LinkStoreError::from)?;
    link_cli::GenericTransactionsDecorator::new(
        storage,
        transition_log,
        link_cli::LogRetentionPolicy::default(),
        link_cli::CommitMode::Sync,
        link_cli_debug_enabled(),
    )
    .map_err(LinkStoreError::from)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
fn replacement_path(database: &Path, label: &str) -> PathBuf {
    static NEXT_REPLACEMENT: AtomicU64 = AtomicU64::new(0);
    let sequence = NEXT_REPLACEMENT.fetch_add(1, Ordering::Relaxed);
    let filename = database
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("memory.links");
    database.with_file_name(format!(
        ".{filename}.{label}.{}.{}.tmp",
        std::process::id(),
        sequence
    ))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
fn replace_file(staged: &Path, destination: &Path) -> std::io::Result<()> {
    match std::fs::rename(staged, destination) {
        Ok(()) => Ok(()),
        Err(_rename_error) if destination.exists() => {
            // `rename` replaces atomically on Unix. Windows requires moving
            // the destination aside first, so retain a recoverable backup if
            // publishing the staged file fails.
            let backup = replacement_path(destination, "backup");
            std::fs::rename(destination, &backup)?;
            match std::fs::rename(staged, destination) {
                Ok(()) => {
                    let _ = std::fs::remove_file(backup);
                    Ok(())
                }
                Err(error) => {
                    let _ = std::fs::rename(&backup, destination);
                    Err(error)
                }
            }
        }
        Err(error) => Err(error),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
fn cleanup_link_cli_files(database: &Path) {
    let _ = std::fs::remove_file(server_link_transition_log_path(database));
    let _ = std::fs::remove_file(link_cli::lock_file_path(database));
    let _ = std::fs::remove_file(database);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
fn link_cli_debug_enabled() -> bool {
    std::env::var("FORMAL_AI_LINK_CLI_DEBUG").as_deref() == Ok("1")
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
            2 if content.starts_with("schema_version ") => validate_schema_version_line(content)?,
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

fn validate_schema_version_line(content: &str) -> Result<(), LinkStoreError> {
    let Some(rest) = content.strip_prefix("schema_version ") else {
        return Err(LinkStoreError::IllFormedLinksNotation(String::from(
            "invalid schema version marker",
        )));
    };
    validate_strict_quoted(rest)?;
    let value = crate::memory::parse_quoted(rest).unwrap_or_default();
    if value.parse::<u32>().is_err() {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "invalid_schema_version:value={value}"
        )));
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
    fields.insert(String::from("accessCount"), event.access_count.to_string());
    fields.insert(
        String::from("writeCount"),
        event.write_count.max(1).to_string(),
    );
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
