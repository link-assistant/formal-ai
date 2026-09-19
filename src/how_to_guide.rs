//! Bounded multi-source synthesis of a "how to X" guide.
//!
//! Issue #991 asks for one contract that both production runtimes execute:
//! pick the enabled, relevant services out of `data/seed/sources-registry.lino`,
//! capture their pages recursively inside declared depth/page/time bounds, keep
//! exact provenance on every accepted step, and resolve copies and
//! contradictions with the issue #709 source-tier policy. This module owns that
//! contract for the Rust side; `src/web/worker/formal_ai_worker_how_to_guide.js` mirrors
//! it on the browser side. The two are held to one answer by replaying the same
//! committed captures through both: `examples/issue_991_how_to_parity.rs` writes
//! `tests/fixtures/issue-991/expected-guides.json` from this path, and both
//! `tests/unit/issue_991_how_to_synthesis.rs` and
//! `tests/web/issue-991-how-to-synthesis.test.mjs` assert against it.
//!
//! Nothing here reaches the network on its own: every byte arrives through
//! [`CachedSourceClient`], so an offline run replays committed captures and a
//! live run refreshes them under the same code path.

pub mod extract;
mod render;

use std::collections::BTreeMap;

use crate::event_log::EventLog;
use crate::needs::NeedKind;
use crate::relative_meta_logic::SourceTier;
use crate::seed::{SourceRecord, percent_encode};
use crate::service_accessibility::ServiceAccessibilityCache;
use crate::source_fetch::{CachedSourceClient, SourceCapture, SourceTransport};
use crate::source_walk::{
    CaptureExtractor, Extracted, LookupBounds, Walk, WalkSourceOutcome, walk_source,
};
use crate::trace_record;

use extract::{Payload, classify, extract_steps, wiki_link_titles};

/// A guide with fewer accepted steps than this is not a procedure; the caller
/// must report insufficient evidence instead of pretending to answer.
pub const MIN_ACCEPTED_STEPS: usize = 2;

/// Declared retrieval bounds.
///
/// Issue #1138 B1, plan 01 L2: this was a second bounds vocabulary, identical
/// in meaning to the one a concept lookup needs and impossible for the compiler
/// to keep consistent with it. It is now an alias of
/// [`crate::source_walk::LookupBounds`], so there is one set of declared bounds
/// in the tree and `max_steps` is spelled `max_items` — the same number, named
/// for what it bounds rather than for the one need kind that first bounded it.
pub use crate::source_walk::LookupBounds as GuideBounds;

/// The user's service opt-outs, read from settings.
///
/// Settings are authoritative in both directions: an explicit `false` silences
/// a service the registry enables by default, and an explicit `true` enables
/// one the registry leaves off. Absence means "registry default".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServicePreferences {
    settings: BTreeMap<String, bool>,
}

impl ServicePreferences {
    /// Preferences from `(settings_key, enabled)` pairs as the UI stores them.
    #[must_use]
    pub fn from_pairs<K: AsRef<str>>(pairs: &[(K, bool)]) -> Self {
        let mut settings = BTreeMap::new();
        for (key, enabled) in pairs {
            settings.insert(key.as_ref().to_owned(), *enabled);
        }
        Self { settings }
    }

    /// Record one explicit setting.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, enabled: bool) -> Self {
        self.settings.insert(key.into(), enabled);
        self
    }

    /// Whether `record` may be consulted.
    #[must_use]
    pub fn allows(&self, record: &SourceRecord) -> bool {
        self.settings
            .get(&record.settings_key)
            .copied()
            .unwrap_or(record.default_enabled)
    }
}

