//! Per-service accessibility memory with a seven-day time-to-live — issue #991.
//!
//! [`crate::source_fetch`] already caches *bodies* for sixty days, but a body
//! cache cannot answer "is this service reachable from here at all?": a service
//! that has never been reached has no body to cache, and a service that answered
//! `403` yesterday would be retried on every prompt forever. Issue #991 asks for
//! the complementary record — the *availability* of each declared external
//! service — kept in the environment's associative memory with a TTL of at least
//! seven days, plus explicit refresh and invalidation.
//!
//! The store is deliberately thin and fully deterministic:
//!
//! * one record per `sources-registry.lino` service id;
//! * `reachable` / `unreachable` plus the exact diagnostic that produced it;
//! * `checked_at` in Unix seconds and the TTL that record was written under, so
//!   a stored record stays interpretable when the default TTL changes;
//! * a record older than its TTL is *stale*, never silently dropped — callers
//!   ask [`ServiceAccessibilityCache::needs_refresh`] and re-probe;
//! * [`ServiceAccessibilityCache::invalidate`] forgets one service and
//!   [`ServiceAccessibilityCache::invalidate_all`] forgets every service, which
//!   is the explicit invalidation path the issue requires.
//!
//! Records are projected into an [`AssociativeMemory`] — the same associative
//! links substrate the rest of the environment memory uses — and persisted as
//! Links Notation next to the source cache, so the CLI, the HTTP server, and the
//! browser worker all read and write the same shape.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::associative_persistence::AssociativeMemory;
use crate::links_format::{format_lino_value, sanitize_lino_value};
use crate::seed::parser::parse_lino;

/// Seven days, the minimum TTL issue #991 requires for availability records.
pub const SERVICE_ACCESSIBILITY_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

/// File the availability records are projected into.
pub const SERVICE_ACCESSIBILITY_FILE: &str = "service-accessibility.lino";

/// Root node name of the Links Notation projection.
const ROOT: &str = "service_accessibility";

/// Whether a declared service answered the last time it was probed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    /// The service answered with usable bytes.
    Reachable,
    /// The service could not be reached (transport error, refusal, or block).
    Unreachable,
}

impl ServiceStatus {
    /// Stable slug used in the projection and in evidence events.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Reachable => "reachable",
            Self::Unreachable => "unreachable",
        }
    }

    /// Parse a slug back, rejecting anything else so a corrupt record is
    /// ignored rather than silently read as "reachable".
    #[must_use]
    pub fn from_slug(value: &str) -> Option<Self> {
        match value {
            "reachable" => Some(Self::Reachable),
            "unreachable" => Some(Self::Unreachable),
            _ => None,
        }
    }

    /// Whether the service may be consulted.
    #[must_use]
    pub const fn is_reachable(self) -> bool {
        matches!(self, Self::Reachable)
    }
}

/// One remembered accessibility observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAccessibilityRecord {
    /// `sources-registry.lino` source id.
    pub service_id: String,
    /// Whether the service answered.
    pub status: ServiceStatus,
    /// The exact diagnostic behind the status (HTTP error, transport message).
    pub detail: String,
    /// Unix seconds at which the probe happened.
    pub checked_at: u64,
    /// TTL this record was written under, in seconds.
    pub ttl_seconds: u64,
}

impl ServiceAccessibilityRecord {
    /// Unix second at which this record stops being authoritative.
    #[must_use]
    pub const fn expires_at(&self) -> u64 {
        self.checked_at.saturating_add(self.ttl_seconds)
    }

    /// Whether the record is still within its TTL at `now`.
    #[must_use]
    pub const fn is_fresh(&self, now: u64) -> bool {
        now < self.expires_at()
    }

    /// Age in seconds at `now` (saturating, so a clock that moved backwards
    /// reports `0` rather than wrapping).
    #[must_use]
    pub const fn age(&self, now: u64) -> u64 {
        now.saturating_sub(self.checked_at)
    }

    /// Stable payload for evidence events.
    #[must_use]
    pub fn trace_payload(&self) -> String {
        format!(
            "service={} status={} checked_at={} ttl_seconds={} detail={}",
            self.service_id,
            self.status.slug(),
            self.checked_at,
            self.ttl_seconds,
            self.detail
        )
    }

    /// The associative-memory expression id for this service.
    #[must_use]
    pub fn memory_id(&self) -> String {
        memory_id(&self.service_id)
    }

    /// The associative-memory expression text for this service.
    #[must_use]
    pub fn memory_text(&self) -> String {
        format!(
            "{ROOT} {} {} {} {}",
            self.service_id,
            self.status.slug(),
            self.checked_at,
            self.ttl_seconds
        )
    }
}

/// Associative-memory expression id for a service accessibility record.
#[must_use]
pub fn memory_id(service_id: &str) -> String {
    format!("{ROOT}:{service_id}")
}

/// The per-environment availability store.
#[derive(Debug, Clone)]
pub struct ServiceAccessibilityCache {
    path: PathBuf,
    ttl_seconds: u64,
    records: BTreeMap<String, ServiceAccessibilityRecord>,
}

