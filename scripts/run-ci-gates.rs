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
//! A lane says which parallel `lint` job runs the gate (PR #1188). The rust
//! stage took 17-28 minutes as one serial list; the `lint` job is a matrix over
//! `lane:` and each leg runs only the gates of its lane (`CI_GATE_LANE`), so the
//! slow gates run beside the rest instead of after them. A gate names its lane
//! with `lane <n>`; without one it runs in lane 1, which is also the only lane
//! that runs the wasm and web stages. `--check` fails when a gate names a lane
//! the matrix does not run -- a gate nobody runs is the failure this registry
//! exists to prevent -- or when a lane runs no gate at all.
//!
//! Usage:
//!     rust-script scripts/run-ci-gates.rs --stage rust   # run one stage
//!     CI_GATE_LANE=2 rust-script scripts/run-ci-gates.rs --stage rust
//!     rust-script scripts/run-ci-gates.rs --list         # print the registry
//!     rust-script scripts/run-ci-gates.rs --check        # validate, run nothing
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
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

/// The lane every gate runs in unless it names another, and the only lane whose
/// `lint` leg runs the wasm and web stages.
const DEFAULT_LANE: u32 = 1;

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
    /// The parallel `lint` leg that runs this gate.
    pub lane: u32,
    /// Shard the gate was read from, for error messages.
    pub source: String,
    /// Why this gate exists: the concrete defect class or incident it prevents.
    /// Must cite an issue or pull request (`#NNN`) or a commit hash (7-40 hex
    /// characters). R1085-16 requires this on every gate.
    pub justification: String,
}

