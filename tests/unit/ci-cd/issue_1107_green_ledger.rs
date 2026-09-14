//! A check that was green on identical inputs is not run twice (issue #1107).
//!
//! `detect-changes` compares the whole pull-request range, so a branch that once
//! touched `src/` re-ran every Rust check on every later docs-only push. The
//! `green-ledger` action keys a marker on the content of a check's inputs; a hit
//! reports itself and skips the body. What these contracts pin is the part that
//! keeps a skip honest: the marker is saved only by a job that succeeded, the
//! skip is announced, `main` never skips, and a job that consults the ledger
//! gates every later step on it -- a step that ran anyway would spend the
//! minutes the ledger exists to save, and one that ran only sometimes would
//! make the job's result depend on which steps happened to be gated.

use std::fs;
use std::path::Path;

const ACTION: &str = ".github/actions/green-ledger/action.yml";

/// Workflows that consult the ledger, with the jobs that carry the `ledger` step.
const LEDGERED: &[(&str, &[&str])] = &[
    (
        ".github/workflows/release.yml",
        &[
            "test",
            "docker-build",
            "box-language-projects",
            "test-e2e-local",
        ],
    ),
    (".github/workflows/macos-core-tests.yml", &["build-archive"]),
    (".github/workflows/agent-cli-e2e.yml", &["agent-cli-e2e"]),
    (".github/workflows/issue-1028-agent-ladder.yml", &["ladder"]),
    (".github/workflows/agentic-cli-matrix.yml", &["build"]),
];

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The lines of one job: from `  <job>:` to the next two-space key.
fn job_lines<'a>(workflow: &'a str, job: &str) -> Vec<&'a str> {
    let header = format!("  {job}:");
    let mut lines = workflow
        .lines()
        .skip_while(|line| line.trim_end() != header);
    let first = lines.next().unwrap_or_else(|| panic!("missing job {job}"));
    std::iter::once(first)
        .chain(lines.take_while(|line| {
            !(line.starts_with("  ") && !line.starts_with("   ") && line.trim_end().ends_with(':'))
        }))
        .collect()
}

/// Steps of a job as (first line, condition) pairs, in order.
fn steps(job: &[&str]) -> Vec<(String, Option<String>)> {
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    for line in job {
        if let Some(rest) = line.strip_prefix("      - ") {
            let condition = rest
                .strip_prefix("if:")
                .map(|value| value.trim().to_owned());
            out.push((rest.to_owned(), condition));
        } else if let Some(rest) = line.strip_prefix("        if:")
            && let Some(last) = out.last_mut()
        {
            // A block scalar (`if: >-`) carries its condition on the following
            // lines; take the marker as the start and let them accumulate.
            last.1 = Some(rest.trim().to_owned());
        } else if let Some(last) = out.last_mut()
            && last
                .1
                .as_ref()
                .is_some_and(|condition| condition.starts_with(['>', '|']))
            && line.starts_with("          ")
        {
            let condition = last.1.take().unwrap_or_default();
            last.1 = Some(format!("{condition} {}", line.trim()));
        }
    }
    out
}

#[test]
fn the_action_saves_only_on_success_and_announces_a_skip() {
    let action = read(ACTION);
    assert!(
        action.contains("uses: actions/cache@v4"),
        "the marker rides on actions/cache, whose post step has post-if: success()"
    );
    assert!(
        action.contains("post-if"),
        "the action must say why a failed job leaves no marker"
    );
    assert!(action.contains("::notice title=$CHECK already green::"));
    assert!(action.contains("GITHUB_STEP_SUMMARY"));
    assert!(
        action.contains("already-green:"),
        "the decision is an output the caller gates on"
    );
    assert!(
        action.contains("git ls-files -s -- $PATHS") && action.contains("git hash-object --stdin"),
        "the key is the content of the named paths, as formal-ai-binary keys its build"
    );
}

