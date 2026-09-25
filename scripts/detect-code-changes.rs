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
//! - Markdown files (*.md) in any folder
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
//!
//! Outputs (written to `GITHUB_OUTPUT`):
//!   - rs-changed: 'true' if any .rs files changed
//!   - toml-changed: 'true' if any .toml files changed
//!   - docs-changed: 'true' if any .md files changed
//!   - workflow-changed: 'true' if any .github/workflows/ files changed
//!   - any-code-changed: 'true' if any non-ignored code files changed
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
use std::io::Write;
use std::path::Path;
use std::process::Command;

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

fn exec(command: &str, args: &[&str]) -> String {
    match Command::new(command).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                eprintln!("Error executing {command} {args:?}");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                String::new()
            }
        }
        Err(e) => {
            eprintln!("Failed to execute {command} {args:?}: {e}");
            String::new()
        }
    }
}

fn set_output(name: &str, value: &str) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT")
        && let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)
    {
        let _ = writeln!(file, "{name}={value}");
    }
    println!("{name}={value}");
}

fn is_merge_commit() -> bool {
    let output = exec("git", &["cat-file", "-p", "HEAD"]);
    output
        .lines()
        .filter(|line| line.starts_with("parent "))
        .count()
        > 1
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

fn get_changed_files() -> Vec<String> {
    // GitHub Actions checks out a synthetic merge commit for pull_request
    // events: HEAD is the merge commit, HEAD^ is the base branch, HEAD^2
    // is the actual PR head. Comparing the two parents covers every commit in
    // the PR, so a docs-only final commit cannot hide earlier code changes.
    let event_name = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    let event_before = env::var("GITHUB_EVENT_BEFORE").ok();
    let merge_commit = is_merge_commit();
    let (base, head, description) =
        comparison_for_event(&event_name, merge_commit, event_before.as_deref());
    if event_name == "pull_request" && merge_commit {
        println!("Merge commit detected (pull_request event)");
    }
    println!("Comparing {base} to {head} ({description})");
    let output = exec("git", &["diff", "--name-only", &base, &head]);

    if output.is_empty() && exec("git", &["rev-parse", "--verify", &base]).is_empty() {
        println!("{base} is not available, listing all files in HEAD");
        let output = exec("git", &["ls-tree", "--name-only", "-r", "HEAD"]);
        return output
            .lines()
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
    }

    output
        .lines()
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn is_ignored_by_ci(file_path: &str) -> bool {
    CI_IGNORED_PATH_PREFIXES
        .iter()
        .any(|prefix| file_path.starts_with(prefix))
}

fn is_excluded_from_code_changes(file_path: &str) -> bool {
    // Exclude markdown files in any folder
    if has_extension(file_path, "md") {
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

fn classify_changes(changed_files: &[String]) -> ChangeFlags {
    let relevant_files: Vec<&String> = changed_files
        .iter()
        .filter(|file| !is_ignored_by_ci(file))
        .collect();
    let code_pattern =
        Regex::new(r"\.(rs|toml|mjs|cjs|js|lino|yml|yaml)$|\.github/workflows/").unwrap();
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
            || file == LAYERED_WORKFLOW
            || file == LAYER_CLASSIFIER
    };
    let ts_tier_input =
        |file: &str| js_tier_input(file) || file.starts_with("ts/");
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
        js_changed: relevant_files.iter().any(|file| js_tier_input(file)),
        ts_changed: relevant_files.iter().any(|file| ts_tier_input(file)),
        rust_changed: relevant_files.iter().any(|file| rust_tier_input(file)),
        any_code_changed: relevant_files
            .iter()
            .filter(|file| !is_excluded_from_code_changes(file))
            .filter(|file| !is_workflow_only_change(file))
            .any(|file| code_pattern.is_match(file)),
    }
}

fn main() {
    println!("Detecting file changes for CI/CD...\n");

    let changed_files = get_changed_files();

    println!("Changed files:");
    if changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &changed_files {
            println!("  {file}");
        }
    }
    println!();

    let flags = classify_changes(&changed_files);
    set_output(
        "rs-changed",
        if flags.rs_changed { "true" } else { "false" },
    );
    set_output(
        "toml-changed",
        if flags.toml_changed { "true" } else { "false" },
    );
    set_output(
        "docs-changed",
        if flags.docs_changed { "true" } else { "false" },
    );
    set_output(
        "workflow-changed",
        if flags.workflow_changed {
            "true"
        } else {
            "false"
        },
    );
    set_output(
        "pipeline-changed",
        if flags.pipeline_changed {
            "true"
        } else {
            "false"
        },
    );
    set_output(
        "agentic-routing-changed",
        if flags.agentic_routing_changed {
            "true"
        } else {
            "false"
        },
    );
    set_output(
        "js-changed",
        if flags.js_changed { "true" } else { "false" },
    );
    set_output(
        "ts-changed",
        if flags.ts_changed { "true" } else { "false" },
    );
    set_output(
        "rust-changed",
        if flags.rust_changed { "true" } else { "false" },
    );

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
            println!("  {file}");
        }
    }
    println!();

    // Check if any code files changed (.rs, .toml, .mjs, .cjs, .js, .lino, .yml,
    // .yaml, or workflow files). .cjs covers the Electron desktop and VS Code
    // extension host sources (extension.*.cjs, lib/*.cjs) so changes there still
    // trigger lint/test. .lino covers seed lexicons and language resources such
    // as js/i18n-catalog.lino: the language-change-parity guard watches
    // those files, so editing one must run lint/test that enforces the guard.
    set_output(
        "any-code-changed",
        if flags.any_code_changed {
            "true"
        } else {
            "false"
        },
    );

    println!("\nChange detection completed.");
}

