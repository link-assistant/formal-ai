//! Trace, evidence, and reader-facing projections of a synthesised guide.
//!
//! The three projections are derived from the same [`HowToGuide`] value, so a
//! rendered guide can never claim a step the trace does not account for.
//!
//! Two kinds of text meet here and are kept strictly apart (R379,
//! `docs/design/no-hardcoded-natural-language.md`):
//!
//! * the trace and evidence projections are *records* — an event slug plus
//!   `key=value` fields, built through [`crate::trace_record`] so the field
//!   names stay tokens rather than sentences;
//! * the reader-facing projection is *prose* — every heading, connective, and
//!   sentence is looked up from `data/seed/multilingual-responses-procedure.lino`
//!   under a `how_to_guide_*` intent, which is what lets the same guide render
//!   in any seeded language.

use crate::event_log::EventLog;
use crate::seed::localized_response;
use crate::trace_record;

use super::{GuideConflict, HowToGuide, MIN_ACCEPTED_STEPS};

/// Event slugs. These name decisions in the evidence log; they are identifiers,
/// not interface text, and are never shown to a reader.
const EVENT_BOUNDS: &str = "how_to:bounds";
const EVENT_SOURCE: &str = "how_to:source";
const EVENT_COPIED_SOURCE: &str = "how_to:copied_source";
const EVENT_CONFLICT: &str = "conflict:source_disagreement";
const EVENT_STEP: &str = "how_to:step";
const EVENT_INSUFFICIENT_EVIDENCE: &str = "how_to:insufficient_evidence";

/// The tier a copied page carries once the #709 policy has demoted it.
const TIER_UNORIGINAL: &str = "unoriginal";

/// The language a guide renders in when the caller does not ask for one.
const DEFAULT_LANGUAGE: &str = "en";

/// How many hex characters of a digest a reader-facing citation shows.
const DIGEST_PREFIX: usize = 12;

/// Deterministic trace lines, one per decision, in a stable order.
pub fn trace(guide: &HowToGuide) -> String {
    let mut lines = vec![trace_record::event(EVENT_BOUNDS, &bounds_payload(guide))];
    for outcome in &guide.outcomes {
        lines.push(trace_record::event(EVENT_SOURCE, &outcome.trace_payload()));
    }
    for copy in &guide.copies {
        lines.push(trace_record::event(
            EVENT_COPIED_SOURCE,
            &copied_source_payload(copy),
        ));
    }
    for conflict in &guide.conflicts {
        lines.push(trace_record::event(
            EVENT_CONFLICT,
            &conflict_payload(conflict),
        ));
    }
    for (index, step) in guide.steps.iter().enumerate() {
        lines.push(trace_record::event(
            EVENT_STEP,
            &step_payload(index, &step.provenance()),
        ));
    }
    if !guide.is_sufficient() {
        lines.push(trace_record::event(
            EVENT_INSUFFICIENT_EVIDENCE,
            &insufficient_evidence_payload(guide),
        ));
    }
    lines.join("\n")
}

/// Append every retrieval and policy decision as evidence events.
pub fn record(guide: &HowToGuide, log: &mut EventLog) {
    log.append(EVENT_BOUNDS, bounds_payload(guide));
    for outcome in &guide.outcomes {
        log.append(EVENT_SOURCE, outcome.trace_payload());
    }
    for copy in &guide.copies {
        log.append(EVENT_COPIED_SOURCE, copied_source_payload(copy));
    }
    for conflict in &guide.conflicts {
        log.append(EVENT_CONFLICT, conflict_event_payload(conflict));
    }
    for (index, step) in guide.steps.iter().enumerate() {
        log.append(EVENT_STEP, step_payload(index, &step.provenance()));
    }
    if !guide.is_sufficient() {
        log.append(
            EVENT_INSUFFICIENT_EVIDENCE,
            insufficient_evidence_payload(guide),
        );
    }
}

fn bounds_payload(guide: &HowToGuide) -> String {
    trace_record::payload_with(
        &[("task", guide.task.clone())],
        &guide.bounds.trace_payload(),
    )
}

fn copied_source_payload(copy: &str) -> String {
    trace_record::payload(&[
        ("url", copy.to_owned()),
        ("tier", TIER_UNORIGINAL.to_owned()),
    ])
}

fn conflict_payload(conflict: &GuideConflict) -> String {
    trace_record::payload(&[
        ("action", conflict.action.clone()),
        ("kept", conflict.kept_source.clone()),
        ("dropped", conflict.dropped_source.clone()),
    ])
}

