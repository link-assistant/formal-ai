#!/usr/bin/env rust-script
//! Named debt only shrinks.
//!
//! `data/meta/debt-ratchet.lino` names measured ceilings, each one a thing the
//! architect has asked to be removed or generalized.
//!
//! Two rules, both in one required command:
//!
//! * every measured value is **exactly** its ceiling. Above it is new debt;
//!   below it is debt already removed whose ceiling was not lowered in the same
//!   commit, which leaves the next commit free to add it back (issue #1138 B9,
//!   plan 09 leaf 2). A measure whose `how` field says `direction upward` has a
//!   floor instead of a ceiling and the comparison inverts.
//! * `--base <rev>` -- no ceiling is higher than at `<rev>` (and no upward
//!   measure's floor lower). `GITHUB_BASE_REF` is honoured when set, so the
//!   pull-request gate compares against the branch it targets without being
//!   told. The flag is **required**: a run with nothing to compare against
//!   answers a weaker question than the gate claims to answer.
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
//!   rust-script scripts/check-debt-ratchet.rs --base <rev> [--repo <path>]
//!   rust-script --test scripts/check-debt-ratchet.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)] // This consumer needs only the live structural gap count.
#[path = "language-parity-lib.rs"]
mod language_parity;

const LEDGER: &str = "data/meta/debt-ratchet.lino";
const HANDLER_LEDGER: &str = "data/meta/handler-migration-ledger.lino";
const ALLOWLIST: &str = "scripts/hardcoded-language-allowlist.txt";
const AUTHORED_LADDER_RULES: &str = "experiments/issue_1028_agent_cli_ladder/rules";
/// The one census of handler source files in the tree (issue #1138 B9, plan 09
/// leaf 3).
///
/// `handler_files` used to be measured here by a second directory walk with
/// slightly different rules from `scripts/check-minimal-core-boundary.rs`'s, and
/// `tests/unit/issue_699_handler_migration.rs` held a third copy as Rust
/// constants. Three copies of one number is how 37/50 drifted from 36/39
/// unnoticed. There is now one census — the boundary ledger's `source` rows,
/// which that gate proves equal to the tree file for file — and this script
/// counts them rather than walking anything.
const BOUNDARY_LEDGER: &str = "data/meta/core-boundary-ledger.lino";
/// Bookkeeping files that carry no domain knowledge: the dispatch `mod.rs`
/// files and the generated `mod` list issue #991 split out of them. They are
/// boundary debt (they are compiled Rust under the handler root) but they are
/// not *handlers*, so the migration count excludes them.
const BOOKKEEPING: [&str; 2] = ["mod.rs", "modules.rs"];

/// The marker a measure puts in its own `how` field when its strict direction is
/// upward rather than downward (plan 00 §6.7 names `store_read_share` and plan
/// 10's four routing measures as the declared exceptions).
const UPWARD_MARKER: &str = "direction upward";