/// Returns true when the justification cites an issue/PR number (`#NNN`) or a
/// commit hash (7-40 lowercase hex characters).
pub fn justification_has_citation(justification: &str) -> bool {
    // Issue/PR citation: #NNN with at least one digit.
    let has_issue = justification
        .split_whitespace()
        .chain(justification.split(|c: char| !c.is_ascii_alphanumeric() && c != '#'))
        .any(|token| {
            token.starts_with('#')
                && token[1..].chars().all(|c| c.is_ascii_digit())
                && token.len() >= 2
        });
    if has_issue {
        return true;
    }
    // Commit hash: a run of 7-40 lowercase hex digits, standing alone (surrounded
    // by non-hex or start/end of string) so a URL slug like "plan-09" is not caught.
    let chars: Vec<char> = justification.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_hexdigit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_hexdigit() {
                i += 1;
            }
            let run = i - start;
            let before_ok = start == 0 || !chars[start - 1].is_ascii_alphanumeric();
            let after_ok = i >= chars.len() || !chars[i].is_ascii_alphanumeric();
            if run >= 7 && run <= 40 && before_ok && after_ok {
                return true;
            }
        } else {
            i += 1;
        }
    }
    false
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
        lane: DEFAULT_LANE,
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
            (2, "justification") => gate.justification = unquote(value),
            (2, "lane") => {
                gate.lane = unquote(value)
                    .parse::<u32>()
                    .ok()
                    .filter(|lane| *lane >= DEFAULT_LANE)
                    .ok_or_else(|| {
                        format!("{shard}: `lane` must be a number from 1, got `{value}`")
                    })?;
            }
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
    if gate.justification.is_empty() {
        return Err(format!(
            "{shard}: gate `{}` has no `justification`; every gate must name the \
             defect class or incident it prevents, citing an issue/PR (#NNN) or a \
             commit hash (R1085-16)",
            gate.name
        ));
    }
    if !justification_has_citation(&gate.justification) {
        return Err(format!(
            "{shard}: gate `{}`'s justification must cite an issue/PR (#NNN) or a \
             commit hash; got: `{}`",
            gate.name, gate.justification
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

/// The lanes the `lint` job's matrix runs, from its `lane: [..]` line.
///
/// `None` when the job is not a lane matrix; then every gate must be in the
/// default lane, because there is no other leg to run it.
fn workflow_lanes(workflow: &str) -> Option<BTreeSet<u32>> {
    job_body(workflow, "lint").lines().find_map(|line| {
        let list = line.trim().strip_prefix("lane: [")?.strip_suffix(']')?;
        Some(
            list.split(',')
                .filter_map(|lane| lane.trim().parse::<u32>().ok())
                .collect(),
        )
    })
}

/// Gates assigned to a lane no `lint` leg runs, and lanes that run no gate.
fn lane_problems(gates: &[Gate], workflow: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let lanes = workflow_lanes(workflow).unwrap_or_else(|| BTreeSet::from([DEFAULT_LANE]));
    for gate in gates {
        if !lanes.contains(&gate.lane) {
            problems.push(format!(
                "gate `{}` ({}) runs in lane {}, but the {WORKFLOW} `lint` matrix runs lanes \
                 {lanes:?}, so nothing would run it",
                gate.name, gate.source, gate.lane
            ));
        } else if gate.stage != STAGES[0] && gate.lane != DEFAULT_LANE {
            problems.push(format!(
                "gate `{}` ({}) is a `{}` stage gate in lane {}; only lane {DEFAULT_LANE} runs \
                 the wasm and web stages",
                gate.name, gate.source, gate.stage, gate.lane
            ));
        }
    }
    for lane in &lanes {
        if !gates.iter().any(|gate| gate.lane == *lane) {
            problems.push(format!(
                "the {WORKFLOW} `lint` matrix runs lane {lane}, but no gate is assigned to it"
            ));
        }
    }
    problems
}

fn registry_problems(gates: &[Gate], workflow: &str) -> Vec<String> {
    let mut problems = lane_problems(gates, workflow);
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
fn run_stage(root: &Path, gates: &[Gate], stage: &str, lane: Option<u32>) -> i32 {
    let selected: Vec<&Gate> = gates
        .iter()
        .filter(|gate| gate.stage == stage && lane.is_none_or(|lane| gate.lane == lane))
        .collect();
    let scope = lane.map_or_else(String::new, |lane| format!(" in lane {lane}"));
    if selected.is_empty() {
        println!("No gates registered for stage `{stage}`{scope}.");
        return 0;
    }
    println!(
        "Running {} gate(s) for stage `{stage}`{scope}.\n",
        selected.len()
    );

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
    // Unset runs every lane, as a local run expects; a malformed value must not
    // quietly select nothing.
    let lane = match std::env::var("CI_GATE_LANE") {
        Ok(value) if !value.trim().is_empty() => match value.trim().parse::<u32>() {
            Ok(lane) => Some(lane),
            Err(_) => {
                println!("::error::CI_GATE_LANE must be a lane number, got `{value}`");
                std::process::exit(2);
            }
        },
        _ => None,
    };
    std::process::exit(run_stage(&root, &gates, stage, lane));
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHARD: &str = "ci_gate check_formatting\n  stage rust\n  \
        description \"rustfmt owns the layout of every Rust file.\"\n  \
        run \"cargo fmt --all -- --check\"\n  \
        justification \"Formatting drift makes diffs noisy and wastes review time; \
        prevented since the registry was introduced in #991.\"\n";

    fn gate() -> Gate {
        parse_gate(SHARD, "data/meta/ci-gates/check-formatting.lino").unwrap()
    }

    #[test]
    fn a_shard_parses_into_one_gate() {
        let gate = gate();
        assert_eq!(gate.name, "check_formatting");
        assert_eq!(gate.stage, "rust");
        assert_eq!(gate.run, "cargo fmt --all -- --check");
        assert_eq!(
            gate.env,
            [] as [(std::string::String, std::string::String); 0]
        );
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
    fn a_gate_without_a_justification_is_rejected() {
        // R1085-16: every gate must carry a justification.
        let source = "ci_gate check_formatting\n  stage rust\n  \
            description \"rustfmt owns the layout.\"\n  run \"cargo fmt\"\n";
        let error = parse_gate(source, "shard.lino").unwrap_err();
        assert!(error.contains("no `justification`"), "{error}");
    }

    #[test]
    fn a_justification_without_a_citation_is_rejected() {
        // The justification must name an issue/PR (#NNN) or a commit hash.
        let source = "ci_gate check_formatting\n  stage rust\n  \
            description \"rustfmt owns the layout.\"\n  \
            run \"cargo fmt\"\n  \
            justification \"prevents formatting drift\"\n";
        let error = parse_gate(source, "shard.lino").unwrap_err();
        assert!(error.contains("must cite an issue/PR"), "{error}");
    }

    #[test]
    fn a_justification_with_an_issue_citation_is_accepted() {
        let source = "ci_gate check_formatting\n  stage rust\n  \
            description \"rustfmt owns the layout.\"\n  \
            run \"cargo fmt\"\n  \
            justification \"prevents formatting drift, introduced in #991\"\n";
        assert!(parse_gate(source, "shard.lino").is_ok());
    }

    #[test]
    fn a_justification_with_a_commit_hash_is_accepted() {
        let source = "ci_gate check_formatting\n  stage rust\n  \
            description \"rustfmt owns the layout.\"\n  \
            run \"cargo fmt\"\n  \
            justification \"prevents formatting drift, commit 2b656e5cc\"\n";
        assert!(parse_gate(source, "shard.lino").is_ok());
    }

    #[test]
    fn justification_citation_detection() {
        assert!(justification_has_citation("introduced in #991"));
        assert!(justification_has_citation("commit 2b656e5cc"));
        assert!(justification_has_citation(
            "fix for #1081 via commit abcdef1234567"
        ));
        // A URL slug with hex chars but fewer than 7 consecutive is not a commit.
        assert!(!justification_has_citation("plan-09 leaf-6"));
        // Plain prose without a citation is rejected.
        assert!(!justification_has_citation("prevents formatting drift"));
        // Exactly 7 hex chars is a valid short hash.
        assert!(justification_has_citation("abc1234"));
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
        assert_eq!(
            registry_problems(&[gate()], &source),
            [] as [std::string::String; 0]
        );
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
        assert_eq!(
            registry_problems(&[gate()], &source),
            [] as [std::string::String; 0]
        );
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
    fn a_gate_runs_in_the_default_lane_unless_it_names_one() {
        assert_eq!(gate().lane, DEFAULT_LANE);
        let source = format!("{SHARD}  lane 3\n");
        let laned = parse_gate(&source, "data/meta/ci-gates/check-formatting.lino").unwrap();
        assert_eq!(laned.lane, 3);
        for bad in ["lane 0", "lane two"] {
            let error = parse_gate(&format!("{SHARD}  {bad}\n"), "shard.lino").unwrap_err();
            assert!(error.contains("`lane` must be a number"), "{error}");
        }
    }

    /// A `lint` job split into lanes, plus a later release job.
    fn laned_workflow(lanes: &str, lint_steps: &str) -> String {
        format!(
            "jobs:\n  lint:\n    strategy:\n      matrix:\n        lane: [{lanes}]\n    \
             steps:\n{lint_steps}  version-check:\n    steps:\n"
        )
    }

    #[test]
    fn every_lane_the_gates_name_is_one_the_matrix_runs() {
        let steps = "      - run: rust-script scripts/run-ci-gates.rs --stage rust\n";
        let mut second = gate();
        second.name = "check_other".to_string();
        second.lane = 2;
        assert_eq!(
            registry_problems(&[gate(), second.clone()], &laned_workflow("1, 2", steps)),
            [] as [std::string::String; 0]
        );

        // A gate in a lane no leg runs is a gate nobody runs.
        let problems = registry_problems(&[gate(), second.clone()], &laned_workflow("1", steps));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("nothing would run it")),
            "{problems:?}"
        );
        // Without a lane matrix there is only the default lane.
        let problems = registry_problems(&[gate(), second], &workflow(steps, ""));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("nothing would run it")),
            "{problems:?}"
        );
    }

    #[test]
    fn a_lane_without_gates_and_a_laned_web_gate_are_rejected() {
        let steps = "      - run: rust-script scripts/run-ci-gates.rs --stage rust\n      \
                     - run: rust-script scripts/run-ci-gates.rs --stage web\n";
        let problems = registry_problems(&[gate()], &laned_workflow("1, 2", steps));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("no gate is assigned")),
            "{problems:?}"
        );

        let mut web = gate();
        web.name = "check_web".to_string();
        web.stage = "web".to_string();
        web.lane = 2;
        let problems = registry_problems(&[gate(), web], &laned_workflow("1, 2", steps));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("only lane 1 runs")),
            "{problems:?}"
        );
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
