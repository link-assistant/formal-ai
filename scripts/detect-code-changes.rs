#!/usr/bin/env rust-script
//! Detect code changes for CI/CD pipeline
//!
//! This script detects what types of files have changed in the triggering event
//! and outputs the results for use in GitHub Actions workflow conditions.
//!
//! Key behavior:
//! - For PRs: detects GitHub Actions' synthetic merge commit and uses
//!   HEAD^..HEAD^2 to cover the complete pull request.
//! - For pushes: compares the event's `before` SHA against HEAD, covering every
//!   commit in a multi-commit push.
//! - Excludes CI evidence and scratch folders from every job-gating output
//! - Excludes additional documentation/example paths from "code changes"
//!
//! Excluded from code changes (don't require changelog fragments):
//! - Root Markdown and nested README.md/CHANGELOG.md documentation
//! - changelog.d/ folder (changelog fragments)
//! - docs/ folder (documentation)
//! - experiments/ folder (experimental scripts)
//! - examples/ folder (example scripts)
//!
//! Excluded from every job-gating output:
//! - experiments/ folder
//! - dev/log/ folder
//! - docs/case-studies/ folder
//!
//! Usage: rust-script scripts/detect-code-changes.rs
//!
//! Environment variables (set by GitHub Actions):
//!   - `GITHUB_EVENT_NAME`: `pull_request` or `push`
//!   - `GITHUB_EVENT_BEFORE`: pre-push SHA from the event payload
//!   - `GITHUB_REF`: the pushed ref
//!   - `RELEASE_PIPELINE`: `true` in the release workflow, whose Auto Release
//!     job needs lint, test and the binary build on a push that has a
//!     changelog fragment waiting (see `release_pending`)
//!
//! Outputs (written to `GITHUB_OUTPUT`):
//!   - rs-changed: 'true' if any .rs files changed
//!   - toml-changed: 'true' if any .toml files changed
//!   - docs-changed: 'true' if any .md files changed
//!   - workflow-changed: 'true' if any .github/workflows/ files changed
//!   - any-code-changed: 'true' if any non-ignored code files changed, or a
//!     release is pending on a push to main in the release workflow
//!   - release-pending: 'true' when that push has a changelog fragment waiting
//!   - agentic-routing-changed: 'true' if agentic routing source changed
//!   - js-changed: 'true' if the js tier's inputs changed (plan 16 L4)
//!   - ts-changed: 'true' if the ts tier's inputs changed (plan 16 L4)
//!   - rust-changed: 'true' if the rust tier's inputs changed (plan 16 L4)
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! regex = "1"
//! ```

use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)] // This CLI uses the fallible writer and retains its own stdout format.
#[path = "github-output.rs"]
mod github_output;

// These paths are CI evidence or non-shipping scratch work. No output used to
// gate a job may be set by a file below one of these prefixes.
const CI_IGNORED_PATH_PREFIXES: &[&str] = &["experiments/", "dev/log/", "docs/case-studies/"];

// The js -> ts -> rust development cycle (issue #1138, plan 16 L4). The tier
// predicates are cumulative downward because each tier consumes the one
// above it: the committed ts tree is a function of the js tree, and the
// dogfood gate in the rust tier binds all three trees. A change therefore
// wakes every tier below its folder and none above it -- that asymmetry is
// the cycle's saving ("once JavaScript version stabilized, we don't
// reexecute it CI/CD"). `data/seed/` wakes every tier (plan 16 risk 3: a
// seed consumed by all three roots must not hide behind an untouched
// folder), and the classifier and the layered workflow gate themselves so
// an edit to the gating rules re-runs the tiers they gate.
const LAYERED_WORKFLOW: &str = ".github/workflows/layered-ci.yml";
const LAYER_CLASSIFIER: &str = "scripts/detect-code-changes.rs";
// The JavaScript suites the js tier runs (`node --test rust/tests/web/`).
// They are the js tier's own tests, so a change to one re-runs that tier; the
// ts tree does not consume them, so the ts tier stands. Before this, an edited
// suite file -- the thing a requirement row names as its pinning test -- woke
// the rust tier (it sits under `rust/`) but never the tier that executes it,
// and merged unexecuted: issue #1017's false-negative class.
const JS_SUITES: &str = "rust/tests/web/";

