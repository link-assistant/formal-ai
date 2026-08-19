//! Regression coverage for the upstream-version pin that issue #1021 needed.
//!
//! The `E2E Tests (agent CLI <-> formal-ai)` job went red on a branch that
//! changed nothing in the Codex startup path: `@openai/codex@0.148.0` was
//! published overnight and drops the ENTER that answers its first-run trust
//! dialog (<https://github.com/openai/codex/issues/39487>), so the TUI leg hung
//! before a single request reached the server under test.
//!
//! `experiments/agentic_cli_matrix/clients.lock` already states the rule that
//! would have prevented it -- "a matrix leg fails because our server changed,
//! not because an upstream CLI shipped overnight" -- but `release.yml` was
//! only following it for the one package `--trust` forced someone to name.
//! These tests hold the rule for every third-party CLI CI installs, so the next
//! floating install is caught at review time rather than by a red job.

use super::workflow_fixtures::release_workflow;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// A scratch directory for one test's stand-in binaries.
fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after 1970")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("issue-1021-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("create the scratch directory");
    path
}

/// Packages the project publishes itself. Tracking these at latest is the
/// point -- an E2E leg that pinned our own client would stop reporting whether
/// today's client still works against today's server.
const OWN_SCOPE: &str = "@link-assistant/";

/// Every workflow in the repository, read as `(file name, contents)`. Reading
/// the directory rather than a list is the point: a rule that only holds for
/// the workflows installing a CLI today is a rule the next one escapes.
fn workflows() -> Vec<(String, String)> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows");
    let mut found: Vec<(String, String)> = fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("a readable directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yml" || extension == "yaml")
        })
        .map(|path| {
            let contents = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
                .replace("\r\n", "\n");
            (
                path.file_name()
                    .expect("a file inside the workflow directory")
                    .to_string_lossy()
                    .into_owned(),
                contents,
            )
        })
        .collect();
    found.sort();
    assert!(
        !found.is_empty(),
        "no workflows found under {}",
        directory.display()
    );
    found
}

/// Split a `bun add -g` command into the package specs it installs, dropping
/// the flags. `bun add -g --ignore-scripts a@1 b@2` yields `["a@1", "b@2"]`.
fn installed_specs(line: &str) -> Vec<&str> {
    line.trim()
        .trim_start_matches("run:")
        .trim()
        .strip_prefix("bun add -g")
        .expect("caller filtered for `bun add -g`")
        .split_whitespace()
        .filter(|argument| !argument.starts_with('-'))
        .collect()
}

/// A version is explicit when the spec carries an `@<version>` after the
/// package name: `@openai/codex@0.147.0` and `opencode-ai@1.18.4` are pinned,
/// `@openai/codex` and `opencode-ai` are not.
fn is_pinned(spec: &str) -> bool {
    let name = spec.strip_prefix('@').unwrap_or(spec);
    name.contains('@')
}

#[test]
fn every_third_party_cli_a_workflow_installs_globally_carries_an_explicit_version() {
    let mut checked = Vec::new();
    for (name, workflow) in workflows() {
        for line in workflow.lines().filter(|line| {
            line.trim()
                .trim_start_matches("run:")
                .trim()
                .starts_with("bun add -g")
        }) {
            for spec in installed_specs(line) {
                if spec.starts_with(OWN_SCOPE) {
                    continue;
                }
                assert!(
                    is_pinned(spec),
                    "{name} installs {spec} without a version; \
                     pin it the way experiments/agentic_cli_matrix/clients.lock requires"
                );
                checked.push(spec.to_string());
            }
        }
    }
    checked.sort();
    assert_eq!(
        checked,
        vec![
            "@anthropic-ai/claude-code@2.1.234".to_string(),
            "@google/gemini-cli@0.55.1".to_string(),
            "@openai/codex@0.147.0".to_string(),
            "opencode-ai@1.18.4".to_string(),
        ],
        "the set of third-party CLIs CI installs changed; \
         re-run experiments/issue_1021_codex_tui_version/run.sh for the new pin"
    );
}

