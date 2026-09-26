//! Deterministic cycle settling for symbolic world-model recalculation.

use std::collections::BTreeMap;

use crate::relative_meta_logic::TruthValue;

/// Collapse a repeated relaxation cycle to the mean truth value of each node.
///
/// The mean is a fixpoint for the affine mutual-contradiction cycle. Callers
/// still verify the returned state with their dependency assessor before
/// reporting convergence.
pub fn collapse_truth_cycle(
    ids: &[String],
    cycle: &[BTreeMap<String, TruthValue>],
) -> Option<BTreeMap<String, TruthValue>> {
    if cycle.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let period = cycle.len() as f64;
    Some(
        ids.iter()
            .map(|id| {
                let sum = cycle
                    .iter()
                    .map(|state| state.get(id).copied().unwrap_or(TruthValue::UNKNOWN).get())
                    .sum::<f64>();
                (id.clone(), TruthValue::new(sum / period))
            })
            .collect(),
    )
}
