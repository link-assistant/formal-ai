//! Issue #894 — no confirmed finding of the four-template CI audit may remain
//! "ready to file" without an upstream issue URL.
//!
//! The 2026-06-14 audit
//! (`docs/case-studies/issue-479/template-comparison/REPORT.md`) closed with
//! drafted-but-unfiled upstream recommendations. A recommendation with no URL is
//! indistinguishable from an unreported gap and, once the templates move on,
//! from a gap that no longer exists. These tests read the report's filing ledger
//! and fail if that state ever returns.

use std::fs;
use std::path::Path;

/// Path of the audit report whose ledger this check enforces.
const REPORT: &str = "docs/case-studies/issue-479/template-comparison/REPORT.md";

/// Heading that opens the filing ledger inside the report.
const LEDGER_HEADING: &str = "## Upstream filing status (revalidated 2026-08-05)";

/// The status vocabulary the ledger documents. Any other status word in a ledger
/// row is rejected, so a new state cannot be introduced without also stating what
/// it means (and whether it needs a URL).
const KNOWN_STATUSES: &[&str] = &["confirmed", "obsolete", "not-applicable", "local"];

/// Every gap that was confirmed against the current template default branches,
/// with the upstream issue it was filed as. `confirmed` is the only status that
/// requires a URL, and every one of these must be present in the ledger.
const CONFIRMED_FILINGS: &[(&str, &str)] = &[
    (
        "U1-js",
        "https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/122",
    ),
    (
        "U1-rust",
        "https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/115",
    ),
    (
        "U1-python",
        "https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/48",
    ),
    (
        "U1-csharp",
        "https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/43",
    ),
    (
        "U2-rust",
        "https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/116",
    ),
    (
        "U2-python",
        "https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/49",
    ),
    (
        "U2-csharp",
        "https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/44",
    ),
    (
        "U3-rust",
        "https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/117",
    ),
];

/// A parsed `| ID | ... | status | filing |` row of the ledger.
#[derive(Debug)]
struct LedgerRow {
    id: String,
    status: String,
    filing: String,
}

#[test]
fn issue_894_every_confirmed_finding_carries_an_upstream_filing_url() {
    let rows = ledger_rows();
    assert!(
        !rows.is_empty(),
        "{REPORT} should contain a parseable filing ledger under {LEDGER_HEADING}",
    );

    for row in &rows {
        assert!(
            KNOWN_STATUSES.contains(&row.status.as_str()),
            "{REPORT} row {} uses undocumented status {:?}; the ledger vocabulary is {KNOWN_STATUSES:?}",
            row.id,
            row.status,
        );

        if row.status != "confirmed" {
            continue;
        }

        assert!(
            row.filing.contains("https://github.com/"),
            "{REPORT} row {} is confirmed but has no upstream issue URL; a confirmed \
             finding may never remain ready-to-file without a filing (issue #894)",
            row.id,
        );
        assert!(
            row.filing.contains("/issues/"),
            "{REPORT} row {} links {:?}, which is not an issue URL",
            row.id,
            row.filing,
        );
    }

    let confirmed = rows.iter().filter(|row| row.status == "confirmed").count();
    assert_eq!(
        confirmed,
        CONFIRMED_FILINGS.len(),
        "{REPORT} should list exactly {} confirmed findings; found {confirmed}",
        CONFIRMED_FILINGS.len(),
    );
}

#[test]
fn issue_894_each_known_filing_is_linked_from_the_report() {
    let rows = ledger_rows();

    for (id, url) in CONFIRMED_FILINGS {
        let row = rows
            .iter()
            .find(|row| row.id == *id)
            .unwrap_or_else(|| panic!("{REPORT} should contain a ledger row for {id}"));
        assert_eq!(
            row.status, "confirmed",
            "{REPORT} row {id} should stay confirmed while {url} tracks it",
        );
        assert!(
            row.filing.contains(url),
            "{REPORT} row {id} should link {url}, found {:?}",
            row.filing,
        );
    }
}

