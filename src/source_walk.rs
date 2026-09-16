//! The one bounded recursive capture walk, need-kind- and
//! extractor-parameterised (issue #1138, plan 01 L2).
//!
//! This module owns plan 00 §4.2's [`SourceLookup`] contract and its
//! [`LookupBounds`]; `how_to_guide::GuideBounds` becomes an alias of the latter
//! so there is one bounds vocabulary in the tree. Nothing here knows a host
//! name: which sources a need kind may consult is read from
//! `data/seed/sources-registry.lino` through [`select_sources`].

use crate::how_to_guide::ServicePreferences;
use crate::needs::{Need, NeedKind};
use crate::seed::SourceRecord;
use crate::service_accessibility::ServiceAccessibilityCache;
use crate::source_fetch::{CachedSourceClient, SourceCapture, SourceTransport};

/// Declared bounds. Every capture is charged against these, so a walk's cost is
/// knowable before it starts. Depth and evidence bounds, never a time or token
/// budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookupBounds {
    pub max_depth: usize,
    pub max_pages_per_service: usize,
    pub max_services: usize,
    pub max_items: usize,
    pub max_capture_age_seconds: u64,
}

impl Default for LookupBounds {
    fn default() -> Self {
        Self {
            max_depth: 2,
            max_pages_per_service: 4,
            max_services: 6,
            max_items: 8,
            max_capture_age_seconds: 7 * 24 * 60 * 60,
        }
    }
}

/// Turns captured bytes into items of one need kind, and names which
/// same-source links are worth following next.
pub trait CaptureExtractor {
    type Item;

    fn extract(
        &self,
        record: &SourceRecord,
        capture: &SourceCapture,
        depth: usize,
        limit: usize,
    ) -> Vec<Self::Item>;

    fn follow(&self, record: &SourceRecord, capture: &SourceCapture, limit: usize) -> Vec<String>;

    fn entry_url(&self, record: &SourceRecord, subject: &str) -> Option<String>;
}

/// What one source produced, or why it produced nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkSourceOutcome {
    pub source_id: String,
    /// `contributed | no_items | disabled | unbound_template | unreachable_cached
    /// | fetch_error | stale_capture`.
    pub status: String,
    pub detail: String,
    pub pages: usize,
    pub items: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WalkOutcome<I> {
    pub subject: String,
    pub items: Vec<I>,
    pub outcomes: Vec<WalkSourceOutcome>,
    pub bounds: LookupBounds,
}

/// Plan 00 §4.2's contract: resolve a need against the sources registry, in
/// registry order for the need's kind, honoring settings opt-outs and licenses.
pub trait SourceLookup {
    fn lookup(
        &mut self,
        need: &Need,
        bounds: &LookupBounds,
    ) -> crate::concept_lookup::LookupOutcome;
}

/// Select the sources this need kind and these settings allow, walk each one
/// inside `bounds`, and return everything the extractor recognised together
/// with an outcome row for every source that could have contributed.
pub fn walk_sources<T: SourceTransport, E: CaptureExtractor>(
    _kind: NeedKind,
    _subject: &str,
    _extractor: &E,
    _client: &CachedSourceClient<T>,
    _preferences: &ServicePreferences,
    _bounds: &LookupBounds,
    _availability: &mut ServiceAccessibilityCache,
    _now: u64,
) -> WalkOutcome<E::Item> {
    todo!("plan 01 leaf L2")
}

/// The sources a need kind may consult, in consultation order: declared kind
/// first, then derived tier descending, then registry order. Total and
/// reproducible.
#[must_use]
pub fn select_sources(
    _kind: NeedKind,
    _subject: &str,
    _preferences: &ServicePreferences,
    _bounds: &LookupBounds,
) -> Vec<SourceRecord> {
    todo!("plan 01 leaf L2")
}
