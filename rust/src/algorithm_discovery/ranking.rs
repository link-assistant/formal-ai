//! Ordering the survivors of the subsumption filter (#1138 B12, plan 12 leaf 5).
//!
//! `subsumes` above stays the *correctness* relation -- a longer validated
//! candidate subsumes a shorter one backed by the same traces -- and this module
//! only orders what survives it, through the same registry heuristic the draft
//! portfolio uses. Before plan 12 the order was an ad-hoc `sort_by` only this
//! module knew about, so two seams that both mean "prefer the cheaper candidate"
//! could disagree without anything noticing.

use crate::selection_heuristics::{
    ActionCost, CandidateRanker, CandidateScore, HeuristicRole, LeastActionRanker,
};

use super::AlgorithmCandidate;

/// Order the subsumption filter's survivors through the registry's `Rank`
/// heuristic, validated candidates first.
///
/// The seeded `key_order` is `code_size,steps,candidate_index`, so a shorter
/// routine backed by the same evidence wins over a longer one and the candidate
/// id remains the final deterministic tie-break. Nothing is dropped: a candidate
/// whose held-out evidence did not pass is not *ranked*, but it is still
/// returned, after the ranked ones, in id order.
pub(super) fn rank_survivors(candidates: Vec<AlgorithmCandidate>) -> Vec<AlgorithmCandidate> {
    let scores: Vec<CandidateScore> = candidates
        .iter()
        .map(|candidate| CandidateScore {
            candidate_id: candidate.id.clone(),
            checks: (
                candidate.held_out.iter().filter(|test| test.passed).count(),
                candidate.held_out.len(),
            ),
            cost: ActionCost {
                steps: u32::try_from(candidate.steps.len()).unwrap_or(u32::MAX),
                code_size: candidate
                    .steps
                    .iter()
                    .map(|step| step.operation.len())
                    .sum(),
                resource_units: 0,
                leaf_count: 1,
            },
        })
        .collect();
    let parameters = crate::method_registry::MethodRegistry::shared()
        .heuristics_for(HeuristicRole::Rank, "")
        .first()
        .map(|heuristic| heuristic.parameters.clone())
        .unwrap_or_default();
    let ranked = LeastActionRanker.rank(&scores, &parameters);

    let mut ordered: Vec<Option<AlgorithmCandidate>> = candidates.into_iter().map(Some).collect();
    let mut out: Vec<AlgorithmCandidate> = Vec::with_capacity(ordered.len());
    for index in ranked {
        if let Some(candidate) = ordered[index].take() {
            out.push(candidate);
        }
    }
    let mut remaining: Vec<AlgorithmCandidate> = ordered.into_iter().flatten().collect();
    remaining.sort_by(|left, right| left.id.cmp(&right.id));
    out.extend(remaining);
    out
}
