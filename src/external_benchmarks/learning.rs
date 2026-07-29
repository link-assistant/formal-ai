//! Failure-derived, review-gated learning artifacts for external benchmarks.
//!
//! This adapter persists only observed failed cases and infrastructure
//! unavailability. It feeds that evidence through the same associative-memory
//! ranking used by Formal AI's other learning reports; it never changes solver
//! behavior or promotes a rule automatically.

use std::fmt::Write as _;

use crate::agentic_coding::external_benchmark_learning;
use crate::links_format::format_lino_value_verbatim;

use super::{vocabulary, SuiteRun};

/// Render a human-review-gated learning proposal from actual run failures.
///
/// A fully passing run has nothing to learn from and returns `None`.
#[must_use]
pub fn render_failure_report(runs: &[SuiteRun]) -> Option<String> {
    let memory = failure_memory(runs)?;
    Some(external_benchmark_learning::render_document_from(&memory))
}

fn failure_memory(runs: &[SuiteRun]) -> Option<String> {
    let mut observations = Vec::new();
    let mut document = String::from("demo_memory\n");
    for run in runs {
        if let Some(reason) = &run.unavailable {
            let id = format!("observation:{}:unavailable", run.suite);
            let content = vocabulary::render(
                "external_benchmark_unavailable_observation",
                &[("suite", &run.suite), ("reason", reason)],
            );
            observation(
                &mut document,
                &id,
                "benchmark_infrastructure_failure",
                &content,
            );
            observations.push(id);
        }
        for (index, outcome) in run
            .outcomes
            .iter()
            .filter(|outcome| !outcome.passed)
            .enumerate()
        {
            let id = format!("observation:{}:failure:{index}", run.suite);
            let content = vocabulary::render(
                "external_benchmark_failure_observation",
                &[
                    ("suite", &run.suite),
                    ("case", &outcome.id),
                    ("detail", &outcome.detail),
                ],
            );
            observation(&mut document, &id, "benchmark_case_failure", &content);
            observations.push(id);
        }
    }
    if observations.is_empty() {
        return None;
    }

    document.push_str(&vocabulary::text("external_benchmark_learning_event"));
    field(&mut document, 4, "kind", "learning_amendment");
    field(&mut document, 4, "role", "assistant");
    field(
        &mut document,
        4,
        "content",
        &vocabulary::text("external_benchmark_learning_lesson"),
    );
    field(&mut document, 4, "conversationId", "issue-698");
    for id in &observations {
        field(&mut document, 4, "evidence", id);
    }
    field(&mut document, 4, "writeCount", "4");
    Some(document)
}

fn observation(document: &mut String, id: &str, kind: &str, content: &str) {
    let _ = writeln!(document, "  event {}", format_lino_value_verbatim(id));
    field(document, 4, "kind", kind);
    field(document, 4, "role", "tool");
    field(document, 4, "content", content);
    field(document, 4, "conversationId", "issue-698");
    field(document, 4, "accessCount", "1");
    field(document, 4, "writeCount", "1");
}

fn field(document: &mut String, indent: usize, name: &str, value: &str) {
    let _ = writeln!(
        document,
        "{}{name} {}",
        " ".repeat(indent),
        format_lino_value_verbatim(value)
    );
}