#[test]
fn issue_894_report_no_longer_holds_unfiled_recommendations() {
    let report = read(repository_root().join(REPORT));

    assert!(
        report.contains(LEDGER_HEADING),
        "{REPORT} should carry the filing ledger heading {LEDGER_HEADING}",
    );
    assert!(
        !report.contains("## Recommended upstream issues to file"),
        "{REPORT} should not reintroduce the pre-filing recommendation section; \
         recommendations belong in the ledger with a status and a URL",
    );
    assert!(
        report.contains("**Revalidated:** 2026-08-05"),
        "{REPORT} should record the revalidation date in its header",
    );

    // Obsolete findings must be marked as such rather than silently dropped.
    for obsolete in ["| L1 |", "| L3 |", "| L4 |", "| L7 |"] {
        let row = report
            .lines()
            .find(|line| line.trim_start().starts_with(obsolete))
            .unwrap_or_else(|| panic!("{REPORT} should keep a ledger row starting {obsolete}"));
        assert!(
            row.contains("obsolete"),
            "{REPORT} row {obsolete} should be marked obsolete, found: {row}",
        );
    }
}

#[test]
fn issue_894_revalidation_evidence_is_preserved() {
    let root = repository_root();

    for relative in [
        "docs/case-studies/issue-894/README.md",
        "docs/case-studies/issue-894/requirements.md",
        "docs/case-studies/issue-894/raw-data/template-heads.json",
        "docs/case-studies/issue-894/raw-data/filed-upstream-issues.json",
        "docs/case-studies/issue-894/raw-data/revalidation-greps.txt",
        "docs/case-studies/issue-894/raw-data/revalidation-greps-2.txt",
        "docs/case-studies/issue-894/raw-data/js-tree.txt",
        "docs/case-studies/issue-894/raw-data/rust-tree.txt",
        "docs/case-studies/issue-894/raw-data/python-tree.txt",
        "docs/case-studies/issue-894/raw-data/csharp-tree.txt",
        "docs/case-studies/issue-894/raw-data/sec-js.md",
        "docs/case-studies/issue-894/raw-data/sec-rust.md",
        "docs/case-studies/issue-894/raw-data/sec-python.md",
        "docs/case-studies/issue-894/raw-data/sec-csharp.md",
        "docs/case-studies/issue-894/raw-data/links-rust.md",
        "docs/case-studies/issue-894/raw-data/links-python.md",
        "docs/case-studies/issue-894/raw-data/links-csharp.md",
        "docs/case-studies/issue-894/raw-data/desktop-rust.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #894 traceability",
        );
    }

    // Every filed issue must appear in the snapshot, so the ledger can be audited
    // against what GitHub actually holds.
    let snapshot =
        read(root.join("docs/case-studies/issue-894/raw-data/filed-upstream-issues.json"));
    for (id, url) in CONFIRMED_FILINGS {
        assert!(
            snapshot.contains(url),
            "filed-upstream-issues.json should snapshot {id} ({url})",
        );
    }
}

#[test]
fn issue_894_requirements_are_traceable() {
    let root = repository_root();

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #894 CI Template Upstream Filings",
            "| R894-1 ",
            "| R894-2 ",
            "| R894-3 ",
            "| R894-4 ",
            "docs/case-studies/issue-894/",
            "tests/unit/docs_requirements_issue_894.rs",
        ],
    );

    let traceability = read(root.join("docs/requirements-traceability.md"));
    assert_contains_all(
        "docs/requirements-traceability.md",
        &traceability,
        &["| R894-1 |", "| R894-4 |"],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-894/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-894/requirements.md",
        &issue_requirements,
        &[
            "R894-1",
            "R894-4",
            "Reproduction",
            "Workaround",
            "Suggested fix",
        ],
    );
}

fn repository_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Parse the `| ID | finding | owner | status | filing |` and
/// `| ID | finding | status | note |` tables that follow [`LEDGER_HEADING`].
///
/// The status is the second-to-last cell and the filing/note the last one, which
/// holds for both table shapes.
fn ledger_rows() -> Vec<LedgerRow> {
    let report = read(repository_root().join(REPORT));
    let ledger = report
        .split_once(LEDGER_HEADING)
        .unwrap_or_else(|| panic!("{REPORT} should contain {LEDGER_HEADING}"))
        .1;
    // The ledger ends at the next top-level section.
    let ledger = ledger.split("\n## ").next().unwrap_or(ledger);

    ledger
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('|'))
        .filter_map(|line| {
            let cells = line
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            if cells.len() < 4 {
                return None;
            }
            let id = cells[0];
            // Skip header and separator rows.
            if id == "ID"
                || id
                    .chars()
                    .all(|character| character == '-' || character == ' ')
            {
                return None;
            }
            Some(LedgerRow {
                id: id.to_string(),
                status: cells[cells.len() - 2].to_string(),
                filing: cells[cells.len() - 1].to_string(),
            })
        })
        .collect()
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}",
        );
    }
}