/// One accepted step and the exact bytes it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuideStep {
    /// The instruction text, compacted from the captured markup.
    pub text: String,
    /// Registry id of the contributing source.
    pub source_id: String,
    /// Human label of the contributing source.
    pub source_name: String,
    /// The exact URL whose bytes carry this step.
    pub source_url: String,
    /// sha256 of those bytes.
    pub sha256: String,
    /// When the bytes were retrieved (unix seconds, as the cache stores them).
    pub fetched_at: String,
    /// Whether the bytes came from the capture cache rather than the network.
    pub cached: bool,
    /// The #709 tier the bytes carry after the copied-source policy.
    pub tier: SourceTier,
    /// License the bytes are quoted under.
    pub license_name: String,
    /// Canonical URL of that license.
    pub license_url: String,
    /// How many link hops past the service's entry request produced the bytes.
    pub depth: usize,
    /// Position of the step within its own source, starting at 1.
    pub position: usize,
}

impl GuideStep {
    /// Project a synthesised guide step into the shared ordered-step record.
    ///
    /// This is the same shape capture extraction emits, so procedure
    /// formalization has one constructor regardless of where the ordered text
    /// came from.
    #[must_use]
    pub fn to_step_record(&self) -> crate::procedure_text::ProcedureStepRecord {
        crate::procedure_text::ProcedureStepRecord {
            ordinal: self.position,
            text: self.text.clone(),
            source_id: self.source_id.clone(),
            source_url: self.source_url.clone(),
            sha256: self.sha256.clone(),
            fetched_at: self.fetched_at.clone(),
            license_name: self.license_name.clone(),
            license_url: self.license_url.clone(),
            depth: self.depth,
        }
    }

    /// Exact provenance for one step, in the order a reviewer checks it.
    #[must_use]
    pub fn provenance(&self) -> String {
        trace_record::payload(&[
            ("source", self.source_id.clone()),
            ("url", self.source_url.clone()),
            ("sha256", self.sha256.clone()),
            ("fetched_at", self.fetched_at.clone()),
            ("cached", self.cached.to_string()),
            ("tier", self.tier.slug().to_owned()),
            ("license", self.license_name.clone()),
            ("depth", self.depth.to_string()),
            ("position", self.position.to_string()),
        ])
    }
}

/// What consulting one service produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuideSourceOutcome {
    /// Registry id of the service.
    pub source_id: String,
    /// Stable status slug (`contributed`, `no_steps`, `disabled`, …).
    pub status: String,
    /// Machine-readable detail: a URL, an error, or a skip reason.
    pub detail: String,
    /// How many pages the service cost.
    pub pages: usize,
    /// How many steps it contributed after policy.
    pub steps: usize,
}

impl GuideSourceOutcome {
    /// The kernel's outcome row, read as a guide's. One walk, one vocabulary:
    /// `items` is `steps` for a procedure need (issue #1138, plan 01 L2).
    fn from_walk(outcome: WalkSourceOutcome) -> Self {
        Self {
            source_id: outcome.source_id,
            status: outcome.status,
            detail: outcome.detail,
            pages: outcome.pages,
            steps: outcome.items,
        }
    }

    fn new(source_id: &str, status: &str, detail: impl Into<String>) -> Self {
        Self {
            source_id: source_id.to_owned(),
            status: status.to_owned(),
            detail: detail.into(),
            pages: 0,
            steps: 0,
        }
    }

    /// Stable one-line payload for the trace.
    #[must_use]
    pub fn trace_payload(&self) -> String {
        trace_record::payload(&[
            ("source", self.source_id.clone()),
            ("status", self.status.clone()),
            ("pages", self.pages.to_string()),
            ("steps", self.steps.to_string()),
            ("detail", self.detail.clone()),
        ])
    }
}

/// A contradiction between two sources about the same step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuideConflict {
    /// The shared action the two sources describe differently.
    pub action: String,
    /// The source whose higher tier won the step.
    pub kept_source: String,
    /// The source whose step was dropped.
    pub dropped_source: String,
    /// The dropped text, kept so the disagreement stays auditable.
    pub dropped_text: String,
}

/// The synthesised guide plus everything needed to audit it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HowToGuide {
    /// The task the guide answers.
    pub task: String,
    /// Accepted steps in presentation order.
    pub steps: Vec<GuideStep>,
    /// Every service considered, in consultation order.
    pub outcomes: Vec<GuideSourceOutcome>,
    /// Contradictions resolved by tier.
    pub conflicts: Vec<GuideConflict>,
    /// URLs dropped because their bytes duplicate a higher-tier capture.
    pub copies: Vec<String>,
    /// The bounds this run was held to.
    pub bounds: GuideBounds,
}

