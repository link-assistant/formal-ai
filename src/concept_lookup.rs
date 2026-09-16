//! Plan 00 §4.2's `SourceLookup`, implemented exactly once (issue #1138, plan
//! 01 L5–L7, L12).
//!
//! A retrieved sense carries the provenance of the exact bytes it was read
//! from, so a gloss can be quoted and attributed but never inlined into a
//! generated program. Which extractor reads which source is declared by the
//! registry's `extractor` field, never by a host-name `match` in Rust.

use std::collections::BTreeSet;

use crate::how_to_guide::ServicePreferences;
use crate::needs::Need;
use crate::relative_meta_logic::SourceTier;
use crate::seed::SourceRecord;
use crate::service_accessibility::ServiceAccessibilityCache;
use crate::source_fetch::{CachedSourceClient, SourceCapture, SourceTransport};
use crate::source_walk::{CaptureExtractor, LookupBounds, SourceLookup, WalkOutcome, WalkSourceOutcome};

/// One retrieved sense of one surface form, with the provenance of the exact
/// bytes it was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptSense {
    pub surface: String,
    pub lemma: String,
    pub language: String,
    pub gloss: String,
    pub part_of_speech: String,
    pub synonyms: Vec<String>,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
    pub cached: bool,
    pub tier: SourceTier,
    pub license_name: String,
    pub license_url: String,
    pub depth: usize,
}

impl ConceptSense {
    /// Stable id over `sha256 + lemma + gloss`; the value the forget /
    /// rediscover round trip compares.
    #[must_use]
    pub fn content_id(&self) -> String {
        todo!("plan 01 leaf L5")
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 01 leaf L5")
    }
}

/// What a lookup produced, or honestly did not.
#[derive(Debug, Clone, PartialEq)]
pub enum LookupOutcome {
    /// Evidence, in consultation order. Every item carries the digest of the
    /// bytes it was read from.
    Found(Vec<ConceptSense>),
    /// No source answered. `consulted` names every source and its observed
    /// outcome, so the absence is attributable rather than bare.
    NotFound { consulted: Vec<WalkSourceOutcome> },
}

/// Registry-declared extractor bindings, chosen by the registry's `extractor`
/// field rather than by a host-name match in Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenseExtractor {
    surface: String,
    language: String,
}

impl SenseExtractor {
    #[must_use]
    pub fn new(_surface: &str, _language: &str) -> Self {
        todo!("plan 01 leaf L5")
    }
}

impl CaptureExtractor for SenseExtractor {
    type Item = ConceptSense;

    fn extract(
        &self,
        _record: &SourceRecord,
        _capture: &SourceCapture,
        _depth: usize,
        _limit: usize,
    ) -> Vec<Self::Item> {
        todo!("plan 01 leaf L5")
    }

    fn follow(&self, _record: &SourceRecord, _capture: &SourceCapture, _limit: usize) -> Vec<String> {
        todo!("plan 01 leaf L5")
    }

    fn entry_url(&self, _record: &SourceRecord, _subject: &str) -> Option<String> {
        todo!("plan 01 leaf L5")
    }
}

/// The single implementation of plan 00 §4.2.
pub struct RegistrySourceLookup<'a, T: SourceTransport> {
    client: &'a CachedSourceClient<T>,
    preferences: &'a ServicePreferences,
    availability: &'a mut ServiceAccessibilityCache,
    bounds: LookupBounds,
    language: String,
    now: u64,
    consulted: BTreeSet<String>,
    outcomes: Vec<WalkSourceOutcome>,
}

impl<'a, T: SourceTransport> RegistrySourceLookup<'a, T> {
    #[must_use]
    pub fn new(
        _client: &'a CachedSourceClient<T>,
        _preferences: &'a ServicePreferences,
        _availability: &'a mut ServiceAccessibilityCache,
        _bounds: LookupBounds,
        _language: &str,
        _now: u64,
    ) -> Self {
        todo!("plan 01 leaf L7")
    }

    /// The outcome row per source, so a caller can report why nothing was found.
    #[must_use]
    pub fn outcomes(&self) -> &[WalkSourceOutcome] {
        todo!("plan 01 leaf L7")
    }
}

impl<T: SourceTransport> SourceLookup for RegistrySourceLookup<'_, T> {
    fn lookup(&mut self, _need: &Need, _bounds: &LookupBounds) -> LookupOutcome {
        todo!("plan 01 leaf L7")
    }
}

/// The thin adapter that keeps `src/coding/concept_discovery.rs` compiling while
/// retrieval lands; removed by plan 01 L18 once every caller names
/// [`SourceLookup`] directly.
pub struct RegistryConceptLookup<'a, T: SourceTransport> {
    inner: RegistrySourceLookup<'a, T>,
}

impl<'a, T: SourceTransport> RegistryConceptLookup<'a, T> {
    #[must_use]
    pub fn new(_inner: RegistrySourceLookup<'a, T>) -> Self {
        todo!("plan 01 leaf L7")
    }
}

impl<T: SourceTransport> crate::coding::concept_discovery::UnknownConceptLookup
    for RegistryConceptLookup<'_, T>
{
    fn lookup(
        &mut self,
        _phrase: &str,
        _depth: usize,
    ) -> Option<crate::coding::concept_discovery::ConceptEvidence> {
        todo!("plan 01 leaf L7")
    }
}

/// Look one surface up, as the trait does, but returning every sense instead of
/// the first. The universal loop and the coding path both call this.
pub fn lookup_surface<T: SourceTransport>(
    _surface: &str,
    _language: &str,
    _client: &CachedSourceClient<T>,
    _preferences: &ServicePreferences,
    _bounds: &LookupBounds,
    _availability: &mut ServiceAccessibilityCache,
    _now: u64,
) -> WalkOutcome<ConceptSense> {
    todo!("plan 01 leaf L7")
}

/// The surfaces in `normalized` that no seeded meaning, no memory link and no
/// declared-name convention accounts for — the words the system must ask about.
/// Language-neutral: it asks the lexicon, never a literal list.
#[must_use]
pub fn unknown_surfaces(_normalized: &str, _language: &str) -> Vec<String> {
    todo!("plan 01 leaf L7")
}
