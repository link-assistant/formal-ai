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
//! checks five things:
//!
//! 1. every path the registry marks `merge union` really is `merge=union` in
//!    `.gitattributes`, and every `merge=union` line in `.gitattributes` is
//!    accounted for by the registry;
//! 2. every command the registry names (`regenerate`, `verify`, `normalize`)
//!    points at a file that exists, so the documented repair is runnable;
//! 3. every union-merged path has a verifier, because a union never blocks a
//!    merge and an unverified union would land silently;
//! 4. every path in `data/meta/merge-conflict-ledger.lino` -- the measured list
//!    of files this repository has actually had to conflict-resolve -- is either
//!    covered by a mechanism or deferred with a stated reason. Without this the
//!    registry would only ever describe the conflicts somebody remembered;
//! 5. a ledger path a union artifact claims really resolves to `merge=union`,
//!    asked of git itself rather than by matching the pattern text. Gitattribute
//!    globs follow gitignore rules, where `*` stops at a `/`, so a pattern like
//!    `data/meta/self-ast/*.lino` silently covers none of the nested files it
//!    reads as though it covers.
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
#[cfg(not(test))]
const LEDGER: &str = "data/meta/merge-conflict-ledger.lino";

/// One `artifact` entry of the registry.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Artifact {
    name: String,
    cause: String,
    merge: Option<String>,
    paths: Vec<String>,
    /// Paths this artifact retired: the list used to live there and was moved
    /// out, so the old path stops being an append point even though the
    /// mechanism does not name it.
    extracted_from: Vec<String>,
    commands: Vec<String>,
    has_verifier: bool,
    /// Set when every possible union of the file is already the correct content,
    /// so there is no canonical form left for a normalizer to restore.
    union_is_terminal: bool,
}

/// Ledger paths the policy knowingly leaves unmechanized, and why.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Deferral {
    name: String,
    reason: String,
    paths: Vec<String>,
}

/// A path the measured ledger ranks, with how often it needed resolving.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LedgerPath {
    path: String,
    events: u32,
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

/// The registry, as the checks below consume it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Policy {
    causes: BTreeSet<String>,
    artifacts: Vec<Artifact>,
    deferrals: Vec<Deferral>,
}

/// Read the `cause`, `artifact` and `deferred` entries of the registry.
fn parse_policy(source: &str) -> Policy {
    let mut policy = Policy::default();
    // Which indent-2 entry the indented lines belong to.
    let mut in_deferral = false;

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
                policy.causes.insert(unquote(value));
                in_deferral = false;
            }
            (2, "artifact") => {
                policy.artifacts.push(Artifact {
                    name: unquote(value),
                    ..Artifact::default()
                });
                in_deferral = false;
            }
            (2, "deferred") => {
                policy.deferrals.push(Deferral {
                    name: unquote(value),
                    ..Deferral::default()
                });
                in_deferral = true;
            }
            (2, _) => in_deferral = false,
            (4, _) if in_deferral => {
                if let Some(deferral) = policy.deferrals.last_mut() {
                    match key {
                        "reason" => deferral.reason = unquote(value),
                        "path" => deferral.paths.push(unquote(value)),
                        _ => {}
                    }
                }
            }
            (4 | 6, _) if !in_deferral => {
                let Some(artifact) = policy.artifacts.last_mut() else {
                    continue;
                };
                match key {
                    "cause" if indent == 4 => artifact.cause = unquote(value),
                    "merge" if indent == 4 => artifact.merge = Some(unquote(value)),
                    "union_is_terminal" if indent == 4 => {
                        artifact.union_is_terminal = unquote(value) == "true"
                    }
                    "path" => artifact.paths.push(unquote(value)),
                    "extracted_from" => artifact.extracted_from.push(unquote(value)),
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
    policy
}

/// The ranked paths of `data/meta/merge-conflict-ledger.lino`.
fn parse_ledger(source: &str) -> Vec<LedgerPath> {
    let mut paths: Vec<LedgerPath> = Vec::new();
    for line in source.lines() {
        let indent = indent_of(line);
        let trimmed = line.trim();
        let Some((key, value)) = trimmed.split_once(' ') else {
            continue;
        };
        match (indent, key) {
            (2, "path") => paths.push(LedgerPath {
                path: unquote(value),
                events: 0,
            }),
            (4, "events") => {
                if let Some(entry) = paths.last_mut() {
                    entry.events = value.trim().parse().unwrap_or(0);
                }
            }
            _ => {}
        }
    }
    paths
}

/// Match a path against a gitattributes pattern.
///
/// The rules that matter here are gitignore's: a pattern with no `/` matches the
/// base name at any depth (`.gitkeep`), a single `*` never crosses a `/`, and
/// `**` crosses any number of them.
fn pattern_matches(pattern: &str, path: &str) -> bool {
    if !pattern.contains('/') {
        let name = path.rsplit('/').next().unwrap_or(path);
        return glob_matches(pattern, name);
    }
    glob_matches(pattern, path)
}

fn glob_matches(pattern: &str, text: &str) -> bool {
    match pattern.split_once("**") {
        Some((head, tail)) => {
            let Some(rest) = strip_glob_prefix(head, text) else {
                return false;
            };
            // `**` spans any number of segments, including none.
            (0..=rest.len()).any(|split| rest.is_char_boundary(split) && glob_matches(tail, &rest[split..]))
        }
        None => match pattern.split_once('*') {
            Some((head, tail)) => {
                let Some(rest) = strip_glob_prefix(head, text) else {
                    return false;
                };
                let limit = rest.find('/').unwrap_or(rest.len());
                (0..=limit).any(|split| rest.is_char_boundary(split) && glob_matches(tail, &rest[split..]))
            }
            None => pattern == text,
        },
    }
}

fn strip_glob_prefix<'a>(head: &str, text: &'a str) -> Option<&'a str> {
    text.strip_prefix(head)
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

