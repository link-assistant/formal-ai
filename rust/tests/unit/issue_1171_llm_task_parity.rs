//! Issue #1171 (E136): the generated LLM task parity surface.
//!
//! `docs/llm-task-parity.md` is generated from
//! `data/meta/llm-task-classes.lino`. These checks pin the rendered table
//! structurally: every row is complete, evidence pointers resolve, the
//! tracking-issue map matches the registry, and the row set matches the
//! registry. The tests do not execute the generator.
//!
//! Remaining wiring: per-class held-out probes and a no-memorization
//! extension (R2), the `benchmark run --suite llm-task-classes` runner (R3),
//! status-surface integration and CI (R4), and the release workflow (R5).

use std::fs;
use std::path::{Path, PathBuf};

const DOCUMENT: &str = "docs/llm-task-parity.md";
const ISSUE_URL_PREFIX: &str = "https://github.com/link-assistant/formal-ai/issues/";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The rows of the document's class table: one vector of trimmed cells per
/// row, in document order.
fn table_rows(document: &str) -> Vec<Vec<String>> {
    let mut lines = document
        .lines()
        .skip_while(|line| !line.starts_with("| Task class |"));
    lines.next(); // the header row
    let separator = lines.next().expect("a separator row after the header");
    assert!(
        separator.starts_with("| --- |"),
        "the table needs a markdown separator: {separator}"
    );
    let mut rows = Vec::new();
    for line in lines {
        if !line.starts_with('|') {
            break;
        }
        let mut cells: Vec<String> = line
            .split('|')
            .skip(1) // the piece before the leading '|'
            .map(|cell| cell.trim().to_owned())
            .collect();
        // A well-formed row ends with '|', so the split yields a trailing
        // empty piece; drop it.
        if cells.last().is_none_or(|last| last.is_empty()) {
            cells.pop();
        }
        rows.push(cells);
    }
    rows
}

/// The class id of a row: the `` `id` `` prefix of the first cell.
fn class_id(row: &[String]) -> String {
    row[0]
        .strip_prefix('`')
        .and_then(|rest| rest.split('`').next())
        .unwrap_or_default()
        .to_owned()
}

/// The class blocks of the data registry: one `(id, fields)` pair per
/// class, with the fields the structural checks need.
fn registry_classes() -> Vec<(String, Vec<String>)> {
    let source = read("data/meta/llm-task-classes.lino");
    let mut classes = Vec::new();
    let mut current: Option<(String, Vec<String>)> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(id) = line.strip_prefix("  class ") {
            if let Some(done) = current.take() {
                classes.push(done);
            }
            current = Some((id.trim().to_owned(), Vec::new()));
        } else if let Some(fields) = current.as_mut() {
            if let Some(name) = trimmed.strip_prefix("name \"") {
                let name = name.strip_suffix('"').unwrap_or(name);
                fields.push(format!("name {name}"));
            } else if trimmed.starts_with("benchmark \"") {
                let name = &trimmed["benchmark \"".len()..];
                let name = &name[..name.find('"').unwrap_or(name.len())];
                fields.push(format!("benchmark {name}"));
            } else if let Some(number) = trimmed.strip_prefix("tracking_issue ") {
                fields.push(format!("tracking_issue {number}"));
            }
        }
    }
    if let Some(done) = current.take() {
        classes.push(done);
    }
    classes
}