fn exec_in(repo: &Path, args: &[&str]) -> std::io::Result<String> {
    let output = Command::new("git").args(args).current_dir(repo).output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "git {args:?} failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

fn set_output(name: &str, value: &str) -> std::io::Result<()> {
    github_output::write_output(name, value)?;
    println!("{name}={value}");
    Ok(())
}

fn usable_before_sha(before: Option<&str>) -> Option<&str> {
    before.filter(|sha| !sha.is_empty() && !sha.chars().all(|character| character == '0'))
}

fn comparison_for_event(
    event_name: &str,
    merge_commit: bool,
    before: Option<&str>,
) -> (String, String, &'static str) {
    if event_name == "pull_request" && merge_commit {
        return (
            "HEAD^".to_string(),
            "HEAD^2".to_string(),
            "complete pull request diff",
        );
    }
    if event_name == "push"
        && let Some(before) = usable_before_sha(before)
    {
        return (before.to_string(), "HEAD".to_string(), "complete push diff");
    }
    (
        "HEAD^".to_string(),
        "HEAD".to_string(),
        "previous commit diff",
    )
}

fn decode_git_paths(output: &str) -> Vec<String> {
    output
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(String::from)
        .collect()
}

fn get_changed_files() -> std::io::Result<Vec<String>> {
    get_changed_files_in_repo(
        Path::new("."),
        &env::var("GITHUB_EVENT_NAME").unwrap_or_default(),
        env::var("GITHUB_EVENT_BEFORE").ok().as_deref(),
    )
}

fn get_changed_files_in_repo(
    repo: &Path,
    event: &str,
    before: Option<&str>,
) -> std::io::Result<Vec<String>> {
    // Even shallow commits retain their parent headers. Only a genuinely initial
    // commit permits the tree fallback; unknown history must not become an empty diff.
    let commit = exec_in(repo, &["cat-file", "-p", "HEAD"])?;
    let parents = commit
        .lines()
        .take_while(|line| !line.is_empty())
        .filter(|line| line.starts_with("parent "))
        .count();
    if event == "pull_request" && parents != 2 {
        return Err(std::io::Error::other(
            "pull_request requires the complete synthetic merge comparison",
        ));
    }
    let (base, head, description) = comparison_for_event(event, parents > 1, before);
    println!("Comparing {base} to {head} ({description})");
    // Preserve removed source paths and literal path bytes (including newlines).
    let output = if parents == 0 && base == "HEAD^" {
        exec_in(repo, &["ls-tree", "--name-only", "-z", "-r", "HEAD"])?
    } else {
        exec_in(
            repo,
            &["diff", "--no-renames", "--name-only", "-z", &base, &head],
        )?
    };
    Ok(decode_git_paths(&output))
}

// Twin of isInertPath in scripts/ci-pull-request-admission.mjs (issue #1637):
// an empty `.gitkeep` placeholder carries no behaviour.
fn is_inert_path(file_path: &str) -> bool {
    file_path == ".gitkeep" || file_path.ends_with("/.gitkeep")
}

fn is_ignored_by_ci(file_path: &str) -> bool {
    is_inert_path(file_path)
        || CI_IGNORED_PATH_PREFIXES
            .iter()
            .any(|prefix| file_path.starts_with(prefix))
}

// Exact twin of documentationPaths in scripts/documentation-only-ci-policy.mjs.
fn is_documentation_path(path: &str) -> bool {
    path.starts_with("docs/")
        || path.starts_with("changelog.d/")
        // GitHub path filters are case-sensitive globs, including a root `.md` file.
        || (!path.contains('/') && matches!(path.rsplit_once('.'), Some((_, "md"))))
        || path.ends_with("/README.md")
        || path.ends_with("/CHANGELOG.md")
}

fn automatic_release_pending(
    event: &str,
    git_ref: &str,
    fragments: &[String],
    changed_files: &[String],
) -> bool {
    // A pending fragment cannot turn a docs-only push into an automatic release,
    // even if GitHub bypasses workflow path filters for an oversized event.
    changed_files
        .iter()
        .any(|path| !is_documentation_path(path))
        && release_pending(event, git_ref, fragments)
}

fn is_excluded_from_code_changes(file_path: &str) -> bool {
    // Unknown Markdown may be runtime input, not documentation.
    if is_documentation_path(file_path) {
        return true;
    }

    // Exclude specific folders from code changes
    let excluded_folders = ["changelog.d/", "docs/", "examples/", "rust/examples/"];

    for folder in &excluded_folders {
        if file_path.starts_with(folder) {
            return true;
        }
    }

    false
}

fn has_extension(file_path: &str, expected: &str) -> bool {
    Path::new(file_path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

#[derive(Debug, Default, Eq, PartialEq)]
// Each boolean maps directly to one GitHub Actions output. Replacing these
// independent flags with a state machine would incorrectly make them exclusive.
#[allow(clippy::struct_excessive_bools)]
struct ChangeFlags {
    rs_changed: bool,
    toml_changed: bool,
    docs_changed: bool,
    workflow_changed: bool,
    pipeline_changed: bool,
    agentic_routing_changed: bool,
    any_code_changed: bool,
    js_changed: bool,
    ts_changed: bool,
    rust_changed: bool,
}

/// A release is pending when a push to `main` finds a changelog fragment in
/// `changelog.d/`: the Auto Release job then has a version to cut, and it
/// runs only after lint, test and the binary build pass. Those jobs are gated
/// on `any-code-changed`, so a push that changed no code (the docs-and-workflow
/// merge of #1150, carrying `20260926_215107_issue-1149-coverage-budget.md`)
/// skipped them, skipped the release, and still reported a green pipeline --
/// the silent deferral of issue #1064, which the no-deferral rule forbids.
fn release_pending(event_name: &str, git_ref: &str, fragments: &[String]) -> bool {
    event_name == "push"
        && git_ref == "refs/heads/main"
        && fragments.iter().any(|name| {
            Path::new(name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
                && name != "README.md"
        })
}

/// Only an absent directory is an empty inventory. Unreadable entries and
/// non-UTF-8 names cannot establish a complete release observation.
fn changelog_fragments(directory: &Path) -> std::io::Result<Vec<String>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(std::io::Error::new(
                error.kind(),
                format!(
                    "Could not read fragment inventory '{}': {error}",
                    directory.display()
                ),
            ));
        }
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            std::io::Error::new(
                error.kind(),
                format!(
                    "Could not read fragment entry in '{}': {error}",
                    directory.display()
                ),
            )
        })?;
        let name = entry.file_name().into_string().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Non-UTF-8 fragment name in '{}'", directory.display()),
            )
        })?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}