impl HowToGuide {
    /// Whether the run found enough corroborated procedure to answer with.
    #[must_use]
    pub const fn is_sufficient(&self) -> bool {
        self.steps.len() >= MIN_ACCEPTED_STEPS
    }

    /// Distinct services that contributed at least one accepted step.
    #[must_use]
    pub fn contributing_sources(&self) -> Vec<String> {
        let mut sources: Vec<String> = Vec::new();
        for step in &self.steps {
            if !sources.contains(&step.source_id) {
                sources.push(step.source_id.clone());
            }
        }
        sources
    }

    /// Deterministic trace, identical for a live capture and its cache replay.
    #[must_use]
    pub fn trace(&self) -> String {
        render::trace(self)
    }

    /// Append every retrieval and policy decision as evidence.
    pub fn record(&self, log: &mut EventLog) {
        render::record(self, log);
    }

    /// The guide rendered for a reader, provenance included.
    #[must_use]
    pub fn markdown(&self) -> String {
        render::markdown(self)
    }

    /// The guide rendered for a reader of `language`.
    ///
    /// The evidence is language-neutral, so only the seeded chrome changes: the
    /// same steps, sources, digests, and bounds are reported either way.
    #[must_use]
    pub fn markdown_in(&self, language: &str) -> String {
        render::markdown_in(self, language)
    }
}

/// The registry sources that may contribute to `task`, in consultation order.
///
/// One line, because the selection is the kernel's (issue #1138 plan 01 L2-L3).
/// A procedure need and a concept need are selected by the same function over
/// the same registry field, so the two can no longer drift; what survives here
/// is the name the how-to callers already use.
#[must_use]
pub fn select_sources(
    task: &str,
    preferences: &ServicePreferences,
    bounds: &GuideBounds,
) -> Vec<SourceRecord> {
    crate::source_walk::select_sources(NeedKind::Procedure, task, preferences, bounds)
}

/// Synthesise a guide for `task` from the enabled registry services.
///
/// `availability` is consulted before a service is contacted and updated after
/// every attempt, so a service known to be down stays skipped for the whole
/// seven-day accessibility TTL instead of costing a request per question.
pub fn synthesize_how_to_guide<T: SourceTransport>(
    task: &str,
    client: &CachedSourceClient<T>,
    preferences: &ServicePreferences,
    bounds: &GuideBounds,
    availability: &mut ServiceAccessibilityCache,
    now: u64,
) -> HowToGuide {
    let mut guide = HowToGuide {
        task: task.trim().to_owned(),
        steps: Vec::new(),
        outcomes: Vec::new(),
        conflicts: Vec::new(),
        copies: Vec::new(),
        bounds: *bounds,
    };
    for record in skipped_sources(&guide.task, preferences) {
        guide.outcomes.push(record);
    }
    let mut collected: Vec<GuideStep> = Vec::new();
    let extractor = StepExtractor { task: &guide.task };
    for record in select_sources(&guide.task, preferences, bounds) {
        if availability.known_unreachable(&record.id, now) {
            guide.outcomes.push(GuideSourceOutcome::new(
                &record.id,
                "unreachable_cached",
                availability
                    .record(&record.id)
                    .map_or_else(String::new, |entry| entry.detail.clone()),
            ));
            continue;
        }
        // `select_sources` already dropped every template this task cannot
        // bind, and reported it; the fallback keeps the walk total.
        let Some(url) = entry_url(&record, &guide.task) else {
            continue;
        };
        let mut outcome = WalkSourceOutcome::new(&record.id, "no_steps", url.clone());
        let steps = walk_source(
            &record,
            &url,
            &extractor,
            &Walk {
                client,
                bounds,
                now,
            },
            availability,
            &mut outcome,
        );
        if !steps.is_empty() {
            outcome.status = String::from("contributed");
        }
        outcome.items = steps.len();
        guide.outcomes.push(GuideSourceOutcome::from_walk(outcome));
        collected.extend(steps);
    }
    let collected = apply_copied_source_policy(collected, &mut guide);
    let collected = apply_conflict_policy(collected, &mut guide);
    guide.steps = order_steps(collected, bounds.max_items);
    guide
}

