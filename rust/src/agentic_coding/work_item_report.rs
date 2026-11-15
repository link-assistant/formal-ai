//! A work-item terminal record is rendered as a localized answer: what
//! stopped the session, which files it read, what it needs and which
//! requirements remain unverified. The record itself is kept in the trace
//! (`[trace] work_item_record=…` and [`latest_work_item_record`]) and never
//! becomes the answer (issue #1560).
//! JavaScript twin: js/agentic/work_item_report.mjs.

use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::AgenticPlan;
use super::work_item_requirements::requirement_naming;
use serde_json::{Value, json};
use std::cell::RefCell;

const LISTED_REQUIREMENTS: usize = 8;
const REQUIREMENT_CHARS: usize = 160;

thread_local! {
    static LATEST_RECORD: RefCell<Option<(String, Value)>> = const { RefCell::new(None) };
}

/// The record behind the latest work-item report rendered on this thread; the
/// in-process view of the trace for callers and tests.
#[must_use]
pub fn latest_work_item_record() -> Option<Value> {
    LATEST_RECORD.with(|latest| latest.borrow().as_ref().map(|(_, record)| record.clone()))
}

/// The record behind `answer` when `answer` is exactly the latest work-item
/// report rendered on this thread.
#[must_use]
pub fn reported_work_item_record(answer: &str) -> Option<Value> {
    LATEST_RECORD.with(|latest| {
        latest
            .borrow()
            .as_ref()
            .filter(|(report, _)| report == answer)
            .map(|(_, record)| record.clone())
    })
}

fn filled(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut text = crate::seed::localized_response(intent, language).unwrap_or_default();
    for (key, value) in values {
        text = text.replace(&format!("{{{key}}}"), value);
    }
    text
}

/// The lines joined into one, at most 160 characters.
fn requirement_line(text: &str) -> String {
    let line = text
        .split('\n')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if line.chars().count() > REQUIREMENT_CHARS {
        format!(
            "{}…",
            line.chars().take(REQUIREMENT_CHARS).collect::<String>()
        )
    } else {
        line
    }
}

/// The path a refusal concerns and the requirement naming it.
pub(super) fn refusal(requirements: &[Value], path: &str) -> Value {
    json!({"path":path,"requirement":requirement_naming(requirements, path)})
}

/// The localized report of a work-item record.
#[must_use]
pub fn work_item_report(record: &Value, task: &str) -> String {
    let language = crate::language::detect(task).slug();
    let subject = record["workItem"]
        .as_str()
        .filter(|item| !item.is_empty())
        .map_or_else(
            || filled("work-item-report-request", language, &[]),
            str::to_owned,
        );
    let reason = record["reason"].as_str().unwrap_or("Unknown");
    let headline = if record["authored"] == json!(true) {
        "work-item-report-changed"
    } else {
        "work-item-report-unchanged"
    };
    let mut paragraphs = vec![filled(
        headline,
        language,
        &[("subject", &subject), ("reason", reason)],
    )];
    let mut read: Vec<&str> = Vec::new();
    for observation in record["observations"].as_array().into_iter().flatten() {
        if let Some(path) = observation["path"].as_str()
            && observation["complete"] == json!(true)
            && !read.contains(&path)
        {
            read.push(path);
        }
    }
    paragraphs.push(if read.is_empty() {
        filled("work-item-report-read-none", language, &[])
    } else {
        let paths = read
            .iter()
            .map(|path| format!("`{path}`"))
            .collect::<Vec<_>>()
            .join(", ");
        filled("work-item-report-read", language, &[("paths", &paths)])
    });
    let requirements = record["requirements"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    if let Some(path) = record["refused"]["path"].as_str() {
        let mut refused = filled("work-item-report-refused", language, &[("path", path)]);
        if let Some(index) = record["refused"]["requirement"]
            .as_u64()
            .and_then(|index| usize::try_from(index).ok())
            .filter(|index| *index < requirements.len())
        {
            let number = (index + 1).to_string();
            let line = requirement_line(requirements[index]["text"].as_str().unwrap_or_default());
            refused.push(' ');
            refused.push_str(&filled(
                "work-item-report-concerns",
                language,
                &[("number", &number), ("requirement", &line)],
            ));
        }
        paragraphs.push(refused);
    } else if reason == "MissingReadTool" {
        paragraphs.push(filled("work-item-report-missing-read-tool", language, &[]));
    }
    if !requirements.is_empty() {
        let count = requirements.len().to_string();
        let mut lines = vec![filled(
            "work-item-report-requirements",
            language,
            &[("count", &count)],
        )];
        for (index, requirement) in requirements.iter().take(LISTED_REQUIREMENTS).enumerate() {
            lines.push(format!(
                "{}. {}",
                index + 1,
                requirement_line(requirement["text"].as_str().unwrap_or_default())
            ));
        }
        if requirements.len() > LISTED_REQUIREMENTS {
            let more = (requirements.len() - LISTED_REQUIREMENTS).to_string();
            lines.push(filled(
                "work-item-report-more",
                language,
                &[("count", &more)],
            ));
        }
        paragraphs.push(lines.join("\n"));
    }
    let schema = record["schema"].as_str().unwrap_or("work-item-record");
    paragraphs.push(filled(
        "work-item-report-trace",
        language,
        &[("schema", schema)],
    ));
    paragraphs.join("\n\n")
}

/// The localized report is the answer; the record stays in the trace.
pub(super) fn work_item_record_final(
    work_item_record: Value,
    task: &str,
    disposition: FinalDisposition,
    origin: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let report = work_item_report(&work_item_record, task);
    super::planner::trace_route_with("work_item_record", || Some(&work_item_record));
    LATEST_RECORD.with(|latest| *latest.borrow_mut() = Some((report.clone(), work_item_record)));
    record(AgenticPlan::Final(report), disposition, origin, result)
}

/// A work-item session never answers with a JSON object. A final whose answer
/// is one is rendered as a report and its object kept as the record; any other
/// plan is returned unchanged.
pub(super) fn reported_work_item_final(
    step: AgenticPlan,
    task: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    if let AgenticPlan::Final(text) = &step
        && let Ok(work_item_record @ Value::Object(_)) = serde_json::from_str::<Value>(text)
    {
        let (disposition, origin) = result
            .as_ref()
            .filter(|resolved| resolved.text == *text)
            .map_or_else(
                || (FinalDisposition::Gap, "work_item_record".to_owned()),
                |resolved| {
                    (
                        if resolved.disposition == FinalDisposition::Failure {
                            FinalDisposition::Failure
                        } else {
                            FinalDisposition::Gap
                        },
                        resolved.origin.clone(),
                    )
                },
            );
        return work_item_record_final(work_item_record, task, disposition, &origin, result);
    }
    step
}

/// Whether `text` parses as a JSON object.
#[must_use]
pub fn parses_as_json_object(text: &str) -> bool {
    serde_json::from_str::<Value>(text).is_ok_and(|value| value.is_object())
}
