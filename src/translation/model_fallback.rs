//! Optional small-model advisor for prompt formalization.
//!
//! The deterministic formalizer remains authoritative: browser or desktop
//! models may only choose from the already-built candidate list. This module
//! keeps the opt-in fallback testable without bundling or downloading any model
//! runtime in the Rust crate.

use std::fmt::Write as _;

use serde::Deserialize;

use super::formalization::FormalizationCandidate;
use super::selection::{
    select_formalization_candidate, FormalizationSelection, FormalizationSelectionConfig,
};

const SHADER_F16_FEATURE: &str = "shader-f16";
const ADVICE_MIN_CONFIDENCE: f32 = 0.5;
const ADVICE_SCORE_BOOST: u16 = 25;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SmallModelHardwareProfile {
    pub webgpu_available: bool,
    pub shader_f16_available: bool,
    pub device_memory_mb: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmallFormalizationModel {
    pub id: &'static str,
    pub label: &'static str,
    pub runtime: &'static str,
    pub source_url: &'static str,
    pub public_rating: u32,
    pub rating_basis: &'static str,
    pub vram_required_mb: u32,
    pub estimated_download_mb: Option<u32>,
    pub low_resource: bool,
    pub required_features: &'static [&'static str],
    pub context_window: u32,
}

const REQUIRED_SHADER_F16: &[&str] = &[SHADER_F16_FEATURE];
const NO_REQUIRED_FEATURES: &[&str] = &[];

const SMALL_FORMALIZATION_MODEL_CATALOG: &[SmallFormalizationModel] = &[
    SmallFormalizationModel {
        id: "SmolLM2-360M-Instruct-q4f16_1-MLC",
        label: "SmolLM2 360M Instruct q4f16",
        runtime: "WebLLM",
        source_url: "https://huggingface.co/mlc-ai/SmolLM2-360M-Instruct-q4f16_1-MLC",
        public_rating: 80_150,
        rating_basis: "Hugging Face downloads, captured for issue #483 research",
        vram_required_mb: 377,
        estimated_download_mb: None,
        low_resource: true,
        required_features: REQUIRED_SHADER_F16,
        context_window: 4096,
    },
    SmallFormalizationModel {
        id: "Qwen2.5-0.5B-Instruct-q4f16_1-MLC",
        label: "Qwen2.5 0.5B Instruct q4f16",
        runtime: "WebLLM",
        source_url: "https://huggingface.co/mlc-ai/Qwen2.5-0.5B-Instruct-q4f16_1-MLC",
        public_rating: 34_924,
        rating_basis: "Hugging Face downloads plus likes, captured for issue #483 research",
        vram_required_mb: 945,
        estimated_download_mb: None,
        low_resource: true,
        required_features: NO_REQUIRED_FEATURES,
        context_window: 4096,
    },
    SmallFormalizationModel {
        id: "SmolLM2-1.7B-Instruct-q4f16_1-MLC",
        label: "SmolLM2 1.7B Instruct q4f16",
        runtime: "WebLLM",
        source_url: "https://huggingface.co/mlc-ai/SmolLM2-1.7B-Instruct-q4f16_1-MLC",
        public_rating: 503,
        rating_basis: "Hugging Face downloads, captured for issue #483 research",
        vram_required_mb: 1775,
        estimated_download_mb: None,
        low_resource: true,
        required_features: REQUIRED_SHADER_F16,
        context_window: 4096,
    },
    SmallFormalizationModel {
        id: "Phi-3.5-mini-instruct-q4f16_1-MLC-1k",
        label: "Phi-3.5 mini Instruct q4f16 1k",
        runtime: "WebLLM",
        source_url: "https://huggingface.co/mlc-ai/Phi-3.5-mini-instruct-q4f16_1-MLC",
        public_rating: 0,
        rating_basis: "WebLLM low-resource alias; no separate Hugging Face model rating",
        vram_required_mb: 2521,
        estimated_download_mb: None,
        low_resource: true,
        required_features: NO_REQUIRED_FEATURES,
        context_window: 1024,
    },
];

#[must_use]
pub const fn small_formalization_model_catalog() -> &'static [SmallFormalizationModel] {
    SMALL_FORMALIZATION_MODEL_CATALOG
}

#[must_use]
pub fn available_small_formalization_models(
    profile: SmallModelHardwareProfile,
) -> Vec<&'static SmallFormalizationModel> {
    let mut models = small_formalization_model_catalog()
        .iter()
        .filter(|model| model_fits_hardware(model, profile))
        .collect::<Vec<_>>();
    models.sort_by(|left, right| {
        right
            .public_rating
            .cmp(&left.public_rating)
            .then_with(|| left.vram_required_mb.cmp(&right.vram_required_mb))
            .then_with(|| left.id.cmp(right.id))
    });
    models
}

