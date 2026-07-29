//! Recursive source gathering, driven by what is still unmet.
//!
//! Issue #844's third requirement: gathering must recurse — a question's sources
//! link to further sources — and it must be *driven by the difference between
//! what is required and what is established*, which is exactly the loop
//! [`crate::option_network`] already implements. So the loop here owns no
//! turn script: it fetches, records which required attributes each document
//! supplies, and asks the network whether anything is still open.
//!
//! Termination is doubly bounded, because either bound alone is unsafe:
//!
//! - **Fixpoint.** A round whose frontier holds no unseen source ends the loop
//!   with `converged = true`. This is what stops a citation cycle
//!   (A links to B links to A) — a URL is fetched at most once per run.
//! - **Depth.** `max_depth` caps how far from the seeds the loop will walk, so
//!   an infinitely deep chain of new URLs still terminates, with
//!   `stopped_at_depth_bound = true` recording *why* it stopped.
//!
//! [`SourceProvider`] and [`SourceCache`] remain the small deterministic seam
//! used by unit fixtures. Production callers use
//! [`execute_captured_gathering`], which runs the same traversal over
//! [`crate::source_fetch::CachedSourceClient`]. That path retains every exact
//! [`crate::source_fetch::SourceCapture`], uses its SHA-256 in the traversal
//! receipt, reports failures as diagnostics rather than evidence, and renders a
//! replayable, review-gated learning proposal.

use super::dedup::SourcedStatement;
use crate::engine::stable_id;
use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::option_network::{Candidate, Constraint, OptionNetwork, Supply, Tier};
use crate::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};

/// The two line kinds of [`GatheringReport::trace`]: one fetch record per
/// document, then the single termination record.
const FETCH_RECORD: &str = "fetch";
const STOP_RECORD: &str = "stop";
use crate::relative_meta_logic::SourceTier;
use std::collections::BTreeMap;

/// A document as returned by a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedSource {
    /// Where it came from — the cache index key and the statement's source label.
    pub url: String,
    /// How much the document's origin is trusted.
    pub tier: SourceTier,
    /// Its text, formalized into statements by [`super::formalize`].
    pub text: String,
    /// Which of the question's required attributes this document supplies.
    pub supplies: Vec<String>,
    /// Further sources it points at — the next round's frontier.
    pub links: Vec<String>,
}

impl FetchedSource {
    /// Build a document from its parts.
    #[must_use]
    pub fn new(url: impl Into<String>, tier: SourceTier, text: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            tier,
            text: text.into(),
            supplies: Vec::new(),
            links: Vec::new(),
        }
    }

    /// Declare that this document answers `attribute`.
    #[must_use]
    pub fn supplying(mut self, attribute: impl Into<String>) -> Self {
        self.supplies.push(attribute.into());
        self
    }

    /// Declare a further source this document links to.
    #[must_use]
    pub fn linking(mut self, url: impl Into<String>) -> Self {
        self.links.push(url.into());
        self
    }

    /// The content address of this document's text.
    #[must_use]
    pub fn digest(&self) -> String {
        stable_id("source", &self.text)
    }
}

/// Where documents come from. Implementations must be deterministic: the same
/// URL yields the same document, or [`None`] when it cannot be retrieved.
pub trait SourceProvider {
    /// Retrieve the document at `url`.
    fn fetch(&mut self, url: &str) -> Option<FetchedSource>;
}

/// Facts derived by a caller from one exact source capture.
///
/// Retrieval owns bytes and provenance; classification owns the trust tier,
/// requirements supplied, and outgoing links. Keeping those responsibilities
/// separate prevents a URL or search result from becoming evidence before its
/// response bytes exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedSourceMetadata {
    /// How much the captured document's origin is trusted.
    pub tier: SourceTier,
    /// Text extracted from the captured bytes and suitable for formalization.
    ///
    /// Keeping extraction in the classifier lets production adapters remove
    /// HTML or decode another text format while the callback still has the
    /// exact [`SourceCapture`] in hand.
    pub text: String,
    /// Required attributes established by the captured bytes.
    pub supplies: Vec<String>,
    /// Further source URLs extracted from the captured bytes.
    pub links: Vec<String>,
}

