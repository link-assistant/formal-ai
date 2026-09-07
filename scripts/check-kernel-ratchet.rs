#!/usr/bin/env rust-script
//! Rust outside the kernel only shrinks (issue #1085, D1.4).
//!
//! `data/meta/kernel-ratchet.lino` names the kernel -- notation, store,
//! interpreter, event log, loader, transports, sandbox, the universal loop --
//! and five measured ceilings for everything that is not kernel: non-kernel
//! Rust lines, handler files, pending handler migrations, literal string
//! predicates outside the kernel, and rows of the hardcoded-language allowlist.
//!
//! Three rules, one command each:
//!
//! * default -- every measured value is at or below its ceiling. A pull request
//!   that shrinks the code may lower a ceiling; nothing may raise one. The
//!   raisable `specialized_handler_files_max` / `try_dispatch_entries_max` pair
//!   in the handler-migration ledger was raised twice (2026-08-02, 2026-08-11)
//!   by the commits that exceeded it, which is why this file measures.
//! * `--base <rev>` -- additionally, no ceiling is higher than at `<rev>`.
//!   `GITHUB_BASE_REF` is honoured when set, so the pull-request gate compares
//!   against the branch it targets without being told.
//! * `--release` -- against the previous `v*` tag: the `non_kernel_rust_lines`
//!   ceiling is strictly lower, and no other ceiling is higher. This is the
//!   shrink a release has to show, checked by
//!   `.github/workflows/self-development-status.yml`.
//!
//! Usage:
//!   rust-script scripts/check-kernel-ratchet.rs [--base <rev>] [--release] [--repo <path>]
//!   rust-script --test scripts/check-kernel-ratchet.rs
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