/// Services the settings opted out of, reported so the trace shows the user's
/// choice rather than an unexplained absence.
fn skipped_sources(task: &str, preferences: &ServicePreferences) -> Vec<GuideSourceOutcome> {
    crate::source_walk::skipped_sources(NeedKind::Procedure, task, preferences)
        .into_iter()
        .map(GuideSourceOutcome::from_walk)
        .collect()
}

/// Bind the registry's API template for this task, or `None` when a required
/// placeholder cannot be filled from the task alone.
///
/// The binder is the kernel's: a template with a `{title}` slot binds the same
/// way whatever need kind is asking.
fn entry_url(record: &SourceRecord, task: &str) -> Option<String> {
    crate::source_walk::entry_url(record, task)
}

/// The task as a wiki page title: `install docker` becomes `Install-Docker` for
/// wikiHow's hyphenated titles and `Install Docker` elsewhere.
#[must_use]
pub fn page_title(task: &str, hyphenated: bool) -> String {
    crate::source_walk::page_title(task, hyphenated)
}

/// A `MediaWiki` full-text search URL on the same wiki as `record`'s endpoint.
///
/// The registry's entry template addresses a page by title, which only works
/// when the task names the page exactly. Full-text search is the recursive step
/// that turns a task into the titles worth parsing.
fn search_url(record: &SourceRecord, task: &str) -> String {
    let base = record.api.split('?').next().unwrap_or(&record.api);
    format!(
        "{base}?action=query&list=search&srsearch={}&srlimit=5&format=json&origin=*",
        percent_encode(task)
    )
}

/// The Stack Exchange answers of one question, best-voted first.
fn answers_url(record: &SourceRecord, question_id: u64) -> String {
    let base = record.api.split("/search").next().unwrap_or(&record.api);
    let site = record
        .api
        .split_once("site=")
        .map_or("stackoverflow", |(_, rest)| {
            rest.split('&').next().unwrap_or("stackoverflow")
        });
    format!(
        "{base}/questions/{question_id}/answers?order=desc&sort=votes&site={site}&filter=withbody"
    )
}

/// A `MediaWiki` `action=parse` URL on the same wiki as `record`'s endpoint.
fn parse_url(record: &SourceRecord, title: &str) -> String {
    let base = record.api.split('?').next().unwrap_or(&record.api);
    format!(
        "{base}?action=parse&page={}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*",
        percent_encode(title)
    )
}

/// Words that carry no topic, so requiring a candidate to repeat them would
/// reject correct pages ("Reverse a string in Python" must match "reverse a
/// string in python" without depending on the article).
const TOPIC_STOPWORDS: &[&str] = &[
    "the", "and", "for", "with", "your", "you", "how", "does", "did", "are", "was", "were", "its",
    "into", "onto", "from", "that", "this",
];

/// Whether `candidate` is about `task`.
///
/// A search endpoint answers with its *best* matches, not with matching pages:
/// Wikibooks offers "Programming Fundamentals/Academic or Scholastic Dishonesty"
/// for "make pancakes" and Stack Overflow offers an ORM question that merely
/// says "pancakes". Following those produces confidently-sourced nonsense, so a
/// candidate contributes only when it repeats *every* topic word of the task.
fn matches_task(task: &str, candidate: &str) -> bool {
    let wanted = topic_words(task);
    if wanted.is_empty() {
        return true;
    }
    let offered = topic_words(candidate);
    wanted.iter().all(|word| offered.contains(word))
}

/// The topic words of a phrase: lowercased, de-punctuated, de-pluralised, with
/// stopwords and one/two-letter fragments dropped.
fn topic_words(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|word| word.len() > 2 && !TOPIC_STOPWORDS.contains(&word.as_str()))
        .map(|word| singular(&word))
        .collect()
}

