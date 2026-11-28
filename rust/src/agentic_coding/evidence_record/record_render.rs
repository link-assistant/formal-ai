//! Render a record and transport its observed content to dependent deliveries.

use super::{DeliveryBinding, substantive_result_line};

/// Render an answer under the constraints attached to its destination.
pub(super) fn render_obligation(obligation: &DeliveryBinding, answer: &str) -> String {
    if !obligation.field_lines.is_empty() {
        let result = substantive_result_line(answer, &obligation.residual);
        let observed_results = exact_field_values(answer, "result");
        let mut next_observed = observed_results.iter();
        let mut rendered = String::new();
        for field in &obligation.field_lines {
            rendered.push_str(field);
            if field.ends_with('=') {
                if field == "result=" {
                    let combined = combined_observation(&observed_results, &obligation.residual)
                        .unwrap_or_else(|| result.to_owned());
                    rendered.push_str(&non_hollow_result(&combined, &obligation.residual));
                } else {
                    rendered.push_str(next_observed.next().map_or(result, String::as_str));
                }
            }
            rendered.push('\n');
        }
        return rendered;
    }
    let answer = non_hollow_result(answer, &obligation.residual);
    obligation.first_line.as_ref().map_or_else(
        || format!("{}\n", answer.trim_end()),
        |line| format!("{line}\n\n{}\n", answer.trim_end()),
    )
}

/// Values carried on exact machine-readable lines in an observation.
///
/// Multi-file reads deliberately render one bare `key=value` line per input so
/// a later delivery can preserve those bytes. Display headings and prose are
/// ignored; only a line whose prefix is exact participates.
fn exact_field_values(answer: &str, key: &str) -> Vec<String> {
    let prefix = format!("{key}=");
    answer
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix(&prefix))
        .map(str::to_owned)
        .collect()
}

/// Retain the finding from every verified step through nested delivery.
pub(super) fn sequence_observation(answer: &str, request: &str) -> String {
    let mut findings = Vec::new();
    for step in answer.split("\n\n") {
        let exact = exact_field_values(step, "result");
        if exact.is_empty() {
            let line = substantive_result_line(step, request);
            if !line.is_empty() {
                findings.push(line.to_owned());
            }
        } else {
            findings.extend(exact);
        }
    }
    combined_observation(&findings, request).unwrap_or_else(|| answer.to_owned())
}

/// State how multiple exact observations compose while retaining each value.
fn combined_observation(values: &[String], request: &str) -> Option<String> {
    match values {
        [] => None,
        [only] => Some(only.clone()),
        many => {
            let values = many.join("; ");
            let language = crate::language::detect(request);
            crate::seed::render_response(
                "evidence-exact-composition",
                language.slug(),
                &[("values", &values)],
            )
            .or(Some(values))
        }
    }
}

/// Keep a terse but decisive source token machine-checkable as a concrete result.
///
/// Structured result consumers reasonably distinguish an observation from a
/// placeholder by requiring a short natural-language statement. A source line
/// can still answer the question completely with only a path and one sentinel,
/// so retain it verbatim and add context instead of discarding or paraphrasing
/// the evidence.
fn non_hollow_result(result: &str, task: &str) -> String {
    let result = result.trim_end();
    let language = crate::language::detect(task).slug();
    let intent = if result.ends_with(':') || result.ends_with('：') {
        "coding_repository_source_observation"
    } else if result.split_whitespace().count() >= 4 {
        return result.to_owned();
    } else {
        "coding_repository_result_observation"
    };
    crate::seed::render_response(intent, language, &[("result", result)])
        .unwrap_or_else(|| result.to_owned())
}

/// The observation carried by a successfully written evidence artifact.
///
/// Remove only the destination's machine header. The remaining bytes are the
/// grounded answer the nested delivery wrote and can feed another artifact.
pub(super) fn written_observation(obligation: &DeliveryBinding, content: &str) -> String {
    let mut lines = content.lines();
    if obligation.first_line.as_deref().is_some_and(|first_line| {
        lines
            .clone()
            .next()
            .is_some_and(|line| line.trim() == first_line)
    }) {
        lines.next();
    }
    lines.collect::<Vec<_>>().join("\n").trim().to_owned()
}
