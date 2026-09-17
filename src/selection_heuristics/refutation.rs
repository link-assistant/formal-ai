//! Refutation-first 2-4-6 hypothesis search (#802; plan 12 leaves 12-14).
//!
//! Each probe must reduce the number of live possibilities, ideally by half or
//! more, and a probe that attempts to *disprove* the leading hypothesis is
//! preferred over one that would only confirm it. Every elimination is backed by
//! an `Evidence` record, so the search replays and the verdict is honest:
//! more than one survivor with no discriminating probe left is
//! `not_confirmed_not_refuted` with the survivors named, never a guess.

use crate::execution_evidence::Evidence;

use super::{Experiment, HypothesisSpace, SearchHypothesis, SearchVerdict};

/// The blocker reported when the live set cannot be narrowed any further.
///
/// A slug rather than a sentence: the surface renders it from seed prose, so the
/// verdict carries no natural-language literal out of the core.
pub(super) const NO_DISCRIMINATING_EXPERIMENT: &str = "no_discriminating_experiment";

impl Experiment {
    /// Live hypotheses surviving the worse of the two outcomes. Lower is better:
    /// this is the halving criterion (#802).
    #[must_use]
    pub fn worst_case_survivors(&self) -> usize {
        self.predicts_yes.len().max(self.predicts_no.len())
    }

    /// Whether this probe is a *disproof attempt* of the leading hypothesis.
    ///
    /// A probe the leading hypothesis predicts will **fail** kills it the moment
    /// it succeeds; a probe it predicts will succeed can only confirm it. That
    /// asymmetry is the whole point of #802's "first try to disprove".
    #[must_use]
    pub fn attempts_refutation_of(&self, leading: &str) -> bool {
        self.predicts_no.iter().any(|name| name == leading)
    }

    /// Whether this probe splits the currently live set: some live hypothesis
    /// predicts it succeeds and some other live one predicts it fails. A probe
    /// every survivor agrees on carries no information.
    fn discriminates(&self, alive: &[&SearchHypothesis]) -> bool {
        let names = |listed: &[String]| {
            alive
                .iter()
                .any(|hypothesis| listed.contains(&hypothesis.hypothesis_id))
        };
        names(&self.predicts_yes) && names(&self.predicts_no)
    }
}

impl HypothesisSpace {
    /// The hypotheses no observation has refuted yet.
    #[must_use]
    pub fn alive(&self) -> Vec<&SearchHypothesis> {
        self.hypotheses
            .iter()
            .filter(|hypothesis| hypothesis.alive)
            .collect()
    }

    /// Apply one observation: kill every hypothesis whose prediction it
    /// contradicts, recording which experiment did it, and retain the record so
    /// the elimination is evidence rather than assertion.
    ///
    /// Returns how many hypotheses this observation newly refuted.
    pub fn observe(&mut self, experiment_id: &str, record: Evidence) -> usize {
        let succeeded = record.reports_success();
        let contradicted: Vec<String> = self
            .experiments
            .iter()
            .find(|experiment| experiment.experiment_id == experiment_id)
            .map(|experiment| {
                if succeeded {
                    experiment.predicts_no.clone()
                } else {
                    experiment.predicts_yes.clone()
                }
            })
            .unwrap_or_default();
        let mut killed = 0;
        for hypothesis in &mut self.hypotheses {
            if !hypothesis.alive {
                continue;
            }
            if contradicted.contains(&hypothesis.hypothesis_id) {
                hypothesis.alive = false;
                hypothesis.refuted_by = Some(experiment_id.to_owned());
                killed += 1;
            }
        }
        self.observations.push(record);
        killed
    }

    /// The honest verdict when the space stops shrinking.
    #[must_use]
    pub fn verdict(&self) -> SearchVerdict {
        let alive = self.alive();
        match alive.len() {
            0 => SearchVerdict::AllRefuted,
            1 => SearchVerdict::Concluded {
                hypothesis_id: alive[0].hypothesis_id.clone(),
            },
            _ => SearchVerdict::NotConfirmedNotRefuted {
                survivors: alive
                    .iter()
                    .map(|hypothesis| hypothesis.hypothesis_id.clone())
                    .collect(),
                blocker: String::from(NO_DISCRIMINATING_EXPERIMENT),
            },
        }
    }

    /// The hypothesis a refuting probe is aimed at: the first live one in
    /// declaration order, so the choice is reproducible.
    fn leading(&self) -> Option<&SearchHypothesis> {
        self.hypotheses.iter().find(|hypothesis| hypothesis.alive)
    }
}

/// Pick the next probe over `space`.
///
/// Ordering is exactly the seeded `key_order`
/// `worst_case_survivors,attempts_refutation,experiment_id`: fewest survivors in
/// the worse outcome first, a disproof attempt ahead of a confirmation at equal
/// power, and the experiment id as the final deterministic tie-break, so the
/// same space always produces the same probe sequence.
pub(super) fn next_experiment(space: &HypothesisSpace) -> Option<&Experiment> {
    let alive = space.alive();
    if alive.len() < 2 {
        return None;
    }
    let leading = space.leading()?.hypothesis_id.clone();
    space
        .experiments
        .iter()
        .filter(|experiment| experiment.discriminates(&alive))
        .min_by(|left, right| {
            left.worst_case_survivors()
                .cmp(&right.worst_case_survivors())
                .then_with(|| {
                    right
                        .attempts_refutation_of(&leading)
                        .cmp(&left.attempts_refutation_of(&leading))
                })
                .then_with(|| left.experiment_id.cmp(&right.experiment_id))
        })
}