impl CapturedSourceMetadata {
    /// Start a classification with text derived from the exact capture.
    #[must_use]
    pub fn new(tier: SourceTier, text: impl Into<String>) -> Self {
        Self {
            tier,
            text: text.into(),
            supplies: Vec::new(),
            links: Vec::new(),
        }
    }

    /// Record an attribute established by the capture.
    #[must_use]
    pub fn supplying(mut self, attribute: impl Into<String>) -> Self {
        self.supplies.push(attribute.into());
        self
    }

    /// Record a source URL extracted from the capture.
    #[must_use]
    pub fn linking(mut self, url: impl Into<String>) -> Self {
        self.links.push(url.into());
        self
    }

    fn normalized(mut self) -> Self {
        self.supplies.sort();
        self.supplies.dedup();
        self.links.sort();
        self.links.dedup();
        self
    }
}

/// One successfully captured and classified document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedSourceObservation {
    /// Exact URL, retrieval time, SHA-256, cache state, and response bytes.
    pub capture: SourceCapture,
    /// Classification derived from those exact bytes.
    pub metadata: CapturedSourceMetadata,
    /// Link distance from the nearest seed.
    pub depth: usize,
}

/// A URL the production traversal could not capture.
///
/// Failures remain outside `sources` and `observations`, so they can be logged
/// and learned from without masquerading as source evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedGatheringFailure {
    /// URL whose capture failed.
    pub url: String,
    /// Link distance from the nearest seed.
    pub depth: usize,
    /// Exact capture-layer diagnostic.
    pub error: FetchError,
}

/// What the cache remembers about one URL: its provenance, and the content
/// address of the body it served.
///
/// Provenance is per-URL and the body is shared, because two URLs can serve the
/// same bytes at *different* trust tiers — an unoriginal repost of a first-party
/// announcement is the ordinary case. Storing the tier, the supplied attributes
/// and the outgoing links per URL is what keeps the shared body from
/// impersonating whichever URL happened to be fetched first.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CacheEntry {
    digest: String,
    tier: SourceTier,
    supplies: Vec<String>,
    links: Vec<String>,
}

/// A content-addressed store of fetched documents.
///
/// Two maps, both ordered so iteration is deterministic: URLs index onto their
/// provenance plus a digest, digests hold bodies. Mirrors and reposts therefore
/// cost one body, however many URLs serve it.
#[derive(Debug, Clone, Default)]
pub struct SourceCache {
    index: BTreeMap<String, CacheEntry>,
    bodies: BTreeMap<String, String>,
}

impl SourceCache {
    /// An empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The cached document for `url`, if it was fetched before.
    ///
    /// Reconstructed from this URL's own provenance and the shared body, so a
    /// cache hit is indistinguishable from the original fetch.
    #[must_use]
    pub fn get(&self, url: &str) -> Option<FetchedSource> {
        let entry = self.index.get(url)?;
        let text = self.bodies.get(&entry.digest)?;
        Some(FetchedSource {
            url: url.to_string(),
            tier: entry.tier,
            text: text.clone(),
            supplies: entry.supplies.clone(),
            links: entry.links.clone(),
        })
    }

    /// Store `source`: its body under its content address, its provenance under
    /// its URL.
    pub fn put(&mut self, source: FetchedSource) {
        let digest = source.digest();
        self.bodies.entry(digest.clone()).or_insert(source.text);
        self.index.insert(
            source.url,
            CacheEntry {
                digest,
                tier: source.tier,
                supplies: source.supplies,
                links: source.links,
            },
        );
    }

    /// How many URLs the cache knows.
    #[must_use]
    pub fn url_count(&self) -> usize {
        self.index.len()
    }

    /// How many distinct bodies it stores. Lower than [`Self::url_count`] when
    /// sources mirror each other.
    #[must_use]
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
}

/// What to gather, and how far to go.
#[derive(Debug, Clone)]
pub struct GatheringPlan {
    /// The subject of the question, used as the option network's subject.
    pub subject: String,
    /// The attributes an answer must cover.
    pub required: Vec<String>,
    /// Where to start.
    pub seeds: Vec<String>,
    /// How many link hops past the seeds to follow. `0` fetches seeds only.
    pub max_depth: usize,
}

