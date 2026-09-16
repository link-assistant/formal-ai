//! The one bounded recursive capture walk, need-kind- and
//! extractor-parameterised (issue #1138, plan 01 L2).
//!
//! Issue #991 paid for a bounded, recursive, accessibility-aware,
//! license-carrying, offline-replayable walk over
//! `data/seed/sources-registry.lino`, and it lived inside
//! `src/how_to_guide.rs`, a module about procedures. A second need kind — what
//! does this word mean — had the choice of copying that walk or doing without
//! one. Copying it is the parity defect #991 was filed to remove, so the walk
//! moves here and `how_to_guide` becomes "the `Procedure` kind with a step
//! extractor" while `concept_lookup` becomes "the `Concept` kind with a sense
//! extractor".
//!
//! This module owns plan 00 §4.2's [`SourceLookup`] contract and its
//! [`LookupBounds`]; `how_to_guide::GuideBounds` is an alias of the latter, so
//! there is one bounds vocabulary in the tree. Nothing here knows a host name:
//! which sources a need kind may consult is read from the registry through
//! [`select_sources`], and what a captured page means is decided by a
//! [`CaptureExtractor`] the caller supplies.

use std::collections::VecDeque;

use crate::how_to_guide::ServicePreferences;
use crate::needs::{Need, NeedKind};
use crate::seed::{SourceRecord, external_trusted_sources};
use crate::service_accessibility::{ServiceAccessibilityCache, ServiceStatus};
use crate::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};
use crate::trace_record;

/// Declared bounds. Every capture is charged against these, so a walk's cost is
/// knowable before it starts. Depth and evidence bounds, never a time or token
/// budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookupBounds {
    /// How many link hops past a service's entry request may be followed.
    pub max_depth: usize,
    /// How many pages a single service may cost, across all depths.
    pub max_pages_per_service: usize,
    /// How many services may be consulted for one subject.
    pub max_services: usize,
    /// How many items the finished answer may contain — steps for a procedure
    /// need, senses for a concept need.
    pub max_items: usize,
    /// The time bound: a capture older than this is reported as stale, because
    /// evidence that has not been re-verified in that long is not evidence the
    /// caller should silently trust.
    pub max_capture_age_seconds: u64,
}

impl Default for LookupBounds {
    /// The bounds `how_to_guide::GuideBounds` has carried since issue #991.
    ///
    /// They are the default for every need kind rather than one kind's private
    /// numbers, because the guard
    /// `tests/unit/issue_1138_source_walk_parity.rs` requires the shared kernel
    /// and the how-to path to select the same sources: two different defaults
    /// would be two different walks again.
    fn default() -> Self {
        Self {
            max_depth: 2,
            max_pages_per_service: 4,
            max_services: 4,
            max_items: 12,
            max_capture_age_seconds: 60 * 60 * 24 * 60,
        }
    }
}

impl LookupBounds {
    /// Stable one-line description used in traces and rendered answers.
    #[must_use]
    pub fn trace_payload(&self) -> String {
        trace_record::payload(&[
            ("max_depth", self.max_depth.to_string()),
            (
                "max_pages_per_service",
                self.max_pages_per_service.to_string(),
            ),
            ("max_services", self.max_services.to_string()),
            ("max_items", self.max_items.to_string()),
            (
                "max_capture_age_seconds",
                self.max_capture_age_seconds.to_string(),
            ),
        ])
    }
}

/// What one captured page yielded, and where the walk may go next from it.
///
/// One return value rather than the three separate methods plan 01 sketched:
/// every payload shape the how-to extractor recognises decides all three things
/// at once — whether it produced items, whether following a link is worth a
/// page, and what to record when it produced nothing — and splitting them into
/// independent calls would have made the kernel re-classify the same bytes
/// three times and still not know why a page was empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extracted<I> {
    /// Items recognised in these bytes.
    pub items: Vec<I>,
    /// Same-source URLs worth capturing next, charged against the page bound.
    pub follow: Vec<String>,
    /// Why this page produced nothing, when that is worth recording.
    pub detail: Option<String>,
}

