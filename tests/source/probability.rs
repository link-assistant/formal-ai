//! Link-native symbolic probability evidence and deterministic ranking.
//!
//! This module intentionally does not perform neural-network inference. It
//! treats probability evidence as ordinary append-only Links Notation records:
//! each record points at a symbolic target, carries provenance and a fixed
//! timestamp supplied by the caller, and can be replayed into the same event
//! log / link-store projection as the rest of the solver trace.

use crate::engine::stable_id;
use crate::event_log::EventLog;
use crate::link_store::{LinkStore, LinkStoreError};
use crate::links_format::format_lino_record;
use crate::memory::MemoryEvent;

/// Supported symbolic probabilistic model families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbabilityModel {
    /// Naive Bayesian-style evidence: independent symbolic observations add
    /// weight to a candidate's posterior score.
    BayesianEvidence,
    /// Markov-style transition evidence: the weight applies only when the
    /// prior selected state matches `transition_from`.
    MarkovTransition,
}

impl ProbabilityModel {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::BayesianEvidence => "bayesian_evidence",
            Self::MarkovTransition => "markov_transition",
        }
    }
}

/// Cached-source provenance attached to probability evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbabilitySourceProvenance {
    pub source_url: String,
    pub fetched_at: String,
    pub sha256: String,
    pub cached: bool,
}

impl ProbabilitySourceProvenance {
    #[must_use]
    pub fn trace_payload(&self) -> String {
        format!(
            "{} fetched_at={} sha256={} cached={}",
            self.source_url, self.fetched_at, self.sha256, self.cached
        )
    }
}

/// One append-only symbolic probability observation.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbabilityEvidence {
    pub id: String,
    pub target: String,
    pub observation: String,
    pub weight: f32,
    pub model: ProbabilityModel,
    pub provenance: String,
    pub recorded_at: String,
    pub source: Option<ProbabilitySourceProvenance>,
    pub transition_from: Option<String>,
}

impl ProbabilityEvidence {
    #[must_use]
    pub fn symbolic(
        target: impl Into<String>,
        observation: impl Into<String>,
        weight: f32,
        provenance: impl Into<String>,
        recorded_at: impl Into<String>,
    ) -> Self {
        let mut evidence = Self {
            id: String::new(),
            target: target.into(),
            observation: observation.into(),
            weight: finite_or_zero(weight),
            model: ProbabilityModel::BayesianEvidence,
            provenance: provenance.into(),
            recorded_at: recorded_at.into(),
            source: None,
            transition_from: None,
        };
        evidence.id = evidence.stable_record_id();
        evidence
    }

    #[must_use]
    pub fn with_model(mut self, model: ProbabilityModel) -> Self {
        self.model = model;
        self.id = self.stable_record_id();
        self
    }

    #[must_use]
    pub fn with_source(mut self, source: ProbabilitySourceProvenance) -> Self {
        self.source = Some(source);
        self.id = self.stable_record_id();
        self
    }

    #[must_use]
    pub fn with_transition_from(mut self, transition_from: impl Into<String>) -> Self {
        self.transition_from = Some(transition_from.into());
        self.id = self.stable_record_id();
        self
    }

    #[must_use]
    pub fn trace_payload(&self) -> String {
        let mut parts = vec![
            format!("id={}", self.id),
            format!("target={}", self.target),
            format!("model={}", self.model.slug()),
            format!("observation={}", self.observation),
            format!("weight={:.6}", self.weight),
            format!("provenance={}", self.provenance),
            format!("recorded_at={}", self.recorded_at),
        ];
        if let Some(transition_from) = &self.transition_from {
            parts.push(format!("transition_from={transition_from}"));
        }
        if let Some(source) = &self.source {
            parts.push(format!("source_url={}", source.source_url));
            parts.push(format!("fetched_at={}", source.fetched_at));
            parts.push(format!("sha256={}", source.sha256));
            parts.push(format!("cached={}", source.cached));
        }
        parts.join(" ")
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut fields = vec![
            ("id", self.id.clone()),
            ("target", self.target.clone()),
            ("observation", self.observation.clone()),
            ("weight", format!("{:.6}", self.weight)),
            ("model", self.model.slug().to_owned()),
            ("provenance", self.provenance.clone()),
            ("recorded_at", self.recorded_at.clone()),
        ];
        if let Some(transition_from) = &self.transition_from {
            fields.push(("transition_from", transition_from.clone()));
        }
        if let Some(source) = &self.source {
            fields.extend([
                ("source_url", source.source_url.clone()),
                ("fetched_at", source.fetched_at.clone()),
                ("sha256", source.sha256.clone()),
                ("cached", source.cached.to_string()),
            ]);
        }
        format_lino_record("probability_evidence", &fields)
    }

