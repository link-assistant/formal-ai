//! Retrieved page → ordered step list with provenance (issue #1138, plan 04 L9,
//! extended by plan 02 L10–L11).
//!
//! One "ordered step with provenance" record for the whole tree:
//! [`ProcedureStepRecord`] is what a synthesised guide step
//! (`how_to_guide::GuideStep::to_step_record`) and a captured page both become,
//! and `formalization::procedures::ExtractedProcedure::from_step_records` is its
//! only consumer-side constructor. HTML handling is delegated to the existing
//! `how_to_guide::extract` helpers so there is one HTML extractor in the tree.

use crate::event_log::EventLog;
use crate::how_to_guide::ServicePreferences;
use crate::how_to_guide::extract::{Payload, classify, compact_step_text, list_items, strip_html};
use crate::needs::NeedKind;
use crate::seed::SourceRecord;
use crate::source_fetch::{CachedSourceClient, SourceCapture, SourceTransport};
use crate::source_walk::{LookupBounds, entry_url_in, select_sources};

/// Fewer than this many items is not a procedure; a one-step guess is refused.
pub const MIN_PROCEDURE_STEPS: usize = 2;

/// One instruction recovered from a retrieved page, with the bytes it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStepRecord {
    pub ordinal: usize,
    pub text: String,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
    pub license_name: String,
    pub license_url: String,
    pub depth: usize,
}

/// How a page's procedure was recovered, recorded so a replay is auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepShape {
    /// `<ol>` / `<li>` items, the wikiHow and Wikipedia "Algorithm" shape.
    OrderedList,
    /// Numbered prose lines ("1." / "Step 2:"), the Stack Exchange shape.
    NumberedProse,
    /// A pseudocode block delimited by `<pre>` / `<code>`.
    PseudocodeBlock,
    /// A definition sentence that states a recurrence or closed form.
    DefinitionSentence,
}

/// Extract an ordered procedure from one capture. Returns `None` rather than a
/// one-step guess: fewer than [`MIN_PROCEDURE_STEPS`] items is not a procedure.
#[must_use]
pub fn steps_from_capture(
    capture: &SourceCapture,
    source: &SourceRecord,
    bounds: &LookupBounds,
) -> Option<(StepShape, Vec<ProcedureStepRecord>)> {
    steps_from_capture_at_depth(capture, source, bounds, 0)
}

/// Whether the bytes this step was read from may be emitted verbatim, or only
/// their abstract shape reused. A share-alike capture is always `ShapeOnly`.
#[must_use]
pub fn reuse_mode(step: &ProcedureStepRecord) -> crate::coding::program_ir::ReuseMode {
    let license = step.license_name.to_ascii_lowercase();
    if license.contains("by-sa")
        || license.contains("share alike")
        || license.contains("gfdl")
        || license.contains("free documentation")
    {
        crate::coding::program_ir::ReuseMode::ShapeOnly
    } else {
        crate::coding::program_ir::ReuseMode::Verbatim
    }
}

/// Retrieve and extract for one need phrase across the registry's coding
/// sources that declare the need kind, in the registry's consultation order.
pub fn retrieve_procedure<T: SourceTransport>(
    phrase: &str,
    prose_language: &str,
    client: &CachedSourceClient<T>,
    bounds: &LookupBounds,
    log: &mut EventLog,
) -> Vec<(StepShape, Vec<ProcedureStepRecord>)> {
    let preferences = ServicePreferences::default();
    let mut procedures = Vec::new();
    for source in select_sources(NeedKind::Procedure, phrase, &preferences, bounds) {
        let Some(url) = entry_url_in(&source, phrase, prose_language) else {
            log.append_fields(
                "procedure_source",
                &[("source", &source.id), ("status", "unbound_template")],
            );
            continue;
        };
        match client.fetch(&url) {
            Ok(capture) => {
                capture.record(log);
                if let Some(procedure) = steps_from_capture_at_depth(&capture, &source, bounds, 0) {
                    let item_count = procedure.1.len().to_string();
                    log.append_fields(
                        "procedure_source",
                        &[
                            ("source", &source.id),
                            ("status", "contributed"),
                            ("items", &item_count),
                        ],
                    );
                    procedures.push(procedure);
                } else {
                    log.append_fields(
                        "procedure_source",
                        &[("source", &source.id), ("status", "no_procedure")],
                    );
                }
            }
            Err(error) => {
                let detail = error.to_string();
                log.append_fields(
                    "procedure_source",
                    &[
                        ("source", &source.id),
                        ("status", "fetch_error"),
                        ("detail", &detail),
                    ],
                );
            }
        }
    }
    procedures
}