impl<I> Default for Extracted<I> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            follow: Vec::new(),
            detail: None,
        }
    }
}

/// Turns captured bytes into items of one need kind, and names which
/// same-source links are worth following next.
pub trait CaptureExtractor {
    /// What one capture yields: a guide step, a concept sense, a part.
    type Item;

    /// Bind the registry's API template for this subject, or `None` when a
    /// required placeholder cannot be filled from the subject alone.
    fn entry_url(&self, record: &SourceRecord, subject: &str) -> Option<String>;

    /// Read one capture. `produced` is how many items the walk already holds
    /// from this service, so an extractor can decide to recurse only when the
    /// shallower pages gave nothing.
    fn read(
        &self,
        record: &SourceRecord,
        capture: &SourceCapture,
        depth: usize,
        produced: usize,
        bounds: &LookupBounds,
    ) -> Extracted<Self::Item>;
}

/// What one source produced, or why it produced nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkSourceOutcome {
    /// Registry id of the service.
    pub source_id: String,
    /// `contributed | no_items | disabled | unbound_template | unreachable_cached
    /// | fetch_error | stale_capture`.
    pub status: String,
    /// Machine-readable detail: a URL, an error, or a skip reason.
    pub detail: String,
    /// How many pages the service cost.
    pub pages: usize,
    /// How many items it contributed.
    pub items: usize,
}

impl WalkSourceOutcome {
    /// One outcome row, before the walk charges pages or items against it.
    #[must_use]
    pub fn new(source_id: &str, status: &str, detail: impl Into<String>) -> Self {
        Self {
            source_id: source_id.to_owned(),
            status: status.to_owned(),
            detail: detail.into(),
            pages: 0,
            items: 0,
        }
    }
}

/// Everything one walk produced, with a row for every source that could have
/// contributed.
#[derive(Debug, Clone, PartialEq)]
pub struct WalkOutcome<I> {
    /// The subject the walk was about.
    pub subject: String,
    /// Items in consultation order.
    pub items: Vec<I>,
    /// One row per source, including the ones that produced nothing.
    pub outcomes: Vec<WalkSourceOutcome>,
    /// The bounds the walk was charged against.
    pub bounds: LookupBounds,
}

/// Plan 00 §4.2's contract: resolve a need against the sources registry, in
/// registry order for the need's kind, honoring settings opt-outs and licenses.
pub trait SourceLookup {
    /// Resolve one need, or report honestly which sources were consulted.
    fn lookup(
        &mut self,
        need: &Need,
        bounds: &LookupBounds,
    ) -> crate::concept_lookup::LookupOutcome;
}

/// The subject as a wiki page title: `install docker` becomes `Install-Docker`
/// for wikiHow's hyphenated titles and `Install Docker` elsewhere.
#[must_use]
pub fn page_title(subject: &str, hyphenated: bool) -> String {
    let words: Vec<String> = subject
        .split(is_word_boundary)
        .filter(|word| !word.is_empty())
        .map(capitalize)
        .collect();
    words.join(if hyphenated { "-" } else { " " })
}

/// Where one written word ends, in any script the seed serves.
///
/// Whitespace, and ASCII punctuation. Deliberately *not* `!is_alphanumeric()`:
/// a Devanagari virama and every other combining mark is a non-alphanumeric
/// character in the middle of a word, so that test cut every Hindi word
/// carrying one into two halves and asked Wikipedia for a title with a space
/// in it. A non-ASCII character is part of the word unless it is whitespace.
#[must_use]
pub fn is_word_boundary(character: char) -> bool {
    character.is_whitespace()
        || (character.is_ascii() && !character.is_ascii_alphanumeric() && character != '_')
}

fn capitalize(word: &str) -> String {
    let mut characters = word.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + characters.as_str()
    })
}