    fn stable_record_id(&self) -> String {
        let source_fingerprint = self.source.as_ref().map_or_else(String::new, |source| {
            format!(
                "{}:{}:{}:{}",
                source.source_url, source.fetched_at, source.sha256, source.cached
            )
        });
        stable_id(
            "probability",
            &format!(
                "{}:{}:{:.6}:{}:{}:{}:{:?}:{}",
                self.target,
                self.observation,
                self.weight,
                self.model.slug(),
                self.provenance,
                self.recorded_at,
                self.transition_from,
                source_fingerprint
            ),
        )
    }

    fn usable_offline(&self, offline: bool) -> bool {
        if !offline {
            return true;
        }
        self.source.as_ref().map_or(true, |source| source.cached)
    }

    fn applies_to_markov_state(&self, markov_from: Option<&str>) -> bool {
        match self.model {
            ProbabilityModel::BayesianEvidence => true,
            ProbabilityModel::MarkovTransition => self.transition_from.as_deref() == markov_from,
        }
    }
}

/// Append-only probability evidence store.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ProbabilityStore {
    records: Vec<ProbabilityEvidence>,
}

impl ProbabilityStore {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    #[must_use]
    pub const fn from_records(records: Vec<ProbabilityEvidence>) -> Self {
        Self { records }
    }

    pub fn record(&mut self, evidence: ProbabilityEvidence) -> String {
        let id = evidence.id.clone();
        self.records.push(evidence);
        id
    }

    pub fn update(
        &mut self,
        target: impl Into<String>,
        observation: impl Into<String>,
        weight: f32,
        provenance: impl Into<String>,
        recorded_at: impl Into<String>,
    ) -> String {
        self.record(ProbabilityEvidence::symbolic(
            target,
            observation,
            weight,
            provenance,
            recorded_at,
        ))
    }

    #[must_use]
    pub fn records(&self) -> &[ProbabilityEvidence] {
        &self.records
    }

    #[must_use]
    pub fn target_weight(&self, target: &str, offline: bool, markov_from: Option<&str>) -> f32 {
        self.records
            .iter()
            .filter(|evidence| evidence.target == target)
            .filter(|evidence| evidence.usable_offline(offline))
            .filter(|evidence| evidence.applies_to_markov_state(markov_from))
            .map(|evidence| evidence.weight)
            .sum()
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut blocks = vec![format_lino_record(
            "probability_store",
            &[("record_count", self.records.len().to_string())],
        )];
        blocks.extend(
            self.records
                .iter()
                .map(ProbabilityEvidence::to_links_notation),
        );
        blocks.join("\n\n")
    }

    pub fn replay_into_event_log(&self, log: &mut EventLog, offline: bool) -> usize {
        let mut replayed = 0;
        for evidence in &self.records {
            if !evidence.usable_offline(offline) {
                if let Some(source) = &evidence.source {
                    log.append("policy:offline", source.trace_payload());
                }
                continue;
            }
            log.append("probability:evidence", evidence.trace_payload());
            log.append("probability:model", evidence.model.slug().to_owned());
            if let Some(source) = &evidence.source {
                log.append("source:http", source.trace_payload());
                if source.cached {
                    log.append("cache_hit", source.source_url.clone());
                }
            }
            replayed += 1;
        }
        replayed
    }

    pub fn append_to_link_store<S: LinkStore>(
        &self,
        store: &mut S,
        offline: bool,
    ) -> Result<usize, LinkStoreError> {
        let mut inserted = 0;
        for evidence in &self.records {
            if !evidence.usable_offline(offline) {
                continue;
            }
            store.append_memory_event(MemoryEvent {
                id: evidence.id.clone(),
                kind: Some(String::from("probability:evidence")),
                content: Some(evidence.to_links_notation()),
                sent_at: Some(evidence.recorded_at.clone()),
                evidence: vec![format!("probability:evidence:{}", evidence.id)],
                ..MemoryEvent::default()
            })?;
            inserted += 1;
        }
        Ok(inserted)
    }
}

