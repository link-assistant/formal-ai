//! Tracked-text readers and label scanning for the documentation contracts.

use std::fs;
use std::path::Path;

pub(super) fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
}

pub(super) fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

pub(super) fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}

/// The repository's own text: the files git tracks. A working-tree walk also read
/// what a job left in the checkout (the coverage archive ran it out of memory).
pub(super) fn tracked_files(root: &Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .expect("git lists the tracked files");
    assert!(output.status.success(), "git ls-files failed");
    String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether a tracked file or directory is verbatim or historical text the lint skips.
pub(super) fn is_skipped_tree(relative: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    // Verbatim external captures archived alongside a case study (the issue/PR
    // JSON snapshots under `docs/case-studies/<issue>/raw-data`) are quoted
    // third-party text, not authored documentation: an issue author may ask for
    // a quick prototype in wording this lint forbids in the project's own prose.
    if relative.starts_with("docs/case-studies/") && relative.ends_with("/raw-data") {
        return true;
    }
    // Recorded self-use sessions are the same kind of verbatim third-party
    // evidence as `raw-data`: `agent.log` contains provider responses and
    // downloaded payloads exactly as observed. Keep authored self-use prose in
    // scope while leaving the immutable transcript bytes untouched.
    if relative.starts_with("docs/case-studies/")
        && relative.contains("/self-use/")
        && name == "agent.log"
    {
        return true;
    }

    // Released changelog text and its provenance map are immutable historical
    // records. They can quote old project terminology without reintroducing it
    // into current product documentation; `docs/changelog/` is its archive.
    if matches!(
        relative,
        "CHANGELOG.md" | "docs/changelog" | "docs/case-studies/issue-711/fragment-release-map.tsv"
    ) || relative.starts_with("docs/changelog/")
    {
        return true;
    }

    matches!(
        relative,
        "ci-logs"
            // Verbatim issue, pull-request, CI, and research captures gathered
            // by the issue solver. Like case-study raw-data, these are external
            // evidence rather than authored product documentation.
            | "dev/log"
            | "logs"
            | "rust/tests/e2e/playwright-report"
            | "rust/tests/e2e/test-results"
            | "data/wikidata-cache"
            | "data/wiktionary-cache"
            | "data/http-cache"
            | "data/seed/api-cache"
            // The content-addressed store of bytes retrieved by `--online`
            // source discovery (official OEIS JSON and the pages it cites).
            // Like the caches above it is verbatim third-party text, and it
            // is git-ignored: the pin governs authored repository prose.
            | "data/source-cache"
            // Git-ignored generated mirrors of already-scanned source: the
            // VS Code packaging step copies js -> vscode/dist-web (with
            // data/seed -> vscode/dist-web/seed) and desktop/lib helpers ->
            // vscode/src/lib/vendor. Scanning the originals is enough.
            | "vscode/dist-web"
            | "vscode/src/lib/vendor"
            // Verbatim third-party CLI output captured by the issue-#671 agentic
            // matrix: `artifacts/` is the git-ignored scratch of a local run and
            // `recorded/` holds the committed transcripts a replay asserts
            // against. Both quote whatever the vendor CLI printed, so editing
            // them to satisfy a prose lint would falsify the evidence.
            | "experiments/agentic_cli_matrix/artifacts"
            | "experiments/agentic_cli_matrix/recorded"
    )
}

pub(super) fn collect_for_haystack<'a>(
    relative: &str,
    source: &str,
    haystack: &str,
    phrase_labels: impl IntoIterator<Item = &'a String>,
    compact_labels: &[String],
    findings: &mut Vec<String>,
) {
    for label in phrase_labels {
        if haystack.contains(label) {
            findings.push(format!("{relative} {source}: {label}"));
        }
    }

    for label in compact_labels {
        if contains_compact_label(haystack, label) {
            findings.push(format!("{relative} {source}: {label}"));
        }
    }
}

fn contains_compact_label(haystack: &str, label: &str) -> bool {
    haystack
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|part| part == label)
}
