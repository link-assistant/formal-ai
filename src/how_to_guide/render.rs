//! Trace, evidence, and reader-facing projections of a synthesised guide.
//!
//! The three projections are derived from the same [`HowToGuide`] value, so a
//! rendered guide can never claim a step the trace does not account for.

use crate::event_log::EventLog;

use super::{HowToGuide, MIN_ACCEPTED_STEPS};

/// Deterministic trace lines, one per decision, in a stable order.
pub fn trace(guide: &HowToGuide) -> String {
    let mut lines = vec![format!(
        "how_to:bounds task={} {}",
        guide.task,
        guide.bounds.trace_payload()
    )];
    for outcome in &guide.outcomes {
        lines.push(format!("how_to:source {}", outcome.trace_payload()));
    }
    for copy in &guide.copies {
        lines.push(format!("how_to:copied_source url={copy} tier=unoriginal"));
    }
    for conflict in &guide.conflicts {
        lines.push(format!(
            "conflict:source_disagreement action={} kept={} dropped={}",
            conflict.action, conflict.kept_source, conflict.dropped_source
        ));
    }
    for (index, step) in guide.steps.iter().enumerate() {
        lines.push(format!(
            "how_to:step rank={} {}",
            index + 1,
            step.provenance()
        ));
    }
    if !guide.is_sufficient() {
        lines.push(format!(
            "how_to:insufficient_evidence steps={} required={MIN_ACCEPTED_STEPS}",
            guide.steps.len()
        ));
    }
    lines.join("\n")
}

/// Append every retrieval and policy decision as evidence events.
pub fn record(guide: &HowToGuide, log: &mut EventLog) {
    log.append(
        "how_to:bounds",
        format!("task={} {}", guide.task, guide.bounds.trace_payload()),
    );
    for outcome in &guide.outcomes {
        log.append("how_to:source", outcome.trace_payload());
    }
    for copy in &guide.copies {
        log.append(
            "how_to:copied_source",
            format!("url={copy} tier=unoriginal"),
        );
    }
    for conflict in &guide.conflicts {
        log.append(
            "conflict:source_disagreement",
            format!(
                "action={} kept={} dropped={} dropped_text={}",
                conflict.action,
                conflict.kept_source,
                conflict.dropped_source,
                conflict.dropped_text
            ),
        );
    }
    for (index, step) in guide.steps.iter().enumerate() {
        log.append(
            "how_to:step",
            format!("rank={} {}", index + 1, step.provenance()),
        );
    }
    if !guide.is_sufficient() {
        log.append(
            "how_to:insufficient_evidence",
            format!("steps={} required={MIN_ACCEPTED_STEPS}", guide.steps.len()),
        );
    }
}

/// The guide as a reader sees it: numbered steps, each with the source it came
/// from, followed by the sources consulted and the licenses their bytes carry.
pub fn markdown(guide: &HowToGuide) -> String {
    let mut sections = vec![format!("## How to {}", guide.task)];
    if guide.is_sufficient() {
        let mut steps = Vec::new();
        for (index, step) in guide.steps.iter().enumerate() {
            steps.push(format!(
                "{}. {} — {} ({}, sha256 {})",
                index + 1,
                step.text,
                step.source_name,
                step.license_name,
                &step.sha256[..step.sha256.len().min(12)],
            ));
        }
        sections.push(steps.join("\n"));
    } else {
        sections.push(format!(
            "Insufficient evidence: {} step(s) were captured and {MIN_ACCEPTED_STEPS} are required, \
             so no procedure is asserted.",
            guide.steps.len()
        ));
    }
    sections.push(sources_section(guide));
    if !guide.conflicts.is_empty() {
        let mut lines = vec![String::from("### Conflicts")];
        for conflict in &guide.conflicts {
            lines.push(format!(
                "- `{}`: kept {}, dropped {} (\"{}\")",
                conflict.action,
                conflict.kept_source,
                conflict.dropped_source,
                conflict.dropped_text
            ));
        }
        sections.push(lines.join("\n"));
    }
    if !guide.copies.is_empty() {
        let mut lines = vec![String::from("### Copied sources ignored")];
        for copy in &guide.copies {
            lines.push(format!("- {copy}"));
        }
        sections.push(lines.join("\n"));
    }
    sections.push(format!("Bounds: {}", guide.bounds.trace_payload()));
    sections.join("\n\n")
}

fn sources_section(guide: &HowToGuide) -> String {
    let mut lines = vec![String::from("### Sources")];
    for outcome in &guide.outcomes {
        lines.push(format!(
            "- {} — {} ({} page(s), {} step(s)): {}",
            outcome.source_id, outcome.status, outcome.pages, outcome.steps, outcome.detail
        ));
    }
    for step in &guide.steps {
        let citation = format!(
            "- {} → {} ({}: {})",
            step.source_id, step.source_url, step.license_name, step.license_url
        );
        if !lines.contains(&citation) {
            lines.push(citation);
        }
    }
    lines.join("\n")
}
