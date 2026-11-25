//! Bounded multi-source synthesis of a "how to X" guide.
//!
//! Issue #991 asks for one contract that both production runtimes execute:
//! pick the enabled, relevant services out of `data/seed/sources-registry.lino`,
//! capture their pages recursively inside declared depth/page/time bounds, keep
//! exact provenance on every accepted step, and resolve copies and
//! contradictions with the issue #709 source-tier policy. This module owns that
//! contract for the Rust side; `js/worker/formal_ai_worker_how_to_guide.js` mirrors
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
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod indexed;
mod render;
mod wiki_how;

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

    pub(crate) fn answer_markdown(&self, language: &str) -> String {
        render::answer_markdown(self, language)
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
    synthesize_how_to_guide_in((task, ""), client, preferences, bounds, availability, now)
}

/// The how-to source asked first and never skipped on a cached failure.
pub const PRIORITY_HOW_TO_SOURCE: &str = "wikihow";

pub(crate) fn synthesize_how_to_guide_in<T: SourceTransport>(
    (task, language): (&str, &str),
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
    let extractor = StepExtractor {
        task: &guide.task,
        language,
        can_search: bounds.max_depth > 0,
    };
    for record in select_sources(&guide.task, preferences, bounds) {
        let Some(url) = extractor.entry_url(&record, &guide.task) else {
            guide.outcomes.push(GuideSourceOutcome::new(
                &record.id,
                "unbound_template",
                record.api.clone(),
            ));
            continue;
        };
        let endpoint = crate::source_walk::endpoint_key(&record, &url);
        // wikiHow is the priority source for "how to X?" (issue #1632, R1). A
        // single timeout, HTTP 429 or 5xx used to mark it unreachable for the
        // whole seven-day accessibility TTL, and every how-to question then
        // answered "no verified instructions" without asking wikiHow again.
        // The cached failure still skips the other services, but the priority
        // source is always probed; a successful fetch refreshes its record.
        if record.id != PRIORITY_HOW_TO_SOURCE && availability.known_unreachable(&endpoint, now) {
            guide.outcomes.push(GuideSourceOutcome::new(
                &record.id,
                "unreachable_cached",
                availability
                    .record(&endpoint)
                    .map_or_else(String::new, |entry| entry.detail.clone()),
            ));
            continue;
        }
        let mut outcome = WalkSourceOutcome::new(&record.id, "no_steps", url.clone());
        let walk = Walk {
            client,
            bounds,
            now,
        };
        let mut steps = walk_source(&record, &url, &extractor, &walk, availability, &mut outcome);
        if record.id == PRIORITY_HOW_TO_SOURCE
            && steps.len() < MIN_ACCEPTED_STEPS
            && let Some(recovered) =
                wiki_how::recover(&record, &url, &extractor, &walk, &mut outcome)
        {
            steps = recovered;
            availability.observe(
                &endpoint,
                crate::service_accessibility::ServiceStatus::Reachable,
                outcome.detail.clone(),
                now,
            );
        }
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
fn search_url(endpoint: &str, task: &str) -> String {
    let base = endpoint.split('?').next().unwrap_or(endpoint);
    format!(
        "{base}?action=query&list=search&srsearch={}&srlimit=5&format=json&origin=*",
        percent_encode(task)
    )
}

fn nominal_search(
    record: &SourceRecord,
    task: &str,
    language: &str,
) -> Option<(String, String, u16)> {
    let policy = crate::rule_interpreter::handler_policy(
        "procedural_how_to",
        &format!("nominal-title-actions-{language}"),
    )?;
    let (action, subject) = task.split_once(' ')?;
    if !policy.split_whitespace().any(|entry| entry == action) {
        return None;
    }
    let declaration = crate::rule_interpreter::handler_table_value(
        "procedure-nominal-title-namespaces",
        &record.id,
    )?;
    let values = crate::seed::parser::split_pipe_list(&declaration);
    let [namespace, prefix] = values.as_slice() else {
        return None;
    };
    Some((
        subject.to_owned(),
        format!("{prefix}:"),
        namespace.parse().ok()?,
    ))
}

fn mediawiki_api(endpoint: &str) -> bool {
    endpoint
        .split('?')
        .next()
        .is_some_and(|base| base.ends_with("/api.php"))
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
fn parse_url(endpoint: &str, title: &str) -> String {
    let base = endpoint.split('?').next().unwrap_or(endpoint);
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
        return false;
    }
    let offered = topic_words(candidate);
    wanted.iter().all(|word| offered.contains(word))
}

/// Publisher titles may use a different aspect of the same infinitive. Apply
/// language policy to whole words and keep every requested topic and polarity.
fn matches_wiki_title(task: &str, candidate: &str, language: &str) -> bool {
    let policy = |key: &str| crate::rule_interpreter::handler_policy("procedural_how_to", key);
    let wanted = topic_words(task);
    let offered = topic_words(candidate);
    if let Some(negation) = policy(&format!("negation-{language}"))
        && negation.split_whitespace().any(|word| {
            let has_word = |text: &str| {
                text.to_lowercase()
                    .split(|character| !crate::source_walk::is_topic_character(character))
                    .any(|token| token == word)
            };
            has_word(task) != has_word(candidate)
        })
    {
        return false;
    }
    let prefix = policy(&format!("aspect-prefix-{language}"));
    let endings = policy(&format!("infinitive-endings-{language}")).unwrap_or_default();
    !wanted.is_empty()
        && wanted.iter().all(|word| {
            offered.contains(word)
                || prefix.as_ref().is_some_and(|prefix| {
                    endings.split_whitespace().any(|ending| {
                        word.strip_suffix(ending)
                            .is_some_and(|stem| stem.chars().count() >= 3)
                    }) && offered.iter().any(|other| {
                        other.strip_prefix(prefix.as_str()) == Some(word.as_str())
                            || word.strip_prefix(prefix.as_str()) == Some(other.as_str())
                    })
                })
        })
}

/// The topic words of a phrase: lowercased, de-punctuated, de-pluralised, with
/// stopwords and one/two-letter fragments dropped.
fn topic_words(value: &str) -> Vec<String> {
    value
        .split(|character| !crate::source_walk::is_topic_character(character))
        .map(str::to_lowercase)
        .filter(|word| {
            (word.len() > 2 || !word.is_ascii()) && !TOPIC_STOPWORDS.contains(&word.as_str())
        })
        .map(|word| singular(&word))
        .collect()
}

fn singular(word: &str) -> String {
    match word.strip_suffix('s') {
        Some(stem) if stem.len() > 2 && !word.ends_with("ss") => stem.to_owned(),
        _ => word.to_owned(),
    }
}

fn nominal_words(text: &str) -> Vec<String> {
    text.split(|character| !crate::source_walk::is_topic_character(character))
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .filter(|word| !matches!(word.as_str(), "a" | "an" | "the"))
        .map(|word| singular(&word))
        .collect()
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
    language: &'a str,
    can_search: bool,
}

impl CaptureExtractor for StepExtractor<'_> {
    type Item = GuideStep;

    fn entry_url(&self, record: &SourceRecord, subject: &str) -> Option<String> {
        crate::source_walk::entry_url_in(record, subject, self.language)
    }

    fn fallback_entry_url(&self, record: &SourceRecord) -> Option<String> {
        if !self.can_search || !mediawiki_api(&record.api) {
            return None;
        }
        let endpoint = self.entry_url(record, self.task)?;
        Some(search_url(&endpoint, self.task))
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
        let endpoint = capture.source_url();
        let url = endpoint.to_owned();
        let is_wiki = mediawiki_api(&record.api);
        let is_wikihow = record.id == "wikihow";
        let nominal = nominal_search(record, task, self.language);
        let nominal_match = |title: &str| {
            nominal.as_ref().is_some_and(|(subject, prefix, _)| {
                title.strip_prefix(prefix).is_some_and(|title| {
                    let words = nominal_words(subject);
                    !words.is_empty() && words == nominal_words(title)
                })
            })
        };
        let full_search = search_url(endpoint, task);
        let nominal_url = nominal.as_ref().map(|(subject, _, namespace)| {
            format!("{}&srnamespace={namespace}", search_url(endpoint, subject))
        });
        let title_matches =
            |title: &str| matches_wiki_title(task, title, self.language) || nominal_match(title);
        let mut read = Extracted::default();
        match classify(capture.bytes()) {
            Payload::Parse { .. } if produced > 0 => {}
            Payload::Parse { title, html } => {
                let relevant = is_wiki && title_matches(&title);
                let found = if !relevant {
                    read.detail = Some(trace_record::line("no_relevant_result", &[("url", url)]));
                    Vec::new()
                } else if is_wikihow {
                    wiki_how::steps(&html, task, self.language, bounds.max_items)
                } else {
                    let labels = crate::rule_interpreter::handler_policy(
                        "procedural_how_to",
                        "ordered-section-labels",
                    )
                    .unwrap_or_default();
                    extract::ordered_procedure_steps(
                        &html,
                        &labels,
                        nominal_match(&title),
                        bounds.max_items,
                    )
                };
                if found.is_empty() && depth < bounds.max_depth {
                    for title in wiki_link_titles(&html, bounds.max_pages_per_service) {
                        if title_matches(&title) {
                            read.follow.push(parse_url(endpoint, &title));
                        }
                    }
                    if read.follow.is_empty() && is_wiki {
                        read.follow.push(full_search);
                        read.follow.extend(nominal_url);
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
            Payload::OpenSearch { titles, .. } | Payload::Search { titles } if is_wiki => {
                let relevant: Vec<&String> = titles
                    .iter()
                    .filter(|title| title_matches(title))
                    .take(bounds.max_pages_per_service)
                    .collect();
                if relevant.is_empty() {
                    read.detail = Some(trace_record::line("no_relevant_result", &[("url", url)]));
                    if endpoint == full_search {
                        read.follow.extend(nominal_url);
                    }
                }
                for title in relevant {
                    read.follow.push(parse_url(endpoint, title));
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
                    read.follow.push(full_search);
                    read.follow.extend(nominal_url);
                }
            }
            Payload::OpenSearch { .. } | Payload::Search { .. } => {}
        }
        // A later relevant capture replaces the rejected page's outcome detail.
        // The accepted steps retain their own capture URL and digest.
        if !read.items.is_empty() {
            read.detail = Some(capture.source_url().to_owned());
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
                .filter(|character| crate::source_walk::is_topic_character(*character))
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
