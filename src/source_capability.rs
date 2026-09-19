//! Execution of source-backed composition and measurement capabilities.
//!
//! Capability routing establishes *what* a request needs. This module performs
//! the next shared step: recursively discover unknown concepts through the
//! trusted-source registry, retain the provenance of every capture, and project
//! only statements or quantities that the captures actually contain. An empty
//! walk is an explicit observation, never permission to invent an essay or a
//! number.

use std::collections::BTreeSet;

use crate::concept_lookup::ConceptSense;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::links_format::push_lino_node;
use crate::solver::SolverConfig;
use crate::solver_handlers::finalize_simple;

/// Assemble an attributable source graph for a composition request.
///
/// Equal content-addressed senses are included once. The result remains Links
/// Notation rather than being expanded into unsupported connective prose: every
/// statement stays adjacent to its URL, digest, and licence, and a later
/// composition procedure can operate on this graph without scraping the
/// rendered answer back into data.
#[must_use]
pub fn compose_source_evidence(prompt: &str, senses: &[ConceptSense]) -> String {
    let senses = distinct_senses(senses);
    let mut out = source_header(
        "compose_from_sources",
        prompt,
        if senses.is_empty() {
            "no_verified_capture"
        } else {
            "evidence_assembled"
        },
        senses.len(),
    );
    for sense in senses {
        push_lino_node(&mut out, 2, "statement", None);
        push_lino_node(&mut out, 4, "text", Some(&sense.gloss));
        push_lino_node(&mut out, 4, "source", Some(&sense.source_id));
        push_lino_node(&mut out, 4, "url", Some(&sense.source_url));
        push_lino_node(&mut out, 4, "sha256", Some(&sense.sha256));
        push_lino_node(&mut out, 4, "license", Some(&sense.license_name));
    }
    out
}

/// Extract sourced quantities from discovered concept senses.
///
/// The same seed-driven quantity parser used by verifiable tasks recognizes
/// values and units in each source gloss. A number without a grounded unit is
/// omitted: returning it as the requested measurement would be an unsupported
/// inference. Provenance travels with every emitted quantity.
#[must_use]
pub fn measurement_source_evidence(prompt: &str, senses: &[ConceptSense]) -> String {
    let senses = distinct_senses(senses);
    let measurements = senses
        .iter()
        .flat_map(|sense| {
            crate::verifiable_task::quantities::extract_quantities(&sense.gloss, &sense.language)
                .into_iter()
                .filter_map(|quantity| quantity.unit.map(|unit| (quantity.value, unit, *sense)))
        })
        .collect::<Vec<_>>();
    let mut out = source_header(
        "concept_measurement_lookup",
        prompt,
        if measurements.is_empty() {
            "no_grounded_quantity"
        } else {
            "measurements_extracted"
        },
        senses.len(),
    );
    for (value, unit, sense) in measurements {
        push_lino_node(&mut out, 2, "measurement", None);
        push_lino_node(&mut out, 4, "value", Some(&value));
        push_lino_node(&mut out, 4, "unit", Some(&unit));
        push_lino_node(&mut out, 4, "source", Some(&sense.source_id));
        push_lino_node(&mut out, 4, "url", Some(&sense.source_url));
        push_lino_node(&mut out, 4, "sha256", Some(&sense.sha256));
    }
    out
}

fn distinct_senses(senses: &[ConceptSense]) -> Vec<&ConceptSense> {
    let mut seen = BTreeSet::new();
    senses
        .iter()
        .filter(|sense| seen.insert(sense.content_id()))
        .collect()
}

fn source_header(capability: &str, prompt: &str, status: &str, source_count: usize) -> String {
    let mut out = String::new();
    push_lino_node(&mut out, 0, "source_capability", None);
    push_lino_node(&mut out, 2, "capability", Some(capability));
    push_lino_node(&mut out, 2, "request", Some(prompt));
    push_lino_node(&mut out, 2, "status", Some(status));
    push_lino_node(&mut out, 2, "source_count", Some(&source_count.to_string()));
    out
}

fn unavailable(capability: &str, prompt: &str, reason: &str) -> String {
    let mut out = String::new();
    push_lino_node(&mut out, 0, "source_capability", None);
    push_lino_node(&mut out, 2, "capability", Some(capability));
    push_lino_node(&mut out, 2, "request", Some(prompt));
    push_lino_node(&mut out, 2, "status", Some("unavailable"));
    push_lino_node(&mut out, 2, "reason", Some(reason));
    push_lino_node(&mut out, 2, "source_count", Some("0"));
    out
}

/// Execute one routed source capability through the common discovery kernel.
pub(crate) fn execute(
    capability: &str,
    prompt: &str,
    config: SolverConfig,
    log: &mut EventLog,
) -> SymbolicAnswer {
    let body = if config.offline {
        // `record_external_search` records the policy boundary. Calling it even
        // though the result is known empty keeps this path observationally the
        // same as every other source lookup.
        let senses = crate::solver_search::record_external_search(
            &config,
            log,
            prompt,
            crate::language::detect(prompt),
        );
        debug_assert!(senses.is_empty());
        unavailable(capability, prompt, "offline")
    } else {
        let senses = crate::solver_search::record_external_search(
            &config,
            log,
            prompt,
            crate::language::detect(prompt),
        );
        for sense in &senses {
            log.append(
                "source_capability:evidence",
                crate::trace_record::payload(&[
                    ("capability", capability.to_owned()),
                    ("content_id", sense.content_id()),
                    ("source", sense.source_id.clone()),
                    ("sha256", sense.sha256.clone()),
                ]),
            );
        }
        match capability {
            "compose_from_sources" => compose_source_evidence(prompt, &senses),
            "concept_measurement_lookup" => measurement_source_evidence(prompt, &senses),
            _ => unavailable(capability, prompt, "unsupported_source_projection"),
        }
    };
    finalize_simple(
        prompt,
        log,
        capability,
        &format!("response:{capability}"),
        &body,
        if body.contains("status \"unavailable\"") {
            0.0
        } else {
            0.8
        },
    )
}