fn conflict_event_payload(conflict: &GuideConflict) -> String {
    trace_record::join(
        &conflict_payload(conflict),
        &trace_record::payload(&[("dropped_text", conflict.dropped_text.clone())]),
    )
}

fn step_payload(index: usize, provenance: &str) -> String {
    trace_record::payload_with(&[("rank", (index + 1).to_string())], provenance)
}

fn insufficient_evidence_payload(guide: &HowToGuide) -> String {
    trace_record::payload(&[
        ("steps", guide.steps.len().to_string()),
        ("required", MIN_ACCEPTED_STEPS.to_string()),
    ])
}

/// The guide as a reader sees it, in the seeded default language.
pub fn markdown(guide: &HowToGuide) -> String {
    markdown_in(guide, DEFAULT_LANGUAGE)
}

/// The guide as a reader sees it: numbered steps, each with the source it came
/// from, followed by the sources consulted and the licenses their bytes carry.
///
/// Every fragment of prose comes from the seed, so asking for another language
/// re-renders the same evidence rather than translating it after the fact.
pub fn markdown_in(guide: &HowToGuide, language: &str) -> String {
    let mut sections = vec![chrome(
        "how_to_guide_heading",
        language,
        &[("task", &guide.task)],
    )];
    if guide.is_sufficient() {
        let mut steps = Vec::new();
        for (index, step) in guide.steps.iter().enumerate() {
            steps.push(chrome(
                "how_to_guide_step",
                language,
                &[
                    ("rank", &(index + 1).to_string()),
                    ("text", &step.text),
                    ("source", &step.source_name),
                    ("license", &step.license_name),
                    (
                        "digest",
                        &step.sha256[..step.sha256.len().min(DIGEST_PREFIX)],
                    ),
                ],
            ));
        }
        sections.push(steps.join("\n"));
    } else {
        sections.push(chrome(
            "how_to_guide_insufficient_evidence",
            language,
            &[
                ("steps", &guide.steps.len().to_string()),
                ("required", &MIN_ACCEPTED_STEPS.to_string()),
            ],
        ));
    }
    sections.push(sources_section(guide, language));
    if !guide.conflicts.is_empty() {
        let mut lines = vec![chrome("how_to_guide_conflicts_heading", language, &[])];
        for conflict in &guide.conflicts {
            lines.push(chrome(
                "how_to_guide_conflict",
                language,
                &[
                    ("action", &conflict.action),
                    ("kept", &conflict.kept_source),
                    ("dropped", &conflict.dropped_source),
                    ("text", &conflict.dropped_text),
                ],
            ));
        }
        sections.push(lines.join("\n"));
    }
    if !guide.copies.is_empty() {
        let mut lines = vec![chrome("how_to_guide_copies_heading", language, &[])];
        for copy in &guide.copies {
            lines.push(chrome("how_to_guide_copy", language, &[("url", copy)]));
        }
        sections.push(lines.join("\n"));
    }
    sections.push(chrome(
        "how_to_guide_bounds",
        language,
        &[("bounds", &guide.bounds.trace_payload())],
    ));
    sections.join("\n\n")
}

fn sources_section(guide: &HowToGuide, language: &str) -> String {
    let mut lines = vec![chrome("how_to_guide_sources_heading", language, &[])];
    for outcome in &guide.outcomes {
        lines.push(chrome(
            "how_to_guide_source_outcome",
            language,
            &[
                ("source", &outcome.source_id),
                ("status", &outcome.status),
                ("pages", &outcome.pages.to_string()),
                ("steps", &outcome.steps.to_string()),
                ("detail", &outcome.detail),
            ],
        ));
    }
    for step in &guide.steps {
        let citation = chrome(
            "how_to_guide_citation",
            language,
            &[
                ("source", &step.source_id),
                ("url", &step.source_url),
                ("license", &step.license_name),
                ("license_url", &step.license_url),
            ],
        );
        if !lines.contains(&citation) {
            lines.push(citation);
        }
    }
    lines.join("\n")
}

/// Look up one seeded guide phrase and substitute its named fields.
///
/// The lookup goes through [`localized_response`] so an unseeded language falls
/// back the way `data/seed/languages.lino` declares, rather than silently
/// answering in English.
fn chrome(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = localized_response(intent, language).unwrap_or_default();
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}
