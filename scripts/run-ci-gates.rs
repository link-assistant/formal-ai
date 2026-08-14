#!/usr/bin/env rust-script
//! Run the CI gates named by `data/meta/ci-gates/`, one shard per gate.
//!
//! Issue #991 review feedback: "All other similar places which may generate
//! conflicts should be fixed in similar way."
//!
//! `.github/workflows/release.yml` is the third most conflicted path in the
//! repository (35 manual resolutions, see `data/meta/merge-conflict-ledger.lino`)
//! and the cause is structural: its `lint` job was one long append-only list of
//! steps, so every branch that added a check appended to the same region.
//!
//! The list cannot simply be union merged — a workflow file is YAML with jobs and
//! conditions in it, and a union of two edits to a job's `if:` can parse and still
//! be wrong. So the *entries* move out of it: each gate is one file under
//! `data/meta/ci-gates/`, named after the gate. Two branches that each add a gate
//! create two different files, which git merges without ever looking at a shared
//! region. The workflow keeps three steps — one per stage — that hand the list
//! back to this runner.
//!
//! A stage says what the gate needs to already be installed, not what it depends
//! on: gates inside a stage are independent, so they run in shard order and all
//! of them run even after one fails. A CI run that reports every broken gate at
//! once is worth more than one that stops at the first.
//!
//! Usage:
//!     rust-script scripts/run-ci-gates.rs --stage rust   # run one stage
//!     rust-script scripts/run-ci-gates.rs --list         # print the registry
//!     rust-script scripts/run-ci-gates.rs --check        # validate, run nothing
//!
//! ```cargo
//! [dependencies]
//! ```

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::Command;

const REGISTRY: &str = "data/meta/ci-gates";
const WORKFLOW: &str = ".github/workflows/release.yml";

/// The stages a gate can ask for, in the order the workflow provides them.
const STAGES: [&str; 3] = ["rust", "wasm", "web"];

/// One registered gate: a shell command plus the stage it needs.
///
/// Public because the unit suite compiles this script as a module
/// (`tests/unit/ci-cd/mod.rs`) and reads the registry through the same parser
/// CI runs, rather than growing a second one that could disagree with it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Gate {
    pub name: String,
    pub stage: String,
    pub description: String,
    pub run: String,
    pub env: Vec<(String, String)>,
    /// Shard the gate was read from, for error messages.
    pub source: String,
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

/// Read one shard. A shard holds exactly one gate, which is what makes adding a
/// gate a whole-file addition instead of an edit to a shared list.
fn parse_gate(source: &str, shard: &str) -> Result<Gate, String> {
    let mut gate = Gate {
        source: shard.to_string(),
        ..Gate::default()
    };
    for line in source.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let trimmed = line.trim();
        let (key, value) = match trimmed.split_once(' ') {
            Some((key, value)) => (key, value.trim()),
            None => (trimmed, ""),
        };
        match (indent_of(line), key) {
            (0, "ci_gate") => gate.name = unquote(value),
            (2, "stage") => gate.stage = unquote(value),
            (2, "description") => gate.description = unquote(value),
            (2, "run") => gate.run = unquote(value),
            (2, "env") => {
                let (name, literal) = value
                    .split_once(' ')
                    .ok_or_else(|| format!("{shard}: `env` needs a name and a value"))?;
                gate.env.push((name.trim().to_string(), unquote(literal)));
            }
            _ => {}
        }
    }
    if gate.name.is_empty() {
        return Err(format!("{shard}: no `ci_gate` name"));
    }
    if gate.run.is_empty() {
        return Err(format!("{shard}: gate `{}` runs nothing", gate.name));
    }
    if gate.description.is_empty() {
        return Err(format!(
            "{shard}: gate `{}` has no description; the workflow log is the only place \
             a reader learns what a failing gate was protecting",
            gate.name
        ));
    }
    if !STAGES.contains(&gate.stage.as_str()) {
        return Err(format!(
            "{shard}: gate `{}` asks for stage `{}`; the workflow provides {}",
            gate.name,
            gate.stage,
            STAGES.join(", ")
        ));
    }
    Ok(gate)
}