fn classify_changes(changed_files: &[String]) -> ChangeFlags {
    let relevant_files: Vec<&String> = changed_files
        .iter()
        .filter(|file| !is_ignored_by_ci(file))
        .collect();
    let code_pattern =
        Regex::new(r"\.(rs|toml|mjs|cjs|js|lino|yml|yaml|[mM][dD])$|\.github/workflows/").unwrap();
    // A workflow is code for the gates that read it, but it is not the code
    // the compiled jobs test. Counting it in `any_code_changed` made every
    // workflow edit buy the macOS archive, the Docker check, six box images
    // and a 25-minute agent end-to-end run; `pipeline_changed` below is what
    // says whether those jobs' own behaviour moved (issue #1107). A workflow
    // edit still runs lint and the workflow gates through `workflow_changed`.
    let is_workflow_only_change = |file: &str| {
        file.starts_with(".github/workflows/") && file != ".github/workflows/release.yml"
    };

    // Plan 16 L4 tier inputs, cumulative downward (see the constants above).
    let js_tier_input = |file: &str| {
        file.starts_with("js/")
            || file.starts_with("data/seed/")
            // The JavaScript server's route table, wording and parity corpus
            // (R1013) are js-tier inputs read by every tier below it.
            || file.starts_with("data/meta/server-")
            || file.starts_with("rust/tests/fixtures/server-parity/")
            || file == "scripts/check-server-parity.mjs"
            // The js -> rust leg's translator driver, its pinned ledger and
            // the committed translation (R1000): the js-rust job checks them.
            || file == "scripts/translate-js-rust.mjs"
            || file == "scripts/check-js-rust-translation.mjs"
            || file == "scripts/lib/rust-projection-presentation.mjs"
            || file == "scripts/self-translation/lexer.mjs"
            || file == "scripts/check-readable-code.mjs"
            || file == "data/meta/js-rust-translation.lino"
            || file.starts_with("rust/tests/fixtures/js-rust-translation/")
            || file == LAYERED_WORKFLOW
            || file == LAYER_CLASSIFIER
    };
    let ts_tier_input = |file: &str| js_tier_input(file) || file.starts_with("ts/");
    let rust_tier_input = |file: &str| {
        ts_tier_input(file)
            || file.starts_with("rust/")
            || file.starts_with(".github/actions/formal-ai-binary/")
    };

    ChangeFlags {
        rs_changed: relevant_files.iter().any(|file| has_extension(file, "rs")),
        toml_changed: relevant_files
            .iter()
            .any(|file| has_extension(file, "toml")),
        docs_changed: relevant_files.iter().any(|file| has_extension(file, "md")),
        workflow_changed: relevant_files
            .iter()
            .any(|file| file.starts_with(".github/workflows/")),
        // Only the files the pipeline itself executes. `workflow_changed` is
        // true for any file under `.github/workflows/`, and the heavy jobs of
        // the pipeline -- the macOS archive, the Docker check, six box images
        // and a 25-minute agent E2E -- were gated on it, so editing an
        // unrelated workflow bought the full 310 job-minutes of a code push
        // (issue #1107). A change to the pipeline's own definition, to a
        // composite action it calls, or to a script those run still unlocks
        // them, because those are the files that change what the jobs do.
        pipeline_changed: relevant_files.iter().any(|file| {
            file.as_str() == ".github/workflows/release.yml"
                || file.starts_with(".github/actions/")
                || file.starts_with("scripts/")
        }),
        // The clients expose different tool vocabularies. A change anywhere in
        // this bounded routing subsystem therefore needs the real four-client
        // replay before merge, even though other feature branches retain the
        // cheaper held-out-only Agent CLI gate (issue #1137). The prefix is
        // the post-L1 crate path: the restructure (plan 16 L1) moved
        // src/ to rust/src/, and the pre-L1 spelling silently disabled this
        // gate on every pull request after it.
        agentic_routing_changed: relevant_files
            .iter()
            .any(|file| file.starts_with("rust/src/agentic_coding/")),
        js_changed: relevant_files
            .iter()
            .any(|file| js_tier_input(file) || file.starts_with(JS_SUITES)),
        ts_changed: relevant_files.iter().any(|file| ts_tier_input(file)),
        rust_changed: relevant_files.iter().any(|file| rust_tier_input(file)),
        any_code_changed: relevant_files
            .iter()
            .filter(|file| !is_excluded_from_code_changes(file))
            .filter(|file| !is_workflow_only_change(file))
            .any(|file| code_pattern.is_match(file)),
    }
}