#[test]
fn the_codex_pin_names_the_upstream_defect_and_the_bisect_that_would_lift_it() {
    let workflow = release_workflow();
    let step = workflow
        .split("- name: Install external agent CLIs")
        .nth(1)
        .and_then(|tail| tail.split("- name:").next())
        .expect("the external agent CLI install step");

    assert!(step.contains("https://github.com/openai/codex/issues/39487"));
    assert!(step.contains("experiments/issue_1021_codex_tui_version"));
    assert!(step.contains("@openai/codex@0.147.0"));
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/issue_1021_codex_tui_version/run.sh")
            .is_file(),
        "the bisect the comment points at must exist"
    );
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/issue_1021_codex_tui_version/codex_trust_dialog_probe.py")
            .is_file(),
        "the wrapper-free reproduction the upstream report cites must exist"
    );
}

/// Evidence a case study cites has to be in the repository, not merely in the
/// working copy that produced it. `.gitignore`'s `logs/` rule swallowed
/// `docs/case-studies/issue-1021/logs/`, and the `!docs/case-studies/**/*.log`
/// re-include could not undo it, because git never descends into an excluded
/// directory. `git add` reported success, committed nothing, and the test that
/// asserts the probe output exists went on passing for the author while failing
/// for every checkout that was not theirs. Only git knows the difference, so
/// this asks git.
#[test]
fn no_case_study_evidence_is_hidden_from_the_repository_by_gitignore() {
    let hidden = Command::new("git")
        .args([
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--",
            "docs/case-studies",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run git ls-files");

    assert!(hidden.status.success(), "git ls-files should succeed");
    let hidden = String::from_utf8_lossy(&hidden.stdout);
    assert!(
        hidden.trim().is_empty(),
        "these case-study files exist on disk but are ignored, so `git add` \
         will report success and commit nothing -- re-include their directory \
         in .gitignore the way `!docs/case-studies/**/logs/` does:\n{hidden}"
    );
}

/// Run `scripts/apt-install-with-retry.sh` against a stand-in `apt-get`, so the
/// retry can be exercised without a package mirror, a network or root.
///
/// `failures` is how many attempts the stand-in refuses before it succeeds;
/// `mode` is `"exit"` for a mirror that answers with an error and `"stall"` for
/// one that answers with nothing, which is the shape run 32272689026 hit.
fn run_apt_retry(failures: u32, mode: &str, attempts: u32) -> std::process::Output {
    let directory = temp_dir("apt-retry");
    let stand_in = directory.join("apt-get");
    fs::write(
        &stand_in,
        "#!/usr/bin/env bash\n\
         printf '%s\\n' \"$*\" >> \"$FAKE_APT_CALLS\"\n\
         if [ \"$(grep -c ' update -q$' \"$FAKE_APT_CALLS\")\" -le \"$FAKE_APT_FAILURES\" ]; then\n\
         \x20 case \"$FAKE_APT_MODE\" in\n\
         \x20   stall) sleep 300 ;;\n\
         \x20   *) echo 'E: Could not resolve host: archive.ubuntu.com' >&2; exit 100 ;;\n\
         \x20 esac\n\
         fi\n\
         exit 0\n",
    )
    .expect("write the stand-in apt-get");
    fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o755))
        .expect("make the stand-in executable");
    let calls = directory.join("calls");
    fs::write(&calls, "").expect("create the call log");

    Command::new("bash")
        .arg(format!(
            "{}/scripts/apt-install-with-retry.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("xvfb")
        // Already unprivileged here; the stand-in needs no escalation.
        .env("FORMAL_AI_APT_PRIVILEGE", "")
        .env("FORMAL_AI_APT_GET", &stand_in)
        .env("FAKE_APT_CALLS", &calls)
        .env("FAKE_APT_FAILURES", failures.to_string())
        .env("FAKE_APT_MODE", mode)
        .env("FORMAL_AI_APT_ATTEMPTS", attempts.to_string())
        .env("FORMAL_AI_APT_ATTEMPT_SECONDS", "3")
        .env("FORMAL_AI_APT_RETRY_DELAY_SECONDS", "0")
        .output()
        .expect("run the apt retry wrapper")
}

/// A stalled mirror is killed at its own deadline and the next attempt runs.
///
/// This is the failure exactly: in run 32272689026 `E2E (opencode-desktop)`
/// spent its whole 300s budget inside one `apt-get`, while `opencode-vscode`
/// and `cursor` installed the same package in 52s from the same commit. A
/// deadline that only reports the stall leaves the pipeline red for something
/// no commit in it caused.
#[test]
fn a_stalled_mirror_is_killed_at_its_own_deadline_and_the_next_attempt_succeeds() {
    let output = run_apt_retry(1, "stall", 2);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "a transient stall must not fail the step: {stdout}{stderr}"
    );
    // 124 is `timeout`'s own status: the attempt was killed, not answered.
    assert!(
        stderr.contains("Attempt 1 exited 124 after 3s of its 3s deadline"),
        "the log must name the killed attempt and its deadline: {stderr}"
    );
    assert!(
        stdout.contains("apt install of xvfb succeeded on attempt 2/2"),
        "the surviving attempt must say which one it was: {stdout}"
    );
}

/// Retrying a *real* failure forever would hide it, so the attempts are counted
/// and the last status is the one the step exits with -- apt's own 100 here,
/// not the wrapper's.
#[test]
fn a_mirror_that_keeps_refusing_is_reported_after_the_last_attempt() {
    let output = run_apt_retry(9, "exit", 3);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(100));
    assert_eq!(stderr.matches("failed attempt").count(), 3);
    assert!(
        stderr.contains("::error title=apt install of xvfb failed every attempt::"),
        "the last attempt must annotate the job, not just exit: {stderr}"
    );
}