/// A candidate whose posterior can be ranked by symbolic probability evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbabilityCandidate {
    pub target: String,
    pub prior_score: f32,
}

impl ProbabilityCandidate {
    #[must_use]
    pub fn new(target: impl Into<String>, prior_score: f32) -> Self {
        Self {
            target: target.into(),
            prior_score: finite_or_zero(prior_score),
        }
    }
}

/// Ranking controls shared by Bayesian and Markov-style helpers.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ProbabilityRankingConfig {
    pub temperature: f32,
    pub offline: bool,
    pub markov_from: Option<String>,
}

/// One ranked candidate with inspectable prior/evidence/posterior fields.
#[derive(Debug, Clone, PartialEq)]
pub struct RankedProbabilityCandidate {
    pub target: String,
    pub prior_score: f32,
    pub evidence_weight: f32,
    pub posterior_score: f32,
    pub probability: f32,
}

/// Deterministic ranking result.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbabilityRanking {
    pub ranked: Vec<RankedProbabilityCandidate>,
    pub margin: f32,
}

impl ProbabilityRanking {
    #[must_use]
    pub fn probability_for(&self, target: &str) -> Option<f32> {
        self.ranked
            .iter()
            .find(|candidate| candidate.target == target)
            .map(|candidate| candidate.probability)
    }

    #[must_use]
    pub fn trace_summary(&self) -> String {
        self.ranked
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{:.6}:{:.6}",
                    candidate.target, candidate.posterior_score, candidate.probability
                )
            })
            .collect::<Vec<_>>()
            .join("|")
    }
}

#[must_use]
pub fn rank_probability_candidates(
    candidates: &[ProbabilityCandidate],
    store: &ProbabilityStore,
    config: ProbabilityRankingConfig,
) -> ProbabilityRanking {
    if candidates.is_empty() {
        return ProbabilityRanking {
            ranked: Vec::new(),
            margin: 0.0,
        };
    }

    let ProbabilityRankingConfig {
        temperature,
        offline,
        markov_from,
    } = config;
    let markov_from = markov_from.as_deref();
    let mut ranked = candidates
        .iter()
        .map(|candidate| {
            let evidence_weight = store.target_weight(&candidate.target, offline, markov_from);
            let posterior_score = candidate.prior_score + evidence_weight;
            RankedProbabilityCandidate {
                target: candidate.target.clone(),
                prior_score: candidate.prior_score,
                evidence_weight,
                posterior_score,
                probability: 0.0,
            }
        })
        .collect::<Vec<_>>();

    let probabilities = softmax_scores(
        &ranked
            .iter()
            .map(|candidate| candidate.posterior_score)
            .collect::<Vec<_>>(),
        temperature,
    );
    for (candidate, probability) in ranked.iter_mut().zip(probabilities) {
        candidate.probability = probability;
    }

    ranked.sort_by(|left, right| {
        right
            .probability
            .total_cmp(&left.probability)
            .then_with(|| right.posterior_score.total_cmp(&left.posterior_score))
            .then_with(|| left.target.cmp(&right.target))
    });
    let margin = match ranked.as_slice() {
        [first, second, ..] => first.probability - second.probability,
        [_] => 1.0,
        [] => 0.0,
    };

    ProbabilityRanking { ranked, margin }
}

fn softmax_scores(scores: &[f32], temperature: f32) -> Vec<f32> {
    if scores.is_empty() {
        return Vec::new();
    }
    let temperature = finite_clamped(temperature, 0.0, 1.0);
    if temperature <= f32::EPSILON {
        let mut probabilities = vec![0.0; scores.len()];
        probabilities[highest_score_index(scores)] = 1.0;
        return probabilities;
    }

    let max_score = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let weights = scores
        .iter()
        .map(|score| ((*score - max_score) / temperature).exp())
        .collect::<Vec<_>>();
    let total = weights.iter().sum::<f32>();
    if !total.is_finite() || total <= f32::EPSILON {
        let uniform = 1.0 / usize_to_f32(scores.len());
        return vec![uniform; scores.len()];
    }
    weights.iter().map(|weight| *weight / total).collect()
}

fn highest_score_index(scores: &[f32]) -> usize {
    scores
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map_or(0, |(index, _)| index)
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn finite_clamped(value: f32, min: f32, max: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        min
    }
}

fn usize_to_f32(value: usize) -> f32 {
    let bounded = u16::try_from(value).unwrap_or(u16::MAX);
    f32::from(bounded.max(1))
}
