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
    static LATEST_RECORD: RefCell<Option<Value>> = const { RefCell::new(None) };
}

/// The record behind the latest work-item report rendered on this thread; the
/// in-process view of the trace for callers and tests.
#[must_use]
pub fn latest_work_item_record() -> Option<Value> {
    LATEST_RECORD.with(|latest| latest.borrow().clone())
}

fn filled(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut text = crate::seed::localized_response(intent, language).unwrap_or_default();
    for (key, value) in values {
        text = text.replace(&format!("{{{key}}}"), value);
    }
    text
}

/// The first line, at most 160 characters.
fn requirement_line(text: &str) -> String {
    let first = text.split('\n').next().unwrap_or_default().trim();
    if first.chars().count() > REQUIREMENT_CHARS {
        format!(
            "{}…",
            first.chars().take(REQUIREMENT_CHARS).collect::<String>()
        )
    } else {
        first.to_owned()
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
            || filled("work_item_report_request", language, &[]),
            str::to_owned,
        );
    let reason = record["reason"].as_str().unwrap_or("Unknown");
    let headline = if record["authored"] == json!(true) {
        "work_item_report_changed"
    } else {
        "work_item_report_unchanged"
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
        filled("work_item_report_read_none", language, &[])
    } else {
        let paths = read
            .iter()
            .map(|path| format!("`{path}`"))
            .collect::<Vec<_>>()
            .join(", ");
        filled("work_item_report_read", language, &[("paths", &paths)])
    });
    let requirements = record["requirements"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    if let Some(path) = record["refused"]["path"].as_str() {
        let mut refused = filled("work_item_report_refused", language, &[("path", path)]);
        if let Some(index) = record["refused"]["requirement"]
            .as_u64()
            .and_then(|index| usize::try_from(index).ok())
            .filter(|index| *index < requirements.len())
        {
            let number = (index + 1).to_string();
            let line = requirement_line(requirements[index]["text"].as_str().unwrap_or_default());
            refused.push(' ');
            refused.push_str(&filled(
                "work_item_report_concerns",
                language,
                &[("number", &number), ("requirement", &line)],
            ));
        }
        paragraphs.push(refused);
    } else if reason == "MissingReadTool" {
        paragraphs.push(filled("work_item_report_missing_read_tool", language, &[]));
    }
    if !requirements.is_empty() {
        let count = requirements.len().to_string();
        let mut lines = vec![filled(
            "work_item_report_requirements",
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
                "work_item_report_more",
                language,
                &[("count", &more)],
            ));
        }
        paragraphs.push(lines.join("\n"));
    }
    let schema = record["schema"].as_str().unwrap_or("work-item-record");
    paragraphs.push(filled(
        "work_item_report_trace",
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
    LATEST_RECORD.with(|latest| *latest.borrow_mut() = Some(work_item_record));
    record(AgenticPlan::Final(report), disposition, origin, result)
}

/// Whether `text` parses as a JSON object.
#[must_use]
pub fn parses_as_json_object(text: &str) -> bool {
    serde_json::from_str::<Value>(text).is_ok_and(|value| value.is_object())
}
