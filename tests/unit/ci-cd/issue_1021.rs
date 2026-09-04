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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
/// package name: `@openai/codex@0.147.0` and `opencode-ai@1.18.25` are pinned,
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
            "@anthropic-ai/claude-code@2.1.252".to_string(),
            "@google/gemini-cli@0.57.0".to_string(),
            "@openai/codex@0.147.0".to_string(),
            "opencode-ai@1.18.25".to_string(),
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
    let started = SystemTime::now();
    let output = run_apt_retry(1, "stall", 2);
    let elapsed = started.elapsed().expect("read the clock");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "a transient stall must not fail the step: {stdout}{stderr}"
    );
    // 124 is the deadline's own status: the attempt was killed, not answered.
    assert!(
        stderr.contains("Attempt 1 exited 124 after") && stderr.contains("of its 3s deadline"),
        "the log must name the killed attempt and its deadline: {stderr}"
    );
    // The claim is that the 300s stall was cut at the 3s deadline, so check the
    // clock, not the wording. The reported figure is whole seconds either side
    // of a poll interval, and pinning it to exactly `3s` is how this assertion
    // failed on a deadline that had done its job in 3.6s -- a test that reads a
    // rounding is not reading the behaviour.
    let reported: u64 = stderr
        .split("Attempt 1 exited 124 after ")
        .nth(1)
        .and_then(|rest| rest.split('s').next())
        .and_then(|seconds| seconds.parse().ok())
        .unwrap_or_else(|| panic!("the attempt must report how long it ran: {stderr}"));
    assert!(
        (3..=6).contains(&reported),
        "the attempt must be killed at its 3s deadline, not before and not much \
         after; it reported {reported}s: {stderr}"
    );
    assert!(
        elapsed < Duration::from_secs(45),
        "two 3s attempts against a 300s stall must not take {elapsed:?}"
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

/// Whether a line lists a path for a trigger filter rather than running it.
///
/// A `paths:`/`paths-ignore:` entry is a bare YAML list item — `- 'x.sh'` — with
/// no command around it. A step that runs the script always reaches it through a
/// `run:` scalar or a `run: >-` continuation, neither of which is a bare item.
fn is_path_filter_entry(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(item) = trimmed.strip_prefix("- ") else {
        return false;
    };
    let item = item.trim().trim_matches(['\'', '"']);
    // A filter entry is exactly one path; a command has arguments or a wrapper.
    !item.contains(char::is_whitespace)
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
            // A step *runs* the wrapper; a `paths:` filter merely names it, and
            // splitting on the step delimiter leaves the whole workflow header
            // as one segment, so a trigger listing the script as a path used to
            // read here as a step that had lost its budget (issue #1064). Only
            // a command line counts.
            if !step.lines().any(|line| {
                line.contains("scripts/apt-install-with-retry.sh") && !is_path_filter_entry(line)
            }) {
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

/// The wrapper's per-attempt deadline was GNU `timeout` until the macOS core
/// slices ran these tests: macOS ships no `timeout`, so every attempt exited
/// 127 and the two tests above failed on a script whose own job was green
/// (run 32282461075, jobs 96170638546 and 96170638704). The replacement lives
/// in `scripts/run-with-deadline.sh` and has to keep `timeout`'s contract --
/// 124 when the deadline expires, the command's own status otherwise -- because
/// the wrapper's log tells a stalled mirror from apt's own failure by that
/// number.
#[test]
fn the_deadline_exits_124_and_kills_the_whole_stalled_tree() {
    let directory = temp_dir("deadline-stall");
    let pid_file = directory.join("descendant-pid");
    let started = std::time::Instant::now();
    let output = Command::new("bash")
        .arg(format!(
            "{}/scripts/run-with-deadline.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("1")
        // The stall is in a *child* of the command, which is where `apt-get`
        // spends one: a deadline that signals only the root leaves it running.
        .arg("bash")
        .arg("-c")
        .arg(format!(
            "sleep 120 & echo $! > {}; wait",
            pid_file.display()
        ))
        .output()
        .expect("run the deadline wrapper");
    let elapsed = started.elapsed();

    assert_eq!(
        output.status.code(),
        Some(124),
        "an expired deadline reports `timeout`'s own status: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        elapsed < Duration::from_secs(30),
        "the deadline must expire long before the command would finish, took {elapsed:?}"
    );
    let descendant = fs::read_to_string(&pid_file).expect("the command recorded its child");
    let descendant = descendant.trim();
    assert!(
        !process_is_running(descendant),
        "the stalled child {descendant} outlived the deadline, so the signal \
         reached only the root of the tree"
    );
}

/// A deadline that expires early is a worse defect than one that expires late:
/// it converts work that was going to finish into a failure, which is the shape
/// of flake this whole retry exists to remove.
///
/// This is not hypothetical. The first draft read elapsed time from bash's
/// `SECONDS`, which is a difference of whole-second clock readings, and killed a
/// 3s deadline after 2.6s (`experiments/issue-1021-deadline-precision`). Nothing
/// in the suite noticed, because every other assertion here is an upper bound.
#[test]
fn the_deadline_never_expires_before_the_time_it_was_given() {
    let started = std::time::Instant::now();
    let output = Command::new("bash")
        .arg(format!(
            "{}/scripts/run-with-deadline.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("3")
        .arg("bash")
        .arg("-c")
        .arg("sleep 120")
        .output()
        .expect("run the deadline wrapper");
    let elapsed = started.elapsed();

    assert_eq!(
        output.status.code(),
        Some(124),
        "the stall must still be killed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        elapsed >= Duration::from_secs(3),
        "a 3s deadline expired after {elapsed:?}, so a command with 3s of work \
         to do would have been killed doing it"
    );
    // Late is allowed, but only by the polling and the fork it costs.
    assert!(
        elapsed < Duration::from_secs(15),
        "a 3s deadline took {elapsed:?}, which is no longer a 3s deadline"
    );
}

/// A command that answers inside its deadline is untouched by it -- including
/// its exit status, which is how `apt-install-with-retry.sh` reports a mirror
/// that is refusing rather than stalling.
#[test]
fn a_command_that_beats_its_deadline_keeps_its_own_status() {
    for expected in [0, 100] {
        let output = Command::new("bash")
            .arg(format!(
                "{}/scripts/run-with-deadline.sh",
                env!("CARGO_MANIFEST_DIR")
            ))
            .arg("30")
            .arg("bash")
            .arg("-c")
            .arg(format!("exit {expected}"))
            .output()
            .expect("run the deadline wrapper");
        assert_eq!(output.status.code(), Some(expected));
    }
}

/// A stand-in `sccache` that answers `--show-stats` with one recognisable
/// counter, so the assertions below test the wrapper's reporting rather than
/// sccache's own output format.
fn stub_sccache(directory: &Path) -> PathBuf {
    let path = directory.join("sccache");
    fs::write(
        &path,
        "#!/usr/bin/env bash\nprintf 'Compile requests 41\\n'\n",
    )
    .expect("write the stand-in sccache");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
        .expect("make the stand-in sccache executable");
    path
}

/// The budget wrapper is what reports that a Rust step ran out of time, and
/// until issue #1021 it reported only the number of seconds. That is not enough
/// to act on: `Test (macos-15-intel / specification)` blew its 1200s budget
/// still compiling dependencies, and the two candidate causes -- a slower
/// runner and a colder compiler cache -- leave the same trace in the log.
/// Deciding between them took reconstructing per-crate rustc timestamps out of
/// a megabyte of raw job log. sccache counts the answer already, so the step
/// that announces the failure asks it.
#[test]
fn a_budget_that_expires_reports_the_compiler_cache_counters() {
    let directory = temp_dir("budget-cache-terminated");
    let sccache = stub_sccache(&directory);
    let output = Command::new("bash")
        .arg(format!(
            "{}/scripts/run-with-budget-warning.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("1")
        .arg("Specification test shard")
        .arg("bash")
        .arg("-c")
        .arg("sleep 120")
        .env("RUSTC_WRAPPER", "sccache")
        .env("SCCACHE_PATH", &sccache)
        .output()
        .expect("run the budget wrapper");

    assert_eq!(
        output.status.code(),
        Some(124),
        "the budget must still terminate the command: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[budget] Compile requests 41"),
        "a terminated Rust step must arrive with the cache counters that say \
         whether the compiler cache was answering, got:\n{stderr}"
    );
}

/// The same counters, one step earlier. The warning fires while the command can
/// still be acted on, which is the moment the counters are worth reading; and
/// a command that then finishes keeps its own exit status, because a diagnostic
/// that changed the result would be a defect rather than a diagnosis.
#[test]
fn a_budget_warning_reports_the_counters_without_touching_the_result() {
    let directory = temp_dir("budget-cache-warning");
    let sccache = stub_sccache(&directory);
    let output = Command::new("bash")
        .arg(format!(
            "{}/scripts/run-with-budget-warning.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("8")
        .arg("Specification test shard")
        .arg("bash")
        .arg("-c")
        .arg("sleep 3")
        .env("TEST_WARN_RATIO_PERCENT", "25")
        .env("RUSTC_WRAPPER", "sccache")
        .env("SCCACHE_PATH", &sccache)
        .output()
        .expect("run the budget wrapper");

    assert_eq!(
        output.status.code(),
        Some(0),
        "the command beat its budget and keeps its own status: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("is approaching its timeout"),
        "the warning must fire at 25% of an 8s budget, got:\n{stderr}"
    );
    assert!(
        stderr.contains("[budget] Compile requests 41"),
        "the warning is the actionable moment, so it carries the counters too, \
         got:\n{stderr}"
    );
}

/// Not every budgeted step compiles Rust -- the Xvfb install of finding 14 is
/// one that does not -- so the counters are asked for only where they mean
/// something. A step with no sccache wrapper must stay exactly as quiet as it
/// was before, or the wrapper trades one unreadable log for another.
#[test]
fn a_budget_that_wraps_no_compiler_reports_no_counters() {
    let directory = temp_dir("budget-cache-absent");
    let sccache = stub_sccache(&directory);
    let output = Command::new("bash")
        .arg(format!(
            "{}/scripts/run-with-budget-warning.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("1")
        .arg("Install Xvfb")
        .arg("bash")
        .arg("-c")
        .arg("sleep 120")
        // The stand-in is reachable; what is missing is any claim that this
        // step compiles through it.
        .env("SCCACHE_PATH", &sccache)
        .env_remove("RUSTC_WRAPPER")
        .output()
        .expect("run the budget wrapper");

    assert_eq!(output.status.code(), Some(124));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Compile requests"),
        "a step that compiles nothing must not report compiler counters, \
         got:\n{stderr}"
    );
}

/// Is this pid still a running process? `kill -0` says yes for a terminated
/// process nobody has reaped yet, and a killed descendant is reparented to a
/// PID 1 that may never reap it, so the state column is read instead: `Z` is a
/// process that has already terminated.
fn process_is_running(pid: &str) -> bool {
    let observation = Command::new("ps")
        .args(["-o", "state=", "-p", pid])
        .output()
        .expect("ps is available");
    if !observation.status.success() {
        return false;
    }
    let state = String::from_utf8_lossy(&observation.stdout);
    let state = state.trim();
    !state.is_empty() && !state.starts_with('Z')
}

/// The defect generalized: `timeout` is a GNU coreutils binary, and half this
/// repository's CI runs on macOS, which does not ship it. One script reaching
/// for it passed review because the job it ships in is Linux; the tests that
/// drive that script are not. So the rule is held for every committed script
/// and workflow rather than for the one that was caught, the way the version
/// pin above is held for every third-party CLI.
#[test]
fn no_committed_script_reaches_for_a_timeout_binary_macos_does_not_have() {
    let tracked = Command::new("git")
        .args(["ls-files", "*.sh", ".github/workflows/*.yml"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run git ls-files");
    assert!(tracked.status.success(), "git ls-files should succeed");
    let tracked = String::from_utf8_lossy(&tracked.stdout);
    let mut checked = 0_usize;

    for path in tracked.lines() {
        if path.starts_with("dev/log/")
            || path.starts_with("docs/case-studies/")
            || path.starts_with("experiments/")
        {
            continue;
        }
        checked += 1;
        let contents =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect(path);
        for (number, line) in contents.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with('#') {
                continue;
            }
            let tokens: Vec<&str> = code.split_whitespace().collect();
            for (index, token) in tokens.iter().enumerate() {
                // The word alone is not the defect -- prose says "timeout" too.
                // What matters is `timeout` standing where a command starts,
                // whatever follows it: the first version of this guard asked
                // what came next instead, and a quoted
                // `timeout "$attempt_seconds"` walked straight through it.
                let starts_a_command = index == 0
                    || matches!(
                        tokens[index - 1],
                        "|" | "||"
                            | "&&"
                            | ";"
                            | "("
                            | "{"
                            | "-"
                            | "!"
                            | "sudo"
                            | "then"
                            | "do"
                            | "else"
                            | "exec"
                            | "command"
                            | "env"
                            | "run:"
                    );
                assert!(
                    *token != "timeout" || !starts_a_command,
                    "{path}:{}: `timeout` is GNU coreutils and the macOS runners \
                     do not have it -- use scripts/run-with-deadline.sh, which \
                     keeps the same 124-on-expiry contract",
                    number + 1
                );
            }
        }
    }
    assert!(
        checked > 30,
        "expected the tracked scripts and workflows to be read, saw {checked}"
    );
}

/// The crate is on edition 2024, and says so in the one place `cargo` reads.
///
/// The edition is not a preference here: `build.rs` exports it as
/// `FORMAL_AI_CRATE_EDITION`, and `src/memory_revision.rs` passes that to the
/// `rustc` it spawns to judge a self-authored version. A manifest that drifts
/// back would silently have the judge compile a different language than the
/// crate is written in -- which is the defect finding 26 records.
#[test]
fn the_crate_is_on_edition_2024_and_the_judge_compiles_the_same_edition() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read Cargo.toml");
    let declared = manifest
        .lines()
        .find_map(|line| line.trim().strip_prefix("edition = "))
        .expect("Cargo.toml should declare an edition")
        .trim()
        .trim_matches('"')
        .to_owned();
    assert_eq!(declared, "2024", "the crate should be on edition 2024");
    assert_eq!(
        env!("FORMAL_AI_CRATE_EDITION"),
        declared,
        "build.rs should export the edition the manifest declares, so the \
         spawned rustc compiles the language this crate is written in"
    );
    assert!(
        manifest
            .lines()
            .any(|line| line.trim().starts_with("rust-version = ")),
        "an edition floor without a rust-version leaves the compiler it needs unstated"
    );
}

/// Edition 2024 does not need nightly, and neither should anything this
/// repository depends on or runs.
///
/// The rule was stated on the pull request and had no gate behind it, so it was
/// true only for as long as nobody reached. The four reaches are enumerated
/// rather than the one that might be tried: a toolchain file, a toolchain
/// action asking for anything but stable, a per-invocation toolchain override,
/// an environment variable that opts a stable compiler into unstable features,
/// and an unstable feature attribute in the crate itself.
///
/// The needles are assembled at run time rather than written out, and the
/// sentence above names them rather than spelling them, because a guard whose
/// own source contains what it forbids has to exempt itself -- and an exemption
/// is exactly the hole the next reach goes through.
#[test]
fn nothing_in_the_tree_reaches_for_a_nightly_toolchain() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let tracked = Command::new("git")
        .args(["ls-files"])
        .current_dir(root)
        .output()
        .expect("run git ls-files");
    assert!(tracked.status.success(), "git ls-files should succeed");
    let tracked = String::from_utf8_lossy(&tracked.stdout);

    // Built from pieces: see the doc comment.
    let feature_attribute = format!("#![{}(", "feature");
    let plus_nightly = format!("+{}", "nightly");
    let bootstrap = format!("RUSTC_{}", "BOOTSTRAP");
    let toolchain_action = format!("dtolnay/rust-{}@", "toolchain");
    let stable_action = format!("{toolchain_action}stable");

    let mut toolchain_actions = 0_usize;
    let mut scanned = 0_usize;

    for path in tracked.lines() {
        assert!(
            !path.starts_with("rust-toolchain"),
            "{path}: a toolchain file pins the whole repository off stable"
        );

        // Archived logs and case-study evidence record what happened; they are
        // not instructions to anything, and rewriting them would be forgery.
        if path.starts_with("dev/log/") || path.starts_with("docs/case-studies/") {
            continue;
        }
        let interesting = path.starts_with("src/")
            || path.starts_with("tests/")
            || path.starts_with("scripts/")
            || path.starts_with(".github/")
            || path.ends_with("Cargo.toml")
            || Path::new(path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("sh"));
        if !interesting {
            continue;
        }
        let Ok(contents) = fs::read_to_string(root.join(path)) else {
            continue; // A binary fixture is not a toolchain request.
        };
        scanned += 1;

        for (number, line) in contents.lines().enumerate() {
            let at = number + 1;
            if let Some(rest) = line.split(&toolchain_action).nth(1) {
                toolchain_actions += 1;
                assert!(
                    line.contains(&stable_action),
                    "{path}:{at}: the toolchain action should ask for @stable, not @{}",
                    rest.split_whitespace().next().unwrap_or(rest)
                );
            }
            assert!(
                !line.contains(&plus_nightly),
                // The needle is interpolated rather than typed, for the same
                // reason it is built from pieces above: this file is scanned
                // too, and a literal here would fail on its own message.
                "{path}:{at}: a {plus_nightly} invocation leaves stable Rust behind"
            );
            assert!(
                !line.contains(&bootstrap),
                "{path}:{at}: this opts a stable compiler into unstable features"
            );
            if path.starts_with("src/") || path.starts_with("tests/") {
                assert!(
                    !line.trim_start().starts_with(&feature_attribute),
                    "{path}:{at}: an unstable feature attribute needs a nightly compiler"
                );
            }
        }
    }

    assert!(
        scanned > 100,
        "the scan should reach the tree, and it reached {scanned} files"
    );
    assert!(
        toolchain_actions >= 20,
        "every CI job that compiles Rust installs a toolchain, and only \
         {toolchain_actions} installs were found -- the scan is missing them"
    );
}