fn singular(word: &str) -> String {
    match word.strip_suffix('s') {
        Some(stem) if stem.len() > 2 && !word.ends_with("ss") => stem.to_owned(),
        _ => word.to_owned(),
    }
}

/// The `Procedure` kind's reading of a capture (issue #1138, plan 01 L2).
///
/// Everything that used to be `capture_service` and is *not* specific to a
/// procedure — the queue, the visited set, the page accounting, the
/// accessibility observation, the staleness bound, the failure classification —
/// now lives once in [`crate::source_walk::walk_source`]. What is left here is
/// the part that only a procedure has: which payload shapes carry steps, and
/// which link on a page is worth one more page.
struct StepExtractor<'a> {
    task: &'a str,
}

impl CaptureExtractor for StepExtractor<'_> {
    type Item = GuideStep;

    fn entry_url(&self, record: &SourceRecord, subject: &str) -> Option<String> {
        entry_url(record, subject)
    }

    fn read(
        &self,
        record: &SourceRecord,
        capture: &SourceCapture,
        depth: usize,
        produced: usize,
        bounds: &LookupBounds,
    ) -> Extracted<Self::Item> {
        let task = self.task;
        let url = capture.source_url().to_owned();
        let is_wiki = record.api.contains("api.php");
        let mut read = Extracted::default();
        match classify(capture.bytes()) {
            Payload::Parse { html, .. } => {
                let found = extract_steps(&html, bounds.max_items);
                if found.is_empty() {
                    for title in wiki_link_titles(&html, bounds.max_pages_per_service) {
                        if matches_task(task, &title) {
                            read.follow.push(parse_url(record, &title));
                        }
                    }
                }
                push_steps(record, capture, depth, &found, &mut read.items);
            }
            Payload::Items { entries } => {
                // Depth 0 is the question search, where relevance still has to be
                // judged; deeper captures are the answers to a question that was
                // already judged relevant, so every entry counts.
                let relevant: Vec<&extract::ItemEntry> = if depth == 0 {
                    entries
                        .iter()
                        .filter(|entry| {
                            matches_task(task, &entry.title) || matches_task(task, &entry.link)
                        })
                        .collect()
                } else {
                    entries.iter().collect()
                };
                if relevant.is_empty() {
                    read.detail = Some(trace_record::line("no_relevant_result", &[("url", url)]));
                }
                for entry in &relevant {
                    let found = extract_steps(&entry.body, bounds.max_items);
                    push_steps(record, capture, depth, &found, &mut read.items);
                }
                if read.items.is_empty() && produced == 0 {
                    // A question body states the problem; the procedure is in the
                    // answers. Following them is the recursion this shape needs.
                    for question in relevant.iter().filter_map(|entry| entry.question_id) {
                        read.follow.push(answers_url(record, question));
                    }
                }
            }
            Payload::Compressed => {
                read.detail = Some(trace_record::line("compressed_payload", &[("url", url)]));
            }
            Payload::OpenSearch { titles, .. } | Payload::Search { titles } => {
                let relevant: Vec<&String> = titles
                    .iter()
                    .filter(|title| matches_task(task, title))
                    .take(bounds.max_pages_per_service)
                    .collect();
                if relevant.is_empty() {
                    read.detail = Some(trace_record::line("no_relevant_result", &[("url", url)]));
                }
                for title in relevant {
                    read.follow.push(parse_url(record, title));
                }
            }
            Payload::Unrecognized { reason } => {
                let should_search = is_wiki && reason.starts_with("api_error");
                read.detail = Some(trace_record::line(
                    "unreadable_payload",
                    &[("reason", reason), ("url", url)],
                ));
                // A title guess that misses is not a dead end: the same wiki can
                // be searched for the task, and the hits parsed one hop deeper.
                if should_search {
                    read.follow.push(search_url(record, task));
                }
            }
        }
        read
    }
}