fn main() -> std::io::Result<()> {
    println!("Detecting file changes for CI/CD...\n");

    let changed_files = get_changed_files()?;

    println!("Changed files:");
    if changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &changed_files {
            println!("  {file:?}");
        }
    }
    println!();

    // Establish the release inventory before emitting any job decision. Other
    // workflows do not consume it and must not acquire a new filesystem input.
    let event = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    let git_ref = env::var("GITHUB_REF").unwrap_or_default();
    let pending = if env::var("RELEASE_PIPELINE").is_ok_and(|value| value == "true")
        && event == "push"
        && git_ref == "refs/heads/main"
    {
        automatic_release_pending(
            &event,
            &git_ref,
            &changelog_fragments(Path::new("changelog.d"))?,
            &changed_files,
        )
    } else {
        false
    };

    let flags = classify_changes(&changed_files);
    set_output(
        "rs-changed",
        if flags.rs_changed { "true" } else { "false" },
    )?;
    set_output(
        "toml-changed",
        if flags.toml_changed { "true" } else { "false" },
    )?;
    set_output(
        "docs-changed",
        if flags.docs_changed { "true" } else { "false" },
    )?;
    set_output(
        "workflow-changed",
        if flags.workflow_changed {
            "true"
        } else {
            "false"
        },
    )?;
    set_output(
        "pipeline-changed",
        if flags.pipeline_changed {
            "true"
        } else {
            "false"
        },
    )?;
    set_output(
        "agentic-routing-changed",
        if flags.agentic_routing_changed {
            "true"
        } else {
            "false"
        },
    )?;
    set_output(
        "js-changed",
        if flags.js_changed { "true" } else { "false" },
    )?;
    set_output(
        "ts-changed",
        if flags.ts_changed { "true" } else { "false" },
    )?;
    set_output(
        "rust-changed",
        if flags.rust_changed { "true" } else { "false" },
    )?;

    // Show code changes after both the CI-wide ignored paths and the broader
    // any-code documentation/example exclusions have been applied.
    let code_changed_files: Vec<&String> = changed_files
        .iter()
        .filter(|file| !is_ignored_by_ci(file) && !is_excluded_from_code_changes(file))
        .collect();

    println!("\nFiles considered as code changes:");
    if code_changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &code_changed_files {
            println!("  {file:?}");
        }
    }
    println!();

    if pending && !flags.any_code_changed {
        println!(
            "A changelog fragment is waiting on main: the release needs lint, test and the build."
        );
    }
    set_output("release-pending", if pending { "true" } else { "false" })?;
    // Check if any code files changed (.rs, .toml, .mjs, .cjs, .js, .lino, .yml,
    // .yaml, or workflow files). .cjs covers the Electron desktop and VS Code
    // extension host sources (extension.*.cjs, lib/*.cjs) so changes there still
    // trigger lint/test. .lino covers seed lexicons and language resources such
    // as js/i18n-catalog.lino: the language-change-parity guard watches
    // those files, so editing one must run lint/test that enforces the guard.
    set_output(
        "any-code-changed",
        if flags.any_code_changed || pending {
            "true"
        } else {
            "false"
        },
    )?;

    println!("\nChange detection completed.");
    Ok(())
}

#[cfg(test)]
#[path = "detect-code-changes-tests.rs"]
mod tests;
