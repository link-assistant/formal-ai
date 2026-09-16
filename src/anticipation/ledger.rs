//! Inspectable outcome ledger for one anticipation plan.

use std::collections::BTreeSet;

use crate::links_format::format_lino_record;
use crate::memory::MemoryEvent;

use super::{AnticipationPlan, PrelearningRun, PREDICTION_HIT_KIND};

pub struct AnticipationLedger<'a> {
    plan: &'a AnticipationPlan,
    prelearning: &'a PrelearningRun,
    events: &'a [MemoryEvent],
}

impl<'a> AnticipationLedger<'a> {
    #[must_use]
    pub const fn new(
        plan: &'a AnticipationPlan,
        prelearning: &'a PrelearningRun,
        events: &'a [MemoryEvent],
    ) -> Self {
        Self {
            plan,
            prelearning,
            events,
        }
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let prediction_ids = self
            .plan
            .predictions
            .iter()
            .map(|prediction| prediction.id.as_str())
            .collect::<BTreeSet<_>>();
        let hits = self
            .events
            .iter()
            .filter(|event| event.kind.as_deref() == Some(PREDICTION_HIT_KIND))
            .filter(|event| {
                event.evidence.iter().any(|link| {
                    link.strip_prefix("anticipation_prediction:")
                        .is_some_and(|id| prediction_ids.contains(id))
                })
            })
            .collect::<Vec<_>>();
        let predictions_hit = hits
            .iter()
            .filter_map(|event| {
                event.evidence.iter().find_map(|link| {
                    link.strip_prefix("anticipation_prediction:")
                        .filter(|id| prediction_ids.contains(id))
                })
            })
            .collect::<BTreeSet<_>>();
        let rate = if self.plan.predictions.is_empty() {
            0
        } else {
            predictions_hit.len().saturating_mul(10_000) / self.plan.predictions.len()
        };
        let mut records = vec![format_lino_record(
            "anticipation_ledger",
            &[
                ("record_type", String::from("anticipation_ledger")),
                ("issue", String::from("705")),
                ("predictions", self.plan.predictions.len().to_string()),
                ("probe_results", self.plan.probes.len().to_string()),
                (
                    "prelearned_sources",
                    self.prelearning.sources.len().to_string(),
                ),
                ("prediction_hits", hits.len().to_string()),
                ("predicted_classes_hit", predictions_hit.len().to_string()),
                ("hit_rate_basis_points", rate.to_string()),
                ("deterministic", String::from("true")),
                ("mode", String::from("proposal_only")),
                ("human_gated", String::from("true")),
            ],
        )];
        records.push(self.plan.links_notation());
        let prelearning = self.prelearning.links_notation();
        if !prelearning.is_empty() {
            records.push(prelearning);
        }
        for hit in hits {
            if let Some(content) = &hit.content {
                records.push(content.clone());
            }
        }
        records.join("\n")
    }
}