/// The marker a measure puts in its own `note` field when the value rises
/// because the old one counted the wrong set, not because debt was added (plan
/// 00 §6.2). The note must also name the value being corrected, so a note left
/// behind after the correction cannot silently permit a second rise.
const CORRECTION_MARKER: &str = "corrected undercount";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Ratchet {
    ceilings: BTreeMap<String, u64>,
    /// Measures whose strict direction is upward; for them the recorded value is
    /// a floor that may only rise.
    upward: BTreeSet<String>,
    /// Per measure, the `note` text, so an announced correction can be
    /// distinguished from a raised ceiling.
    notes: BTreeMap<String, String>,
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

/// Parse the ledger's `ceiling` blocks. Written by hand so this script has no
/// dependency and the same parser runs at any revision.
fn parse_ratchet(text: &str) -> Result<Ratchet, String> {
    let mut ceilings = BTreeMap::new();
    let mut upward = BTreeSet::new();
    let mut notes = BTreeMap::new();
    let mut measure: Option<String> = None;
    let mut named: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "ceiling" {
            measure = None;
            named = None;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("measure ") {
            measure = Some(unquote(rest));
            named = measure.clone();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("how ")
            && let Some(name) = named.as_ref()
            && unquote(rest).contains(UPWARD_MARKER)
        {
            upward.insert(name.clone());
        }
        if let Some(rest) = trimmed.strip_prefix("note ")
            && let Some(name) = named.as_ref()
        {
            notes.insert(name.clone(), unquote(rest));
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
    Ok(Ratchet {
        ceilings,
        upward,
        notes,
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

/// Every specialized handler source the core-boundary ledger covers, minus
/// bookkeeping and generic interpreters promoted into the minimal core.
///
/// The one definition, read from data. `check-minimal-core-boundary.rs` proves
/// these rows are exactly the files on disk, so counting rows and counting files
/// are the same measurement — with the difference that there is now one place
/// where "which files are handlers" is written down.
fn handler_source_files(root: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(root.join(BOUNDARY_LEDGER))
        .map_err(|error| format!("{BOUNDARY_LEDGER}: {error}"))?;
    let mut files = Vec::new();
    let mut path: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("  source ") {
            path = Some(rest.trim().to_owned());
            continue;
        }
        if let Some(rest) = line.trim().strip_prefix("disposition ")
            && let Some(source) = path.take()
        {
            let name = source.rsplit('/').next().unwrap_or("");
            if unquote(rest) == "migrate" && !BOOKKEEPING.contains(&name) {
                files.push(source);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Read one file, naming it if it is missing.
fn read(root: &Path, path: &str) -> Result<String, String> {
    fs::read_to_string(root.join(path)).map_err(|error| format!("{path}: {error}"))
}

/// The slice of `text` between the first `open` and the next `close` after it.
fn between<'a>(text: &'a str, open: &str, close: &str) -> &'a str {
    text.split_once(open)
        .and_then(|(_, tail)| tail.split_once(close))
        .map_or("", |(body, _)| body)
}

/// Native dispatch entries that are still a compiled `try_*` arm.
///
/// The same reading `tests/unit/issue_699_handler_migration.rs` performs, so the
/// test and the gate cannot disagree about what a dispatch entry is.
fn try_dispatch_entries(root: &Path) -> Result<u64, String> {
    let dispatch = read(root, "rust/src/solver_dispatch.rs")?;
    Ok(between(&dispatch, "const HANDLER_FUNCTIONS", "];")
        .lines()
        .filter(|line| {
            line.split_once(',')
                .is_some_and(|(_, function)| function.trim_start().starts_with("try_"))
        })
        .count() as u64)
}

/// Hard-coded promotion predicates: one `handler:<name>` literal per promotion
/// the prompt formalizer decides in Rust instead of reading from seed data.
fn promotion_predicates(root: &Path) -> Result<u64, String> {
    let text = read(root, "rust/src/intent_formalization/prompt_relevants.rs")?;
    Ok(text.matches("\"handler:").count() as u64)
}

/// Method names the dispatcher special-cases: direct `name == "…"`
/// comparisons or a name-based runtime match. Typed attribute matches do not
/// count: their method-to-runtime association lives in seed data.
fn dispatch_name_special_cases(root: &Path) -> Result<u64, String> {
    let text = read(root, "rust/src/meta_method_dispatch.rs")?;
    let comparisons = text.matches("name == \"").count();
    let name_matches = between(&text, "match name", "_ => return None")
        .matches("\" =>")
        .count();
    Ok((comparisons + name_matches) as u64)
}

/// Handler names the browser worker states as literals rather than deriving from
/// `data/seed/handler-precedence.lino`.
fn worker_sync_handler_literals(root: &Path) -> Result<u64, String> {
    let worker_dir = root.join("js/worker");
    let mut dispatch_sources = Vec::new();
    for entry in
        fs::read_dir(&worker_dir).map_err(|error| format!("{}: {error}", worker_dir.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", worker_dir.display()))?;
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "js") {
            continue;
        }
        let text =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        if text.contains("function synchronousHandlerCandidates") {
            dispatch_sources.push((path, text));
        }
    }
    if dispatch_sources.len() != 1 {
        return Err(format!(
            "expected exactly one synchronousHandlerCandidates owner under js/worker, found {}",
            dispatch_sources.len()
        ));
    }
    let text = &dispatch_sources[0].1;
    Ok(text.matches("name: \"").count() as u64)
}

/// Routing decisions served by the link store rather than the seed tables.
///
/// The one measure here whose strict direction is upward (plan 00 §6.7). It is a
/// count of the store-reading entry points that exist, not a percentage: the
/// ledger's `value` field is an integer, and a number this plan has not yet run
/// may not be stated as a fraction it has not measured (plan 00 §6.8).
fn store_read_share(root: &Path) -> Result<u64, String> {
    let mut files = Vec::new();
    rust_files(&root.join("rust/src"), &mut files)?;
    let mut total = 0_u64;
    for file in &files {
        let path = relative(root, file);
        let text = fs::read_to_string(file).map_err(|error| format!("{path}: {error}"))?;
        total += text.matches("from_store(").count() as u64;
    }
    Ok(total)
}

/// Consolidated documentation test suites directly below `tests/unit`.
///
/// This is the same filesystem census as Plan 11's count test: files and
/// directories both count because either one is a top-level Rust suite surface.
fn docs_requirements_suites(root: &Path) -> Result<u64, String> {
    let entries =
        fs::read_dir(root.join("rust/tests/unit")).map_err(|error| format!("tests/unit: {error}"))?;
    Ok(entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("docs_"))
        .count() as u64)
}

/// Exact per-leaf edit documents are useful historical evidence, but each is
/// also a memoized benchmark answer. Count the directory structurally so a
/// rename or deletion lowers the reviewed debt instead of leaving a stale
/// literal in a test.
fn authored_ladder_rules(root: &Path) -> Result<u64, String> {
    let directory = root.join(AUTHORED_LADDER_RULES);
    let entries = fs::read_dir(&directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    Ok(entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "lino")
        })
        .count() as u64)
}

/// Measure every value in a checkout.
fn measure(root: &Path) -> Result<BTreeMap<String, u64>, String> {
    let mut files = Vec::new();
    rust_files(&root.join("rust/src"), &mut files)?;
    let mut literals = 0_u64;
    let handler_files = handler_source_files(root)?.len() as u64;
    for file in &files {
        let path = relative(root, file);
        let text = fs::read_to_string(file).map_err(|error| format!("{path}: {error}"))?;
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
    measured.insert(
        "try_dispatch_entries".to_owned(),
        try_dispatch_entries(root)?,
    );
    measured.insert(
        "promotion_predicates".to_owned(),
        promotion_predicates(root)?,
    );
    measured.insert(
        "dispatch_name_special_cases".to_owned(),
        dispatch_name_special_cases(root)?,
    );
    measured.insert(
        "worker_sync_handler_literals".to_owned(),
        worker_sync_handler_literals(root)?,
    );
    measured.insert("store_read_share".to_owned(), store_read_share(root)?);
    measured.insert(
        "docs_requirements_suites".to_owned(),
        docs_requirements_suites(root)?,
    );
    measured.insert(
        "authored_ladder_rules".to_owned(),
        authored_ladder_rules(root)?,
    );
    measured.insert(
        "language_parity_gaps".to_owned(),
        language_parity::current_gap_count(root)?,
    );
    Ok(measured)
}

/// Every measured value **exactly at** its ceiling.
///
/// Issue #1138 B9, plan 09 leaf 2: this was an at-or-below comparison, which is
/// how `literal_predicates` sat at 548 against a ceiling of 549 for weeks — the
/// gate was green while the ledger stated a number the tree had already beaten,
/// so the next commit could add a literal back for free. The rule is now
/// `check-minimal-core-boundary.rs`'s exact two-sided one: a value above its
/// ceiling fails, and a value below it fails too, naming the remedy "lower the
/// reviewed ceiling in this commit".
///
/// A measure whose `how` field declares `direction upward` inverts the
/// comparison: for it, above the floor is the improvement and below it is the
/// regression.
fn check_measured(ratchet: &Ratchet, measured: &BTreeMap<String, u64>) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, ceiling) in &ratchet.ceilings {
        let Some(value) = measured.get(name) else {
            failures.push(format!("ceiling `{name}` has no measurement"));
            continue;
        };
        let upward = ratchet.upward.contains(name);
        if (!upward && value > ceiling) || (upward && value < ceiling) {
            failures.push(format!(
                "{name}: measured {value}, ceiling {ceiling}; move the behaviour into data/seed or \
                 data/meta rules instead of raising the ceiling (issue #1085 D1)"
            ));
        } else if (!upward && value < ceiling) || (upward && value > ceiling) {
            failures.push(format!(
                "{name}: improved from {ceiling} to {value}; lower the reviewed ceiling in \
                 data/meta/debt-ratchet.lino in this commit"
            ));
        }
    }
    failures
}

/// No ceiling higher than before — and, for an upward measure, no floor lower.
fn check_against_previous(previous: &Ratchet, current: &Ratchet) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, before) in &previous.ceilings {
        let Some(now) = current.ceilings.get(name) else {
            failures.push(format!(
                "ceiling `{name}` was removed; a ratchet is not lowered by deleting it"
            ));
            continue;
        };
        if current.upward.contains(name) {
            if now < before {
                failures.push(format!(
                    "{name}: floor lowered from {before} to {now}; an upward measure's floor can \
                     only move up"
                ));
            }
        } else if now > before && !announces_correction(current, name, *before) {
            failures.push(format!(
                "{name}: ceiling raised from {before} to {now}; a ceiling can only move down. If \
                 the old value counted the wrong set, say so in this measure's `note` as \
                 \"{CORRECTION_MARKER}\" and name the {before} it corrects (plan 00 §6.2)"
            ));
        }
    }
    failures
}

/// Whether this measure's own `note` announces that `before` was an undercount.
///
/// Plan 00 §6.2 permits exactly this: a value that rises because the old one
/// counted the wrong set, in a commit that changes no behaviour. The note must
/// name the number it corrects, so a note left in place after the correction
/// cannot silently permit a second rise — `before` is then the corrected value
/// and no longer appears in the text.
fn announces_correction(ratchet: &Ratchet, measure: &str, before: u64) -> bool {
    let Some(note) = ratchet.notes.get(measure) else {
        return false;
    };
    if !note.contains(CORRECTION_MARKER) {
        return false;
    }
    let before = before.to_string();
    note.split(|character: char| !character.is_ascii_digit())
        .any(|token| token == before)
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
                return Err("usage: check-debt-ratchet.rs --base <rev> [--repo <path>]".to_owned());
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    // Issue #1138 B9, plan 09 leaf 2: `--base` was optional, so the only check
    // that catches a *raised* ceiling ran on a pull request and nowhere else. A
    // run without a base answers a weaker question than the one the gate claims
    // to answer, so it is refused rather than silently downgraded.
    if base.is_none() {
        return Err(
            "--base <rev> is required: without it the ceilings are compared against nothing and a \
             raised ceiling passes. Use --base origin/main locally; CI supplies GITHUB_BASE_REF."
                .to_owned(),
        );
    }
    Ok(Options { repo, base })
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
                for (name, before) in &previous.ceilings {
                    let now = ratchet.ceilings.get(name).copied().unwrap_or(*before);
                    if now > *before && announces_correction(&ratchet, name, *before) {
                        println!(
                            "  ({name}: {before} -> {now}, announced as a {CORRECTION_MARKER} in \
                             the ledger's note)"
                        );
                    }
                }
                failures.extend(check_against_previous(&previous, &ratchet));
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
        Ratchet {
            ceilings,
            upward: BTreeSet::new(),
            notes: BTreeMap::new(),
        }
    }

    #[test]
    fn the_ledger_parses_into_ceilings() {
        let parsed = parse_ratchet(SAMPLE).expect("the sample parses");
        assert_eq!(parsed.ceilings["literal_predicates"], 10);
        assert_eq!(parsed.ceilings["handler_files"], 2);
    }

    #[test]
    fn authored_ladder_rules_are_measured_from_the_live_directory() {
        assert_eq!(
            authored_ladder_rules(&repo_root()).expect("authored ladder rule census"),
            32
        );
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

    /// Issue #1138 B9, plan 09 leaf 2. `literal_predicates` measured 548
    /// against a ceiling of 549 and the gate was green, so the ledger stated a
    /// number the tree had already beaten and the next commit could add the
    /// literal back for free. Below the ceiling is now a failure that names the
    /// remedy.
    #[test]
    fn a_measurement_below_its_ceiling_fails_and_asks_for_the_ceiling_to_be_lowered() {
        let failures = check_measured(&ratchet(10, 2), &{
            let mut measured = BTreeMap::new();
            measured.insert("literal_predicates".to_owned(), 9);
            measured.insert("handler_files".to_owned(), 2);
            measured
        });
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(
            failures[0].contains("literal_predicates: improved from 10 to 9")
                && failures[0].contains("lower the reviewed ceiling"),
            "{failures:?}"
        );
    }

    /// A measure that declares `direction upward` in its own `how` field has a
    /// floor, not a ceiling: below it is the regression and above it is the
    /// improvement that must be recorded (plan 00 §6.7).
    #[test]
    fn an_upward_measure_inverts_both_comparisons() {
        let text = "debt_ratchet\n  ceiling\n    measure store_read_share\n    value 3\n    \
                    how \"share of reads served by the link store; strict direction upward\"\n";
        let parsed = parse_ratchet(text).expect("the sample parses");
        assert!(parsed.upward.contains("store_read_share"));

        let below = check_measured(
            &parsed,
            &BTreeMap::from([("store_read_share".to_owned(), 2)]),
        );
        assert_eq!(below.len(), 1, "{below:?}");
        assert!(below[0].contains("measured 2, ceiling 3"), "{below:?}");

        let above = check_measured(
            &parsed,
            &BTreeMap::from([("store_read_share".to_owned(), 4)]),
        );
        assert_eq!(above.len(), 1, "{above:?}");
        assert!(above[0].contains("improved from 3 to 4"), "{above:?}");

        assert!(
            check_measured(
                &parsed,
                &BTreeMap::from([("store_read_share".to_owned(), 3)])
            )
            .is_empty()
        );
    }

    /// Plan 00 §6.2's one permitted rise, and the reason it cannot become a
    /// standing bypass: the note must name the number it corrects, so the same
    /// note is inert against the corrected value.
    #[test]
    fn an_announced_corrected_undercount_may_rise_once_and_not_twice() {
        let mut corrected = ratchet(10, 46);
        corrected.notes.insert(
            "handler_files".to_owned(),
            "corrected undercount. 42 counted one directory only.".to_owned(),
        );
        assert!(
            check_against_previous(&ratchet(10, 42), &corrected).is_empty(),
            "a rise the ledger announces and explains is the correction plan 00 §6.2 allows"
        );

        let mut again = corrected.clone();
        again.ceilings.insert("handler_files".to_owned(), 47);
        let failures = check_against_previous(&corrected, &again);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(
            failures[0].contains("ceiling raised from 46 to 47"),
            "the same note must not permit a second rise: {failures:?}"
        );

        let mut unexplained = ratchet(10, 46);
        unexplained.notes.insert(
            "handler_files".to_owned(),
            "corrected undercount.".to_owned(),
        );
        assert_eq!(
            check_against_previous(&ratchet(10, 42), &unexplained).len(),
            1,
            "a marker that names no corrected value explains nothing"
        );
    }

    /// The only check that catches a raised ceiling is the base comparison, so a
    /// run with no base answers a weaker question than the gate claims to.
    #[test]
    fn a_run_without_a_base_is_refused() {
        // SAFETY: single-threaded test process; the variable is removed so the
        // ambient CI environment cannot supply the base this test withholds.
        unsafe { env::remove_var("GITHUB_BASE_REF") };
        let error = parse_options(Vec::new()).err().expect("a base is required");
        assert!(error.contains("--base <rev> is required"), "{error}");
        assert!(
            parse_options(["--base".to_owned(), "origin/main".to_owned()]).is_ok(),
            "an explicit base is accepted"
        );
    }

    #[test]
    fn the_committed_ledger_measures_exactly_its_own_ceilings() {
        let root = repo_root();
        let text = fs::read_to_string(root.join(LEDGER)).expect("the ledger is committed");
        let parsed = parse_ratchet(&text).expect("the committed ledger parses");
        let measured = measure(&root).expect("the checkout measures");
        let failures = check_measured(&parsed, &measured);
        assert!(failures.is_empty(), "{failures:?}");
    }
}