/// Bind the registry's API template for this subject, or `None` when a required
/// placeholder cannot be filled from the subject alone (GitHub needs an owner
/// and a repository, for instance).
///
/// One binder for every need kind: the registry's templates name `{title}`,
/// `{query}`, `{lemma}` and `{language}`, and which of them a template uses is
/// the registry's business, not a caller's.
#[must_use]
pub fn entry_url(record: &SourceRecord, subject: &str) -> Option<String> {
    entry_url_in(record, subject, "")
}

/// [`entry_url`], with a language code for the templates that serve one.
///
/// An empty `language` means "whichever language this endpoint leads with":
/// the first code the registry's `api_language` declares. A source that
/// declares no `api_language` has no language to bind, so a `{language}` slot
/// stays literal and the template is reported unbound rather than requested in
/// a language nobody said it serves. A `language` the record does not serve
/// binds nothing at all — that is the honest `unbound_template` outcome, and it
/// is what keeps an English gloss from being handed back for a Hindi question.
#[must_use]
pub fn entry_url_in(record: &SourceRecord, subject: &str, language: &str) -> Option<String> {
    if !language.is_empty() && !record.serves_language(language) {
        return None;
    }
    let language = if language.is_empty() {
        record.api_language.first().map_or("", String::as_str)
    } else {
        language
    };
    let hyphenated = record.host().contains("wikihow");
    let title = page_title(subject, hyphenated);
    let mut bindings: Vec<(&str, &str)> = vec![
        ("title", title.as_str()),
        ("query", subject),
        ("lemma", subject),
    ];
    if !language.is_empty() {
        bindings.push(("language", language));
    }
    let url = record.api_url_in(language, &bindings);
    (!url.contains('{')).then_some(url)
}

/// The sources a need kind may consult, in consultation order: declared kind
/// first, then derived tier descending, then registry order. Total and
/// reproducible.
///
/// Sources the settings opt out of, sources that do not answer this kind, and
/// sources whose API template the subject alone cannot bind never appear — an
/// unbindable service must not consume one of the `max_services` slots a
/// bindable one could have used.
#[must_use]
pub fn select_sources(
    kind: NeedKind,
    subject: &str,
    preferences: &ServicePreferences,
    bounds: &LookupBounds,
) -> Vec<SourceRecord> {
    let mut selected: Vec<SourceRecord> = candidates(kind)
        .into_iter()
        .filter(|record| preferences.allows(record) && entry_url(record, subject).is_some())
        .collect();
    if kind == NeedKind::Procedure {
        // `how_to_role` survives as the ordering hint *inside* the procedure
        // kind, which is what keeps the #991 guide byte-identical. It is not a
        // second axis: for every other kind the registry's declared order is
        // the consultation order, so a dictionary is consulted before an
        // encyclopedia because the file says so and not because Rust does.
        selected.sort_by_key(|record| {
            (
                record.how_to_role,
                u8::MAX - record.tier.weight_percent(),
                record.id.clone(),
            )
        });
    } else {
        selected.sort_by_key(|record| {
            (
                u8::MAX - record.tier.weight_percent(),
                record.registry_index(),
            )
        });
    }
    selected.truncate(bounds.max_services);
    selected
}

/// Every registry source that declares it answers `kind`, before settings and
/// before template binding.
#[must_use]
pub fn candidates(kind: NeedKind) -> Vec<SourceRecord> {
    crate::seed::sources_for_need_kind(kind)
        .into_iter()
        .filter(|record| !record.service_group.is_empty())
        .collect()
}

/// Sources this need kind could have used and did not: the settings opted them
/// out, or the subject cannot bind their template.
///
/// Reported rather than silently dropped, so an absence is attributable to the
/// user's own choice or to the shape of the question.
#[must_use]
pub fn skipped_sources(
    kind: NeedKind,
    subject: &str,
    preferences: &ServicePreferences,
) -> Vec<WalkSourceOutcome> {
    candidates(kind)
        .into_iter()
        .filter_map(|record| {
            if !preferences.allows(&record) {
                Some(WalkSourceOutcome::new(
                    &record.id,
                    "disabled",
                    record.settings_key.clone(),
                ))
            } else if entry_url(&record, subject).is_none() {
                Some(WalkSourceOutcome::new(
                    &record.id,
                    "unbound_template",
                    record.api.clone(),
                ))
            } else {
                None
            }
        })
        .collect()
}