/// The issue number inside a tracking cell like
/// `[#1174](https://github.com/link-assistant/formal-ai/issues/1174)`.
fn tracking_number(row: &[String]) -> String {
    let cell = &row[6];
    let start = cell.find('#').expect("the tracking cell names its issue");
    let digits: String = cell[start + 1..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    assert!(!digits.is_empty(), "the tracking issue is a number: {cell}");
    digits
}

#[test]
fn every_row_has_seven_non_empty_columns() {
    for (index, row) in table_rows(&read(DOCUMENT)).iter().enumerate() {
        assert_eq!(
            row.len(),
            7,
            "row {} (`{}`) must have the seven columns: {row:?}",
            index + 1,
            class_id(row)
        );
        for (column, cell) in row.iter().enumerate() {
            assert!(
                !cell.is_empty(),
                "row {} column {column} is empty",
                index + 1
            );
        }
    }
}

#[test]
fn every_evidence_pointer_resolves() {
    for row in table_rows(&read(DOCUMENT)) {
        let cell = &row[5];
        if let Some(open) = cell.find("](")
            && cell.starts_with('[')
        {
            let url = &cell[open + 2..cell.len() - 1];
            assert!(
                url.starts_with(ISSUE_URL_PREFIX),
                "`{}` points at an issue, not elsewhere: {url}",
                class_id(&row)
            );
        } else {
            let path = cell.trim_matches('`');
            assert!(
                repo_root().join(path).exists(),
                "`{}` names evidence that must exist: {path}",
                class_id(&row)
            );
        }
    }
}

#[test]
fn tracking_links_pin_the_issue_map() {
    let expected = [
        ("factual_qa", "1172"),
        ("explanation", "1173"),
        ("summarization", "1174"),
        ("units_and_dates", "1176"),
        ("math_word", "1176"),
        ("code_generation", "1177"),
        ("regex", "1177"),
        ("creative_writing", "1178"),
        ("fact_checking", "1179"),
        ("repository_qa", "1180"),
        ("formalization", "1186"),
        ("multi_turn_conversation", "1171"),
        ("agentic_coding", "1162"),
    ];
    let rows = table_rows(&read(DOCUMENT));
    let by_id = |id: &str| {
        rows.iter()
            .find(|row| class_id(row) == id)
            .unwrap_or_else(|| panic!("the registry carries `{id}`"))
    };
    for (id, issue) in expected {
        assert_eq!(
            tracking_number(by_id(id)),
            issue,
            "`{id}` tracks issue #{issue}"
        );
    }
}

#[test]
fn document_rows_match_the_generator_registry() {
    let registry = registry_classes();
    let rows = table_rows(&read(DOCUMENT));
    let document_ids: Vec<String> = rows.iter().map(|row| class_id(row)).collect();
    let registry_ids: Vec<String> = registry.iter().map(|(id, _)| id.clone()).collect();
    assert_eq!(
        document_ids, registry_ids,
        "the document renders the registry's rows, in its order"
    );
    for ((id, fields), row) in registry.iter().zip(rows.iter()) {
        for field in fields {
            if let Some(name) = field.strip_prefix("name ") {
                assert!(
                    row[0].ends_with(name),
                    "`{id}` names its class: {name} vs {}",
                    row[0]
                );
            } else if let Some(benchmark) = field.strip_prefix("benchmark ") {
                assert!(
                    row[2].contains(benchmark),
                    "`{id}` names the benchmark {benchmark} in {}",
                    row[2]
                );
            } else if let Some(number) = field.strip_prefix("tracking_issue ") {
                assert_eq!(
                    tracking_number(row),
                    number,
                    "`{id}` tracks issue #{number}"
                );
            }
        }
    }
}

#[test]
fn registry_covers_the_r1_additions_beyond_the_probed_classes() {
    let ids: Vec<String> = table_rows(&read(DOCUMENT))
        .iter()
        .map(|row| class_id(row))
        .collect();
    for extra in [
        "multi_turn_conversation",
        "ocr_image_description",
        "long_document_qa",
        "classification",
        "extraction",
    ] {
        assert!(ids.contains(&extra.to_owned()), "R1 requires `{extra}`");
    }
    assert!(
        ids.len() >= 36,
        "the 34 probed classes plus the R1 additions: {} rows",
        ids.len()
    );
}

#[test]
fn first_measurement_history_is_pinned() {
    let document = read(DOCUMENT);
    for fragment in [
        "0 of 34 useful",
        "3 wrong",
        "0.347.0",
        "`factual_qa`",
        "`units_and_dates`",
        "`repository_qa`",
        "22 further probes",
    ] {
        assert!(
            document.contains(fragment),
            "the history section records {fragment}"
        );
    }
}

#[test]
fn generated_header_names_the_generator_and_its_input() {
    let document = read(DOCUMENT);
    let mut lines = document.lines();
    let header = lines.next().expect("a first line");
    assert!(
        header.contains("generate-llm-task-parity.rs") && header.contains("Never edited by hand"),
        "the first line is the generated-by marker: {header}"
    );
    let input = lines.next().expect("an input line");
    assert_eq!(
        input, "Input: `data/meta/llm-task-classes.lino`",
        "the generated document names its data-owned input"
    );
}