/// Every shard in the registry, in file-name order.
pub fn load_registry(root: &Path) -> Result<Vec<Gate>, String> {
    let dir = root.join(REGISTRY);
    let mut shards: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|error| format!("{}: {error}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("lino"))
        .collect();
    shards.sort();

    let mut gates = Vec::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for shard in shards {
        let label = shard
            .strip_prefix(root)
            .unwrap_or(&shard)
            .to_string_lossy()
            .to_string();
        let source = fs::read_to_string(&shard).map_err(|error| format!("{label}: {error}"))?;
        let gate = parse_gate(&source, &label)?;
        let expected = format!("{REGISTRY}/{}.lino", gate.name.replace('_', "-"));
        if label != expected {
            return Err(format!(
                "{label}: gate `{}` belongs in `{expected}`. Naming the shard after the gate \
                 is what stops two branches from claiming the same file",
                gate.name
            ));
        }
        if !names.insert(gate.name.clone()) {
            return Err(format!("{label}: gate `{}` is registered twice", gate.name));
        }
        gates.push(gate);
    }
    if gates.is_empty() {
        return Err(format!("{REGISTRY}/ registers no gates"));
    }
    Ok(gates)
}

/// The body of one job in the workflow, by its two-space-indented name.
///
/// Only the `lint` job is a gate list. The release jobs each run one or two
/// checks whose results feed a job output or a conditional, so they are steps
/// with logic in them, not entries in a list, and the ledger does not show them
/// growing.
fn job_body<'a>(workflow: &'a str, job: &str) -> &'a str {
    let header = format!("\n  {job}:\n");
    let Some(start) = workflow.find(&header).map(|index| index + header.len()) else {
        return "";
    };
    let rest = &workflow[start..];
    let end = rest
        .match_indices("\n  ")
        .find(|(index, _)| {
            let after = &rest[index + 3..];
            after
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic())
                && after
                    .lines()
                    .next()
                    .is_some_and(|line| line.trim_end().ends_with(':'))
        })
        .map_or(rest.len(), |(index, _)| index);
    &rest[..end]
}

/// Gate commands the lint job still runs inline, bypassing the registry.
///
/// This is the check that keeps the append point closed: without it the list
/// would grow back one convenient step at a time.
fn inline_gate_steps(workflow: &str) -> Vec<String> {
    job_body(workflow, "lint")
        .lines()
        .map(str::trim)
        .filter(|line| {
            let Some(command) = line.strip_prefix("run: ") else {
                return false;
            };
            command.starts_with("rust-script scripts/check-")
                || command.starts_with("rust-script --test scripts/check-")
                || command.starts_with("npm run --prefix tests/e2e check:")
        })
        .map(str::to_string)
        .collect()
}

/// Stages the workflow actually invokes.
fn invoked_stages(workflow: &str) -> BTreeSet<String> {
    let mut stages = BTreeSet::new();
    for line in workflow.lines() {
        if let Some(rest) = line.trim().split_once("run-ci-gates.rs --stage ") {
            stages.insert(rest.1.split_whitespace().next().unwrap_or("").to_string());
        }
    }
    stages
}

fn registry_problems(gates: &[Gate], workflow: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for step in inline_gate_steps(workflow) {
        problems.push(format!(
            "{WORKFLOW} runs a gate inline: `{step}`. Add a shard under {REGISTRY}/ instead, \
             so the next branch that adds a gate does not touch this file"
        ));
    }
    let invoked = invoked_stages(workflow);
    // A set, so a stage with several gates is reported once and the problems
    // come out in a stable order however the shards happened to be read.
    let registered: BTreeSet<&str> = gates.iter().map(|gate| gate.stage.as_str()).collect();
    for stage in &registered {
        if !invoked.contains(*stage) {
            problems.push(format!(
                "stage `{stage}` has registered gates but {WORKFLOW} never runs it"
            ));
        }
    }
    problems
}

/// One gate rendered as the workflow step it replaced.
fn gate_step(gate: &Gate) -> String {
    let mut step = format!("      - name: {}\n", gate.name);
    if !gate.env.is_empty() {
        step.push_str("        env:\n");
        for (name, value) in &gate.env {
            writeln!(step, "          {name}: {value}").expect("writing to a String never fails");
        }
    }
    writeln!(step, "        run: {}", gate.run).expect("writing to a String never fails");
    step
}

/// The whole command surface CI executes, workflow and registry as one text.
///
/// Moving the gates out of `release.yml` broke every test that asked "does CI
/// run this?" by searching the workflow for a command. The question stayed
/// right; only the answer moved. This splices each stage's registered gates
/// back in at the step that runs them, so those tests keep asking it against
/// the whole surface -- and because the gates land where CI reaches them,
/// position in this text still means position in the run, which is what the
/// ordering assertions depend on.
pub fn workflow_surface(workflow: &str, gates: &[Gate]) -> String {
    let mut surface = workflow.to_string();
    for stage in STAGES {
        let marker = format!("run: rust-script scripts/run-ci-gates.rs --stage {stage}");
        let Some(index) = surface.find(&marker) else {
            continue;
        };
        let steps: String = gates
            .iter()
            .filter(|gate| gate.stage == stage)
            .map(gate_step)
            .collect();
        if steps.is_empty() {
            continue;
        }
        surface.insert_str(index + marker.len(), &format!("\n{}", steps.trim_end()));
    }
    surface
}