fn model_fits_hardware(
    model: &SmallFormalizationModel,
    profile: SmallModelHardwareProfile,
) -> bool {
    if !profile.webgpu_available {
        return false;
    }
    if !model
        .required_features
        .iter()
        .all(|feature| feature_supported(feature, profile))
    {
        return false;
    }
    profile
        .device_memory_mb
        .map_or(model.low_resource, |memory_mb| {
            model.vram_required_mb <= memory_mb
        })
}

fn feature_supported(feature: &str, profile: SmallModelHardwareProfile) -> bool {
    match feature {
        SHADER_F16_FEATURE => profile.shader_f16_available,
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormalizationModelAdvice {
    pub model_id: String,
    pub selected_option: String,
    pub confidence: f32,
    pub raw_output: String,
}

#[derive(Debug, Deserialize)]
struct JsonAdvice {
    #[serde(default)]
    selected_option: String,
    #[serde(default, rename = "selectedOption")]
    selected_option_camel: String,
    #[serde(default)]
    confidence: Option<f32>,
}

#[must_use]
pub fn parse_formalization_model_advice(
    model_id: &str,
    raw_output: &str,
) -> FormalizationModelAdvice {
    let parsed = serde_json::from_str::<JsonAdvice>(raw_output).ok();
    let selected_option = parsed
        .as_ref()
        .map(|advice| {
            if advice.selected_option.trim().is_empty() {
                advice.selected_option_camel.trim()
            } else {
                advice.selected_option.trim()
            }
        })
        .filter(|selected| !selected.is_empty())
        .map_or_else(
            || raw_output.trim().trim_matches('"').to_owned(),
            str::to_owned,
        );
    let confidence = parsed.and_then(|advice| advice.confidence).unwrap_or(0.5);
    FormalizationModelAdvice {
        model_id: model_id.to_owned(),
        selected_option,
        confidence,
        raw_output: raw_output.to_owned(),
    }
}

#[must_use]
pub fn formalization_model_option_id(index: usize) -> String {
    format!("option_{}", index + 1)
}

#[must_use]
pub fn build_formalization_model_prompt(
    user_prompt: &str,
    candidates: &[FormalizationCandidate],
) -> String {
    let mut output = String::new();
    let _ = writeln!(
        output,
        "You are an optional formalization advisor. Select the best existing formalization option for the user prompt."
    );
    let _ = writeln!(
        output,
        "Return JSON only: {{\"selected_option\":\"option_1\",\"confidence\":0.0}}."
    );
    let _ = writeln!(
        output,
        "Do not create new anchors, terms, predicates, or explanations."
    );
    let _ = writeln!(output, "User prompt: {user_prompt}");
    let _ = writeln!(output, "Options:");
    for (index, candidate) in candidates.iter().enumerate() {
        let _ = writeln!(output, "{}:", formalization_model_option_id(index));
        let _ = writeln!(output, "  summary: {}", candidate.compact_summary());
        let _ = writeln!(output, "  score: {}", candidate.score);
        let _ = writeln!(output, "  links_notation:");
        for line in candidate.to_links_notation().lines() {
            let _ = writeln!(output, "    {line}");
        }
    }
    output
}

#[must_use]
pub fn apply_formalization_model_advice(
    candidates: &[FormalizationCandidate],
    fallback_enabled: bool,
    advice: Option<&FormalizationModelAdvice>,
) -> Vec<FormalizationCandidate> {
    let mut ranked = candidates.to_vec();
    if !fallback_enabled || ranked.len() < 2 {
        return ranked;
    }
    let Some(advice) = advice else {
        return ranked;
    };
    if !advice.confidence.is_finite() || advice.confidence < ADVICE_MIN_CONFIDENCE {
        return ranked;
    }
    let Some(index) = advised_candidate_index(&ranked, &advice.selected_option) else {
        return ranked;
    };

    let mut selected = ranked.remove(index);
    let top_score = ranked
        .iter()
        .map(|candidate| candidate.score)
        .max()
        .unwrap_or(selected.score)
        .max(selected.score);
    selected.score = selected
        .score
        .max(top_score.saturating_add(ADVICE_SCORE_BOOST));
    ranked.insert(0, selected);
    ranked
}

#[must_use]
pub fn select_formalization_candidate_with_model_advice(
    candidates: &[FormalizationCandidate],
    config: FormalizationSelectionConfig,
    impulse: &str,
    fallback_enabled: bool,
    advice: Option<&FormalizationModelAdvice>,
) -> FormalizationSelection {
    let ranked = apply_formalization_model_advice(candidates, fallback_enabled, advice);
    select_formalization_candidate(&ranked, config, impulse)
}

fn advised_candidate_index(candidates: &[FormalizationCandidate], selected: &str) -> Option<usize> {
    let selected = selected.trim();
    if selected.is_empty() {
        return None;
    }
    candidates
        .iter()
        .enumerate()
        .find_map(|(index, candidate)| {
            let summary = candidate.compact_summary();
            let target = format!("formalization:{summary}");
            if selected == formalization_model_option_id(index)
                || selected == summary
                || selected == target
            {
                Some(index)
            } else {
                None
            }
        })
}
