//! Failure-derived, review-gated learning artifacts for external benchmarks.
//!
//! This adapter persists only observed failed cases and infrastructure
//! unavailability. It feeds that evidence through the same associative-memory
//! ranking used by Formal AI's other learning reports; it never changes solver
//! behavior or promotes a rule automatically.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::agentic_coding::external_benchmark_learning;
use crate::learning_cycle::{UPSTREAM_BENCHMARKS_FRONTIER, parse_frontier_record};
use crate::links_format::format_lino_value_verbatim;

use super::{SuiteRun, vocabulary};

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

/// Rewrite the committed upstream-failure frontier from this run's failed cases.
///
/// Issue #1085 (D5.1): every failing upstream case becomes a learning-cycle
/// frontier input. `existing` is the committed record; items of suites that ran
/// now are replaced (a suite that passed fully drops out), items of suites that
/// did not run are kept, and the whole record is re-ranked so the diff is
/// stable.
#[must_use]
pub fn render_frontier_record(existing: &str, runs: &[SuiteRun], date: &str) -> String {
    let refreshed: BTreeSet<&str> = runs.iter().map(|run| run.suite.as_str()).collect();
    let mut items: Vec<(String, String, String)> = parse_frontier_record(existing)
        .into_iter()
        .filter(|item| !refreshed.contains(item.variation.as_str()))
        .map(|item| (item.variation, item.query, item.prompt))
        .collect();
    for run in runs {
        for outcome in run.outcomes.iter().filter(|outcome| !outcome.passed) {
            let prompt = if outcome.prompt_excerpt.is_empty() {
                one_line(&outcome.detail, 200)
            } else {
                outcome.prompt_excerpt.clone()
            };
            items.push((
                run.suite.clone(),
                format!("{}/{}", run.suite, outcome.id),
                prompt,
            ));
        }
    }
    items.sort();
    let count = items.len().to_string();
    let mut document = String::from("learning_frontier\n");
    field(&mut document, 2, "record_type", "learning_frontier_record");
    field(&mut document, 2, "frontier", UPSTREAM_BENCHMARKS_FRONTIER);
    field(&mut document, 2, "issue", "1085");
    field(
        &mut document,
        2,
        "recorded_from",
        &vocabulary::text("external_benchmark_frontier_recorded_from"),
    );
    field(&mut document, 2, "recorded_on", date);
    field(
        &mut document,
        2,
        "summary",
        &vocabulary::text("external_benchmark_frontier_summary"),
    );
    field(&mut document, 2, "total_prompts", &count);
    field(&mut document, 2, "learning_frontier", &count);
    for (index, (variation, query, prompt)) in items.iter().enumerate() {
        let _ = writeln!(document, "  frontier_prompt");
        field(&mut document, 4, "rank", &(index + 1).to_string());
        field(&mut document, 4, "query", query);
        field(&mut document, 4, "language", "en");
        field(&mut document, 4, "variation", variation);
        field(&mut document, 4, "prompt", prompt);
        field(&mut document, 4, "engine_intent", "benchmark_failure");
    }
    document
}

/// Split a rendered frontier document into pages that each honour
/// `line_budget` lines, at `frontier_prompt` boundaries.
///
/// The 1500-line data cap (issue #960) applies to every committed LiNo file,
/// and one honest prompt per failed case grows without bound, so the writer
/// spills continuation pages to `<stem>-partN.lino` files. Every page repeats
/// the document header, so each one is a self-describing record the ordinary
/// parser accepts, and the concatenation replays to exactly the frontier that
/// was split: ranks are assigned before the split and are not renumbered.
#[must_use]
pub fn split_frontier_document(document: &str, line_budget: usize) -> Vec<String> {
    let header_end = document
        .find("\n  frontier_prompt\n")
        .map_or(document.len(), |at| at + 1);
    let header = &document[..header_end];
    let header_lines = header.lines().count();
    // Slicing on the bare opener keeps each chunk opener-free and
    // newline-terminated, so re-attaching `  frontier_prompt\n` reconstructs
    // the item byte-for-byte.
    let items: Vec<&str> = document[header_end..]
        .split("  frontier_prompt\n")
        .filter(|item| !item.is_empty())
        .collect();

    let mut pages: Vec<String> = Vec::new();
    let mut page = String::from(header);
    let mut page_lines = header_lines;
    for item in items {
        // The replayed line cost of one item is its own lines plus the
        // `  frontier_prompt` opener the split strips and re-attaches.
        let item_lines = item.lines().count() + 1;
        if page_lines + item_lines > line_budget && page_lines > header_lines {
            pages.push(std::mem::take(&mut page));
            page.push_str(header);
            page_lines = header_lines;
        }
        page.push_str("  frontier_prompt\n");
        page.push_str(item);
        page_lines += item_lines;
    }
    pages.push(page);
    pages
}

/// The header-only page a provisioned part file carries while the frontier is
/// too small to fill it: a parseable document with no frontier prompts.
#[must_use]
pub fn placeholder_frontier_page() -> &'static str {
    "learning_frontier\n  record_type \"learning_frontier_record\"\n"
}

/// Write the split pages: the base page to `base_path`, continuation pages to
/// the `<stem>-partN.lino` siblings next to it.
///
/// Parts the pages no longer fill are reset to [`placeholder_frontier_page`]
/// rather than deleted: the embedded record compiles in exactly
/// `provisioned_parts` part files, so the committed file set must keep
/// existing whatever the frontier's size, and a placeholder guarantees no
/// stale frontier item survives a shrinking rewrite. Returns the written
/// paths, base page last.
///
/// # Errors
/// When any page cannot be written, or the pages exceed the provisioned part
/// count — which would leave a real page outside the embedded record; the
/// provisioning must be widened instead.
pub fn write_frontier_pages(
    base_path: &std::path::Path,
    pages: &[String],
    provisioned_parts: usize,
) -> std::io::Result<Vec<std::path::PathBuf>> {
    if pages.len() - 1 > provisioned_parts {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "the frontier needs {} continuation parts but only {provisioned_parts} are provisioned; widen the embedding before writing",
                pages.len() - 1
            ),
        ));
    }
    let parent = base_path.parent().map_or_else(
        || std::path::PathBuf::from("."),
        std::path::Path::to_path_buf,
    );
    std::fs::create_dir_all(&parent)?;
    let stem = base_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut written = Vec::new();
    for (index, page) in pages.iter().enumerate().skip(1) {
        let path = parent.join(format!("{stem}-part{index}.lino"));
        std::fs::write(&path, page)?;
        written.push(path);
    }
    for index in pages.len()..=provisioned_parts {
        let path = parent.join(format!("{stem}-part{index}.lino"));
        std::fs::write(&path, placeholder_frontier_page())?;
        written.push(path);
    }
    std::fs::write(base_path, &pages[0])?;
    written.push(base_path.to_path_buf());
    Ok(written)
}

/// The first non-empty line of `text`, whitespace collapsed, quotes and
/// backslashes replaced, cut to `limit` characters: a value every Links
/// Notation reader in the repository accepts on one line.
#[must_use]
pub fn one_line(text: &str, limit: usize) -> String {
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default();
    let collapsed = line
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(['"', '\\'], "'");
    if collapsed.chars().count() <= limit {
        return collapsed;
    }
    let mut cut: String = collapsed.chars().take(limit).collect();
    cut.push('…');
    cut
}
