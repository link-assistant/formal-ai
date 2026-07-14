use crate::event_log::EventLog;
use crate::translation::{
    FormalizationAnchorKind, FormalizationCandidate, FormalizationDecision, FormalizationRole,
    FormalizationSelection, FormalizationSelectionReason,
};

pub fn record_formalization(log: &mut EventLog, candidate: &FormalizationCandidate) {
    if candidate.slots.is_empty() {
        return;
    }
    log.append("formalization", candidate.compact_summary());
    for slot in &candidate.slots {
        let kind = match (slot.role, slot.anchor.kind) {
            (FormalizationRole::Subject, FormalizationAnchorKind::WikidataItem) => {
                "formalization:subject_q"
            }
            (FormalizationRole::Predicate, FormalizationAnchorKind::WikidataProperty) => {
                "formalization:predicate_p"
            }
            (FormalizationRole::Object, FormalizationAnchorKind::WikidataItem) => {
                "formalization:object_q"
            }
            (_, FormalizationAnchorKind::WikidataItem) => "formalization:item_q",
            (_, FormalizationAnchorKind::WikidataProperty) => "formalization:property_p",
            (
                _,
                FormalizationAnchorKind::WikipediaArticle
                | FormalizationAnchorKind::WiktionaryEntry,
            ) => "formalization:fallback",
            (_, FormalizationAnchorKind::RawText) => "formalization:raw",
        };
        log.append(kind, slot.anchor.id.clone());
    }
    for term in &candidate.unresolved_terms {
        log.append("formalization_unresolved", term.clone());
    }
}

pub fn record_formalization_selection(log: &mut EventLog, selection: &FormalizationSelection) {
    for (index, candidate) in selection.candidates.iter().enumerate() {
        let probability = selection.probabilities.get(index).copied().unwrap_or(0.0);
        log.append(
            "candidate",
            format!(
                "formalization:{index} score={} probability={probability:.6} {}",
                candidate.score,
                candidate.compact_summary()
            ),
        );
    }

    // Issue #661 (R384): expose each interpretation's posterior as an explicit,
    // inspectable `statement_weight` link so every formalized message is
    // inspectable as a probability-weighted statement. The weights are the same
    // softmax posteriors already carried by the `candidate` events above, but
    // surfaced as their own link kind whose values sum to 1 across candidates.
    // Trace-only by design: the weights live in the evidence links, never in the
    // plain reply (see `curate_thinking_event`, which drops this kind).
    for (index, candidate) in selection.candidates.iter().enumerate() {
        let weight = selection.probabilities.get(index).copied().unwrap_or(0.0);
        log.append(
            "statement_weight",
            format!(
                "formalization:{index} weight={weight:.6} {}",
                candidate.compact_summary()
            ),
        );
    }

    match &selection.decision {
        FormalizationDecision::NoCandidate => {
            log.append("policy:temperature_selection", "no_candidate".to_owned());
        }
        FormalizationDecision::Selected {
            index,
            probability,
            margin,
            epsilon,
            reason,
        } => {
            log.append(
                "policy:temperature_selection",
                format!(
                    "selected=formalization:{index} probability={probability:.6} \
                     margin={margin:.6} epsilon={epsilon:.6} reason={}",
                    selection_reason_slug(*reason)
                ),
            );
            if *reason == FormalizationSelectionReason::GuessedUnderAmbiguity {
                log.append(
                    "policy:guessed_under_ambiguity",
                    format!("selected=formalization:{index}"),
                );
            }
        }
        FormalizationDecision::Clarify {
            top_index,
            runner_up_index,
            margin,
            epsilon,
            ..
        } => {
            log.append(
                "policy:clarify_under_ambiguity",
                format!(
                    "top=formalization:{top_index} runner_up=formalization:{runner_up_index} \
                     margin={margin:.6} epsilon={epsilon:.6}"
                ),
            );
        }
    }
}

const fn selection_reason_slug(reason: FormalizationSelectionReason) -> &'static str {
    match reason {
        FormalizationSelectionReason::OnlyCandidate => "only_candidate",
        FormalizationSelectionReason::ClearlyBest => "clearly_best",
        FormalizationSelectionReason::GuessedUnderAmbiguity => "guessed_under_ambiguity",
    }
}
