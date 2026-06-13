//! Symbolic probabilistic reasoning tests.
//!
//! Issue #279 requires probability evidence to remain link-native and
//! deterministic: symbolic evidence can change candidate ranking, but it must
//! not introduce neural inference, hidden weights, or nondeterministic replay.
//! Issue #449 layers the interpretable experiential-learning mechanisms
//! (counted utility, thresholds, similarity fallback, episode feedback) on top.
//!
//! The suite is split into thematic submodules to stay under the per-file line
//! cap; each submodule pulls the shared imports and helper through `super::*`.

use formal_ai::probability::{
    rank_probability_candidates, symbolic_cosine_similarity, ProbabilityCandidate,
    ProbabilityDecisionPolicy, ProbabilityEvidence, ProbabilityModel, ProbabilityRankingConfig,
    ProbabilitySourceProvenance, ProbabilityStore,
};
use formal_ai::translation::{
    formalization_probability_target, formalize_prompt_candidates, select_formalization_candidate,
    select_formalization_candidate_with_policy,
    select_formalization_candidate_with_probability_store, FormalizationDecision,
    FormalizationSelectionConfig,
};
use formal_ai::{EventLog, MemoryStore, SolverConfig, UniversalSolver};

const fn ambiguous_config() -> FormalizationSelectionConfig {
    FormalizationSelectionConfig {
        temperature: 0.7,
        guess_probability: 0.0,
        questioning_rigor: 1.0,
    }
}

mod counted_utility;
mod decision_policy;
mod evidence_core;
mod multilingual;
mod ranking_mechanics;
mod similarity_fallback;