impl GatheringPlan {
    /// A plan with no requirements and no seeds.
    #[must_use]
    pub fn new(subject: impl Into<String>, max_depth: usize) -> Self {
        Self {
            subject: subject.into(),
            required: Vec::new(),
            seeds: Vec::new(),
            max_depth,
        }
    }

    /// Require that the gathered sources cover `attribute`.
    #[must_use]
    pub fn requiring(mut self, attribute: impl Into<String>) -> Self {
        self.required.push(attribute.into());
        self
    }

    /// Start from `url`.
    #[must_use]
    pub fn seeded_with(mut self, url: impl Into<String>) -> Self {
        self.seeds.push(url.into());
        self
    }
}

/// One fetch, in the order it happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRecord {
    /// The URL fetched.
    pub url: String,
    /// Link hops from the nearest seed.
    pub depth: usize,
    /// Content address of the body.
    pub digest: String,
    /// `true` when the body came from the cache and the provider was not called.
    pub from_cache: bool,
    /// Required attributes this document supplied.
    pub supplies: Vec<String>,
}

/// The outcome of one gathering run.
#[derive(Debug, Clone)]
pub struct GatheringReport {
    /// Every fetch, in order.
    pub fetches: Vec<FetchRecord>,
    /// Every statement formalized out of the fetched documents, tagged with its
    /// source — the input to [`super::dedup::deduplicate`].
    pub observations: Vec<SourcedStatement>,
    /// The deepest link hop reached.
    pub depth_reached: usize,
    /// `true` when the loop ran out of unseen sources (a fixpoint) rather than
    /// hitting a bound.
    pub converged: bool,
    /// `true` when the depth bound stopped the loop while unseen sources
    /// remained.
    pub stopped_at_depth_bound: bool,
    /// Attributes still unsupplied when the loop stopped.
    pub open_attributes: Vec<String>,
    /// How many fetches were served from the cache.
    pub cache_hits: usize,
}

impl GatheringReport {
    /// Did the gathered sources cover every required attribute?
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.open_attributes.is_empty()
    }

    /// A deterministic, byte-comparable render of the run: one line per fetch
    /// plus the termination verdict. Cache state is deliberately omitted, so a
    /// warm replay of the same captures produces byte-identical text.
    ///
    /// Every line is a keyword followed by `name=value` fields, so the trace is
    /// a machine record with no natural language to translate (R379).
    #[must_use]
    pub fn trace(&self) -> String {
        let mut lines = Vec::with_capacity(self.fetches.len() + 1);
        for record in &self.fetches {
            lines.push(
                [
                    FETCH_RECORD.to_owned(),
                    format!("url={}", record.url),
                    format!("depth={}", record.depth),
                    format!("digest={}", record.digest),
                    format!("supplies=[{}]", record.supplies.join(",")),
                ]
                .join(" "),
            );
        }
        lines.push(
            [
                STOP_RECORD.to_owned(),
                format!("depth={}", self.depth_reached),
                format!("converged={}", self.converged),
                format!("depth_bound={}", self.stopped_at_depth_bound),
                format!("open=[{}]", self.open_attributes.join(",")),
            ]
            .join(" "),
        );
        lines.join("\n")
    }
}

/// Production gathering result over the exact-capture boundary.
#[derive(Debug, Clone)]
pub struct CapturedGatheringReport {
    /// Subject used by the unmet-difference option network.
    pub subject: String,
    /// Required attributes, normalized for deterministic replay.
    pub required: Vec<String>,
    /// The shared recursive traversal result.
    pub report: GatheringReport,
    /// Every successfully captured source, in traversal order.
    pub sources: Vec<CapturedSourceObservation>,
    /// Capture failures, retained only as diagnostics.
    pub failures: Vec<CapturedGatheringFailure>,
}