#[cfg(not(test))]
fn repository_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("git rev-parse");
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
}

#[cfg(not(test))]
fn run_stage(root: &Path, gates: &[Gate], stage: &str) -> i32 {
    let selected: Vec<&Gate> = gates.iter().filter(|gate| gate.stage == stage).collect();
    if selected.is_empty() {
        println!("No gates registered for stage `{stage}`.");
        return 0;
    }
    println!("Running {} gate(s) for stage `{stage}`.\n", selected.len());

    let mut failed: Vec<&str> = Vec::new();
    for gate in selected {
        println!("::group::{} — {}", gate.name, gate.description);
        println!("$ {}", gate.run);
        let mut command = Command::new("bash");
        command.arg("-o").arg("pipefail").arg("-c").arg(&gate.run);
        command.current_dir(root);
        for (name, value) in &gate.env {
            command.env(name, value);
        }
        let status = command.status();
        println!("::endgroup::");
        match status {
            Ok(status) if status.success() => {}
            Ok(status) => {
                println!(
                    "::error::gate `{}` failed ({status}); see {}",
                    gate.name, gate.source
                );
                failed.push(&gate.name);
            }
            Err(error) => {
                println!("::error::gate `{}` could not start: {error}", gate.name);
                failed.push(&gate.name);
            }
        }
    }

    if failed.is_empty() {
        println!("\nEvery gate in stage `{stage}` passed.");
        return 0;
    }
    // Every gate runs even after one fails: a run that reports all the broken
    // gates at once saves the round trips a fail-fast list would cost.
    println!(
        "\n::error::{} gate(s) failed in stage `{stage}`: {}",
        failed.len(),
        failed.join(", ")
    );
    1
}

