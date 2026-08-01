//! Symbolic next-request anticipation for idle dreaming (issue #705).
//!
//! It predicts from Markov transitions over request classes, expands seeded
//! vocabulary, probes offline, and preserves proposal-only consent/provenance/TTL.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::engine::{normalize_prompt, stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::intent_formalization::formalize_intent;
use crate::learning_cycle::{run_learning_cycle, FrontierItem, LearningCycleRun};
use crate::links_format::format_lino_record;
use crate::memory::{MemoryEvent, MemoryStore};
use crate::probability::{ProbabilityEvidence, ProbabilityModel};
use crate::seed;
use crate::solver::{SolverConfig, UniversalSolver};
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};
use crate::source_research::execute_source_research;

mod expansion;
use expansion::expand_class;
mod ledger;
pub use ledger::AnticipationLedger;

pub const ANTICIPATION_FRONTIER: &str = "anticipation";
pub const ANTICIPATION_PREDICTION_KIND: &str = "anticipation_prediction";
pub const ANTICIPATION_SOURCE_KIND: &str = "anticipation_source";
pub const PREDICTION_HIT_KIND: &str = "prediction_hit";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentClass {
    pub id: String,
    pub intent: String,
    pub kind: String,
    pub route: Option<String>,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntentTransition {
    pub from: IntentClass,
    pub to: IntentClass,
    pub count: usize,
    pub probability: f32,
    pub evidence: ProbabilityEvidence,
    pub evidence_links: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptVariant {
    pub prompt: String,
    pub source: String,
    pub base_event_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PredictedClass {
    pub id: String,
    pub class: IntentClass,
    pub rank: usize,
    pub count: usize,
    pub probability: f32,
    pub transition_evidence_id: String,
    pub evidence_links: Vec<String>,
    pub variants: Vec<PromptVariant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeStatus {
    Passed,
    Unknown,
    Failed,
}

impl ProbeStatus {
    const fn slug(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Unknown => "unknown",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeResult {
    pub prediction_id: String,
    pub prompt: String,
    pub base_event_id: String,
    pub variation_source: String,
    pub expected_class: String,
    pub actual_class: String,
    pub engine_intent: String,
    pub language: String,
    pub status: ProbeStatus,
}

#[derive(Debug, Clone)]
pub struct AnticipationPlan {
    pub current_class: Option<IntentClass>,
    pub transitions: Vec<IntentTransition>,
    pub predictions: Vec<PredictedClass>,
    pub probes: Vec<ProbeResult>,
    pub frontier: Vec<FrontierItem>,
    pub learning_cycle: LearningCycleRun,
}

impl AnticipationPlan {
    #[must_use]
    pub fn why_prediction(&self, prediction_id: &str) -> Option<String> {
        let prediction = self
            .predictions
            .iter()
            .find(|prediction| prediction.id == prediction_id)?;
        let mut explanation = String::new();
        let _ = write!(explanation, "prediction={}", prediction.id);
        let _ = write!(explanation, " class={}", prediction.class.id);
        let _ = write!(
            explanation,
            " transition_evidence={}",
            prediction.transition_evidence_id
        );
        let _ = write!(explanation, " count={}", prediction.count);
        let _ = write!(explanation, " probability={:.6}", prediction.probability);
        Some(explanation)
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut records = vec![format_lino_record(
            "anticipation_plan",
            &[
                ("record_type", String::from("anticipation_plan")),
                ("issue", String::from("705")),
                ("model", String::from("markov_transition")),
                ("deterministic", String::from("true")),
                (
                    "current_class",
                    self.current_class
                        .as_ref()
                        .map(|class| class.id.clone())
                        .unwrap_or_default(),
                ),
                ("predictions", self.predictions.len().to_string()),
                ("probes", self.probes.len().to_string()),
                ("frontier_items", self.frontier.len().to_string()),
            ],
        )];
        records.extend(
            self.transitions
                .iter()
                .map(|transition| transition.evidence.to_links_notation()),
        );
        for prediction in &self.predictions {
            let mut fields = vec![
                ("record_type", String::from("anticipation_prediction")),
                ("id", prediction.id.clone()),
                ("class", prediction.class.id.clone()),
                ("rank", prediction.rank.to_string()),
                ("count", prediction.count.to_string()),
                ("probability", format!("{:.6}", prediction.probability)),
                (
                    "transition_evidence",
                    prediction.transition_evidence_id.clone(),
                ),
            ];
            for link in &prediction.evidence_links {
                fields.push(("evidence", link.clone()));
            }
            records.push(format_lino_record("anticipation_prediction", &fields));
            for variant in &prediction.variants {
                records.push(format_lino_record(
                    "anticipation_variant",
                    &[
                        ("prediction", prediction.id.clone()),
                        ("prompt", variant.prompt.clone()),
                        ("source", variant.source.clone()),
                        ("base_event", variant.base_event_id.clone()),
                    ],
                ));
            }
        }
        for probe in &self.probes {
            records.push(format_lino_record(
                "anticipation_probe",
                &[
                    ("prediction", probe.prediction_id.clone()),
                    ("prompt", probe.prompt.clone()),
                    ("expected_class", probe.expected_class.clone()),
                    ("actual_class", probe.actual_class.clone()),
                    ("engine_intent", probe.engine_intent.clone()),
                    ("status", probe.status.slug().to_owned()),
                    ("variation_source", probe.variation_source.clone()),
                ],
            ));
        }
        records.push(self.learning_cycle.links_notation());
        records.join("\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnticipationConfig {
    pub max_predictions: usize,
    pub max_variations_per_prediction: usize,
    pub source_page_limit: usize,
    pub ttl_seconds: u64,
}

impl Default for AnticipationConfig {
    fn default() -> Self {
        Self {
            max_predictions: 3,
            max_variations_per_prediction: 16,
            source_page_limit: 1,
            ttl_seconds: 3_600,
        }
    }
}

#[derive(Clone)]
struct Observation {
    event_id: String,
    prompt: String,
    class: IntentClass,
    recorded_at: String,
    conversation_id: Option<String>,
}

#[must_use]
pub fn intent_class_for_prompt(prompt: &str) -> IntentClass {
    let solver = offline_probe_solver();
    classify_prompt(&solver, prompt).0
}

#[must_use]
pub fn plan_anticipation(events: &[MemoryEvent], config: &AnticipationConfig) -> AnticipationPlan {
    let observations = observations(events);
    let transitions = transition_counts(&observations);
    let current_class = observations.last().map(|item| item.class.clone());
    let mut predictions = current_class.as_ref().map_or_else(Vec::new, |current| {
        transitions
            .iter()
            .filter(|transition| transition.from.id == current.id)
            .map(|transition| PredictedClass {
                id: stable_id(
                    "anticipation_prediction",
                    &format!(
                        "{}:{}:{}",
                        current.id, transition.to.id, transition.evidence.id
                    ),
                ),
                class: transition.to.clone(),
                rank: 0,
                count: transition.count,
                probability: transition.probability,
                transition_evidence_id: transition.evidence.id.clone(),
                evidence_links: transition.evidence_links.clone(),
                variants: expand_class(&transition.to, &observations, config),
            })
            .collect::<Vec<_>>()
    });
    predictions.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.class.id.cmp(&right.class.id))
    });
    predictions.truncate(config.max_predictions);
    for (index, prediction) in predictions.iter_mut().enumerate() {
        prediction.rank = index + 1;
    }

    let solver = offline_probe_solver();
    let mut probes = Vec::new();
    let mut frontier = Vec::new();
    for prediction in &predictions {
        for variant in &prediction.variants {
            let (actual, engine_intent) = classify_prompt(&solver, &variant.prompt);
            let status = if engine_intent == "unknown" {
                ProbeStatus::Unknown
            } else if actual.id == prediction.class.id {
                ProbeStatus::Passed
            } else {
                ProbeStatus::Failed
            };
            let probe = ProbeResult {
                prediction_id: prediction.id.clone(),
                prompt: variant.prompt.clone(),
                base_event_id: variant.base_event_id.clone(),
                variation_source: variant.source.clone(),
                expected_class: prediction.class.id.clone(),
                actual_class: actual.id,
                engine_intent: engine_intent.clone(),
                language: crate::language::detect(&variant.prompt).slug().to_owned(),
                status,
            };
            if status != ProbeStatus::Passed {
                frontier.push(FrontierItem {
                    rank: frontier.len() + 1,
                    query: variant.prompt.clone(),
                    language: probe.language.clone(),
                    variation: frontier_variation(&variant.source),
                    prompt: variant.prompt.clone(),
                    engine_intent,
                });
            }
            probes.push(probe);
        }
    }
    let learning_cycle = run_learning_cycle(ANTICIPATION_FRONTIER, &frontier);
    AnticipationPlan {
        current_class,
        transitions,
        predictions,
        probes,
        frontier,
        learning_cycle,
    }
}

fn offline_probe_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        compute_budget: 0,
        draft_count: 1,
        ..SolverConfig::default()
    })
}

fn classify_prompt(solver: &UniversalSolver, prompt: &str) -> (IntentClass, String) {
    let answer = solver.solve(prompt);
    let language = crate::language::detect(prompt);
    let formalization = formalize_intent(prompt, language.slug(), None);
    let operations = seed::operation_vocabulary().detect(&normalize_prompt(prompt));
    let intent = answer.intent;
    (
        IntentClass {
            id: format!("intent:{}", identifier(&intent)),
            intent: intent.clone(),
            kind: formalization.kind.slug().to_owned(),
            route: formalization.route,
            operations,
        },
        intent,
    )
}

fn observations(events: &[MemoryEvent]) -> Vec<Observation> {
    events
        .iter()
        .enumerate()
        .filter(|(_, event)| event.role.as_deref() == Some("user"))
        .filter_map(|(index, event)| {
            let prompt = event.content.as_deref()?.trim();
            if prompt.is_empty() {
                return None;
            }
            let mut class = intent_class_for_prompt(prompt);
            if let Some(intent) = event.intent.as_deref().filter(|intent| !intent.is_empty()) {
                intent.clone_into(&mut class.intent);
                class.id = format!("intent:{}", identifier(intent));
            }
            Some(Observation {
                event_id: event.id.clone(),
                prompt: prompt.to_owned(),
                class,
                recorded_at: event
                    .sent_at
                    .clone()
                    .unwrap_or_else(|| format!("append:{index}")),
                conversation_id: event.conversation_id.clone(),
            })
        })
        .collect()
}

#[allow(clippy::cast_precision_loss)] // ProbabilityEvidence stores its symbolic weight as f32.
fn transition_counts(observations: &[Observation]) -> Vec<IntentTransition> {
    let mut groups: BTreeMap<(String, String), Vec<(&Observation, &Observation)>> = BTreeMap::new();
    for pair in observations.windows(2) {
        let from = &pair[0];
        let to = &pair[1];
        if matches!(
            (&from.conversation_id, &to.conversation_id),
            (Some(left), Some(right)) if left != right
        ) {
            continue;
        }
        groups
            .entry((from.class.id.clone(), to.class.id.clone()))
            .or_default()
            .push((from, to));
    }
    let mut outgoing: BTreeMap<String, usize> = BTreeMap::new();
    for ((from, _), pairs) in &groups {
        *outgoing.entry(from.clone()).or_default() += pairs.len();
    }
    groups
        .into_values()
        .map(|pairs| {
            let from = pairs[0].0.class.clone();
            let to = pairs[0].1.class.clone();
            let count = pairs.len();
            let total = outgoing.get(&from.id).copied().unwrap_or(count).max(1);
            let probability = count as f32 / total as f32;
            let evidence_links = pairs
                .iter()
                .flat_map(|(left, right)| {
                    [
                        format!("memory:{}", left.event_id),
                        format!("memory:{}", right.event_id),
                    ]
                })
                .collect::<Vec<_>>();
            let provenance = evidence_links.join(",");
            let recorded_at = pairs
                .last()
                .map(|(_, item)| item.recorded_at.clone())
                .unwrap_or_default();
            let evidence = ProbabilityEvidence::symbolic(
                to.id.clone(),
                format!("count={count};outgoing={total}"),
                probability,
                provenance,
                recorded_at,
            )
            .with_model(ProbabilityModel::MarkovTransition)
            .with_transition_from(from.id.clone());
            IntentTransition {
                from,
                to,
                count,
                probability,
                evidence,
                evidence_links,
            }
        })
        .collect()
}

fn frontier_variation(source: &str) -> String {
    format!("anticipation_{}", identifier(source))
}

fn identifier(value: &str) -> String {
    let mut out = String::new();
    let mut separator = false;
    for ch in value.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            if separator && !out.is_empty() {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
            separator = false;
        } else {
            separator = true;
        }
    }
    if out.is_empty() {
        stable_id("intent_class", value)
    } else {
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnticipationConsent {
    Denied,
    Granted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrelearningStatus {
    ConsentRequired,
    Captured,
    NoSource,
    FetchFailed,
}

impl PrelearningStatus {
    const fn slug(self) -> &'static str {
        match self {
            Self::ConsentRequired => "consent_required",
            Self::Captured => "captured",
            Self::NoSource => "no_source",
            Self::FetchFailed => "fetch_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrelearningAttempt {
    pub prediction_id: String,
    pub query: String,
    pub status: PrelearningStatus,
    pub diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrelearnedSource {
    pub id: String,
    pub prediction_id: String,
    pub class_id: String,
    pub base_event_id: String,
    pub query: String,
    pub aliases: Vec<String>,
    pub answer: String,
    pub result_url: String,
    pub source_url: String,
    pub fetched_at: String,
    pub sha256: String,
    pub cached: bool,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrelearningRun {
    pub attempts: Vec<PrelearningAttempt>,
    pub sources: Vec<PrelearnedSource>,
}

impl PrelearningRun {
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut records = Vec::new();
        for attempt in &self.attempts {
            records.push(format_lino_record(
                "anticipation_prelearning_attempt",
                &[
                    ("prediction", attempt.prediction_id.clone()),
                    ("query", attempt.query.clone()),
                    ("status", attempt.status.slug().to_owned()),
                    ("diagnostic", attempt.diagnostic.clone()),
                ],
            ));
        }
        for source in &self.sources {
            records.push(source_record(source, None));
        }
        records.join("\n")
    }
}

pub fn prelearn_predictions<T: SourceTransport>(
    plan: &AnticipationPlan,
    client: &CachedSourceClient<T>,
    consent: AnticipationConsent,
    config: &AnticipationConfig,
) -> PrelearningRun {
    let candidates = plan
        .predictions
        .iter()
        .filter_map(|prediction| {
            plan.probes
                .iter()
                .find(|probe| {
                    probe.prediction_id == prediction.id && probe.status != ProbeStatus::Passed
                })
                .map(|probe| (prediction, probe))
        })
        .collect::<Vec<_>>();
    let mut run = PrelearningRun::default();
    for (prediction, probe) in candidates {
        if consent == AnticipationConsent::Denied {
            run.attempts.push(PrelearningAttempt {
                prediction_id: prediction.id.clone(),
                query: probe.prompt.clone(),
                status: PrelearningStatus::ConsentRequired,
                diagnostic: String::from("fetch_consent_required"),
            });
            continue;
        }
        match execute_source_research(client, &probe.prompt, config.source_page_limit) {
            Err(error) => run.attempts.push(PrelearningAttempt {
                prediction_id: prediction.id.clone(),
                query: probe.prompt.clone(),
                status: PrelearningStatus::FetchFailed,
                diagnostic: error.to_string(),
            }),
            Ok(research) => {
                let Some(result) = research.search.fused.first() else {
                    run.attempts.push(PrelearningAttempt {
                        prediction_id: prediction.id.clone(),
                        query: probe.prompt.clone(),
                        status: PrelearningStatus::NoSource,
                        diagnostic: String::from("search_result_absent"),
                    });
                    continue;
                };
                let answer = if result.excerpt.trim().is_empty() {
                    result.title.trim()
                } else {
                    result.excerpt.trim()
                };
                if answer.is_empty() {
                    run.attempts.push(PrelearningAttempt {
                        prediction_id: prediction.id.clone(),
                        query: probe.prompt.clone(),
                        status: PrelearningStatus::NoSource,
                        diagnostic: String::from("search_excerpt_absent"),
                    });
                    continue;
                }
                let capture = research
                    .pages
                    .iter()
                    .find(|page| page.ranking.url == result.url)
                    .map(|page| &page.capture)
                    .or_else(|| research.search.captures.first());
                let Some(capture) = capture else {
                    continue;
                };
                let fetched_at = capture.fetched_at().parse::<u64>().unwrap_or_default();
                let aliases = prediction
                    .variants
                    .iter()
                    .filter(|variant| variant.base_event_id == probe.base_event_id)
                    .map(|variant| variant.prompt.clone())
                    .collect::<Vec<_>>();
                let id = stable_id(
                    "anticipation_source",
                    &format!("{}:{}:{}", prediction.id, result.url, capture.sha256()),
                );
                run.sources.push(PrelearnedSource {
                    id,
                    prediction_id: prediction.id.clone(),
                    class_id: prediction.class.id.clone(),
                    base_event_id: probe.base_event_id.clone(),
                    query: probe.prompt.clone(),
                    aliases,
                    answer: answer.to_owned(),
                    result_url: result.url.clone(),
                    source_url: capture.source_url().to_owned(),
                    fetched_at: capture.fetched_at().to_owned(),
                    sha256: capture.sha256().to_owned(),
                    cached: capture.cached(),
                    expires_at: fetched_at.saturating_add(config.ttl_seconds),
                });
                run.attempts.push(PrelearningAttempt {
                    prediction_id: prediction.id.clone(),
                    query: probe.prompt.clone(),
                    status: PrelearningStatus::Captured,
                    diagnostic: result.url.clone(),
                });
            }
        }
    }
    run
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnticipationOutcome {
    pub probability_records: usize,
    pub prediction_records: usize,
    pub prelearned_aliases: usize,
}

impl AnticipationOutcome {
    #[must_use]
    pub const fn recorded_events(self) -> usize {
        self.probability_records + self.prediction_records + self.prelearned_aliases
    }
}

pub fn apply_anticipation(
    store: &mut MemoryStore,
    plan: &AnticipationPlan,
    prelearning: &PrelearningRun,
) -> AnticipationOutcome {
    let mut known = store
        .events()
        .iter()
        .map(|event| event.id.clone())
        .collect::<BTreeSet<_>>();
    let mut outcome = AnticipationOutcome::default();
    for transition in &plan.transitions {
        let id = transition.evidence.id.clone();
        if known.insert(id.clone()) {
            store.append(MemoryEvent {
                id,
                kind: Some(String::from("probability_evidence")),
                role: Some(String::from("system")),
                intent: Some(String::from("markov_transition")),
                inputs: Some(transition.from.id.clone()),
                outputs: Some(transition.to.id.clone()),
                content: Some(transition.evidence.to_links_notation()),
                evidence: transition.evidence_links.clone(),
                write_count: 1,
                ..MemoryEvent::default()
            });
            outcome.probability_records += 1;
        }
    }
    for prediction in &plan.predictions {
        if known.insert(prediction.id.clone()) {
            store.append(MemoryEvent {
                id: prediction.id.clone(),
                kind: Some(String::from(ANTICIPATION_PREDICTION_KIND)),
                role: Some(String::from("system")),
                intent: Some(prediction.class.id.clone()),
                inputs: plan.current_class.as_ref().map(|class| class.id.clone()),
                outputs: Some(prediction.class.id.clone()),
                content: Some(format_lino_record(
                    "anticipation_prediction",
                    &[
                        ("rank", prediction.rank.to_string()),
                        ("count", prediction.count.to_string()),
                        ("probability", format!("{:.6}", prediction.probability)),
                        (
                            "transition_evidence",
                            prediction.transition_evidence_id.clone(),
                        ),
                    ],
                )),
                evidence: prediction
                    .evidence_links
                    .iter()
                    .cloned()
                    .chain(std::iter::once(format!(
                        "probability_evidence:{}",
                        prediction.transition_evidence_id
                    )))
                    .collect(),
                write_count: 1,
                ..MemoryEvent::default()
            });
            outcome.prediction_records += 1;
        }
    }
    for source in &prelearning.sources {
        for alias in &source.aliases {
            let id = stable_id(
                "anticipation_source_alias",
                &format!("{}:{}", source.id, normalize_prompt(alias)),
            );
            if !known.insert(id.clone()) {
                continue;
            }
            store.append(MemoryEvent {
                id,
                kind: Some(String::from(ANTICIPATION_SOURCE_KIND)),
                role: Some(String::from("system")),
                intent: Some(source.class_id.clone()),
                inputs: Some(alias.clone()),
                outputs: Some(source.answer.clone()),
                content: Some(source_record(source, Some(alias))),
                sent_at: Some(source.fetched_at.clone()),
                demo_label: Some(source.query.clone()),
                evidence: vec![
                    format!("anticipation_prediction:{}", source.prediction_id),
                    format!("source:http:{}", source_trace(source)),
                ],
                write_count: 1,
                ..MemoryEvent::default()
            });
            outcome.prelearned_aliases += 1;
        }
    }
    outcome
}

#[must_use]
pub fn answer_from_prelearned_cache(
    prompt: &str,
    events: &[MemoryEvent],
) -> Option<SymbolicAnswer> {
    answer_from_prelearned_cache_at(prompt, events, epoch_seconds())
}

#[must_use]
pub fn answer_from_prelearned_cache_at(
    prompt: &str,
    events: &[MemoryEvent],
    now: u64,
) -> Option<SymbolicAnswer> {
    let normalized = normalize_prompt(prompt);
    let event = events.iter().rev().find(|event| {
        event.kind.as_deref() == Some(ANTICIPATION_SOURCE_KIND)
            && event
                .inputs
                .as_deref()
                .is_some_and(|alias| normalize_prompt(alias) == normalized)
            && content_field(event.content.as_deref(), "expires_at")
                .and_then(|value| value.parse::<u64>().ok())
                .is_some_and(|expires_at| now <= expires_at)
    })?;
    let body = event.outputs.as_deref()?;
    let prediction = content_field(event.content.as_deref(), "prediction").unwrap_or_default();
    let source_trace = content_field(event.content.as_deref(), "source_trace").unwrap_or_default();
    let source_url = content_field(event.content.as_deref(), "source_url").unwrap_or_default();
    let mut log = EventLog::new();
    log.append("anticipation_prediction", prediction);
    log.append("source:http", source_trace);
    log.append("cache_hit", source_url);
    Some(crate::solver_handlers::finalize_simple(
        prompt,
        &mut log,
        "anticipation_cache",
        "response:anticipation_cache",
        body,
        1.0,
    ))
}

#[must_use]
pub fn prediction_hit_event(
    events: &[MemoryEvent],
    prompt: &str,
    actual_request_id: &str,
) -> Option<MemoryEvent> {
    let class = intent_class_for_prompt(prompt);
    let prediction = events.iter().rev().find(|event| {
        event.kind.as_deref() == Some(ANTICIPATION_PREDICTION_KIND)
            && event.intent.as_deref() == Some(class.id.as_str())
    })?;
    let id = stable_id(
        "prediction_hit",
        &format!("{}:{actual_request_id}", prediction.id),
    );
    Some(MemoryEvent {
        id,
        kind: Some(String::from(PREDICTION_HIT_KIND)),
        role: Some(String::from("system")),
        intent: Some(class.id.clone()),
        inputs: Some(prediction.id.clone()),
        outputs: Some(actual_request_id.to_owned()),
        content: Some(format_lino_record(
            "prediction_hit",
            &[
                ("prediction", prediction.id.clone()),
                ("actual_request", actual_request_id.to_owned()),
                ("actual_class", class.id),
            ],
        )),
        evidence: vec![
            format!("anticipation_prediction:{}", prediction.id),
            actual_request_id.to_owned(),
        ],
        write_count: 1,
        ..MemoryEvent::default()
    })
}

pub fn run_idle_anticipation(
    memory_path: &Path,
    store: &mut MemoryStore,
) -> io::Result<AnticipationOutcome> {
    let solver_config = SolverConfig::from_env();
    let config = AnticipationConfig {
        ttl_seconds: solver_config.cache_ttl_seconds,
        ..AnticipationConfig::default()
    };
    let plan = plan_anticipation(store.events(), &config);
    let consent = if crate::lexeme_import::live_api_enabled() {
        AnticipationConsent::Granted
    } else {
        AnticipationConsent::Denied
    };
    let cache_dir =
        std::env::var("FORMAL_AI_SOURCE_CACHE_DIR").unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(cache_dir, CurlSourceTransport)
        .with_online(consent == AnticipationConsent::Granted)
        .with_ttl_seconds(config.ttl_seconds);
    let prelearning = prelearn_predictions(&plan, &client, consent, &config);
    let outcome = apply_anticipation(store, &plan, &prelearning);
    let ledger = AnticipationLedger::new(&plan, &prelearning, store.events()).links_notation();
    crate::memory::write_locked_atomic(
        &anticipation_ledger_path(memory_path),
        &format!("{ledger}\n"),
    )?;
    Ok(outcome)
}

#[must_use]
pub fn anticipation_ledger_path(memory_path: &Path) -> PathBuf {
    memory_path.with_extension("anticipation.lino")
}

fn source_record(source: &PrelearnedSource, alias: Option<&str>) -> String {
    let mut fields = vec![
        ("record_type", String::from("prelearned_source")),
        ("id", source.id.clone()),
        ("prediction", source.prediction_id.clone()),
        ("class", source.class_id.clone()),
        ("base_event", source.base_event_id.clone()),
        ("query", source.query.clone()),
        ("result_url", source.result_url.clone()),
        ("source_url", source.source_url.clone()),
        ("fetched_at", source.fetched_at.clone()),
        ("sha256", source.sha256.clone()),
        ("cached", source.cached.to_string()),
        ("expires_at", source.expires_at.to_string()),
        ("source_trace", source_trace(source)),
    ];
    if let Some(alias) = alias {
        fields.push(("alias", alias.to_owned()));
    }
    format_lino_record("anticipation_source", &fields)
}

fn source_trace(source: &PrelearnedSource) -> String {
    let mut trace = source.source_url.clone();
    let _ = write!(trace, " fetched_at={}", source.fetched_at);
    let _ = write!(trace, " sha256={}", source.sha256);
    let _ = write!(trace, " cached={}", source.cached);
    trace
}

fn content_field(content: Option<&str>, key: &str) -> Option<String> {
    content?.lines().find_map(|line| {
        let line = line.trim();
        let (name, value) = line.split_once(' ')?;
        (name == key).then(|| {
            value
                .trim()
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or_else(|| value.trim())
                .replace("\"\"", "\"")
        })
    })
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