fn push_steps(
    record: &SourceRecord,
    capture: &SourceCapture,
    depth: usize,
    found: &[String],
    steps: &mut Vec<GuideStep>,
) {
    for text in found {
        if steps.iter().any(|step| &step.text == text) {
            continue;
        }
        let position = steps.len() + 1;
        steps.push(GuideStep {
            text: text.clone(),
            source_id: record.id.clone(),
            source_name: record.name.clone(),
            source_url: capture.source_url().to_owned(),
            sha256: capture.sha256().to_owned(),
            fetched_at: capture.fetched_at().to_owned(),
            cached: capture.cached(),
            tier: record.tier,
            license_name: record.license_name.clone(),
            license_url: record.license_url.clone(),
            depth,
            position,
        });
    }
}

/// Identical bytes under two URLs mean one of them is a copy. The higher tier
/// keeps the capture; the copy is demoted to [`SourceTier::Unoriginal`] and
/// contributes nothing, exactly as `search_fusion::effective_classifications`
/// decides it for search results.
fn apply_copied_source_policy(steps: Vec<GuideStep>, guide: &mut HowToGuide) -> Vec<GuideStep> {
    let mut owner: BTreeMap<String, (String, u8)> = BTreeMap::new();
    for step in &steps {
        let weight = step.tier.weight_percent();
        match owner.get(&step.sha256) {
            None => {
                owner.insert(step.sha256.clone(), (step.source_url.clone(), weight));
            }
            Some((existing_url, existing_weight))
                if existing_url != &step.source_url && weight > *existing_weight =>
            {
                let demoted = existing_url.clone();
                owner.insert(step.sha256.clone(), (step.source_url.clone(), weight));
                if !guide.copies.contains(&demoted) {
                    guide.copies.push(demoted);
                }
            }
            Some((existing_url, _)) if existing_url != &step.source_url => {
                if !guide.copies.contains(&step.source_url) {
                    guide.copies.push(step.source_url.clone());
                }
            }
            Some(_) => {}
        }
    }
    steps
        .into_iter()
        .filter(|step| !guide.copies.contains(&step.source_url))
        .collect()
}

/// Two sources describing the same action differently is a contradiction, not
/// two steps. The higher tier wins and the disagreement is recorded.
fn apply_conflict_policy(steps: Vec<GuideStep>, guide: &mut HowToGuide) -> Vec<GuideStep> {
    let mut kept: Vec<GuideStep> = Vec::new();
    for step in steps {
        let action = action_key(&step.text);
        let Some(index) = kept
            .iter()
            .position(|existing| action_key(&existing.text) == action)
        else {
            kept.push(step);
            continue;
        };
        if kept[index].source_id == step.source_id || kept[index].text == step.text {
            continue;
        }
        if step.tier.weight_percent() > kept[index].tier.weight_percent() {
            guide.conflicts.push(GuideConflict {
                action: action.clone(),
                kept_source: step.source_id.clone(),
                dropped_source: kept[index].source_id.clone(),
                dropped_text: kept[index].text.clone(),
            });
            kept[index] = step;
        } else {
            guide.conflicts.push(GuideConflict {
                action,
                kept_source: kept[index].source_id.clone(),
                dropped_source: step.source_id.clone(),
                dropped_text: step.text.clone(),
            });
        }
    }
    kept
}

/// The action a step describes, used to detect two sources disagreeing about
/// the same move: the first three meaningful words, lowercased.
fn action_key(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|character| character.is_alphanumeric())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|word| !word.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Presentation order: primary sources first, then by tier, then by the order
/// each source itself listed the step.
fn order_steps(mut steps: Vec<GuideStep>, max_steps: usize) -> Vec<GuideStep> {
    // Tier first, then depth: a page the service answered directly is more
    // direct evidence than one reached by following a search result, so equal
    // tiers rank the shallower capture ahead of the deeper one.
    steps.sort_by_key(|step| {
        (
            u8::MAX - step.tier.weight_percent(),
            step.depth,
            step.source_id.clone(),
            step.position,
        )
    });
    steps.truncate(max_steps);
    steps
}
