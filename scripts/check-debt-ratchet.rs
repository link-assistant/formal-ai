#!/usr/bin/env rust-script
//! Named debt only shrinks.
//!
//! `data/meta/debt-ratchet.lino` names four measured ceilings, each one a thing
//! the architect has asked to be removed: handler files still awaiting
//! migration to data, pending migrations in the ledger, per-case literal string
//! predicates standing in for a rule, and rows of the hardcoded-language
//! allowlist (R379).
//!
//! Two rules, one command each:
//!
//! * default -- every measured value is at or below its ceiling. A pull request
//!   that removes debt may lower a ceiling; nothing may raise one.
//! * `--base <rev>` -- additionally, no ceiling is higher than at `<rev>`.
//!   `GITHUB_BASE_REF` is honoured when set, so the pull-request gate compares
//!   against the branch it targets without being told.
//!
//! No release depends on any ceiling here. This file carried a name built on a
//! division of `src` into a privileged part and the rest until 2026-09-12. That
//! division was never requested by the architect and is withdrawn (VISION.md,
//! docs/architect-notes/): all of `src` serves the meta algorithm. The measure
//! that counted Rust lines outside it existed only to serve the division and is
//! gone with it, along with the `--release` mode that required it to fall before
//! every release.
//!
//! Usage:
//!   rust-script scripts/check-debt-ratchet.rs [--base <rev>] [--repo <path>]
//!   rust-script --test scripts/check-debt-ratchet.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LEDGER: &str = "data/meta/debt-ratchet.lino";
const HANDLER_LEDGER: &str = "data/meta/handler-migration-ledger.lino";
const ALLOWLIST: &str = "scripts/hardcoded-language-allowlist.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Ratchet {
    ceilings: BTreeMap<String, u64>,
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

/// Parse the ledger's `ceiling` blocks. Written by hand so this script has no
/// dependency and the same parser runs at any revision.
fn parse_ratchet(text: &str) -> Result<Ratchet, String> {
    let mut ceilings = BTreeMap::new();
    let mut measure: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "ceiling" {
            measure = None;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("measure ") {
            measure = Some(unquote(rest));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("value ") {
            let Some(name) = measure.take() else {
                return Err(format!("`{trimmed}` has no `measure` above it"));
            };
            let value = unquote(rest)
                .parse::<u64>()
                .map_err(|error| format!("{trimmed}: {error}"))?;
            ceilings.insert(name, value);
        }
    }
    if ceilings.is_empty() {
        return Err("the ratchet names no ceiling".to_owned());
    }
    Ok(Ratchet { ceilings })
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Measure the four values in a checkout.
fn measure(root: &Path) -> Result<BTreeMap<String, u64>, String> {
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files)?;
    let mut literals = 0_u64;
    let mut handler_files = 0_u64;
    for file in &files {
        let path = relative(root, file);
        let text = fs::read_to_string(file).map_err(|error| format!("{path}: {error}"))?;
        if path.starts_with("src/solver_handlers/") {
            let name = path.rsplit('/').next().unwrap_or("");
            if name != "mod.rs" && name != "modules.rs" {
                handler_files += 1;
            }
        }
        // Every file under src/, with no kernel exemption: all of src serves
        // the meta algorithm: see VISION.md and docs/architect-notes/.
        literals +=
            (text.matches("contains(\"").count() + text.matches("starts_with(\"").count()) as u64;
    }
    let handler_ledger = fs::read_to_string(root.join(HANDLER_LEDGER))
        .map_err(|error| format!("{HANDLER_LEDGER}: {error}"))?;
    let pending = handler_ledger.matches("status pending").count() as u64;
    let allowlist = fs::read_to_string(root.join(ALLOWLIST))
        .map_err(|error| format!("{ALLOWLIST}: {error}"))?;
    let rows = allowlist
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .count() as u64;
    let mut measured = BTreeMap::new();
    measured.insert("handler_files".to_owned(), handler_files);
    measured.insert("handler_migration_pending".to_owned(), pending);
    measured.insert("literal_predicates".to_owned(), literals);
    measured.insert("hardcoded_language_rows".to_owned(), rows);
    Ok(measured)
}

/// Every measured value at or below its ceiling.
fn check_measured(ratchet: &Ratchet, measured: &BTreeMap<String, u64>) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, ceiling) in &ratchet.ceilings {
        let Some(value) = measured.get(name) else {
            failures.push(format!("ceiling `{name}` has no measurement"));
            continue;
        };
        if value > ceiling {
            failures.push(format!(
                "{name}: measured {value}, ceiling {ceiling}; move the behaviour into data/seed or \
                 data/meta rules instead of raising the ceiling (issue #1085 D1)"
            ));
        }
    }
    failures
}

