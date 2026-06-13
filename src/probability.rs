//! Link-native symbolic probability evidence and deterministic ranking.
//!
//! This module intentionally does not perform neural-network inference. It
//! treats probability evidence as ordinary append-only Links Notation records:
//! each record points at a symbolic target, carries provenance and a fixed
//! timestamp supplied by the caller, and can be replayed into the same event
//! log / link-store projection as the rest of the solver trace.

use std::cmp::Ordering;
use std::collections::BTreeMap;

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

    /// Reinforce a whole episode's trajectory in one shot — the deterministic,
    /// append-only counterpart of the paper's global feedback (episode-wide
    /// one-shot update) from arXiv:2605.00940.
    ///
    /// Given an ordered `path` of visited states `[s0, s1, ..., sn]`, this
    /// appends one [`ProbabilityModel::MarkovTransition`] record per adjacent
    /// pair `(s_i -> s_{i+1})`, each carrying the shared episode `reward` as its
    /// utility `U` and the same `provenance`/`recorded_at` stamp, so the entire
    /// episode is reinforced together rather than transition by transition. The
    /// recorded evidence is then visible to [`Self::target_weight`] /
    /// [`Self::target_evidence_count`] under the matching `markov_from` state,
    /// exactly like any other transition observation.
    ///
    /// Returns the ids of the appended records in path order. A `path` with
    /// fewer than two states has no transitions, so it records nothing and
    /// returns an empty vector.
    pub fn reinforce_transition_path<S: AsRef<str>>(
        &mut self,
        path: &[S],
        reward: f32,
        provenance: impl Into<String>,
        recorded_at: impl Into<String>,
    ) -> Vec<String> {
        let provenance = provenance.into();
        let recorded_at = recorded_at.into();
        path.windows(2)
            .map(|pair| {
                let from = pair[0].as_ref();
                let to = pair[1].as_ref();
                self.record(
                    ProbabilityEvidence::symbolic(
                        to,
                        format!("episode_transition:{from}->{to}"),
                        reward,
                        provenance.clone(),
                        recorded_at.clone(),
                    )
                    .with_model(ProbabilityModel::MarkovTransition)
                    .with_transition_from(from),
                )
            })
            .collect()
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

    /// Count the number of append-only observations that support `target`.
    ///
    /// This is the symbolic analogue of the evidence count `C` from Kolonin's
    /// "Interpretable Experiential Learning" (arXiv:2605.00940): every recorded
    /// observation is one unit of evidence for a transition/answer, kept
    /// separate from the accumulated utility (`target_weight`) so that a rarely
    /// seen high-weight transition can be told apart from a frequently confirmed
    /// one. The same offline and Markov-state filters as [`Self::target_weight`]
    /// apply, so utility and count always describe the same evidence subset.
    #[must_use]
    pub fn target_evidence_count(
        &self,
        target: &str,
        offline: bool,
        markov_from: Option<&str>,
    ) -> usize {
        self.records
            .iter()
            .filter(|evidence| evidence.target == target)
            .filter(|evidence| evidence.usable_offline(offline))
            .filter(|evidence| evidence.applies_to_markov_state(markov_from))
            .count()
    }

    /// Reuse the nearest stored target's evidence when `target` has none of its
    /// own — the symbolic counterpart of the paper's cosine-similarity `SS`
    /// fallback over stored situations.
    ///
    /// Among the distinct targets that carry usable evidence under the same
    /// offline/Markov filters (excluding `target` itself), this returns the one
    /// whose [`symbolic_cosine_similarity`] to `target` is highest and at least
    /// `threshold`. Ties are broken by target name so the choice is
    /// deterministic. Returns `None` when nothing clears the threshold.
    #[must_use]
    pub fn nearest_similar_evidence(
        &self,
        target: &str,
        offline: bool,
        markov_from: Option<&str>,
        threshold: f32,
    ) -> Option<SimilarEvidence> {
        let mut seen: Vec<&str> = Vec::new();
        let mut best: Option<SimilarEvidence> = None;
        for evidence in &self.records {
            let other = evidence.target.as_str();
            if other == target || seen.contains(&other) {
                continue;
            }
            seen.push(other);
            let count = self.target_evidence_count(other, offline, markov_from);
            if count == 0 {
                continue;
            }
            let similarity = symbolic_cosine_similarity(target, other);
            if similarity < threshold {
                continue;
            }
            let candidate = SimilarEvidence {
                matched_target: other.to_owned(),
                weight: self.target_weight(other, offline, markov_from),
                count,
                similarity,
            };
            let replace = best.as_ref().map_or(true, |current| {
                match similarity.total_cmp(&current.similarity) {
                    Ordering::Greater => true,
                    Ordering::Equal => candidate.matched_target < current.matched_target,
                    Ordering::Less => false,
                }
            });
            if replace {
                best = Some(candidate);
            }
        }
        best
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

/// Evidence borrowed from the nearest stored target by the `SS` fallback.
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarEvidence {
    /// The stored target whose evidence is being reused.
    pub matched_target: String,
    /// The matched target's accumulated utility `U` (before similarity scaling).
    pub weight: f32,
    /// The matched target's evidence count `C`.
    pub count: usize,
    /// Symbolic cosine similarity between the queried and matched targets.
    pub similarity: f32,
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
///
/// The optional fields below port the decision-policy hyperparameters from
/// Kolonin's "Interpretable Experiential Learning" (arXiv:2605.00940). Their
/// defaults (`counted_utility = false`, both thresholds `None`) reproduce the
/// paper's recommended `CU = False`, `TU = 0`, `TC = 1` baseline, which is
/// exactly the additive behavior this module shipped before they were added, so
/// existing callers are unaffected unless they opt in.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ProbabilityRankingConfig {
    pub temperature: f32,
    pub offline: bool,
    pub markov_from: Option<String>,
    /// Counted-utility policy (the paper's `CU`). When `true`, a candidate's
    /// learned utility is scaled by its evidence count (`U` becomes `U * C`), so
    /// a frequently confirmed transition outranks a rarely seen one of equal
    /// per-observation weight. When `false` the ranking uses the accumulated
    /// utility directly (`argmax(U)`).
    pub counted_utility: bool,
    /// Minimum accumulated transition utility (the paper's `TU`). A candidate
    /// whose evidence weight is below this threshold has its learned evidence
    /// withheld and falls back to its structural prior. `None` disables the gate.
    pub min_transition_utility: Option<f32>,
    /// Minimum evidence count (the paper's `TC`). A candidate observed fewer
    /// times than this threshold has its learned evidence withheld and falls
    /// back to its structural prior. `None` disables the gate.
    pub min_transition_count: Option<usize>,
    /// Similarity threshold for the inexact-state fallback (the paper's `SS`).
    /// When a candidate has *no* exact evidence of its own, the ranker reuses
    /// the nearest stored target whose symbolic cosine similarity to the
    /// candidate is at least this threshold, scaling the borrowed utility by the
    /// similarity. `None` disables the fallback, so only exact evidence counts.
    pub similarity_threshold: Option<f32>,
}

impl ProbabilityRankingConfig {
    /// Overlay the paper's decision-policy hyperparameters (`CU`/`TU`/`TC`/`SS`)
    /// onto this config, leaving the deterministic transport knobs
    /// (`temperature`, `offline`, `markov_from`) untouched. This is the seam
    /// every call site uses to honour a centrally configured
    /// [`ProbabilityDecisionPolicy`] without re-spelling each field.
    #[must_use]
    pub const fn with_decision_policy(mut self, policy: ProbabilityDecisionPolicy) -> Self {
        self.counted_utility = policy.counted_utility;
        self.min_transition_utility = policy.min_transition_utility;
        self.min_transition_count = policy.min_transition_count;
        self.similarity_threshold = policy.similarity_threshold;
        self
    }

    /// Extract the decision-policy hyperparameters from this config.
    #[must_use]
    pub const fn decision_policy(&self) -> ProbabilityDecisionPolicy {
        ProbabilityDecisionPolicy {
            counted_utility: self.counted_utility,
            min_transition_utility: self.min_transition_utility,
            min_transition_count: self.min_transition_count,
            similarity_threshold: self.similarity_threshold,
        }
    }
}

/// Interpretable decision-policy hyperparameters from Kolonin's paper.
///
/// These are the `CU`/`TU`/`TC`/`SS` knobs of "Interpretable Experiential
/// Learning" (arXiv:2605.00940), grouped as one `Copy` unit so a single policy
/// can be threaded through every ranking call site instead of being re-spelled
/// field by field.
///
/// The default is the paper's recommended baseline (`CU=False`, `TU=0`,
/// `TC=1`, no similarity fallback), which is exactly the additive behaviour
/// this module shipped before the policy existed, so a defaulted policy is a
/// no-op overlay.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ProbabilityDecisionPolicy {
    /// Counted-utility switch `CU`: rank by `argmax(U·C)` instead of `argmax(U)`.
    pub counted_utility: bool,
    /// Transition-utility threshold `TU`: withhold evidence below this utility.
    pub min_transition_utility: Option<f32>,
    /// Transition-count threshold `TC`: withhold evidence below this count.
    pub min_transition_count: Option<usize>,
    /// Inexact-state similarity threshold `SS`: reuse the nearest stored target's
    /// evidence (scaled by similarity) when a candidate has none of its own.
    pub similarity_threshold: Option<f32>,
}

/// One ranked candidate with inspectable prior/evidence/posterior fields.
///
/// `evidence_weight` is the accumulated utility `U` and `evidence_count` is the
/// evidence count `C` for this candidate (after any `TU`/`TC` gating). Keeping
/// both visible is what makes a decision locally interpretable in the sense of
/// arXiv:2605.00940: every ranked option carries the utility and the number of
/// observations that produced it.
#[derive(Debug, Clone, PartialEq)]
pub struct RankedProbabilityCandidate {
    pub target: String,
    pub prior_score: f32,
    pub evidence_weight: f32,
    pub evidence_count: usize,
    /// Provenance of the evidence behind this candidate: `1.0` when it is the
    /// candidate's own (exact) evidence, or the symbolic cosine similarity
    /// `< 1.0` when it was borrowed from the nearest stored target through the
    /// `SS` fallback. Surfaced so a fallback-driven decision stays interpretable.
    pub similarity: f32,
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
        counted_utility,
        min_transition_utility,
        min_transition_count,
        similarity_threshold,
    } = config;
    let markov_from = markov_from.as_deref();
    let mut ranked = candidates
        .iter()
        .map(|candidate| {
            let direct_weight = store.target_weight(&candidate.target, offline, markov_from);
            let direct_count = store.target_evidence_count(&candidate.target, offline, markov_from);
            // State-similarity fallback (SS): when a target carries no direct
            // evidence we borrow it from the most similar previously seen target,
            // attenuated by the symbolic cosine similarity between their names.
            // This mirrors the paper's `SS` inexact-match path without changing
            // any directly-evidenced decision.
            let (raw_weight, raw_count, similarity) = if direct_count == 0 {
                if let Some(found) = similarity_threshold.and_then(|threshold| {
                    store.nearest_similar_evidence(
                        &candidate.target,
                        offline,
                        markov_from,
                        threshold,
                    )
                }) {
                    (
                        found.weight * found.similarity,
                        found.count,
                        found.similarity,
                    )
                } else {
                    (direct_weight, direct_count, 1.0)
                }
            } else {
                (direct_weight, direct_count, 1.0)
            };
            // Transition utility/count thresholds (TU/TC): an under-evidenced
            // transition is not trusted as a learned candidate, so its evidence
            // is withheld and the candidate falls back to its structural prior.
            let below_utility =
                min_transition_utility.is_some_and(|threshold| raw_weight < threshold);
            let below_count = min_transition_count.is_some_and(|threshold| raw_count < threshold);
            let (evidence_weight, evidence_count, similarity) = if below_utility || below_count {
                (0.0, 0, 1.0)
            } else {
                (raw_weight, raw_count, similarity)
            };
            // Counted-utility policy (CU): scale the learned utility by how many
            // times the transition was confirmed (`U` becomes `U * C`).
            let contribution = if counted_utility {
                evidence_weight * count_to_f32(evidence_count)
            } else {
                evidence_weight
            };
            let posterior_score = candidate.prior_score + contribution;
            RankedProbabilityCandidate {
                target: candidate.target.clone(),
                prior_score: candidate.prior_score,
                evidence_weight,
                evidence_count,
                similarity,
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

/// Convert an evidence count into a scaling factor for the counted-utility
/// policy. Unlike [`usize_to_f32`], a count of zero stays `0.0` (an unevidenced
/// candidate contributes nothing), and counts are saturated at `u16::MAX` to
/// avoid precision loss for absurdly large symbolic histories.
fn count_to_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).unwrap_or(u16::MAX))
}

/// Deterministic bag-of-words cosine similarity between two symbolic targets.
///
/// This is the non-neural counterpart of the paper's `SS` state-similarity
/// score. Names are tokenized on any non-alphanumeric boundary and lowercased,
/// then compared as multisets of tokens. The result lies in `0.0..=1.0`; it is
/// `0.0` when either side has no tokens and `1.0` for identical token bags.
#[must_use]
pub fn symbolic_cosine_similarity(a: &str, b: &str) -> f32 {
    let left = tokenize_symbolic(a);
    let right = tokenize_symbolic(b);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let left_counts = bag_of_words(&left);
    let right_counts = bag_of_words(&right);
    let mut dot = 0.0f32;
    for (token, left_count) in &left_counts {
        if let Some(right_count) = right_counts.get(token) {
            dot = count_to_f32(*left_count).mul_add(count_to_f32(*right_count), dot);
        }
    }
    let left_norm = vector_norm(&left_counts);
    let right_norm = vector_norm(&right_counts);
    if left_norm <= f32::EPSILON || right_norm <= f32::EPSILON {
        return 0.0;
    }
    (dot / (left_norm * right_norm)).clamp(0.0, 1.0)
}

fn tokenize_symbolic(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn bag_of_words(tokens: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for token in tokens {
        *counts.entry(token.clone()).or_insert(0) += 1;
    }
    counts
}

fn vector_norm(counts: &BTreeMap<String, usize>) -> f32 {
    let sum_of_squares = counts
        .values()
        .map(|count| {
            let value = count_to_f32(*count);
            value * value
        })
        .sum::<f32>();
    sum_of_squares.sqrt()
}
