//! Apply a reviewed memory plan without losing reconstruction provenance.
use super::retention::reconstruction_record;
use super::support::estimate_event_bytes;
use super::{
    DreamingConfig, DreamingDurability, DreamingOutcome, DreamingPlan, algorithm_candidate_event,
    amendment_event, candidate_failure_event, pattern_event, plan_memory_dreaming,
    synthesized_trial_event,
};
use crate::memory::{MemoryEvent, MemoryStore};
use std::collections::BTreeSet;

#[must_use]
pub fn apply_dreaming_plan(store: &mut MemoryStore, plan: &DreamingPlan) -> DreamingOutcome {
    // A saved plan is a proposal, not authority to delete a record whose
    // provenance or reconstruction proof has changed since planning.
    let current = plan_memory_dreaming(
        store.events(),
        &DreamingConfig {
            daydreaming_enabled: plan.daydreaming_enabled,
            target_free_ratio_percent: plan.target_free_ratio_percent,
            storage_capacity_bytes: plan.storage_capacity_bytes,
            free_bytes: plan.free_bytes,
            incoming_bytes: plan.incoming_bytes,
        },
    );
    let selected_ids = plan
        .actions
        .iter()
        .filter(|action| {
            current
                .actions
                .iter()
                .any(|now| now.event_id == action.event_id)
                && current
                    .observations
                    .iter()
                    .filter(|observation| observation.event_id == action.event_id)
                    .all(|observation| observation.durability.is_reclaimable())
        })
        .map(|action| action.event_id.as_str())
        .collect::<BTreeSet<_>>();

    // Bake each learned generalization into memory as a retained learning record
    // *before* forgetting the specifics it covers. Applying an unchanged plan
    // twice must not duplicate amendments, so we skip ids already present.
    let existing_ids: BTreeSet<String> = store
        .events()
        .iter()
        .map(|event| event.id.clone())
        .collect();
    let reconstruction_records = store
        .events()
        .iter()
        .filter(|event| selected_ids.contains(event.id.as_str()))
        .filter(|event| {
            current.observations.iter().any(|observation| {
                observation.event_id == event.id
                    && observation.durability == DreamingDurability::RecomputableCache
            })
        })
        .filter_map(reconstruction_record)
        .filter(|event| !existing_ids.contains(&event.id))
        .collect::<Vec<_>>();
    let new_amendments: Vec<MemoryEvent> = plan
        .amendments
        .iter()
        .filter(|amendment| !existing_ids.contains(&amendment.id))
        .map(amendment_event)
        .collect();
    let learned_amendments = new_amendments.len();
    let new_patterns = plan
        .patterns
        .iter()
        .map(pattern_event)
        .filter(|event| !existing_ids.contains(&event.id))
        .collect::<Vec<_>>();
    let learned_patterns = new_patterns.len();
    let new_algorithm_candidates = plan
        .algorithm_candidates
        .iter()
        .map(algorithm_candidate_event)
        .filter(|event| !existing_ids.contains(&event.id))
        .collect::<Vec<_>>();
    let learned_algorithm_candidates = new_algorithm_candidates.len();
    // Failed simulations are learning material, not noise: preserve each one as
    // a retained record so later dreaming rounds (and refinement) can consume it.
    let new_failures = plan
        .candidate_tasks
        .iter()
        .filter(|candidate| !candidate.passed)
        .map(candidate_failure_event)
        .filter(|event| !existing_ids.contains(&event.id))
        .collect::<Vec<_>>();
    let recorded_failures = new_failures.len();
    let new_trials = plan
        .synthesized_tasks
        .iter()
        .map(synthesized_trial_event)
        .filter(|event| !existing_ids.contains(&event.id))
        .collect::<Vec<_>>();
    let recorded_trials = new_trials.len();

    if selected_ids.is_empty()
        && new_amendments.is_empty()
        && new_patterns.is_empty()
        && new_algorithm_candidates.is_empty()
        && new_failures.is_empty()
        && new_trials.is_empty()
    {
        return DreamingOutcome {
            removed_events: 0,
            estimated_reclaimed_bytes: 0,
            learned_amendments: 0,
            learned_patterns: 0,
            recorded_failures: 0,
            recorded_trials: 0,
            learned_algorithm_candidates: 0,
        };
    }

    let initial_len = store.len();
    let initial_bytes: u64 = store.events().iter().map(estimate_event_bytes).sum();
    let mut retained = store
        .events()
        .iter()
        .filter(|event| !selected_ids.contains(event.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let removed_events = initial_len - retained.len();
    retained.extend(reconstruction_records);
    retained.extend(new_amendments);
    retained.extend(new_patterns);
    retained.extend(new_algorithm_candidates);
    retained.extend(new_failures);
    retained.extend(new_trials);
    *store = MemoryStore::from_events(retained);
    // Measure the final store, so every present and future retained record kind
    // is charged. A growing learning pass has zero reclaimed bytes, not a
    // positive saving computed from discarded payloads alone.
    let retained_bytes: u64 = store.events().iter().map(estimate_event_bytes).sum();
    let estimated_reclaimed_bytes = initial_bytes.saturating_sub(retained_bytes);
    DreamingOutcome {
        removed_events,
        estimated_reclaimed_bytes,
        learned_amendments,
        learned_patterns,
        recorded_failures,
        recorded_trials,
        learned_algorithm_candidates,
    }
}