/// No ceiling higher than before; with `strict_shrink`, the line ceiling lower.
fn check_against_previous(
    previous: &Ratchet,
    current: &Ratchet,
) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, before) in &previous.ceilings {
        let Some(now) = current.ceilings.get(name) else {
            failures.push(format!(
                "ceiling `{name}` was removed; a ratchet is not lowered by deleting it"
            ));
            continue;
        };
        if now > before {
            failures.push(format!(
                "{name}: ceiling raised from {before} to {now}; a ceiling can only move down"
            ));
        }
    }
    failures
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| format!("could not run git {args:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn ratchet_at(repo: &Path, revision: &str) -> Result<Option<Ratchet>, String> {
    match git(repo, &["show", &format!("{revision}:{LEDGER}")]) {
        Ok(text) => parse_ratchet(&text).map(Some),
        Err(error)
            if error.contains("does not exist") || error.contains("exists on disk, but not in") =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

struct Options {
    repo: PathBuf,
    base: Option<String>,
}

fn parse_options(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut repo = PathBuf::from(".");
    let mut base = env::var("GITHUB_BASE_REF")
        .ok()
        .filter(|value| !value.is_empty())
        .map(|value| format!("origin/{value}"));
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo" => repo = PathBuf::from(args.next().ok_or("--repo requires a value")?),
            "--base" => base = Some(args.next().ok_or("--base requires a value")?),
            "--help" | "-h" => {
                return Err(
                    "usage: check-debt-ratchet.rs [--base <rev>] [--repo <path>]"
                        .to_owned(),
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Options {
        repo,
        base,
    })
}

fn run() -> Result<(), String> {
    let options = parse_options(env::args().skip(1))?;
    let repo = if options.repo == Path::new(".") {
        PathBuf::from(git(&options.repo, &["rev-parse", "--show-toplevel"])?)
    } else {
        options.repo.clone()
    };
    let text =
        fs::read_to_string(repo.join(LEDGER)).map_err(|error| format!("{LEDGER}: {error}"))?;
    let ratchet = parse_ratchet(&text)?;
    let measured = measure(&repo)?;
    println!("debt ratchet ({LEDGER}):");
    for (name, ceiling) in &ratchet.ceilings {
        println!(
            "  {name}: measured {} / ceiling {ceiling}",
            measured.get(name).copied().unwrap_or(0)
        );
    }
    let mut failures = check_measured(&ratchet, &measured);

    if let Some(base) = &options.base {
        match ratchet_at(&repo, base) {
            Ok(Some(previous)) => {
                failures.extend(check_against_previous(&previous, &ratchet))
            }
            Ok(None) => println!("  ({base} has no {LEDGER}; nothing to compare against)"),
            Err(error) => println!("  (skipping the base comparison: {error})"),
        }
    }

    if failures.is_empty() {
        println!("debt ratchet holds");
        return Ok(());
    }
    for failure in &failures {
        println!("::error file={LEDGER}::{failure}");
    }
    Err(format!("{} debt ratchet failure(s)", failures.len()))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("check-debt-ratchet: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Locate the repository root. rust-script compiles this file from a
    /// temporary manifest, so `CARGO_MANIFEST_DIR` is not the repository; walk
    /// up from the script's own location instead.
    fn repo_root() -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if root.join(LEDGER).is_file() {
            return root;
        }
        PathBuf::from(file!())
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or(root)
    }

    const SAMPLE: &str = "debt_ratchet\n  ceiling\n    measure literal_predicates\n    value 10\n  ceiling\n    measure handler_files\n    value 2\n";

    fn ratchet(literals: u64, handlers: u64) -> Ratchet {
        let mut ceilings = BTreeMap::new();
        ceilings.insert("literal_predicates".to_owned(), literals);
        ceilings.insert("handler_files".to_owned(), handlers);
        Ratchet { ceilings }
    }

    #[test]
    fn the_ledger_parses_into_ceilings() {
        let parsed = parse_ratchet(SAMPLE).expect("the sample parses");
        assert_eq!(parsed.ceilings["literal_predicates"], 10);
        assert_eq!(parsed.ceilings["handler_files"], 2);
    }

    /// The division of `src` into a privileged part and the rest is withdrawn
    /// (VISION.md, docs/architect-notes/), so the ledger names no such path and
    /// no measure counts Rust lines. All of `src/` serves the meta algorithm.
    #[test]
    fn the_ledger_names_no_kernel_and_measures_no_rust_line_count() {
        let text = fs::read_to_string(repo_root().join(LEDGER)).expect("the ledger is committed");
        assert!(
            !text.contains("kernel"),
            "the ledger still names the withdrawn kernel split:\n{text}"
        );
        let parsed = parse_ratchet(&text).expect("the committed ledger parses");
        for name in parsed.ceilings.keys() {
            assert!(
                !name.contains("rust_lines"),
                "`{name}` counts Rust lines, which measures an emission target and not the system"
            );
        }
    }

    #[test]
    fn a_measurement_above_its_ceiling_fails_and_names_the_remedy() {
        let failures = check_measured(&ratchet(10, 2), &{
            let mut measured = BTreeMap::new();
            measured.insert("literal_predicates".to_owned(), 11);
            measured.insert("handler_files".to_owned(), 2);
            measured
        });
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("literal_predicates: measured 11, ceiling 10"));
    }

    #[test]
    fn a_ceiling_can_move_down_and_never_up() {
        // (previous, current): lowering 10 -> 9 is allowed, raising 10 -> 11 is not.
        assert!(check_against_previous(&ratchet(10, 2), &ratchet(9, 2)).is_empty());
        let raised = check_against_previous(&ratchet(10, 2), &ratchet(11, 2));
        assert_eq!(raised.len(), 1, "{raised:?}");
    }

    /// No release depends on any ceiling here, so the checker offers no mode
    /// that could make one: a requirement may not obstruct progression to the
    /// vision (VISION.md, docs/architect-notes/).
    #[test]
    fn there_is_no_release_mode() {
        let source = fs::read_to_string(file!()).unwrap_or_default();
        assert!(
            !source.contains("\"--release\""),
            "a --release mode reintroduces a release condition on a measured ceiling"
        );
    }

    #[test]
    fn the_committed_ledger_measures_at_or_below_its_own_ceilings() {
        let root = repo_root();
        let text = fs::read_to_string(root.join(LEDGER)).expect("the ledger is committed");
        let parsed = parse_ratchet(&text).expect("the committed ledger parses");
        let measured = measure(&root).expect("the checkout measures");
        let failures = check_measured(&parsed, &measured);
        assert!(failures.is_empty(), "{failures:?}");
    }
}
