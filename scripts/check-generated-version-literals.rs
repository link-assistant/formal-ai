#!/usr/bin/env rust-script
//! No memorized third-party versions under `data/` (issue #1168 R7).
//!
//! A generated workflow resolves its pins at generation time
//! (`rust/src/version_resolution.rs`); the templates it renders from are
//! data. A literal 40-hex commit SHA after `@`, or a bare toolchain version
//! behind `version:` / `java-version:` / `kotlin-version:` /
//! `python-version:`, is therefore a pin that stopped being resolved -- it
//! will ship stale forever, because nothing in the tree moves it.
//!
//! The shipped baseline in `data/seed/toolchains.lino` is exempt by shape,
//! not by name: its `latest_sha "…"` fields are not `@`-prefixed and its
//! `latest_tag "…"` fields are not `version:` keys, so the gate reads them
//! as data rather than as pins. Captured external evidence under
//! `data/cache/` and `data/source-cache/` (the git-ignored store fetched
//! pages are written to at run time) and the issue bodies captured verbatim
//! under `data/benchmarks/issue-requirements/` (PR #1188 R1188-U20) is not
//! generated code and is not scanned.
//!
//! This gate is red on a tree whose templates still carry the literals; the
//! same change that rewrites them to `{placeholders}` (R6) turns it green.
//!
//! Usage:
//!   rust-script scripts/check-generated-version-literals.rs [--repo <path>]

// The unit crate includes this file as a module and drives `scan` directly.
#![cfg_attr(test, allow(dead_code))]

use std::fs;
use std::path::{Path, PathBuf};

/// One memorized pin the tree should have resolved instead.
#[derive(Debug)]
pub struct Finding {
    pub path: PathBuf,
    pub line: usize,
    pub detail: String,
}

fn main() {
    let mut repo = PathBuf::from(".");
    let mut arguments = std::env::args().skip(1);
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--repo" => {
                repo = arguments
                    .next()
                    .unwrap_or_else(|| usage("--repo needs a path"))
                    .into();
            }
            other => usage(&format!("unknown flag {other}")),
        }
    }
    let data = repo.join("data");
    if !data.is_dir() {
        usage("the repository root must contain data/");
    }
    let findings = scan_data(&data);
    for finding in &findings {
        println!(
            "{}:{} {}",
            finding.path.display(),
            finding.line,
            finding.detail
        );
    }
    println!(
        "generated-version literals: {} finding(s) under data/ (captured evidence excluded)",
        findings.len()
    );
    if findings.is_empty() {
        println!("generated-version literals: clean");
        return;
    }
    println!(
        "generated-version literals: FAIL - resolve these pins or replace them with {{placeholders}}"
    );
    std::process::exit(1);
}

fn usage(message: &str) -> ! {
    eprintln!("check-generated-version-literals: {message}");
    std::process::exit(2);
}

/// Directories under `data/` that hold captured evidence, not generated code.
///
/// A bare name matches a directory of that name at any depth; a path matches
/// that directory under `data/`.
const CAPTURED_EVIDENCE: [&str; 3] = ["cache", "source-cache", "benchmarks/issue-requirements"];

/// Every memorized pin under `data/`, captured evidence excluded.
pub fn scan_data(data: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    walk(data, data, &mut findings);
    findings
}

fn walk(root: &Path, dir: &Path, findings: &mut Vec<Finding>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_captured_evidence(root, &path) {
                continue;
            }
            walk(root, &path, findings);
        } else if path.is_file() {
            scan_file(root, &path, findings);
        }
    }
}

/// Whether the directory `path` under the `data/` root holds captured evidence.
fn is_captured_evidence(root: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    CAPTURED_EVIDENCE.iter().any(|captured| {
        if captured.contains('/') {
            relative == Path::new(captured)
        } else {
            path.file_name().is_some_and(|name| name == *captured)
        }
    })
}