/// Two defects the first CI run of this action found, both of which made every
/// consumer job fail to even start.
///
/// GitHub evaluates `${{ }}` inside an action's `description:` as a template,
/// not as prose, and `github.*` is not in scope in a composite action's
/// manifest -- an example expression written in the description of `enabled`
/// therefore failed the manifest to load for every job that used it. And an
/// input interpolated straight into a `run:` block is a code-injection surface
/// (zizmor `template-injection`), so every value this action passes to the
/// shell goes through `env:` instead.
#[test]
fn the_action_neither_templates_its_prose_nor_interpolates_into_a_script() {
    let action = read(ACTION);
    let (documentation, steps) = action
        .split_once("\nruns:")
        .expect("the manifest declares its inputs before its steps");
    // `outputs.<id>.value` is the one place above `runs:` where an expression
    // belongs; a description is prose and is evaluated all the same.
    let mut in_outputs = false;
    for line in documentation.lines() {
        if !line.starts_with(char::is_whitespace) {
            in_outputs = line.starts_with("outputs:");
        }
        if in_outputs && line.trim_start().starts_with("value:") {
            continue;
        }
        assert!(
            !line.contains("${{"),
            "an action's description and input descriptions are evaluated as templates, \
             and `github.*` is not in scope in a composite manifest: write the expression \
             in prose, or the manifest fails to load for every job that uses it:\n{line}"
        );
    }
    for block in steps.split("      run: |").skip(1) {
        let script = block.split("\n    - ").next().unwrap_or(block);
        assert!(
            !script.contains("${{"),
            "a value interpolated into a run block is a code-injection surface; \
             pass it through env: instead:\n{script}"
        );
    }
}

#[test]
fn every_ledgered_job_gates_all_later_steps_and_names_its_own_workflow() {
    for (path, jobs) in LEDGERED {
        let workflow = read(path);
        let file_name = Path::new(path).file_name().unwrap().to_str().unwrap();
        for job in *jobs {
            let lines = job_lines(&workflow, job);
            let text = lines.join("\n");
            let ledger = text
                .find("uses: ./.github/actions/green-ledger")
                .unwrap_or_else(|| panic!("{path} job {job} does not consult the ledger"));
            let block = &text[ledger..];
            let block = &block[..block.find("\n      - ").unwrap_or(block.len())];
            assert!(
                block.contains(file_name),
                "{path} job {job}: the ledger paths must include {file_name}, so editing the check re-runs it"
            );
            assert!(
                block.contains("enabled: ${{ github.ref != 'refs/heads/main' }}"),
                "{path} job {job}: main must never skip"
            );
            let all = steps(&lines);
            let ledger_index = all
                .iter()
                .position(|(first, _)| first.contains("Skip when identical inputs already passed"))
                .expect("ledger step");
            for (first, condition) in &all[ledger_index + 1..] {
                let condition = condition.as_deref().unwrap_or("");
                // Cleanup that costs seconds and must run whatever happened
                // (`!cancelled()`) is the one shape allowed through ungated.
                if condition.contains("!cancelled()") && !condition.contains("already-green") {
                    continue;
                }
                assert!(
                    condition.contains("already-green"),
                    "{path} job {job}: step `{first}` is not gated on the ledger (if: {condition:?})"
                );
            }
        }
    }
}

#[test]
fn consumers_of_a_ledgered_producer_gate_on_its_output() {
    // A skipped producer uploads no artifact; its consumers must not try to
    // download one. They read the producer's `already-green` output instead.
    for (path, producer, consumers) in [
        (
            ".github/workflows/macos-core-tests.yml",
            "build-archive",
            &["test-archive"][..],
        ),
        (
            ".github/workflows/agentic-cli-matrix.yml",
            "build",
            &["matrix", "learn"][..],
        ),
    ] {
        let workflow = read(path);
        let producer_text = job_lines(&workflow, producer).join("\n");
        assert!(
            producer_text.contains("already-green: ${{ steps.ledger.outputs.already-green }}"),
            "{path} job {producer} must expose already-green as a job output"
        );
        for consumer in consumers {
            let lines = job_lines(&workflow, consumer);
            let needle = format!("needs.{producer}.outputs.already-green != 'true'");
            for (first, condition) in steps(&lines) {
                assert!(
                    condition.as_deref().unwrap_or("").contains(&needle),
                    "{path} job {consumer}: step `{first}` must gate on {needle}"
                );
            }
        }
    }
}