impl CapturedGatheringReport {
    /// Append truthful source and failure events to the common event log.
    pub fn record(&self, log: &mut EventLog) {
        for source in &self.sources {
            source.capture.record(log);
        }
        for failure in &self.failures {
            log.append(
                "error:fetch",
                format!(
                    "url={} depth={} error={}",
                    failure.url, failure.depth, failure.error
                ),
            );
        }
    }

    /// Deterministic, proposal-only auto-learning projection.
    ///
    /// Cache-hit state is omitted so a live execution and its offline replay
    /// produce identical proposals. Promotion into durable memory remains a
    /// later human-gated operation.
    #[must_use]
    pub fn learning_proposal(&self) -> String {
        let mut records = vec![format_lino_record(
            "multi_source_gathering",
            &[
                ("subject", self.subject.clone()),
                ("required", self.required.join("|")),
                ("captured_sources", self.sources.len().to_string()),
                ("failures", self.failures.len().to_string()),
                ("converged", self.report.converged.to_string()),
                (
                    "stopped_at_depth_bound",
                    self.report.stopped_at_depth_bound.to_string(),
                ),
                ("open", self.report.open_attributes.join("|")),
            ],
        )];
        for source in &self.sources {
            records.push(format_lino_record(
                "source_observation",
                &[
                    ("url", source.capture.source_url().to_owned()),
                    ("fetched_at", source.capture.fetched_at().to_owned()),
                    ("sha256", source.capture.sha256().to_owned()),
                    ("depth", source.depth.to_string()),
                ],
            ));
            records.push(format_lino_record(
                "source_classification",
                &[
                    ("url", source.capture.source_url().to_owned()),
                    ("tier", source.metadata.tier.slug().to_owned()),
                    ("supplies", source.metadata.supplies.join("|")),
                    ("links", source.metadata.links.join("|")),
                ],
            ));
        }
        for failure in &self.failures {
            records.push(format_lino_record(
                "source_failure",
                &[
                    ("url", failure.url.clone()),
                    ("depth", failure.depth.to_string()),
                    ("diagnostic", failure.error.to_string()),
                ],
            ));
        }
        records.join("\n")
    }
}

/// Map a source's trust tier onto the option network's provenance ladder, so the
/// gathering loop and the probability calculation rank origins the same way.
#[must_use]
pub const fn research_tier(tier: SourceTier) -> Tier {
    match tier {
        SourceTier::OriginalFirstParty => Tier::Authentic,
        SourceTier::OriginalJournalism => Tier::OfficialCompatible,
        SourceTier::IndependentCorroboration | SourceTier::Unoriginal => Tier::GenericCompatible,
    }
}

struct LoadedSource {
    document: FetchedSource,
    digest: String,
    from_cache: bool,
}

/// Gather sources for `plan`, recursing through their links.
///
/// Breadth-first from the seeds. Each round: fetch the frontier (cache first),
/// formalize each document into observations, tell the option network which
/// required attributes the document supplied, then ask the network whether
/// anything is still unmet. The loop ends when the question is closed, when a
/// round has nothing unseen left to fetch (fixpoint), or when the next round
/// would pass `max_depth`.
pub fn gather(
    plan: &GatheringPlan,
    provider: &mut dyn SourceProvider,
    cache: &mut SourceCache,
) -> GatheringReport {
    run_gathering(plan, |url, _depth| {
        let cached = cache.get(url);
        let from_cache = cached.is_some();
        let document = cached.or_else(|| provider.fetch(url))?;
        if !from_cache {
            cache.put(document.clone());
        }
        Some(LoadedSource {
            digest: document.digest(),
            document,
            from_cache,
        })
    })
}