impl ServiceAccessibilityCache {
    /// An empty in-memory store rooted at `dir`.
    #[must_use]
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join(SERVICE_ACCESSIBILITY_FILE),
            ttl_seconds: SERVICE_ACCESSIBILITY_TTL_SECONDS,
            records: BTreeMap::new(),
        }
    }

    /// Override the TTL new observations are written under. Values below the
    /// seven-day floor are raised to it: the issue states the TTL requirement as
    /// "≥ 7 days", so a caller cannot accidentally weaken it.
    #[must_use]
    pub fn with_ttl_seconds(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = ttl_seconds.max(SERVICE_ACCESSIBILITY_TTL_SECONDS);
        self
    }

    /// TTL new observations are written under.
    #[must_use]
    pub const fn ttl_seconds(&self) -> u64 {
        self.ttl_seconds
    }

    /// Where the projection is persisted.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load the projection from disk. A missing file is an empty store; an
    /// unreadable or malformed one is also empty, because a lost availability
    /// record only costs one re-probe.
    #[must_use]
    pub fn load(dir: impl AsRef<Path>) -> Self {
        let mut cache = Self::new(dir);
        if let Ok(text) = fs::read_to_string(&cache.path) {
            cache.records = parse_projection(&text);
        }
        cache
    }

    /// Every remembered record, ordered by service id.
    #[must_use]
    pub const fn records(&self) -> &BTreeMap<String, ServiceAccessibilityRecord> {
        &self.records
    }

    /// The remembered record for `service_id`, fresh or stale.
    #[must_use]
    pub fn record(&self, service_id: &str) -> Option<&ServiceAccessibilityRecord> {
        self.records.get(service_id)
    }

    /// The record for `service_id` only while it is still within its TTL.
    #[must_use]
    pub fn fresh_record(&self, service_id: &str, now: u64) -> Option<&ServiceAccessibilityRecord> {
        self.records
            .get(service_id)
            .filter(|record| record.is_fresh(now))
    }

    /// Whether the service must be probed again: no record at all, or a record
    /// past its TTL.
    #[must_use]
    pub fn needs_refresh(&self, service_id: &str, now: u64) -> bool {
        self.fresh_record(service_id, now).is_none()
    }

    /// Whether a *fresh* record says the service is unreachable, which is the
    /// only condition under which a caller skips a service it is allowed to use.
    #[must_use]
    pub fn known_unreachable(&self, service_id: &str, now: u64) -> bool {
        self.fresh_record(service_id, now)
            .is_some_and(|record| !record.status.is_reachable())
    }

    /// Record one probe result, replacing any earlier record for the service.
    pub fn observe(
        &mut self,
        service_id: impl Into<String>,
        status: ServiceStatus,
        detail: impl Into<String>,
        now: u64,
    ) -> ServiceAccessibilityRecord {
        let record = ServiceAccessibilityRecord {
            service_id: service_id.into(),
            status,
            detail: sanitize_lino_value(&detail.into()),
            checked_at: now,
            ttl_seconds: self.ttl_seconds,
        };
        self.records
            .insert(record.service_id.clone(), record.clone());
        record
    }

    /// Explicitly forget one service so the next call re-probes it.
    pub fn invalidate(&mut self, service_id: &str) -> Option<ServiceAccessibilityRecord> {
        self.records.remove(service_id)
    }

    /// Explicitly forget every service.
    pub fn invalidate_all(&mut self) -> usize {
        let count = self.records.len();
        self.records.clear();
        count
    }

    /// Drop records that are already past their TTL, returning how many went.
    /// Expiry is lazy everywhere else; this is the eager form a maintenance
    /// command can call.
    pub fn prune_expired(&mut self, now: u64) -> usize {
        let before = self.records.len();
        self.records.retain(|_, record| record.is_fresh(now));
        before - self.records.len()
    }

    /// The Links Notation projection of the store.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from(ROOT);
        out.push('\n');
        for record in self.records.values() {
            out.push_str("  service ");
            out.push_str(&format_lino_value(&record.service_id));
            out.push('\n');
            out.push_str("    status ");
            out.push_str(record.status.slug());
            out.push('\n');
            out.push_str("    detail ");
            out.push_str(&format_lino_value(&record.detail));
            out.push('\n');
            out.push_str("    checked_at ");
            out.push_str(&record.checked_at.to_string());
            out.push('\n');
            out.push_str("    ttl_seconds ");
            out.push_str(&record.ttl_seconds.to_string());
            out.push('\n');
        }
        out
    }

    /// The associative-memory view of the store: one persisted expression per
    /// service, linked to the registry root so degree-based retention treats the
    /// availability network as one connected structure.
    #[must_use]
    pub fn associative_memory(&self) -> AssociativeMemory {
        let mut memory = AssociativeMemory::new();
        memory.persist_identified(ROOT, ROOT);
        for record in self.records.values() {
            let id = record.memory_id();
            memory.persist_identified(id.clone(), record.memory_text());
            memory.associate(ROOT, &id);
        }
        memory
    }

    /// Persist the projection, creating the directory when needed.
    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, self.links_notation())
    }
}

fn parse_projection(text: &str) -> BTreeMap<String, ServiceAccessibilityRecord> {
    let tree = parse_lino(text);
    let mut records = BTreeMap::new();
    for root in tree.children.iter().filter(|node| node.name == ROOT) {
        for entry in root.children.iter().filter(|node| node.name == "service") {
            let service_id = entry.id.clone();
            let Some(status) = ServiceStatus::from_slug(entry.find_child_value("status")) else {
                continue;
            };
            let Ok(checked_at) = entry.find_child_value("checked_at").parse::<u64>() else {
                continue;
            };
            let ttl_seconds = entry
                .find_child_value("ttl_seconds")
                .parse::<u64>()
                .unwrap_or(SERVICE_ACCESSIBILITY_TTL_SECONDS);
            records.insert(
                service_id.clone(),
                ServiceAccessibilityRecord {
                    service_id,
                    status,
                    detail: entry.find_child_value("detail").to_owned(),
                    checked_at,
                    ttl_seconds,
                },
            );
        }
    }
    records
}

/// Current Unix second, or `0` when the clock is before the epoch.
#[must_use]
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