#[cfg(not(test))]
fn main() {
    let root = repository_root();
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    let gates = match load_registry(&root) {
        Ok(gates) => gates,
        Err(problem) => {
            println!("::error::{problem}");
            std::process::exit(1);
        }
    };
    let workflow = fs::read_to_string(root.join(WORKFLOW)).unwrap_or_default();

    if arguments.iter().any(|argument| argument == "--list") {
        for stage in STAGES {
            println!("stage {stage}:");
            for gate in gates.iter().filter(|gate| gate.stage == stage) {
                println!("  {:<40} {}", gate.name, gate.description);
            }
        }
        return;
    }

    let problems = registry_problems(&gates, &workflow);
    for problem in &problems {
        println!("::error::{problem}");
    }
    if !problems.is_empty() {
        std::process::exit(1);
    }

    if arguments.iter().any(|argument| argument == "--check") {
        println!(
            "{} gate(s) registered across {} stage(s); the workflow runs every stage.",
            gates.len(),
            STAGES.len()
        );
        return;
    }

    let Some(stage) = arguments
        .iter()
        .position(|argument| argument == "--stage")
        .and_then(|index| arguments.get(index + 1))
    else {
        println!(
            "::error::pass --stage <{}>, --list or --check",
            STAGES.join("|")
        );
        std::process::exit(2);
    };
    std::process::exit(run_stage(&root, &gates, stage));
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHARD: &str = "ci_gate check_formatting\n  stage rust\n  \
        description \"rustfmt owns the layout of every Rust file.\"\n  \
        run \"cargo fmt --all -- --check\"\n";

    fn gate() -> Gate {
        parse_gate(SHARD, "data/meta/ci-gates/check-formatting.lino").unwrap()
    }

    #[test]
    fn a_shard_parses_into_one_gate() {
        let gate = gate();
        assert_eq!(gate.name, "check_formatting");
        assert_eq!(gate.stage, "rust");
        assert_eq!(gate.run, "cargo fmt --all -- --check");
        assert!(gate.env.is_empty());
    }

    #[test]
    fn per_gate_environment_is_read() {
        let source = format!("{SHARD}  env RUSTDOCFLAGS \"-D warnings\"\n  env DOCS_RS \"1\"\n");
        let gate = parse_gate(&source, "data/meta/ci-gates/check-formatting.lino").unwrap();
        assert_eq!(
            gate.env,
            [
                ("RUSTDOCFLAGS".to_string(), "-D warnings".to_string()),
                ("DOCS_RS".to_string(), "1".to_string()),
            ]
        );
    }

    #[test]
    fn a_gate_without_a_description_is_rejected() {
        let source = "ci_gate check_formatting\n  stage rust\n  run \"cargo fmt\"\n";
        let error = parse_gate(source, "shard.lino").unwrap_err();
        assert!(error.contains("no description"), "{error}");
    }

    #[test]
    fn an_unknown_stage_is_rejected() {
        let source = SHARD.replace("stage rust", "stage everything");
        let error = parse_gate(&source, "shard.lino").unwrap_err();
        assert!(error.contains("the workflow provides"), "{error}");
    }

    /// A workflow whose lint job contains `steps`, plus a later release job.
    fn workflow(lint_steps: &str, release_steps: &str) -> String {
        format!(
            "jobs:\n  lint:\n    steps:\n{lint_steps}  version-check:\n    steps:\n{release_steps}"
        )
    }

    #[test]
    fn a_gate_the_workflow_still_runs_inline_fails() {
        // The whole point of the registry: if a branch can append one more
        // `run:` line to the lint job, the append-only list grows straight back.
        let source = workflow(
            "      - name: Check something\n        \
             run: rust-script scripts/check-something.rs\n      \
             - run: rust-script scripts/run-ci-gates.rs --stage rust\n",
            "",
        );
        let problems = registry_problems(&[gate()], &source);
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("runs a gate inline")),
            "{problems:?}"
        );
    }

    #[test]
    fn a_check_outside_the_lint_job_stays_inline() {
        // The release jobs run one check each, guarding a job output or a
        // conditional. They are steps with logic, not a list, so the registry
        // does not claim them and must not report them.
        let source = workflow(
            "      - run: rust-script scripts/run-ci-gates.rs --stage rust\n",
            "      - run: rust-script scripts/check-version-modification.rs\n",
        );
        assert!(registry_problems(&[gate()], &source).is_empty());
    }

    #[test]
    fn a_stage_the_workflow_never_runs_fails() {
        let problems = registry_problems(&[gate()], &workflow("      - run: echo hello\n", ""));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("never runs it")),
            "{problems:?}"
        );
    }

    #[test]
    fn a_registry_the_workflow_fully_runs_has_no_problems() {
        let source = workflow(
            "      - run: rust-script scripts/run-ci-gates.rs --stage rust\n",
            "",
        );
        assert!(registry_problems(&[gate()], &source).is_empty());
    }

    #[test]
    fn the_surface_places_a_gate_where_its_stage_runs() {
        // The tests that grep for a command also compare positions -- "the
        // documentation build runs before packaging". Splicing a gate in at its
        // stage step keeps those comparisons meaningful; appending it to the end
        // of the file would silently invert every one of them.
        let source = workflow(
            "      - run: rust-script scripts/run-ci-gates.rs --stage rust\n",
            "      - run: cargo package\n",
        );
        let surface = workflow_surface(&source, &[gate()]);

        assert!(
            surface.contains("run: cargo fmt --all -- --check"),
            "{surface}"
        );
        assert!(
            surface.find("cargo fmt").unwrap() < surface.find("cargo package").unwrap(),
            "{surface}"
        );
    }

    #[test]
    fn the_surface_carries_each_gates_environment() {
        let mut with_env = gate();
        with_env.env = vec![("RUSTDOCFLAGS".to_string(), "-D warnings".to_string())];
        let source = workflow(
            "      - run: rust-script scripts/run-ci-gates.rs --stage rust\n",
            "",
        );

        let surface = workflow_surface(&source, &[with_env]);

        assert!(surface.contains("RUSTDOCFLAGS: -D warnings"), "{surface}");
    }

    #[test]
    fn a_stage_with_no_registered_gates_leaves_the_workflow_alone() {
        let source = workflow(
            "      - run: rust-script scripts/run-ci-gates.rs --stage web\n",
            "",
        );

        assert_eq!(workflow_surface(&source, &[gate()]), source);
    }

    #[test]
    fn the_shard_name_must_match_the_gate_name() {
        let root = std::env::temp_dir().join("issue-991-ci-gate-registry");
        let dir = root.join(REGISTRY);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("formatting.lino"), SHARD).unwrap();
        let error = load_registry(&root).unwrap_err();
        assert!(error.contains("belongs in"), "{error}");
        fs::rename(
            dir.join("formatting.lino"),
            dir.join("check-formatting.lino"),
        )
        .unwrap();
        assert_eq!(load_registry(&root).unwrap().len(), 1);
        let _ = fs::remove_dir_all(&root);
    }
}