/// Execute recursive gathering through the real, replayable capture client.
///
/// `classify` sees the exact capture and is the only route by which trust,
/// supplied attributes, or outgoing links enter the traversal. A failed capture
/// is retained in [`CapturedGatheringReport::failures`] and contributes no
/// statement or source evidence.
pub fn execute_captured_gathering<T, C>(
    plan: &GatheringPlan,
    client: &CachedSourceClient<T>,
    classify: C,
) -> CapturedGatheringReport
where
    T: SourceTransport,
    C: Fn(&SourceCapture) -> CapturedSourceMetadata,
{
    let mut sources = Vec::new();
    let mut failures = Vec::new();
    let report = run_gathering(plan, |url, depth| match client.fetch(url) {
        Ok(capture) => {
            let metadata = classify(&capture).normalized();
            let document = FetchedSource {
                url: capture.source_url().to_owned(),
                tier: metadata.tier,
                text: metadata.text.clone(),
                supplies: metadata.supplies.clone(),
                links: metadata.links.clone(),
            };
            let loaded = LoadedSource {
                digest: capture.sha256().to_owned(),
                from_cache: capture.cached(),
                document,
            };
            sources.push(CapturedSourceObservation {
                capture,
                metadata,
                depth,
            });
            Some(loaded)
        }
        Err(error) => {
            failures.push(CapturedGatheringFailure {
                url: url.to_owned(),
                depth,
                error,
            });
            None
        }
    });
    let mut required = plan.required.clone();
    required.sort();
    required.dedup();
    CapturedGatheringReport {
        subject: plan.subject.clone(),
        required,
        report,
        sources,
        failures,
    }
}

fn run_gathering(
    plan: &GatheringPlan,
    mut fetch: impl FnMut(&str, usize) -> Option<LoadedSource>,
) -> GatheringReport {
    let mut network = OptionNetwork::new(plan.subject.clone());
    for attribute in &plan.required {
        network.require(Constraint::nominal(attribute.clone(), "known"));
    }

    let mut seen: Vec<String> = Vec::new();
    let mut frontier: Vec<String> = dedupe_urls(&plan.seeds, &seen);
    let mut fetches: Vec<FetchRecord> = Vec::new();
    let mut observations: Vec<SourcedStatement> = Vec::new();
    let mut cache_hits = 0;
    let mut depth = 0;
    let mut depth_reached = 0;
    let mut converged = false;
    let mut stopped_at_depth_bound = false;

    loop {
        if frontier.is_empty() {
            // Nothing unseen left to ask for: the recursion reached its
            // fixpoint. This is the citation-cycle exit.
            converged = true;
            break;
        }
        if depth > plan.max_depth {
            stopped_at_depth_bound = true;
            break;
        }
        depth_reached = depth;
        let mut next: Vec<String> = Vec::new();
        for url in &frontier {
            seen.push(url.clone());
            let Some(loaded) = fetch(url, depth) else {
                // An unreachable source is recorded by its absence: it is
                // `seen`, so the loop never retries it, and it supplies nothing.
                continue;
            };
            let LoadedSource {
                document,
                digest,
                from_cache,
            } = loaded;
            if from_cache {
                cache_hits += 1;
            }
            let supplied: Vec<String> = plan
                .required
                .iter()
                .filter(|attribute| document.supplies.contains(attribute))
                .cloned()
                .collect();
            let mut candidate = Candidate::new(document.url.clone(), research_tier(document.tier));
            for attribute in &supplied {
                candidate = candidate.supplying(attribute.clone(), Supply::nominal("known"));
            }
            network.observe(candidate);
            for statement in super::formalize(&document.text) {
                observations.push(SourcedStatement::new(
                    statement,
                    document.url.clone(),
                    document.tier,
                ));
            }
            fetches.push(FetchRecord {
                url: document.url.clone(),
                depth,
                digest,
                from_cache,
                supplies: supplied,
            });
            next.extend(document.links.iter().cloned());
        }
        if network.is_closed() && !plan.required.is_empty() {
            // The unmet difference is empty: every required attribute has a
            // source. Recursing further would add sources the question does not
            // need.
            converged = true;
            break;
        }
        frontier = dedupe_urls(&next, &seen);
        depth += 1;
    }

    GatheringReport {
        fetches,
        observations,
        depth_reached,
        converged,
        stopped_at_depth_bound,
        open_attributes: network.open_attributes(),
        cache_hits,
    }
}

/// The unseen members of `urls`, de-duplicated, in first-seen order.
fn dedupe_urls(urls: &[String], seen: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for url in urls {
        if url.trim().is_empty() || seen.contains(url) || out.contains(url) {
            continue;
        }
        out.push(url.clone());
    }
    out
}
