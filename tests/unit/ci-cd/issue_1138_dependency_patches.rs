//! Dependency policy gate for issue #1138 (plan 16, leaf L6).
//!
//! The doctrine: when a translator or a layer check needs a feature a
//! self-maintained dependency has not released, the workaround is a
//! `[patch]`-style source install, and the debt it creates is tracked where
//! debt belongs -- in an issue, not in the manifest. Every patch or source
//! install in `Cargo.toml` must carry a comment beside it referencing, by
//! URL, the issue that lists what is patched and what blocks using the
//! latest release.
//!
//! The referenced issues are written generally (never Formal-AI-specific),
//! so the general case moves into the dependency and this repository's own
//! code shrinks; the patch is a bridge, and the issue is the plan to remove
//! it. A patch without an issue is invisible debt: nothing fails when the
//! upstream release lands, so nothing ever removes the patch.
//!
//! This gate is deliberately offline. It checks reference presence and URL
//! shape against the manifest text and cross-checks the lockfile for git
//! sources that a manifest comment might have missed; it does not verify
//! that the issue is still open, because the unit suite runs `--offline`.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// A patch or source-install entry found in a manifest, with the reference
/// (or lack of one) in the comments directly above it.
struct PatchEntry {
    line_number: usize,
    dependency: String,
    issue_reference: Option<String>,
}

const ISSUE_URL_PATTERN: &str = "https://github.com/link-assistant/formal-ai/issues/";

/// Scan manifest text for entries that install a dependency from a source
/// other than a registry: dependency lines carrying `git =` (or `path =`
/// pointing outside the workspace) and any `[patch.` section content. Each
/// entry is attributed the nearest block of preceding `#` comments.
fn patch_entries(manifest: &str) -> Vec<PatchEntry> {
    let mut entries = Vec::new();
    let mut pending_comment: Vec<String> = Vec::new();
    let mut in_patch_section = false;
    for (index, line) in manifest.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            pending_comment.push(trimmed.trim_start_matches('#').trim().to_owned());
            continue;
        }
        if trimmed.starts_with('[') {
            // Section headers keep the pending comment block: a reference
            // written above `[patch.crates-io]` attributes to the entries
            // inside it, matching how a reader finds it.
            in_patch_section = trimmed.starts_with("[patch");
            continue;
        }
        let is_source_install =
            trimmed.contains("git =") || (in_patch_section && trimmed.contains('='));
        if !is_source_install || trimmed.is_empty() {
            if !trimmed.is_empty() {
                pending_comment.clear();
            }
            continue;
        }
        let dependency = trimmed
            .split('=')
            .next()
            .map(str::trim)
            .unwrap_or_default()
            .to_owned();
        let issue_reference = pending_comment.iter().find_map(|comment| {
            comment.find(ISSUE_URL_PATTERN).map(|start| {
                comment[start..]
                    .trim_end_matches(|c: char| !c.is_ascii_alphanumeric())
                    .to_owned()
            })
        });
        entries.push(PatchEntry {
            line_number: index + 1,
            dependency,
            issue_reference,
        });
        pending_comment.clear();
    }
    entries
}

fn entry_is_referenced(entry: &PatchEntry) -> bool {
    match &entry.issue_reference {
        None => false,
        Some(url) => {
            let tail = url.strip_prefix(ISSUE_URL_PATTERN).unwrap_or_default();
            !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit())
        }
    }
}

/// Git dependencies the lockfile actually resolved -- the ground truth that
/// survives manifest reformatting. A registry-only lockfile yields none.
fn lockfile_git_sources(lockfile: &str) -> Vec<String> {
    lockfile
        .lines()
        .filter(|line| line.trim_start().starts_with("source ="))
        .filter(|line| line.contains("git+"))
        .map(|line| line.trim().to_owned())
        .collect()
}

#[test]
fn every_patch_entry_references_an_issue_by_url() {
    let entries = patch_entries(&repository_file("Cargo.toml"));
    let unreferenced: Vec<&PatchEntry> =
        entries.iter().filter(|e| !entry_is_referenced(e)).collect();
    assert!(
        unreferenced.is_empty(),
        "patched or source-installed dependencies without a tracking issue \
         beside them: {}",
        unreferenced
            .iter()
            .map(|entry| format!(
                "{} (Cargo.toml line {})",
                entry.dependency, entry.line_number
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[test]
fn lockfile_git_sources_are_all_declared_in_the_manifest() {
    let declared = lockfile_git_sources(&repository_file("Cargo.lock"));
    assert!(
        declared.is_empty(),
        "Cargo.lock resolved git sources that Cargo.toml does not declare as \
         tracked patch entries: {}",
        declared.join(", ")
    );
}

#[test]
fn the_gate_fires_on_an_unreferenced_patch() {
    let bad_manifest = "[patch.crates-io]\nsome-dep = { git = \"https://example.com/some-dep\" }\n";
    let entries = patch_entries(bad_manifest);
    assert_eq!(entries.len(), 1, "the fixture patch must be discovered");
    assert!(
        !entry_is_referenced(&entries[0]),
        "a patch without an issue reference must fail the gate"
    );

    let good_manifest = "# tracked in https://github.com/link-assistant/formal-ai/issues/1200\n[patch.crates-io]\nsome-dep = { git = \"https://example.com/some-dep\" }\n";
    let entries = patch_entries(good_manifest);
    assert_eq!(entries.len(), 1);
    assert!(
        entry_is_referenced(&entries[0]),
        "a patch with a well-formed issue reference must pass"
    );

    let malformed_manifest = "# see the team board\n[patch.crates-io]\nsome-dep = { git = \"https://example.com/some-dep\" }\n";
    let entries = patch_entries(malformed_manifest);
    assert!(
        !entry_is_referenced(&entries[0]),
        "a comment without the issue URL must fail the gate"
    );
}