fn steps_from_capture_at_depth(
    capture: &SourceCapture,
    source: &SourceRecord,
    bounds: &LookupBounds,
    depth: usize,
) -> Option<(StepShape, Vec<ProcedureStepRecord>)> {
    if bounds.max_items < MIN_PROCEDURE_STEPS || depth > bounds.max_depth {
        return None;
    }
    let fetched_at = capture.fetched_at().parse::<u64>().unwrap_or(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    if fetched_at > 0 && now.saturating_sub(fetched_at) > bounds.max_capture_age_seconds {
        return None;
    }
    let bodies = capture_bodies(capture);
    for body in bodies {
        let (shape, texts) = extract_step_texts(&body, bounds.max_items);
        if texts.len() < MIN_PROCEDURE_STEPS {
            continue;
        }
        let records = texts
            .into_iter()
            .take(bounds.max_items)
            .enumerate()
            .map(|(index, text)| ProcedureStepRecord {
                ordinal: index + 1,
                text,
                source_id: source.id.clone(),
                source_url: capture.source_url().to_owned(),
                sha256: capture.sha256().to_owned(),
                fetched_at: capture.fetched_at().to_owned(),
                license_name: source.license_name.clone(),
                license_url: source.license_url.clone(),
                depth,
            })
            .collect();
        return Some((shape, records));
    }
    None
}

fn capture_bodies(capture: &SourceCapture) -> Vec<String> {
    match classify(capture.bytes()) {
        Payload::Parse { html, .. } => vec![html],
        Payload::Items { entries } => entries.into_iter().map(|entry| entry.body).collect(),
        Payload::Unrecognized { .. } => std::str::from_utf8(capture.bytes())
            .map(|text| vec![text.to_owned()])
            .unwrap_or_default(),
        Payload::OpenSearch { .. } | Payload::Search { .. } | Payload::Compressed => Vec::new(),
    }
}

fn extract_step_texts(body: &str, limit: usize) -> (StepShape, Vec<String>) {
    let ordered = ordered_list_items(body)
        .into_iter()
        .map(|item| compact_step_text(&item))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
    if ordered.len() >= MIN_PROCEDURE_STEPS {
        return (StepShape::OrderedList, unique_steps(ordered, limit));
    }

    if let Some(code) = delimited_body(body, "pre").or_else(|| delimited_body(body, "code")) {
        let lines = strip_html(code)
            .lines()
            .map(compact_step_text)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        if lines.len() >= MIN_PROCEDURE_STEPS {
            return (StepShape::PseudocodeBlock, unique_steps(lines, limit));
        }
    }

    let plain = strip_html(body);
    let numbered = plain
        .lines()
        .filter_map(numbered_instruction)
        .map(compact_step_text)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if numbered.len() >= MIN_PROCEDURE_STEPS {
        return (StepShape::NumberedProse, unique_steps(numbered, limit));
    }

    let definitions = sentence_steps(&plain, limit);
    (StepShape::DefinitionSentence, definitions)
}

fn ordered_list_items(html: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<ol") {
        let tail = &rest[start..];
        let Some(open_end) = tail.find('>') else {
            break;
        };
        let content = &tail[open_end + 1..];
        let Some(close) = content.find("</ol>") else {
            break;
        };
        items.extend(list_items(&content[..close]));
        rest = &content[close + "</ol>".len()..];
    }
    items
}

fn delimited_body<'a>(html: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}");
    let start = html.find(&open)?;
    let content = html[start..].find('>')? + start + 1;
    let close = format!("</{tag}>");
    let end = html[content..].find(&close)? + content;
    Some(&html[content..end])
}

fn numbered_instruction(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    trimmed[digits..]
        .strip_prefix(['.', ')', ':'])
        .map(str::trim)
        .filter(|instruction| !instruction.is_empty())
}

fn sentence_steps(text: &str, limit: usize) -> Vec<String> {
    unique_steps(
        text.split_inclusive(['.', '!', '?'])
            .map(compact_step_text)
            .filter(|sentence| !sentence.is_empty())
            .collect(),
        limit,
    )
}

fn unique_steps(candidates: Vec<String>, limit: usize) -> Vec<String> {
    let mut steps = Vec::new();
    for candidate in candidates {
        if !steps.contains(&candidate) {
            steps.push(candidate);
            if steps.len() >= limit {
                break;
            }
        }
    }
    steps
}