/// How an artifact accounts for a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Coverage {
    /// The mechanism acts on this path itself.
    Mechanized,
    /// The list that used to conflict here was moved out. The old file is left
    /// alone -- it is ordinary code again -- so it is *not* union merged.
    Retired,
}

/// The artifact that accounts for `path`, and how.
fn covering_artifact<'a>(artifacts: &'a [Artifact], path: &str) -> Option<(&'a Artifact, Coverage)> {
    artifacts.iter().find_map(|artifact| {
        if artifact
            .paths
            .iter()
            .any(|pattern| pattern_matches(pattern, path))
        {
            return Some((artifact, Coverage::Mechanized));
        }
        artifact
            .extracted_from
            .iter()
            .any(|pattern| pattern_matches(pattern, path))
            .then_some((artifact, Coverage::Retired))
    })
}

/// Every rule the registry has to satisfy, as human-readable failures.
fn violations(
    policy: &Policy,
    attributes: &BTreeSet<String>,
    ledger: &[LedgerPath],
    exists: &dyn Fn(&str) -> bool,
    union_in_git: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let causes = &policy.causes;
    let artifacts = &policy.artifacts;
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

    for deferral in &policy.deferrals {
        if deferral.reason.trim().is_empty() {
            failures.push(format!(
                "deferral `{}` states no reason; a deferral without one is an omission, not a decision",
                deferral.name
            ));
        }
        if deferral.paths.is_empty() {
            failures.push(format!(
                "deferral `{}` lists no path, so it defers nothing",
                deferral.name
            ));
        }
        for path in &deferral.paths {
            if ledger.iter().all(|entry| !pattern_matches(path, &entry.path)) {
                failures.push(format!(
                    "deferral `{}` defers `{}`, which no longer appears in the ledger; \
                     drop the entry rather than carrying a stale exemption",
                    deferral.name, path
                ));
            }
        }
    }

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for entry in ledger {
        if !seen.insert(entry.path.as_str()) {
            failures.push(format!(
                "`{}` appears twice in the ledger; a union merge left it stale. \
                 Rerun `python3 scripts/analyze-merge-conflicts.py --ledger`",
                entry.path
            ));
            continue;
        }
        let deferred = policy.deferrals.iter().any(|deferral| {
            deferral
                .paths
                .iter()
                .any(|pattern| pattern_matches(pattern, &entry.path))
        });
        match covering_artifact(artifacts, &entry.path) {
            Some((artifact, Coverage::Mechanized)) => {
                if artifact.merge.as_deref() == Some("union") && !union_in_git(&entry.path) {
                    failures.push(format!(
                        "artifact `{}` claims `{}` ({} resolutions) is union merged, but \
                         `git check-attr merge` disagrees: the pattern does not actually reach \
                         the file. A `*` in a gitattributes pattern stops at a `/`",
                        artifact.name, entry.path, entry.events
                    ));
                }
            }
            Some((_, Coverage::Retired)) => {}
            None if !deferred => failures.push(format!(
                "`{}` needed manual conflict resolution {} times and the policy neither covers \
                 nor defers it. Add it to an artifact, or add a `deferred` entry saying why the \
                 conflict is irreducible",
                entry.path, entry.events
            )),
            None => {}
        }
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
    let ledger_source = std::fs::read_to_string(root.join(LEDGER)).unwrap_or_else(|error| {
        println!("::error::Could not read {LEDGER}: {error}");
        println!("::error::Run `python3 scripts/analyze-merge-conflicts.py --ledger` to write it.");
        std::process::exit(1);
    });

    let policy = parse_policy(&policy);
    let ledger = parse_ledger(&ledger_source);
    let attributes = union_attributes(&attributes_source);
    let exists = |path: &str| root.join(path).exists();
    let union_in_git = |path: &str| {
        std::process::Command::new("git")
            .args(["check-attr", "merge", "--", path])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains("merge: union"))
            .unwrap_or(false)
    };

    println!("\nChecking the merge-conflict policy in {POLICY}...\n");
    println!(
        "  {} structural cause(s), {} artifact(s), {} union-merged path(s), \
         {} ledger path(s), {} deferral(s)\n",
        policy.causes.len(),
        policy.artifacts.len(),
        attributes.len(),
        ledger.len(),
        policy.deferrals.len()
    );
    for artifact in &policy.artifacts {
        println!(
            "  {:<26} {:<22} {} path(s)",
            artifact.name,
            artifact.cause,
            artifact.paths.len()
        );
    }

    let failures = violations(&policy, &attributes, &ledger, &exists, &union_in_git);
    if failures.is_empty() {
        println!(
            "\nThe registry, .gitattributes, the measured ledger and the repository agree.\n"
        );
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

    /// A ledger whose every path the SAMPLE registry accounts for.
    const SAMPLE_LEDGER: &str = "merge_conflict_ledger\n  \
        path \"src/lib.rs\"\n    \
        events 59\n  \
        path \"src/web/worker/formal_ai_worker_07.js\"\n    \
        events 11\n";

    fn always_exists(_path: &str) -> bool {
        true
    }

    fn always_union(_path: &str) -> bool {
        true
    }

    fn check(policy: &Policy, attributes: &BTreeSet<String>) -> Vec<String> {
        violations(
            policy,
            attributes,
            &parse_ledger(SAMPLE_LEDGER),
            &always_exists,
            &always_union,
        )
    }

    #[test]
    fn the_registry_parses_into_causes_and_artifacts() {
        let policy = parse_policy(SAMPLE);
        assert_eq!(
            policy.causes,
            BTreeSet::from([
                "append_only_list".to_string(),
                "sequential_file_name".to_string()
            ])
        );
        assert_eq!(policy.artifacts.len(), 2);
        assert_eq!(policy.artifacts[0].paths, ["src/lib.rs"]);
        assert_eq!(policy.artifacts[0].merge.as_deref(), Some("union"));
        assert!(policy.artifacts[0].has_verifier);
        assert_eq!(policy.artifacts[1].merge, None);
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
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        assert!(check(&parse_policy(SAMPLE), &attributes).is_empty());
    }

    #[test]
    fn a_union_path_missing_from_gitattributes_fails() {
        let failures = check(&parse_policy(SAMPLE), &BTreeSet::new());
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("src/lib.rs") && failure.contains(".gitattributes")),
            "{failures:?}"
        );
    }

    #[test]
    fn an_unregistered_union_path_fails() {
        let attributes =
            BTreeSet::from(["src/lib.rs".to_string(), "src/surprise.rs".to_string()]);
        let failures = check(&parse_policy(SAMPLE), &attributes);
        assert!(
            failures.iter().any(|failure| failure.contains("src/surprise.rs")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_union_artifact_without_a_verifier_fails() {
        let source = SAMPLE.replace("    verify \"rust-script scripts/normalize-ordered-lists.rs\"\n", "");
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = check(&parse_policy(&source), &attributes);
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
        let policy = parse_policy(&source);
        assert!(policy.artifacts[0].union_is_terminal);
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        assert!(check(&policy, &attributes).is_empty());
    }

    #[test]
    fn a_command_pointing_at_a_missing_script_fails() {
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(
            &parse_policy(SAMPLE),
            &attributes,
            &parse_ledger(SAMPLE_LEDGER),
            &|_path| false,
            &always_union,
        );
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
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = check(&parse_policy(&source), &attributes);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("sequential_file_name")),
            "renaming a cause leaves the artifact that names it unbacked: {failures:?}"
        );
    }

    #[test]
    fn a_measured_conflict_site_the_policy_ignores_fails() {
        // The whole point of the ledger: a path the repository has actually had
        // to conflict-resolve cannot be silently absent from the policy.
        let ledger = format!("{SAMPLE_LEDGER}  path \"README.md\"\n    events 35\n");
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(
            &parse_policy(SAMPLE),
            &attributes,
            &parse_ledger(&ledger),
            &always_exists,
            &always_union,
        );
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("README.md") && failure.contains("35 times")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_deferral_with_a_reason_accounts_for_a_ledger_path() {
        let source = format!(
            "{SAMPLE}  deferred shared_prose\n    reason \"Two branches editing the same prose.\"\n    path \"README.md\"\n"
        );
        let ledger = format!("{SAMPLE_LEDGER}  path \"README.md\"\n    events 35\n");
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(
            &parse_policy(&source),
            &attributes,
            &parse_ledger(&ledger),
            &always_exists,
            &always_union,
        );
        assert!(failures.is_empty(), "{failures:?}");
    }

    #[test]
    fn a_deferral_without_a_reason_fails() {
        let source = format!("{SAMPLE}  deferred shared_prose\n    path \"README.md\"\n");
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = check(&parse_policy(&source), &attributes);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("shared_prose") && failure.contains("states no reason")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_union_pattern_git_does_not_actually_apply_fails() {
        // The bug this test exists for: `data/meta/self-ast/*.lino` reads as
        // though it covers the whole census, but a gitattributes `*` stops at a
        // `/`, so every nested shard was in fact left unprotected.
        let ledger = "merge_conflict_ledger\n  path \"src/lib.rs\"\n    events 59\n";
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(
            &parse_policy(SAMPLE),
            &attributes,
            &parse_ledger(ledger),
            &always_exists,
            &|_path| false,
        );
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("git check-attr merge` disagrees")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_gitattributes_pattern_matches_the_way_git_matches_it() {
        assert!(pattern_matches("data/meta/self-ast/*.lino", "data/meta/self-ast/index.lino"));
        assert!(
            !pattern_matches("data/meta/self-ast/*.lino", "data/meta/self-ast/src/lib.lino"),
            "a single `*` never crosses a directory separator"
        );
        assert!(pattern_matches("data/meta/self-ast/**", "data/meta/self-ast/src/lib.lino"));
        assert!(
            pattern_matches(".gitkeep", "dev/log/issues/999/.gitkeep"),
            "a pattern with no slash matches the base name at any depth"
        );
        assert!(pattern_matches("src/lib.rs", "src/lib.rs"));
        assert!(!pattern_matches("src/lib.rs", "src/solver.rs"));
    }

    #[test]
    fn an_extracted_from_path_counts_as_covered() {
        // `src/web/formal_ai_worker.js` is the second most conflicted file in
        // the history, and the fix moved its list out rather than renaming it.
        let source = SAMPLE.replace(
            "    path \"src/lib.rs\"\n",
            "    path \"src/lib.rs\"\n      extracted_from \"src/web/formal_ai_worker.js\"\n",
        );
        let ledger = "merge_conflict_ledger\n  path \"src/web/formal_ai_worker.js\"\n    events 99\n";
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(
            &parse_policy(&source),
            &attributes,
            &parse_ledger(ledger),
            &always_exists,
            &always_union,
        );
        assert!(failures.is_empty(), "{failures:?}");
    }

    #[test]
    fn a_ledger_left_stale_by_a_union_merge_fails() {
        let ledger = format!("{SAMPLE_LEDGER}  path \"src/lib.rs\"\n    events 61\n");
        let attributes = BTreeSet::from(["src/lib.rs".to_string()]);
        let failures = violations(
            &parse_policy(SAMPLE),
            &attributes,
            &parse_ledger(&ledger),
            &always_exists,
            &always_union,
        );
        assert!(
            failures.iter().any(|failure| failure.contains("appears twice in the ledger")),
            "{failures:?}"
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
