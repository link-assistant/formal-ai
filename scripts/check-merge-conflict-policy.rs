#!/usr/bin/env rust-script
//! Keep the merge-conflict policy and the repository in agreement.
//!
//! Issue #991 review feedback: "the end result should be that probability of
//! conflicts in the future reduced to zero ... all other similar places which
//! may generate conflicts should be fixed in similar way."
//!
//! `data/meta/merge-conflict-policy.lino` is the reviewed registry: it names
//! every structural cause of a merge conflict this repository has actually
//! seen, the mechanism assigned to it, and the paths that mechanism covers.
//! A registry only stays true if something fails when it drifts, so this gate
//! checks three things:
//!
//! 1. every path the registry marks `merge union` really is `merge=union` in
//!    `.gitattributes`, and every `merge=union` line in `.gitattributes` is
//!    accounted for by the registry;
//! 2. every command the registry names (`regenerate`, `verify`, `normalize`)
//!    points at a file that exists, so the documented repair is runnable;
//! 3. every union-merged path has a verifier, because a union never blocks a
//!    merge and an unverified union would land silently.
//!
//! Usage: rust-script scripts/check-merge-conflict-policy.rs
//!
//! ```cargo
//! [dependencies]
//! ```

use std::collections::BTreeSet;

#[cfg(not(test))]
const POLICY: &str = "data/meta/merge-conflict-policy.lino";
#[cfg(not(test))]
const ATTRIBUTES: &str = ".gitattributes";