/// A retry is only an improvement while it can finish -- the rule
/// `desktop/scripts/package-macos-with-retry.sh` learned on the macOS runners.
/// Attempts that outlive the step budget would convert a transient stall into a
/// terminated step, which is the failure being fixed, so the wrapper refuses to
/// start and says which number to change.
#[test]
fn attempts_that_cannot_fit_the_step_budget_are_refused_before_the_first_one() {
    let output = Command::new("bash")
        .arg(format!(
            "{}/scripts/apt-install-with-retry.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("xvfb")
        .env("TEST_BUDGET_SECONDS", "100")
        .env("FORMAL_AI_APT_ATTEMPTS", "3")
        .env("FORMAL_AI_APT_ATTEMPT_SECONDS", "90")
        .env("FORMAL_AI_APT_RETRY_DELAY_SECONDS", "5")
        .output()
        .expect("run the apt retry wrapper");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr.contains("3 attempts of 90s plus 5s delays need 280s, but the step budget is 100s"),
        "the refusal must show the arithmetic it refused on: {stderr}"
    );
}

/// The guard above only helps while the two numbers meet. They are set in
/// different places -- the step's `env:` block and the wrapper's defaults -- so
/// a workflow could compose a retry whose worst case exceeds the budget above
/// it and only find out when a mirror stalls, which is once every few dozen
/// runs. Check the arithmetic at review time instead, for every step that
/// composes them, not only for the one that has one today.
#[test]
fn every_budgeted_retry_in_a_workflow_fits_the_budget_it_runs_under() {
    let mut checked = 0_usize;
    for (name, workflow) in workflows() {
        for step in workflow.split("\n      - name: ") {
            if !step.contains("scripts/apt-install-with-retry.sh") {
                continue;
            }
            checked += 1;
            let step_name = step.lines().next().unwrap_or_default();
            let setting = |key: &str| -> u64 {
                step.lines()
                    .find_map(|line| line.trim().strip_prefix(&format!("{key}:")))
                    .unwrap_or_else(|| {
                        panic!("{name}: step `{step_name}` retries apt but sets no {key}")
                    })
                    .trim()
                    .parse()
                    .expect("a whole number of seconds")
            };

            let budget = setting("TEST_BUDGET_SECONDS");
            let attempts = setting("FORMAL_AI_APT_ATTEMPTS");
            let attempt_seconds = setting("FORMAL_AI_APT_ATTEMPT_SECONDS");
            let delay = setting("FORMAL_AI_APT_RETRY_DELAY_SECONDS");
            let worst_case = attempts * attempt_seconds + (attempts - 1) * delay;
            assert!(
                worst_case <= budget,
                "{name}: step `{step_name}` needs {worst_case}s in the worst case \
                 ({attempts} attempts of {attempt_seconds}s plus {delay}s delays) \
                 but is budgeted {budget}s, so the last attempt would be \
                 terminated rather than answered"
            );
        }
    }
    assert_eq!(
        checked, 1,
        "expected the agentic matrix's Xvfb install to be the one budgeted retry"
    );
}