/// Everything a service walk needs besides the service itself.
pub struct Walk<'a, T: SourceTransport> {
    /// The cache-backed client every byte arrives through.
    pub client: &'a CachedSourceClient<T>,
    /// The bounds every capture is charged against.
    pub bounds: &'a LookupBounds,
    /// The wall clock the staleness bound is measured against.
    pub now: u64,
}

/// Walk one service inside the declared bounds, returning its items.
///
/// The queue, the visited set, the page accounting, the accessibility
/// observation, the staleness bound and the failure classification all live
/// here, once. What the bytes *mean* is the extractor's business.
pub fn walk_source<T: SourceTransport, E: CaptureExtractor>(
    record: &SourceRecord,
    entry_url: &str,
    extractor: &E,
    walk: &Walk<'_, T>,
    availability: &mut ServiceAccessibilityCache,
    outcome: &mut WalkSourceOutcome,
) -> Vec<E::Item> {
    let Walk {
        client,
        bounds,
        now,
    } = *walk;
    let mut queue: VecDeque<(String, usize)> = VecDeque::from([(entry_url.to_owned(), 0)]);
    let mut visited: Vec<String> = Vec::new();
    let mut items: Vec<E::Item> = Vec::new();
    while let Some((url, depth)) = queue.pop_front() {
        if outcome.pages >= bounds.max_pages_per_service || visited.contains(&url) {
            continue;
        }
        visited.push(url.clone());
        let capture = match client.fetch(&url) {
            Ok(capture) => capture,
            Err(error) => {
                // Only the service's *declared* entry endpoint speaks for the
                // service. wikiHow answers `action=parse` and 500s on
                // `list=search`; letting the fallback's failure mark the whole
                // service unreachable would blank its working endpoint for the
                // seven-day accessibility TTL.
                observe_failure(
                    record,
                    &url,
                    &error,
                    availability,
                    now,
                    outcome,
                    url == entry_url,
                );
                break;
            }
        };
        outcome.pages += 1;
        availability.observe(
            &endpoint_key(record, &url),
            ServiceStatus::Reachable,
            trace_record::line("captured", &[("url", url.clone())]),
            now,
        );
        let age = now.saturating_sub(capture.fetched_at().parse::<u64>().unwrap_or(now));
        if age > bounds.max_capture_age_seconds {
            outcome.detail = trace_record::line(
                "stale_capture",
                &[("age_seconds", age.to_string()), ("url", url.clone())],
            );
        }
        let read = extractor.read(record, &capture, depth, items.len(), bounds);
        if let Some(detail) = read.detail {
            outcome.detail = detail;
        }
        items.extend(read.items);
        if depth < bounds.max_depth {
            for next in read.follow {
                queue.push_back((next, depth + 1));
            }
        }
    }
    items
}