fn scan_file(root: &Path, path: &Path, findings: &mut Vec<Finding>) {
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };
    for (index, line) in text.lines().enumerate() {
        if let Some(offset) = literal_action_sha(line) {
            findings.push(Finding {
                path: path.strip_prefix(root.parent().unwrap_or(root)).unwrap_or(path).to_path_buf(),
                line: index + 1,
                detail: format!(
                    "literal action SHA at column {}: pin the resolved SHA or a {{placeholder}} instead ({})",
                    offset + 1,
                    &line[offset..(offset + 41).min(line.len())]
                ),
            });
        }
        if let Some(key) = literal_toolchain_version(line) {
            findings.push(Finding {
                path: path
                    .strip_prefix(root.parent().unwrap_or(root))
                    .unwrap_or(path)
                    .to_path_buf(),
                line: index + 1,
                detail: format!(
                    "literal toolchain version behind `{key}`: resolve it at generation time"
                ),
            });
        }
    }
}

/// The offset of an `@` followed by exactly forty hex characters, when the
/// line has one.
fn literal_action_sha(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    (0..bytes.len().saturating_sub(40)).find(|&offset| {
        bytes[offset] == b'@'
            && bytes[offset + 1..offset + 41]
                .iter()
                .all(u8::is_ascii_hexdigit)
            && bytes
                .get(offset + 41)
                .is_none_or(|b| !b.is_ascii_hexdigit())
    })
}

/// The `version:`-shaped key this line pins a bare version behind, when it
/// does. `latest_tag "v7.0.1"` baseline fields carry no such key.
fn literal_toolchain_version(line: &str) -> Option<&'static str> {
    for key in [
        "kotlin-version:",
        "python-version:",
        "java-version:",
        "version:",
    ] {
        let Some(offset) = line.find(key) else {
            continue;
        };
        let value = line[offset + key.len()..].trim_start();
        let quoted = value.chars().next().is_some_and(|c| c == '\'' || c == '"');
        let digits = if quoted {
            value.chars().nth(1)
        } else {
            value.chars().next()
        };
        if digits.is_some_and(|c| c.is_ascii_digit()) {
            return Some(match key {
                "kotlin-version:" => "kotlin-version",
                "python-version:" => "python-version",
                "java-version:" => "java-version",
                _ => "version",
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_forty_hex_sha_after_at_is_a_finding() {
        assert_eq!(
            literal_action_sha(
                "      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"
            ),
            Some(30)
        );
    }

    #[test]
    fn a_tag_ref_is_not_a_sha_finding() {
        assert_eq!(
            literal_action_sha("      - uses: actions/checkout@v7"),
            None
        );
    }

    #[test]
    fn a_sixty_four_hex_digest_without_at_is_not_a_finding() {
        assert_eq!(
            literal_action_sha(
                "  sha256 \"a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2\""
            ),
            None
        );
    }

    #[test]
    fn the_baseline_fields_are_data_not_pins() {
        assert_eq!(
            literal_action_sha("  latest_sha \"3d3c42e5aac5ba805825da76410c181273ba90b1\""),
            None
        );
        assert_eq!(literal_toolchain_version("  latest_tag \"v7.0.1\""), None);
        assert_eq!(
            literal_toolchain_version("  releases_url \"https://api.github.com\""),
            None
        );
    }

    #[test]
    fn quoted_and_bare_toolchain_versions_are_findings() {
        assert_eq!(
            literal_toolchain_version("          version: '2.3.10'"),
            Some("version")
        );
        assert_eq!(
            literal_toolchain_version("          java-version: '21'"),
            Some("java-version")
        );
        assert_eq!(
            literal_toolchain_version("          python-version: '3.14'"),
            Some("python-version")
        );
        assert_eq!(
            literal_toolchain_version("          kotlin-version: \"2.3.10\""),
            Some("kotlin-version")
        );
    }

    #[test]
    fn a_non_version_key_is_left_alone() {
        assert_eq!(
            literal_toolchain_version("          verbose: 'quite'"),
            None
        );
    }
}
