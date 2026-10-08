//! Evidence contracts for the requirement rows of issue #1017.
//!
//! `issue_1017` pins the workflow invariants the issue introduced. This file
//! pins what the requirement rows cite besides workflows: the diagnostic
//! ledger, the template comparison, the upstream reports, the verbose modes
//! and the delivery documents. A row that says "Implemented" then names a test
//! that fails when its evidence goes, instead of a path nobody re-reads.

use std::fs;
use std::path::{Path, PathBuf};

/// The canonical archive of issue #1017 / pull request #1018.
const ARCHIVE: &str = "dev/log/issues/1017/pulls/1018";

fn repository_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .join(path)
}

fn repository_file(path: &str) -> String {
    fs::read_to_string(repository_path(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// The text between the `start` heading and the next `end` heading.
fn between<'a>(document: &'a str, start: &str, end: &str) -> &'a str {
    let from = document
        .find(start)
        .unwrap_or_else(|| panic!("missing heading {start:?}"));
    let rest = &document[from + start.len()..];
    let to = rest
        .find(end)
        .unwrap_or_else(|| panic!("missing heading {end:?} after {start:?}"));
    &rest[..to]
}

/// The non-header rows of every Markdown table in `text`.
fn table_rows(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.starts_with("| ") && !line.starts_with("| ---"))
        .collect()
}

/// R1017-3: every collected diagnostic is fixed or kept with a reason.
///
/// Every annotation and warning- or error-shaped line is either fixed
/// (section 4.1, one row per defect) or kept with a stated reason (section
/// 4.2). The ledger's counts must match the archived source files, so a ledger
/// that silently loses a defect -- or a source that grows without the ledger
/// following -- fails here.
#[test]
fn every_collected_diagnostic_is_fixed_or_kept_with_a_reason() {
    let ledger = repository_file(&format!("{ARCHIVE}/README.md"));

    let fixed = between(&ledger, "### 4.1 Defects fixed", "### 4.2 ");
    let mut numbers: Vec<u32> = table_rows(fixed)
        .into_iter()
        .filter_map(|row| row.strip_prefix("| D"))
        .filter_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse().ok()
        })
        .collect();
    numbers.dedup();
    assert_eq!(
        numbers,
        (1..=21).collect::<Vec<u32>>(),
        "section 4.1 must carry D1..D21 in order, each with a root cause and a fix"
    );
    for row in table_rows(fixed).into_iter().skip(1) {
        assert!(
            row.matches(" | ").count() >= 3,
            "a fixed defect names its diagnostic, root cause and fix: {row}"
        );
    }

    let kept = between(&ledger, "### 4.2 ", "## 5. ");
    let dispositions = table_rows(kept).len().saturating_sub(1);
    assert!(
        dispositions >= 9,
        "section 4.2 states why each remaining diagnostic class is kept, found {dispositions}"
    );

    let annotations = repository_file(&format!("{ARCHIVE}/annotations/all-annotations.tsv"));
    let annotation_count = annotations.lines().count();
    assert!(
        ledger.contains(&format!(
            "`annotations/all-annotations.tsv` ({annotation_count} annotations)"
        )),
        "the ledger must state the {annotation_count} archived annotations it classifies"
    );
    let soft_warnings = repository_file(&format!("{ARCHIVE}/analysis/soft-warnings.txt"));
    let warning_count = soft_warnings.lines().count();
    let grouped = format!("{},{:03}", warning_count / 1000, warning_count % 1000);
    assert!(
        ledger.contains(&format!("({grouped} warning- or error-shaped lines)")),
        "the ledger must state the {grouped} archived warning- or error-shaped lines it classifies"
    );
}

/// R1017-8: the template comparison keeps its sources and its verdicts.
///
/// The comparison is against complete, immutable template trees, and the Hive
/// Mind checklist is answered item by item, with each deviation argued under
/// its own heading.
#[test]
fn template_trees_and_the_best_practice_checklist_are_retained() {
    for template in ["rust-template", "js-template", "python-template"] {
        let tree = repository_path(&format!("{ARCHIVE}/references/templates/{template}"));
        let entries = fs::read_dir(&tree)
            .unwrap_or_else(|error| panic!("read {}: {error}", tree.display()))
            .count();
        assert!(
            entries > 0,
            "{template} must be archived as a complete tree"
        );
    }
    assert!(
        repository_path(&format!("{ARCHIVE}/references/CI-CD-BEST-PRACTICES.md")).is_file(),
        "the Hive Mind guidance the checklist answers must be archived"
    );
    let diffs = fs::read_dir(repository_path(&format!(
        "{ARCHIVE}/analysis/template-diffs"
    )))
    .expect("the per-workflow template diffs are archived")
    .count();
    assert!(
        diffs >= 4,
        "expected one diff per compared workflow, found {diffs}"
    );

    let ledger = repository_file(&format!("{ARCHIVE}/README.md"));
    let comparison = between(
        &ledger,
        "## 5. Template and best-practice comparison",
        "## 6. ",
    );
    let checklist = table_rows(comparison).len().saturating_sub(1);
    assert!(
        checklist >= 13,
        "every Hive Mind checklist item needs a verdict, found {checklist}"
    );
    assert!(
        comparison.contains("Deliberately **not** adopted")
            && comparison.contains("### Deviation:"),
        "each practice not adopted must be argued, not left implicit"
    );
}