#[cfg(test)]
mod tests {
    use super::{
        ChangeFlags, LAYERED_WORKFLOW, LAYER_CLASSIFIER, classify_changes, comparison_for_event,
    };

    #[test]
    fn pull_requests_compare_the_complete_base_to_head_range() {
        assert_eq!(
            comparison_for_event("pull_request", true, None),
            (
                "HEAD^".to_string(),
                "HEAD^2".to_string(),
                "complete pull request diff"
            )
        );
    }

    #[test]
    fn pushes_compare_every_commit_in_the_event() {
        assert_eq!(
            comparison_for_event("push", true, Some("before-sha")),
            (
                "before-sha".to_string(),
                "HEAD".to_string(),
                "complete push diff"
            ),
            "a pushed merge commit must not be mistaken for GitHub's synthetic PR merge"
        );
    }

    #[test]
    fn missing_push_before_sha_falls_back_to_the_previous_commit() {
        assert_eq!(
            comparison_for_event("push", false, Some("000000")),
            (
                "HEAD^".to_string(),
                "HEAD".to_string(),
                "previous commit diff"
            )
        );
    }

    #[test]
    fn ignored_only_changes_set_no_flags_for_pushes_or_pull_requests() {
        for event_name in ["push", "pull_request"] {
            for path in [
                "experiments/repro.rs",
                "experiments/README.md",
                "experiments/repro.mjs",
                "dev/log/run.rs",
                "docs/case-studies/issue-846/repro.rs",
            ] {
                assert_eq!(
                    classify_changes(&[path.to_string()]),
                    ChangeFlags::default(),
                    "{path} must be ignored on {event_name}"
                );
            }
        }
    }

    #[test]
    fn shipping_changes_still_set_their_flags() {
        assert_eq!(
            classify_changes(&[
                "src/lib.rs".to_string(),
                "guide.md".to_string(),
                ".github/workflows/release.yml".to_string(),
            ]),
            ChangeFlags {
                rs_changed: true,
                docs_changed: true,
                workflow_changed: true,
                pipeline_changed: true,
                any_code_changed: true,
                ..ChangeFlags::default()
            }
        );
    }