/// One `artifact` entry of the registry.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Artifact {
    name: String,
    cause: String,
    merge: Option<String>,
    paths: Vec<String>,
    commands: Vec<String>,
    has_verifier: bool,
    /// Set when every possible union of the file is already the correct content,
    /// so there is no canonical form left for a normalizer to restore.
    union_is_terminal: bool,
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_string()
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Read the `cause` and `artifact` entries of the registry.
fn parse_policy(source: &str) -> (BTreeSet<String>, Vec<Artifact>) {
    let mut causes = BTreeSet::new();
    let mut artifacts: Vec<Artifact> = Vec::new();

    for line in source.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = indent_of(line);
        let trimmed = line.trim();
        let (key, value) = match trimmed.split_once(' ') {
            Some((key, value)) => (key, value.trim()),
            None => (trimmed, ""),
        };

        match (indent, key) {
            (2, "cause") => {
                causes.insert(unquote(value));
            }
            (2, "artifact") => artifacts.push(Artifact {
                name: unquote(value),
                ..Artifact::default()
            }),
            (4 | 6, _) => {
                let Some(artifact) = artifacts.last_mut() else {
                    continue;
                };
                match key {
                    "cause" if indent == 4 => artifact.cause = unquote(value),
                    "merge" if indent == 4 => artifact.merge = Some(unquote(value)),
                    "union_is_terminal" if indent == 4 => {
                        artifact.union_is_terminal = unquote(value) == "true"
                    }
                    "path" => artifact.paths.push(unquote(value)),
                    "regenerate" | "normalize" => artifact.commands.push(unquote(value)),
                    "verify" => {
                        artifact.commands.push(unquote(value));
                        artifact.has_verifier = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    (causes, artifacts)
}

/// The paths `.gitattributes` marks `merge=union`.
fn union_attributes(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (path, attributes) = line.split_once(' ')?;
            attributes
                .split_whitespace()
                .any(|attribute| attribute == "merge=union")
                .then(|| path.to_string())
        })
        .collect()
}

/// The file a shell command runs, for commands that name one.
fn command_target(command: &str) -> Option<String> {
    command
        .split_whitespace()
        .find(|word| {
            (word.starts_with("scripts/") || word.starts_with("examples/"))
                && (word.ends_with(".rs") || word.ends_with(".py") || word.ends_with(".sh"))
        })
        .map(str::to_string)
}

/// Every rule the registry has to satisfy, as human-readable failures.
fn violations(
    causes: &BTreeSet<String>,
    artifacts: &[Artifact],
    attributes: &BTreeSet<String>,
    exists: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut failures = Vec::new();
    let mut registered_union: BTreeSet<String> = BTreeSet::new();

    for artifact in artifacts {
        if !causes.contains(&artifact.cause) {
            failures.push(format!(
                "artifact `{}` names cause `{}`, which the registry does not define",
                artifact.name, artifact.cause
            ));
        }
        if artifact.paths.is_empty() {
            failures.push(format!("artifact `{}` registers no path", artifact.name));
        }
        if artifact.merge.as_deref() == Some("union") {
            if !artifact.has_verifier && !artifact.union_is_terminal {
                failures.push(format!(
                    "artifact `{}` is union merged but registers no verify command; \
                     a union never blocks a merge, so an unverified union lands silently. \
                     Add a verify command, or declare `union_is_terminal true` if every \
                     union of this path is already the correct content",
                    artifact.name
                ));
            }
            for path in &artifact.paths {
                registered_union.insert(path.clone());
                if !attributes.contains(path) {
                    failures.push(format!(
                        "`{path}` is union merged in the registry but not in .gitattributes"
                    ));
                }
            }
        }
        for command in &artifact.commands {
            if let Some(target) = command_target(command) {
                if !exists(&target) {
                    failures.push(format!(
                        "artifact `{}` runs `{command}`, but `{target}` does not exist",
                        artifact.name
                    ));
                }
            }
        }
    }

    for path in attributes.difference(&registered_union) {
        failures.push(format!(
            "`{path}` is union merged in .gitattributes but the registry does not cover it"
        ));
    }
    failures
}

#[cfg(not(test))]
fn main() {
    let root = std::env::current_dir().expect("Failed to get current directory");
    let policy = std::fs::read_to_string(root.join(POLICY)).unwrap_or_else(|error| {
        println!("::error::Could not read {POLICY}: {error}");
        std::process::exit(1);
    });
    let attributes_source = std::fs::read_to_string(root.join(ATTRIBUTES)).unwrap_or_else(|error| {
        println!("::error::Could not read {ATTRIBUTES}: {error}");
        std::process::exit(1);
    });

    let (causes, artifacts) = parse_policy(&policy);
    let attributes = union_attributes(&attributes_source);
    let exists = |path: &str| root.join(path).exists();

    println!("\nChecking the merge-conflict policy in {POLICY}...\n");
    println!(
        "  {} structural cause(s), {} artifact(s), {} union-merged path(s)\n",
        causes.len(),
        artifacts.len(),
        attributes.len()
    );
    for artifact in &artifacts {
        println!(
            "  {:<26} {:<22} {} path(s)",
            artifact.name,
            artifact.cause,
            artifact.paths.len()
        );
    }

    let failures = violations(&causes, &artifacts, &attributes, &exists);
    if failures.is_empty() {
        println!("\nThe registry, .gitattributes and the repository agree.\n");
        return;
    }
    println!();
    for failure in &failures {
        println!("::error::{failure}");
    }
    println!(
        "\n{} merge-conflict policy violation(s). Update {POLICY} or {ATTRIBUTES}.\n",
        failures.len()
    );
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "merge_conflict_policy\n  \
        cause append_only_list\n    \
        description \"...\"\n  \
        cause sequential_file_name\n    \
        description \"...\"\n  \
        artifact rust_declaration_lists\n    \
        cause append_only_list\n    \
        merge union\n    \
        verify \"rust-script scripts/normalize-ordered-lists.rs\"\n    \
        file\n      \
        path \"src/lib.rs\"\n  \
        artifact worker_modules\n    \
        cause sequential_file_name\n    \
        path \"src/web/worker/formal_ai_worker_*.js\"\n";

    fn always_exists(_path: &str) -> bool {
        true
    }

    #[test]
    fn the_registry_parses_into_causes_and_artifacts() {
        let (causes, artifacts) = parse_policy(SAMPLE);
        assert_eq!(
            causes,
            BTreeSet::from([
                "append_only_list".to_string(),
                "sequential_file_name".to_string()
            ])
        );
        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[0].paths, ["src/lib.rs"]);
        assert_eq!(artifacts[0].merge.as_deref(), Some("union"));
        assert!(artifacts[0].has_verifier);
        assert_eq!(artifacts[1].merge, None);
    }

    #[test]
    fn gitattributes_union_lines_are_recognised_and_comments_ignored() {
        let attributes = union_attributes(
            "# src/ignored.rs merge=union\n*.lino text eol=lf\nsrc/lib.rs merge=union\n",
        );
        assert_eq!(attributes, BTreeSet::from(["src/lib.rs".to_string()]));
    }

    #[test]
    fn a_registry_that_matches_the_repository_reports_nothing() {
        let (causes, artifacts) = parse_policy(SAMPLE);
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        assert!(violations(&causes, &artifacts, &attributes, &always_exists).is_empty());
    }

    #[test]
    fn a_union_path_missing_from_gitattributes_fails() {
        let (causes, artifacts) = parse_policy(SAMPLE);
        let failures = violations(&causes, &artifacts, &BTreeSet::new(), &always_exists);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("src/lib.rs") && failure.contains(".gitattributes")),
            "{failures:?}"
        );
    }

    #[test]
    fn an_unregistered_union_path_fails() {
        let (causes, artifacts) = parse_policy(SAMPLE);
        let attributes =
            BTreeSet::from(["src/lib.rs".to_string(), "src/surprise.rs".to_string()]);
        let failures = violations(&causes, &artifacts, &attributes, &always_exists);
        assert!(
            failures.iter().any(|failure| failure.contains("src/surprise.rs")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_union_artifact_without_a_verifier_fails() {
        let source = SAMPLE.replace("    verify \"rust-script scripts/normalize-ordered-lists.rs\"\n", "");
        let (causes, artifacts) = parse_policy(&source);
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(&causes, &artifacts, &attributes, &always_exists);
        assert!(
            failures.iter().any(|failure| failure.contains("registers no verify command")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_union_artifact_that_declares_union_is_terminal_needs_no_verifier() {
        let source = SAMPLE
            .replace(
                "    verify \"rust-script scripts/normalize-ordered-lists.rs\"\n",
                "    union_is_terminal true\n",
            );
        let (causes, artifacts) = parse_policy(&source);
        assert!(artifacts[0].union_is_terminal);
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        assert!(violations(&causes, &artifacts, &attributes, &always_exists).is_empty());
    }

    #[test]
    fn a_command_pointing_at_a_missing_script_fails() {
        let (causes, artifacts) = parse_policy(SAMPLE);
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(&causes, &artifacts, &attributes, &|_path| false);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("scripts/normalize-ordered-lists.rs")),
            "{failures:?}"
        );
    }

    #[test]
    fn an_undefined_cause_fails() {
        let source = SAMPLE.replace("cause sequential_file_name\n    description", "cause renamed\n    description");
        let (causes, artifacts) = parse_policy(&source);
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(&causes, &artifacts, &attributes, &always_exists);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("sequential_file_name")),
            "renaming a cause leaves the artifact that names it unbacked: {failures:?}"
        );
    }

    #[test]
    fn a_command_target_is_the_script_it_runs() {
        assert_eq!(
            command_target("rust-script scripts/audit-seed-metadata.rs --write").as_deref(),
            Some("scripts/audit-seed-metadata.rs")
        );
        assert_eq!(
            command_target("python3 scripts/close-total.py").as_deref(),
            Some("scripts/close-total.py")
        );
        assert_eq!(
            command_target("cargo run --example regenerate_self_ast_census"),
            None,
            "a cargo example is not a path on disk"
        );
    }
}