/// R1017-9: every upstream report records where it was filed.
///
/// Each upstream report is kept as the exact body that was filed and records
/// where it was filed; the three template reports carry a reproduction, a
/// workaround and a code-level fix.
#[test]
fn every_upstream_report_records_where_it_was_filed() {
    let reports = [
        (
            "rust-template-step-execution-budget.md",
            "https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/135",
        ),
        (
            "js-template-step-execution-budget.md",
            "https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/137",
        ),
        (
            "python-template-step-execution-budget.md",
            "https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/60",
        ),
        (
            "codeql-rust-macro-expansion-data-point.md",
            "https://github.com/github/codeql/issues/19982#issuecomment-5309221141",
        ),
        (
            "codeql-rust-sysroot-pin-measured-result.md",
            "https://github.com/github/codeql/issues/19982#issuecomment-5309264165",
        ),
        (
            "electron-builder-async-task-manager-stale-error.md",
            "https://github.com/electron-userland/electron-builder/issues/10091",
        ),
        (
            "meta-language-quadratic-point-at-byte.md",
            "https://github.com/link-foundation/meta-language/issues/193",
        ),
    ];
    for (file, url) in reports {
        let body = repository_file(&format!("{ARCHIVE}/upstream-reports/{file}"));
        assert!(
            body.contains(url),
            "{file} must record that it was filed as {url}"
        );
        if file.contains("-template-") {
            for section in ["## Reproduction", "## Workaround", "## Suggested code fix"] {
                assert!(body.contains(section), "{file} lacks {section}");
            }
        }
    }
}

/// R1017-10: the slow-init and command traces are opt-in.
///
/// The two tracing modes beside the budget heartbeat are opt-in through
/// `cli_env::flag_enabled`, and nothing CI runs switches them on, so a green
/// run stays quiet. The heartbeat has its own test:
/// `step_budgets_within_job_clocks::budget_wrapper_heartbeat_is_available_but_off_by_default`.
#[test]
fn the_slow_init_and_command_traces_are_available_but_off_by_default() {
    let modes = [
        (
            "rust/src/agentic_coding/self_ast.rs",
            "FORMAL_AI_TRACE_SLOW_INIT",
        ),
        ("rust/src/agent.rs", "FORMAL_AI_TRACE_COMMANDS"),
    ];
    for (source, variable) in modes {
        assert!(
            repository_file(source)
                .contains(&format!("crate::cli_env::flag_enabled(\"{variable}\")")),
            "{source} must read {variable} through the opt-in flag helper"
        );
        if std::env::var_os(variable).is_none() {
            assert!(
                !formal_ai::cli_env::flag_enabled(variable),
                "{variable} must be off when it is not set"
            );
        }
        for (file, body) in super::workflow_fixtures::ci_shell_files() {
            assert!(
                !body.contains(variable),
                "{file} switches {variable} on; the trace must stay opt-in"
            );
        }
    }
}

/// R1017-12: the evidence and the delivery documents are retained.
///
/// The archive, both case studies and the changelog entry name the issue and
/// the single pull request that delivered it.
#[test]
fn the_archive_and_delivery_documents_name_the_issue_and_its_pull_request() {
    for path in [
        "dev/log/issues/1017/pulls/1018/README.md",
        "docs/case-studies/issue-1017/README.md",
        "docs/case-studies/pull-request-1018/README.md",
    ] {
        let document = repository_file(path);
        for marker in ["#1017", "#1018"] {
            assert!(document.contains(marker), "{path} is missing {marker}");
        }
    }
    for source in ["annotations", "analysis", "ci-logs", "runs"] {
        assert!(
            repository_path(&format!("{ARCHIVE}/{source}")).is_dir(),
            "the archive must retain {source}/ so every claim re-derives offline"
        );
    }
    assert!(
        crate::assembled_docs::changelog().contains("Fix issue #1017 in pull request #1018"),
        "the changelog must record the delivery"
    );
}
