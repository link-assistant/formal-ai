//! Temperature-based selection over formalization candidates.
//!
//! Scores in the formalization layer are stored as 0..1000 basis points. This
//! module normalizes them to 0.0..1.0 before applying softmax so the public
//! `SolverConfig::temperature` range stays useful.

use crate::translation::{FormalizationCandidate, FormalizationRole};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormalizationSelectionConfig {
    pub temperature: f32,
    pub guess_probability: f32,
    pub questioning_rigor: f32,
}

impl FormalizationSelectionConfig {
    #[must_use]
    pub fn normalized(self) -> Self {
        Self {
            temperature: finite_clamped(self.temperature, 0.0, 1.0),
            guess_probability: finite_clamped(self.guess_probability, 0.0, 1.0),
            questioning_rigor: finite_clamped(self.questioning_rigor, 0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormalizationSelectionReason {
    OnlyCandidate,
    ClearlyBest,
    GuessedUnderAmbiguity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormalizationDecision {
    NoCandidate,
    Selected {
        index: usize,
        probability: f32,
        margin: f32,
        epsilon: f32,
        reason: FormalizationSelectionReason,
    },
    Clarify {
        question: String,
        top_index: usize,
        runner_up_index: usize,
        margin: f32,
        epsilon: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormalizationSelection {
    pub candidates: Vec<FormalizationCandidate>,
    pub probabilities: Vec<f32>,
    pub decision: FormalizationDecision,
}

impl FormalizationSelection {
    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        match &self.decision {
            FormalizationDecision::Selected { index, .. } => Some(*index),
            FormalizationDecision::NoCandidate | FormalizationDecision::Clarify { .. } => None,
        }
    }

    #[must_use]
    pub fn selected_candidate(&self) -> Option<&FormalizationCandidate> {
        self.selected_index()
            .and_then(|index| self.candidates.get(index))
    }

    #[must_use]
    pub const fn is_clarification(&self) -> bool {
        matches!(&self.decision, FormalizationDecision::Clarify { .. })
    }
}

#[must_use]
pub fn softmax_formalization_scores(
    candidates: &[FormalizationCandidate],
    temperature: f32,
) -> Vec<f32> {
    if candidates.is_empty() {
        return Vec::new();
    }

    let temperature = finite_clamped(temperature, 0.0, 1.0);
    if temperature <= f32::EPSILON {
        let mut probabilities = vec![0.0; candidates.len()];
        probabilities[highest_score_index(candidates)] = 1.0;
        return probabilities;
    }

    let logits = candidates
        .iter()
        .map(|candidate| f32::from(candidate.score) / 1000.0)
        .collect::<Vec<_>>();
    let max_logit = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let weights = logits
        .iter()
        .map(|logit| ((*logit - max_logit) / temperature).exp())
        .collect::<Vec<_>>();
    let total = weights.iter().sum::<f32>();
    if !total.is_finite() || total <= f32::EPSILON {
        let uniform = 1.0 / usize_to_f32(candidates.len());
        return vec![uniform; candidates.len()];
    }
    weights.iter().map(|weight| *weight / total).collect()
}

#[must_use]
pub fn select_formalization_candidate(
    candidates: &[FormalizationCandidate],
    config: FormalizationSelectionConfig,
    impulse: &str,
) -> FormalizationSelection {
    let config = config.normalized();
    let candidates = candidates.to_vec();
    let probabilities = softmax_formalization_scores(&candidates, config.temperature);
    if candidates.is_empty() {
        return FormalizationSelection {
            candidates,
            probabilities,
            decision: FormalizationDecision::NoCandidate,
        };
    }
    if candidates.len() == 1 {
        return FormalizationSelection {
            candidates,
            probabilities,
            decision: FormalizationDecision::Selected {
                index: 0,
                probability: 1.0,
                margin: 1.0,
                epsilon: probability_margin_epsilon(config.questioning_rigor),
                reason: FormalizationSelectionReason::OnlyCandidate,
            },
        };
    }

    let ranked = ranked_indices(&candidates, &probabilities);
    let top_index = ranked[0];
    let runner_up_index = ranked[1];
    let margin = probabilities[top_index] - probabilities[runner_up_index];
    let epsilon = probability_margin_epsilon(config.questioning_rigor);

    if margin > epsilon {
        let probability = probabilities[top_index];
        return FormalizationSelection {
            candidates,
            probabilities,
            decision: FormalizationDecision::Selected {
                index: top_index,
                probability,
                margin,
                epsilon,
                reason: FormalizationSelectionReason::ClearlyBest,
            },
        };
    }

    if should_clarify(config) {
        return FormalizationSelection {
            decision: FormalizationDecision::Clarify {
                question: clarifying_question(&candidates, top_index, runner_up_index),
                top_index,
                runner_up_index,
                margin,
                epsilon,
            },
            candidates,
            probabilities,
        };
    }

    let index = sample_index(
        &probabilities,
        impulse,
        &selection_salt(&candidates, config),
    );
    let probability = probabilities[index];
    FormalizationSelection {
        candidates,
        probabilities,
        decision: FormalizationDecision::Selected {
            index,
            probability,
            margin,
            epsilon,
            reason: FormalizationSelectionReason::GuessedUnderAmbiguity,
        },
    }
}

fn finite_clamped(value: f32, min: f32, max: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        min
    }
}

fn highest_score_index(candidates: &[FormalizationCandidate]) -> usize {
    candidates
        .iter()
        .enumerate()
        .max_by_key(|(_, candidate)| candidate.score)
        .map_or(0, |(index, _)| index)
}

fn ranked_indices(candidates: &[FormalizationCandidate], probabilities: &[f32]) -> Vec<usize> {
    let mut indices = (0..candidates.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| {
        probabilities[*right]
            .total_cmp(&probabilities[*left])
            .then_with(|| candidates[*right].score.cmp(&candidates[*left].score))
            .then_with(|| left.cmp(right))
    });
    indices
}

fn probability_margin_epsilon(questioning_rigor: f32) -> f32 {
    0.23f32.mul_add(finite_clamped(questioning_rigor, 0.0, 1.0), 0.02)
}

fn should_clarify(config: FormalizationSelectionConfig) -> bool {
    config.questioning_rigor * (1.0 - config.guess_probability) > 0.5
}

fn sample_index(probabilities: &[f32], impulse: &str, salt: &str) -> usize {
    let draw = seeded_unit_interval(impulse, salt);
    let mut cumulative = 0.0;
    for (index, probability) in probabilities.iter().enumerate() {
        cumulative += *probability;
        if draw <= cumulative {
            return index;
        }
    }
    probabilities.len().saturating_sub(1)
}

fn selection_salt(
    candidates: &[FormalizationCandidate],
    config: FormalizationSelectionConfig,
) -> String {
    let summaries = candidates
        .iter()
        .map(FormalizationCandidate::compact_summary)
        .collect::<Vec<_>>()
        .join("|");
    format!(
        "temperature={:.4};guess={:.4};rigor={:.4};{summaries}",
        config.temperature, config.guess_probability, config.questioning_rigor
    )
}

fn seeded_unit_interval(impulse: &str, salt: &str) -> f32 {
    let hash = fnv1a64(&format!("{impulse}\n{salt}"));
    let bucket = u16::try_from(hash >> 48).unwrap_or(u16::MAX);
    f32::from(bucket) / f32::from(u16::MAX)
}

fn fnv1a64(value: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn usize_to_f32(value: usize) -> f32 {
    let bounded = u16::try_from(value).unwrap_or(u16::MAX);
    f32::from(bounded.max(1))
}

fn clarifying_question(
    candidates: &[FormalizationCandidate],
    top_index: usize,
    runner_up_index: usize,
) -> String {
    let top = &candidates[top_index];
    let runner_up = &candidates[runner_up_index];
    if let Some(question) = predicate_clarifying_question(top, runner_up) {
        return question;
    }
    format!(
        "Which interpretation did you mean: {} or {}?",
        top.compact_summary(),
        runner_up.compact_summary()
    )
}

fn predicate_clarifying_question(
    top: &FormalizationCandidate,
    runner_up: &FormalizationCandidate,
) -> Option<String> {
    let top_subject = top.slot(FormalizationRole::Subject)?;
    let top_predicate = top.slot(FormalizationRole::Predicate)?;
    let top_object = top.slot(FormalizationRole::Object)?;
    let other_subject = runner_up.slot(FormalizationRole::Subject)?;
    let other_predicate = runner_up.slot(FormalizationRole::Predicate)?;
    let other_object = runner_up.slot(FormalizationRole::Object)?;

    if top_subject.surface != other_subject.surface || top_object.surface != other_object.surface {
        return None;
    }
    if top_predicate.anchor.id == other_predicate.anchor.id {
        return None;
    }

    Some(format!(
        "Should I read \"{}\" as \"{} {} {}\" or \"{} {} {}\"?",
        top.source_text,
        top_subject.surface,
        top_predicate.anchor.label,
        top_object.surface,
        other_subject.surface,
        other_predicate.anchor.label,
        other_object.surface
    ))
}
