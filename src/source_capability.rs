//! Execution of source-backed composition and measurement capabilities.
//!
//! Capability routing establishes *what* a request needs. This module performs
//! the next shared step: recursively discover unknown concepts through the
//! trusted-source registry, retain the provenance of every capture, and project
//! only statements or quantities that the captures actually contain. An empty
//! walk is an explicit observation, never permission to invent an essay or a
//! number.

use std::collections::BTreeSet;
use std::fmt::Write as _;

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

/// Compose an attributable document over the discovered concept senses.
///
/// The document is assembled from exactly what the captures contain: a title,
/// the localized boundary note, every distinct gloss with its citation marker,
/// and a sources footer naming each URL and licence. No connective prose is
/// generated — a "few pages" request with one captured statement yields one
/// statement, because padding to the requested length would be invention. An
/// empty graph composes nothing and says so in the response language.
#[must_use]
pub fn compose_document(prompt: &str, senses: &[ConceptSense]) -> String {
    let language = crate::language::detect(prompt).slug();
    let senses = distinct_senses(senses);
    if senses.is_empty() {
        return crate::seed::localized_response("compose_no_verified_capture", language)
            .unwrap_or_else(|| {
                String::from(
                    "No verified source statement was captured for this request, so nothing \
                     was composed.",
                )
            });
    }
    let boundary_note = crate::seed::localized_response("compose_boundary_note", language)
        .unwrap_or_else(|| {
            String::from(
                "Every statement below is carried by a cited source; nothing was added beyond \
                 them.",
            )
        });
    let sources_heading = crate::seed::localized_response("compose_sources_heading", language)
        .unwrap_or_else(|| String::from("Sources"));
    let subject = crate::solver_handlers::detect_web_search_query(prompt)
        .unwrap_or_else(|| prompt.trim().to_owned());
    let mut out = String::new();
    let _ = writeln!(out, "# {subject}");
    let _ = writeln!(out);
    let _ = writeln!(out, "{boundary_note}");
    let _ = writeln!(out);
    for (index, sense) in senses.iter().enumerate() {
        let marker = index + 1;
        let _ = writeln!(out, "{} [{marker}]", sense.gloss);
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## {sources_heading}");
    let _ = writeln!(out);
    for (index, sense) in senses.iter().enumerate() {
        let marker = index + 1;
        let _ = writeln!(
            out,
            "[{marker}] {} - {} ({})",
            sense.source_id, sense.source_url, sense.license_name
        );
    }
    out
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
    let senses = crate::solver_search::record_external_search(
        &config,
        log,
        prompt,
        crate::language::detect(prompt),
    );
    let mut composed_without_capture = false;
    let body = if config.offline {
        // The walk above already recorded the policy boundary, keeping this
        // path observationally the same as every other source lookup.
        debug_assert!(senses.is_empty());
        unavailable(capability, prompt, "offline")
    } else {
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
            "compose_from_sources" => {
                log.append(
                    "compose:source_graph",
                    compose_source_evidence(prompt, &senses),
                );
                composed_without_capture = senses.is_empty();
                compose_document(prompt, &senses)
            }
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
        if body.contains("status \"unavailable\"") || composed_without_capture {
            0.0
        } else {
            0.8
        },
    )
}