/// Select the sources this need kind and these settings allow, walk each one
/// inside `bounds`, and return everything the extractor recognised together
/// with an outcome row for every source that could have contributed.
pub fn walk_sources<T: SourceTransport, E: CaptureExtractor>(
    kind: NeedKind,
    subject: &str,
    extractor: &E,
    client: &CachedSourceClient<T>,
    preferences: &ServicePreferences,
    bounds: &LookupBounds,
    availability: &mut ServiceAccessibilityCache,
    now: u64,
) -> WalkOutcome<E::Item> {
    let subject = subject.trim().to_owned();
    let mut walked = WalkOutcome {
        subject: subject.clone(),
        items: Vec::new(),
        outcomes: skipped_sources(kind, &subject, preferences),
        bounds: *bounds,
    };
    for record in select_sources(kind, &subject, preferences, bounds) {
        // `select_sources` drops the templates the subject alone cannot bind
        // and reports them. An extractor may refuse a source for a reason the
        // selector cannot see — a language the endpoint does not serve, above
        // all — and that refusal is recorded rather than swallowed, so the
        // absence of a gloss stays attributable to a named source.
        let Some(url) = extractor.entry_url(&record, &subject) else {
            walked.outcomes.push(WalkSourceOutcome::new(
                &record.id,
                "unbound_template",
                record.api.clone(),
            ));
            continue;
        };
        // The accessibility fact is about the *endpoint*, and it is checked
        // after the endpoint is known for exactly that reason. A source can
        // publish its material through more than one surface (issue #1138,
        // plan 01 L10: the Free Dictionary API for the language Wiktionary's
        // registry entry leads with, each language edition's own MediaWiki API
        // for the rest). Keyed by source id alone, one endpoint answering
        // HTTP 522 blanked the other endpoint of the same source for the whole
        // seven-day TTL, and the language it served was never requested — the
        // same defect as a 404 speaking for a service, one level down.
        let endpoint = endpoint_key(&record, &url);
        if availability.known_unreachable(&endpoint, now) {
            walked.outcomes.push(WalkSourceOutcome::new(
                &record.id,
                "unreachable_cached",
                availability
                    .record(&endpoint)
                    .map_or_else(String::new, |entry| entry.detail.clone()),
            ));
            continue;
        }
        let mut outcome = WalkSourceOutcome::new(&record.id, "no_items", url.clone());
        let items = walk_source(
            &record,
            &url,
            extractor,
            &Walk {
                client,
                bounds,
                now,
            },
            availability,
            &mut outcome,
        );
        if !items.is_empty() {
            outcome.status = String::from("contributed");
        }
        outcome.items = items.len();
        walked.outcomes.push(outcome);
        walked.items.extend(items);
    }
    walked
}

/// The accessibility key of one source *endpoint*: the source id and the host
/// that answered for it.
///
/// A source with one endpoint keys the same string every time, so this changes
/// nothing for the twelve single-endpoint sources. A source that publishes two
/// surfaces gets one record per surface, which is what makes "this endpoint is
/// down" stop meaning "this project is down".
#[must_use]
pub fn endpoint_key(record: &SourceRecord, url: &str) -> String {
    let host = url
        .split_once("://")
        .map_or(url, |(_, rest)| rest)
        .split('/')
        .next()
        .unwrap_or_default();
    if host.is_empty() {
        record.id.clone()
    } else {
        format!("{}@{host}", record.id)
    }
}

/// Classify one fetch failure without letting a fallback endpoint's failure
/// speak for the whole service.
pub fn observe_failure(
    record: &SourceRecord,
    url: &str,
    error: &FetchError,
    availability: &mut ServiceAccessibilityCache,
    now: u64,
    outcome: &mut WalkSourceOutcome,
    is_entry_endpoint: bool,
) {
    let status = if is_entry_endpoint {
        if matches!(error, FetchError::OfflineCacheMiss(_)) {
            // An offline replay without this capture says nothing about whether
            // the service is up, so it must not poison the accessibility record.
            "offline_cache_miss"
        } else if !error.speaks_for_the_service() {
            // The service answered and said it has no such page. That is a fact
            // about this subject in this language, not about the service: one
            // absent Russian article must not blank Wikipedia for every
            // language for the seven-day accessibility TTL.
            "no_entry"
        } else {
            availability.observe(
                &endpoint_key(record, url),
                ServiceStatus::Unreachable,
                error.to_string(),
                now,
            );
            "unreachable"
        }
    } else {
        "fallback_failed"
    };
    outcome.status = String::from(status);
    outcome.detail = trace_record::line(&error.to_string(), &[("url", url.to_owned())]);
}

/// The registry sources of the `external_trusted` group, in registry order.
///
/// Kept here so a caller that wants the group rather than a need kind does not
/// reach past this module into the seed reader.
#[must_use]
pub fn trusted_sources() -> Vec<SourceRecord> {
    external_trusted_sources()
}