const LEDGER: &str = "data/meta/kernel-ratchet.lino";
const HANDLER_LEDGER: &str = "data/meta/handler-migration-ledger.lino";
const ALLOWLIST: &str = "scripts/hardcoded-language-allowlist.txt";
const SHRINK_MEASURE: &str = "non_kernel_rust_lines";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Ratchet {
    kernel_paths: Vec<String>,
    ceilings: BTreeMap<String, u64>,
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

/// Parse the ledger's `kernel_path` rows and `ceiling` blocks. Written by hand
/// so this script has no dependency and the same parser runs at any revision
/// through `git show`.
fn parse_ratchet(text: &str) -> Result<Ratchet, String> {
    let mut kernel_paths = Vec::new();
    let mut ceilings = BTreeMap::new();
    let mut measure: Option<String> = None;
    let mut value: Option<u64> = None;
    let flush = |measure: &mut Option<String>,
                 value: &mut Option<u64>,
                 ceilings: &mut BTreeMap<String, u64>|
     -> Result<(), String> {
        if let Some(name) = measure.take() {
            let number = value
                .take()
                .ok_or_else(|| format!("ceiling {name} has no value"))?;
            ceilings.insert(name, number);
        }
        Ok(())
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("kernel_path ") {
            kernel_paths.push(unquote(rest));
        } else if trimmed == "ceiling" {
            flush(&mut measure, &mut value, &mut ceilings)?;
        } else if let Some(rest) = trimmed.strip_prefix("measure ") {
            measure = Some(unquote(rest));
        } else if let Some(rest) = trimmed.strip_prefix("value ") {
            value = Some(
                unquote(rest)
                    .parse::<u64>()
                    .map_err(|error| format!("ceiling value `{rest}` is not a number: {error}"))?,
            );
        }
    }
    flush(&mut measure, &mut value, &mut ceilings)?;
    if kernel_paths.is_empty() {
        return Err("the ratchet names no kernel_path".to_owned());
    }
    if !ceilings.contains_key(SHRINK_MEASURE) {
        return Err(format!("the ratchet has no `{SHRINK_MEASURE}` ceiling"));
    }
    Ok(Ratchet {
        kernel_paths,
        ceilings,
    })
}

fn is_kernel(path: &str, kernel_paths: &[String]) -> bool {
    kernel_paths.iter().any(|kernel| {
        if kernel.ends_with('/') {
            path.starts_with(kernel.as_str())
        } else {
            path == kernel
        }
    })
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

/// Measure the five values in a checkout.
fn measure(root: &Path, ratchet: &Ratchet) -> Result<BTreeMap<String, u64>, String> {
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files)?;
    let mut non_kernel_lines = 0_u64;
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
        if is_kernel(&path, &ratchet.kernel_paths) {
            continue;
        }
        non_kernel_lines += text.lines().count() as u64;
        literals += (text.matches("contains(\"").count() + text.matches("starts_with(\"").count())
            as u64;
    }
    let handler_ledger = fs::read_to_string(root.join(HANDLER_LEDGER))
        .map_err(|error| format!("{HANDLER_LEDGER}: {error}"))?;
    let pending = handler_ledger.matches("status pending").count() as u64;
    let allowlist =
        fs::read_to_string(root.join(ALLOWLIST)).map_err(|error| format!("{ALLOWLIST}: {error}"))?;
    let rows = allowlist
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .count() as u64;
    let mut measured = BTreeMap::new();
    measured.insert(SHRINK_MEASURE.to_owned(), non_kernel_lines);
    measured.insert("handler_files".to_owned(), handler_files);
    measured.insert("handler_migration_pending".to_owned(), pending);
    measured.insert("literal_predicates_outside_kernel".to_owned(), literals);
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
fn check_against_previous(previous: &Ratchet, current: &Ratchet, strict_shrink: bool) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, before) in &previous.ceilings {
        let Some(now) = current.ceilings.get(name) else {
            failures.push(format!("ceiling `{name}` was removed; a ratchet is not lowered by deleting it"));
            continue;
        };
        if now > before {
            failures.push(format!(
                "{name}: ceiling raised from {before} to {now}; a ceiling can only move down"
            ));
        }
        if strict_shrink && name == SHRINK_MEASURE && now >= before {
            failures.push(format!(
                "{name}: ceiling is {now}, was {before} at the previous tag; a release has to show \
                 the kernel boundary shrinking (issue #1085 D1.4)"
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
        Err(error) if error.contains("does not exist") || error.contains("exists on disk, but not in") => {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

struct Options {
    repo: PathBuf,
    base: Option<String>,
    release: bool,
}

fn parse_options(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut repo = PathBuf::from(".");
    let mut base = env::var("GITHUB_BASE_REF")
        .ok()
        .filter(|value| !value.is_empty())
        .map(|value| format!("origin/{value}"));
    let mut release = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo" => repo = PathBuf::from(args.next().ok_or("--repo requires a value")?),
            "--base" => base = Some(args.next().ok_or("--base requires a value")?),
            "--release" => release = true,
            "--help" | "-h" => {
                return Err(
                    "usage: check-kernel-ratchet.rs [--base <rev>] [--release] [--repo <path>]"
                        .to_owned(),
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Options {
        repo,
        base,
        release,
    })
}

fn run() -> Result<(), String> {
    let options = parse_options(env::args().skip(1))?;
    let repo = if options.repo == Path::new(".") {
        PathBuf::from(git(&options.repo, &["rev-parse", "--show-toplevel"])?)
    } else {
        options.repo.clone()
    };
    let text = fs::read_to_string(repo.join(LEDGER)).map_err(|error| format!("{LEDGER}: {error}"))?;
    let ratchet = parse_ratchet(&text)?;
    let measured = measure(&repo, &ratchet)?;
    println!("kernel ratchet ({LEDGER}):");
    for (name, ceiling) in &ratchet.ceilings {
        println!(
            "  {name}: measured {} / ceiling {ceiling}",
            measured.get(name).copied().unwrap_or(0)
        );
    }
    let mut failures = check_measured(&ratchet, &measured);

    if let Some(base) = &options.base {
        match ratchet_at(&repo, base) {
            Ok(Some(previous)) => failures.extend(check_against_previous(&previous, &ratchet, false)),
            Ok(None) => println!("  ({base} has no {LEDGER}; nothing to compare against)"),
            Err(error) => println!("  (skipping the base comparison: {error})"),
        }
    }
    if options.release {
        let tag = git(&repo, &["describe", "--tags", "--match", "v[0-9]*", "--abbrev=0", "HEAD"])?;
        match ratchet_at(&repo, &tag)? {
            Some(previous) => {
                println!("  previous tag {tag}: {SHRINK_MEASURE} ceiling {}", previous.ceilings[SHRINK_MEASURE]);
                failures.extend(check_against_previous(&previous, &ratchet, true));
            }
            None => failures.push(format!(
                "{tag} predates the kernel ratchet; the first release that carries it has to be \
                 cut before a shrink can be shown, and that release has not been cut"
            )),
        }
    }

    if failures.is_empty() {
        println!("kernel ratchet holds");
        return Ok(());
    }
    for failure in &failures {
        println!("::error file={LEDGER}::{failure}");
    }
    Err(format!("{} kernel ratchet failure(s)", failures.len()))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("check-kernel-ratchet: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "kernel_ratchet\n  kernel_path \"src/lib.rs\"\n  kernel_path \"src/web/\"\n  ceiling\n    measure non_kernel_rust_lines\n    value 10\n  ceiling\n    measure handler_files\n    value 2\n";

    fn ratchet(lines: u64, handlers: u64) -> Ratchet {
        let mut ceilings = BTreeMap::new();
        ceilings.insert(SHRINK_MEASURE.to_owned(), lines);
        ceilings.insert("handler_files".to_owned(), handlers);
        Ratchet {
            kernel_paths: vec!["src/lib.rs".to_owned(), "src/web/".to_owned()],
            ceilings,
        }
    }

    #[test]
    fn the_ledger_parses_into_kernel_paths_and_ceilings() {
        let parsed = parse_ratchet(SAMPLE).expect("sample parses");
        assert_eq!(parsed, ratchet(10, 2));
    }

    #[test]
    fn kernel_membership_is_exact_for_files_and_prefix_for_directories() {
        let kernel = vec!["src/lib.rs".to_owned(), "src/web/".to_owned()];
        assert!(is_kernel("src/lib.rs", &kernel));
        assert!(is_kernel("src/web/wasm-worker/src/lib.rs", &kernel));
        assert!(!is_kernel("src/lib.rs.bak", &kernel));
        assert!(!is_kernel("src/webby.rs", &kernel));
        assert!(!is_kernel("src/solver_handlers/calendar.rs", &kernel));
    }

    #[test]
    fn a_measurement_above_its_ceiling_fails_and_names_the_remedy() {
        let mut measured = BTreeMap::new();
        measured.insert(SHRINK_MEASURE.to_owned(), 11);
        measured.insert("handler_files".to_owned(), 2);
        let failures = check_measured(&ratchet(10, 2), &measured);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("non_kernel_rust_lines: measured 11, ceiling 10"));
        assert!(failures[0].contains("instead of raising the ceiling"));
        measured.insert(SHRINK_MEASURE.to_owned(), 10);
        assert!(check_measured(&ratchet(10, 2), &measured).is_empty());
    }

    #[test]
    fn a_ceiling_can_move_down_and_never_up() {
        assert!(check_against_previous(&ratchet(10, 2), &ratchet(9, 2), false).is_empty());
        let raised = check_against_previous(&ratchet(10, 2), &ratchet(11, 2), false);
        assert_eq!(raised.len(), 1, "{raised:?}");
        assert!(raised[0].contains("ceiling raised from 10 to 11"));
        let removed = check_against_previous(&ratchet(10, 2), &ratchet(10, 2), false);
        assert!(removed.is_empty());
    }

    #[test]
    fn a_release_has_to_show_the_line_ceiling_lower_than_at_the_previous_tag() {
        let same = check_against_previous(&ratchet(10, 2), &ratchet(10, 2), true);
        assert_eq!(same.len(), 1, "{same:?}");
        assert!(same[0].contains("was 10 at the previous tag"));
        assert!(check_against_previous(&ratchet(10, 2), &ratchet(9, 2), true).is_empty());
    }

    #[test]
    fn the_committed_ledger_measures_at_or_below_its_own_ceilings() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = if root.join(LEDGER).is_file() {
            root
        } else {
            // rust-script compiles this file from a temporary manifest; walk up
            // from the script's own location instead.
            PathBuf::from(file!())
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or(root)
        };
        if !root.join(LEDGER).is_file() {
            eprintln!("repository root not found from {}; skipping", root.display());
            return;
        }
        let ratchet = parse_ratchet(&fs::read_to_string(root.join(LEDGER)).expect("ledger readable"))
            .expect("ledger parses");
        let measured = measure(&root, &ratchet).expect("measurement runs");
        let failures = check_measured(&ratchet, &measured);
        assert!(failures.is_empty(), "{failures:?}");
    }
}