    /// The flag that decides whether a push pays for the heavy jobs.
    ///
    /// Issue #1107: `workflow_changed` is true for every file under
    /// `.github/workflows/`, so editing a scheduled benchmark unlocked the
    /// macOS archive, the Docker check, six box images and a 25-minute agent
    /// end-to-end run. `pipeline_changed` is true only for the files those
    /// jobs actually execute.
    #[test]
    fn an_unrelated_workflow_does_not_unlock_the_pipeline_heavy_jobs() {
        let unrelated =
            classify_changes(&[".github/workflows/external-benchmarks.yml".to_string()]);
        assert!(
            unrelated.workflow_changed,
            "the workflow gates still have to run for it"
        );
        assert!(
            !unrelated.pipeline_changed,
            "a scheduled benchmark's workflow does not change what the pipeline jobs do"
        );
        assert!(
            !unrelated.any_code_changed,
            "a workflow is not the compiled code the heavy jobs test; counting it here is what \
             made every workflow edit pay for the macOS archive, the Docker check, six box \
             images and a 25-minute agent end-to-end run"
        );

        for path in [
            ".github/workflows/release.yml",
            ".github/actions/author-with-formal-ai/action.yml",
            "scripts/author-change-with-formal-ai.sh",
        ] {
            assert!(
                classify_changes(&[path.to_string()]).pipeline_changed,
                "{path} changes what a pipeline job runs"
            );
        }
    }

    #[test]
    fn agentic_source_changes_request_the_four_client_replay() {
        for path in [
            "rust/src/agentic_coding/capability_router.rs",
            "rust/src/agentic_coding/planner.rs",
            "rust/src/agentic_coding/new_router.rs",
        ] {
            assert!(
                classify_changes(&[path.to_string()]).agentic_routing_changed,
                "{path} can change client-specific routing"
            );
        }

        for path in [
            "rust/src/coding/composition.rs",
            "docs/agentic_coding/README.md",
        ] {
            assert!(
                !classify_changes(&[path.to_string()]).agentic_routing_changed,
                "{path} is outside the agentic routing source boundary"
            );
        }
    }

    /// The js -> ts -> rust cycle's tier matrix (issue #1138, plan 16 L4).
    ///
    /// Cumulative downward, never upward: the committed ts tree is a
    /// function of the js tree and the dogfood gate binds all three, so a
    /// change wakes every tier below its folder and none above it. That
    /// asymmetry is the instruction's saving -- a rust iteration lets the
    /// js and ts tiers stand on their last green runs.
    #[test]
    fn tier_gates_are_cumulative_downward_and_never_upward() {
        let rust_only = classify_changes(&["rust/src/lib.rs".to_string()]);
        assert!(rust_only.rust_changed, "a rust change wakes the rust tier");
        assert!(
            !rust_only.js_changed && !rust_only.ts_changed,
            "a rust change lets the js and ts tiers carry their last green runs forward"
        );

        let js_only = classify_changes(&["js/app.js".to_string()]);
        assert!(
            js_only.js_changed && js_only.ts_changed && js_only.rust_changed,
            "a js change wakes every tier below it: the ts tree is a function of \
             the js tree, so a sleeping ts tier would carry a stale-green tier \
             past the plan 16 L3 regeneration contract"
        );

        let ts_only = classify_changes(&["ts/app.ts".to_string()]);
        assert!(
            ts_only.ts_changed && ts_only.rust_changed,
            "a ts change still reaches the tier that regenerates it"
        );
        assert!(
            !ts_only.js_changed,
            "the cycle never ascends: a ts change does not reexecute the js tier"
        );

        let seed = classify_changes(&["data/seed/tools.lino".to_string()]);
        assert!(
            seed.js_changed && seed.ts_changed && seed.rust_changed,
            "a seed consumed by all three roots wakes every tier (plan 16 risk 3)"
        );

        let docs = classify_changes(&["README.md".to_string()]);
        assert!(
            !docs.js_changed && !docs.ts_changed && !docs.rust_changed,
            "a docs-only change wakes no tier"
        );
    }

    /// The gating files re-run the tiers they gate, so the cycle's own
    /// rules can never go stale on a green branch.
    #[test]
    fn the_gating_files_rerun_the_tiers_they_gate() {
        for path in [LAYERED_WORKFLOW, LAYER_CLASSIFIER] {
            let flags = classify_changes(&[path.to_string()]);
            assert!(
                flags.js_changed && flags.ts_changed && flags.rust_changed,
                "{path} edits the tier rules, so every tier must re-run"
            );
        }

        let binary_action =
            classify_changes(&[".github/actions/formal-ai-binary/action.yml".to_string()]);
        assert!(
            binary_action.rust_changed,
            "the binary action changes what the rust tier executes"
        );
        assert!(
            !binary_action.js_changed && !binary_action.ts_changed,
            "the binary action feeds only the rust tier"
        );
    }
}
