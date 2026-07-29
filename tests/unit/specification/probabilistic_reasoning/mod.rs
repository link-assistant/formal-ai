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

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

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
use formal_ai::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::{EventLog, MemoryStore, SolverConfig, UniversalSolver};

static SOURCE_FIXTURE_IDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
struct ProbabilitySourceTransport;

impl SourceTransport for ProbabilitySourceTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        Ok(format!("probability source fixture for {url}\n").into_bytes())
    }
}

const fn probability_source_time() -> u64 {
    1_753_444_800
}

fn captured_probability_source(url: &str, cached: bool) -> ProbabilitySourceProvenance {
    let cache = std::env::temp_dir().join(format!(
        "formal-ai-probability-source-{}-{}",
        std::process::id(),
        SOURCE_FIXTURE_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let online = CachedSourceClient::new(&cache, ProbabilitySourceTransport)
        .with_online(true)
        .with_clock(probability_source_time);
    let live = online.fetch(url).expect("capture probability fixture");
    let capture = if cached {
        CachedSourceClient::new(&cache, ProbabilitySourceTransport)
            .fetch(url)
            .expect("replay probability fixture")
    } else {
        live
    };
    let provenance = ProbabilitySourceProvenance::from_source_capture(&capture);
    fs::remove_dir_all(cache).expect("remove probability fixture cache");
    provenance
}

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
