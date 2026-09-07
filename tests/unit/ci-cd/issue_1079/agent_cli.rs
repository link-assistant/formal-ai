//! Issue #1079, D11 and D12: the two Agent CLI end-to-end jobs.
//!
//! Both failed on a *session title*. `@link-assistant/agent` summarizes every
//! session by default, and `--compaction-model same` -- the flag every harness
//! here passed to keep that summary on the session's own model -- is silently
//! ignored, so the summarizer reached the hosted `opencode/big-pickle` and its
//! rejection, neither awaited nor caught, aborted the running turn. The job
//! log then said `Process completed with exit code 1.` and nothing else, so
//! the failure could not explain itself either.
//!
//! These tests pin the fix at every place that launches the client, and pin
//! the diagnostic that makes the next such failure readable from the log.

use std::fs;
use std::path::Path;

use super::github_yaml_files;

/// Every `experiments/**.sh` path a workflow names, in the order they appear.
///
/// The reference is textual on purpose: a workflow may name a harness in a
/// `run:` block, in an `if:` guard or inside a heredoc, and a sweep that only
/// understood one of those spellings would silently measure less than it
/// claims.
fn experiment_script_references(body: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut start = 0;
    while let Some(index) = body[start..].find("experiments/") {
        let begin = start + index;
        let end = body[begin..]
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '`' || c == ')')
            .map_or(body.len(), |offset| begin + offset);
        if Path::new(&body[begin..end])
            .extension()
            .is_some_and(|extension| extension == "sh")
        {
            found.push(body[begin..end].to_string());
        }
        start = end.max(begin + 1);
    }
    found
}

/// The sibling libraries a script `source`s, resolved against its own
/// directory.
///
/// `experiments/agentic_cli_matrix/run_leg.sh` reaches the Agent CLI through
/// `lib.sh`, so a sweep that stopped at the file the workflow names would
/// declare that harness clean without ever reading the line that launches the
/// client.
fn sourced_siblings(path: &str, body: &str) -> Vec<String> {
    let Some((directory, _)) = path.rsplit_once('/') else {
        return Vec::new();
    };
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed
                .strip_prefix("source ")
                .or_else(|| trimmed.strip_prefix(". "))?;
            let token = rest.split_whitespace().next()?.trim_matches('"');
            let name = token.rsplit('/').next()?;
            Path::new(name)
                .extension()
                .is_some_and(|extension| extension == "sh")
                .then(|| format!("{directory}/{name}"))
        })
        .collect()
}

/// Every shell script CI reaches, whether a workflow names it directly or
/// another script it names does.
fn shell_scripts_ci_runs() -> Vec<(String, String)> {
    let mut queue: Vec<String> = Vec::new();
    for path in github_yaml_files() {
        let body = fs::read_to_string(&path).expect("readable workflow");
        queue.extend(experiment_script_references(&body));
    }

    let mut seen: Vec<String> = Vec::new();
    let mut scripts: Vec<(String, String)> = Vec::new();
    while let Some(path) = queue.pop() {
        if seen.contains(&path) {
            continue;
        }
        seen.push(path.clone());
        let Ok(body) = fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))) else {
            continue;
        };
        let body = body.replace("\r\n", "\n");
        queue.extend(experiment_script_references(&body));
        queue.extend(sourced_siblings(&path, &body));
        scripts.push((path, body));
    }
    scripts.sort();
    scripts
}

/// Issue #1079, D11. `Proactive failure report Agent CLI E2E` failed twice in a
/// row on this branch with a provider error from a service the harness never
/// configured:
///
/// ```text
/// Error from provider (Console): Request is missing x-opencode-session and cannot be routed efficiently
/// ```
///
/// The harness points the Agent CLI at a local `formal-ai serve`, and that is
/// the only provider in its config. But `--summarize-session` defaults to true
/// and the summarizer does not use the session's model: it loads the head of
/// the default compaction cascade, `opencode/big-pickle`, over the hosted
/// gateway. `agent-stream.raw.log` records the decision --
/// `"service":"session.summary","providerID":"opencode","modelID":"big-pickle"`
/// -- and the rejection arrives as an `UnhandledRejection` on stderr, which
/// `scripts/classify-agent-cli-stderr.sh` refuses to hide.
///
/// This repository already knew: commit f9cee7b68, "fix(ci): disable hosted
/// Agent summarization", added `--no-summarize-session` in July 2026 -- to one
/// harness, pinned by a gate that reads that one file. Fourteen of the
/// twenty-five harnesses CI runs never got it.
///
/// The compaction half is spelled `--compaction-models "(same)"` rather than
/// `--compaction-model same`, because the singular flag is silently ignored:
/// `src/cli/model-config.js` takes the cascade branch whenever
/// `argv['compaction-models']` is set, and yargs always sets it to the default
/// cascade. Reproduced against `@link-assistant/agent` 0.26.0: passing
/// `--compaction-model same` still logs
/// `models: ["opencode/big-pickle", "kilo/minimax-m2.5-free", "same"],
/// source: "default"`, while `--compaction-models "(same)"` logs
/// `models: ["same"], source: "cli"`. Filed upstream; the report is in
/// `dev/log/issues/1079/pulls/1080/upstream-reports/`.
#[test]
fn every_agent_cli_harness_ci_runs_keeps_the_session_local() {
    let harnesses: Vec<(String, String)> = shell_scripts_ci_runs()
        .into_iter()
        // `--disable-stdin` is an Agent CLI flag; no other client in these
        // harnesses accepts it.
        .filter(|(_, body)| body.contains("--disable-stdin"))
        .collect();

    assert!(
        harnesses
            .iter()
            .any(|(path, _)| path == "experiments/agent_cli_e2e/run_issue_864.sh"),
        "the sweep must reach the harness that failed, found {:?}",
        harnesses.iter().map(|(path, _)| path).collect::<Vec<_>>()
    );

    for (path, body) in &harnesses {
        assert!(
            body.contains("--no-summarize-session"),
            "{path} drives the Agent CLI against a local provider but leaves \
             `--summarize-session` at its default, so the client calls the \
             hosted `opencode/big-pickle` summarizer between turns and the run \
             dies on a gateway this harness never configured (issue #1079)"
        );
        assert!(
            body.contains("--compaction-models \"(same)\""),
            "{path} must pin compaction to the session's own model with \
             `--compaction-models \"(same)\"`. The singular `--compaction-model \
             same` reads as the fix but is ignored: the CLI takes the cascade \
             branch whenever the cascade argument is set, and it is always set \
             to a default that begins with hosted models (issue #1079)"
        );
    }
}

/// Every job of a workflow, keyed by name.
///
/// `workflow_fixtures::job_block` answers "give me this one job of
/// `release.yml`"; a sweep needs the opposite -- every job of every workflow,
/// without naming any of them, so a job added tomorrow is measured too.
fn workflow_jobs(body: &str) -> Vec<(String, String)> {
    let Some(jobs_at) = body.find("\njobs:\n") else {
        return Vec::new();
    };
    let mut jobs: Vec<(String, String)> = Vec::new();
    for line in body[jobs_at + "\njobs:\n".len()..].lines() {
        let is_job_header = line.starts_with("  ")
            && !line.starts_with("   ")
            && line.trim_end().ends_with(':')
            && !line.trim_start().starts_with('#');
        if is_job_header {
            jobs.push((line.trim().trim_end_matches(':').to_string(), String::new()));
        } else if let Some((_, collected)) = jobs.last_mut() {
            collected.push_str(line);
            collected.push('\n');
        }
    }
    jobs
}

/// Issue #1079, D11, second layer. The flags on the harness only bind the
/// invocations that harness spells out; a job that also launches the client
/// some other way -- a ladder that shells out per node, a step that resumes a
/// session -- would still reach for the hosted summarizer.
///
/// `LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION=false` binds the whole job, and the
/// CLI reports the result in its first stream event
/// (`{"type":"config","summarizeSession":false,...}`), so the belt is
/// observable rather than assumed. `release.yml`'s `test-agent-cli-e2e` job
/// has carried it since issue #819; the two workflows that grew their own
/// Agent CLI job afterwards did not inherit it, which is the same
/// one-file-gate shape as the missing flags.
#[test]
fn every_ci_job_that_launches_the_agent_cli_disables_hosted_summarization() {
    let agent_harnesses: Vec<String> = shell_scripts_ci_runs()
        .into_iter()
        .filter(|(_, body)| body.contains("--disable-stdin"))
        .map(|(path, _)| path)
        .collect();

    let mut checked = 0;
    for path in github_yaml_files() {
        let body = fs::read_to_string(&path)
            .expect("readable workflow")
            .replace("\r\n", "\n");
        for (job, job_body) in workflow_jobs(&body) {
            let launches_agent = experiment_script_references(&job_body)
                .iter()
                .any(|script| agent_harnesses.contains(script));
            if !launches_agent {
                continue;
            }
            checked += 1;
            assert!(
                job_body.contains("LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION: \"false\""),
                "job `{job}` of {} runs an Agent CLI harness but does not set \
                 LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION=false, so any client \
                 the job starts outside the harness command line still calls \
                 the hosted `opencode/big-pickle` summarizer (issue #1079)",
                path.display()
            );
        }
    }

    assert!(
        checked >= 3,
        "the sweep must find the jobs that run the Agent CLI, found {checked}"
    );
}

/// Issue #1079, D11, third place. The same wrong flag is baked into the
/// product: `formal-ai with agent ...` appends `no_summarize_args` from the
/// seed to every wrapped invocation, so every user of the wrapper -- not just
/// CI -- was sending a flag the CLI ignores and getting the hosted summarizer
/// they asked to be rid of.
///
/// The value is `(same)` rather than `same` because `--compaction-models`
/// parses a parenthesized list; `formal-ai with` passes argv directly, so the
/// parentheses that a shell would quote away belong in the argument itself.
#[test]
fn the_wrapper_pins_agent_compaction_to_the_session_model() {
    let agent = formal_ai::seed::client_integrations()
        .into_iter()
        .find(|integration| integration.id == "agent")
        .expect("the seed registry must describe the Agent CLI");

    assert_eq!(
        agent.invocation.no_summarize_args,
        vec![
            "--no-summarize-session".to_string(),
            "--compaction-models".to_string(),
            "(same)".to_string(),
        ],
        "`formal-ai with agent` must disable summarization *and* pin \
         compaction to the session's own model. `--compaction-model same` is \
         silently ignored whenever the plural default is set, which it always \
         is (issue #1079)"
    );
}

/// Issue #1079, D12. The `agent-cli-failure-report` job of run 34061511110
/// failed with exactly one line of evidence:
///
/// ```text
/// ##[error]Process completed with exit code 1.
/// ```
///
/// Two seconds, no message, no stack. The cause -- an unhandled rejection from
/// the hosted summarizer -- was in the uploaded artifact, and only there. The
/// harnesses redirect the client's streams to files and classify them
/// afterwards with `scripts/classify-agent-cli-stderr.sh`, which *does* print
/// an unexpected diagnostic; but `set -e` ends the harness on the client's own
/// non-zero exit, one line before that classification runs. The step that
/// would have spoken is unreachable exactly when it has something to say.
///
/// `experiments/agentic_cli_matrix/lib.sh` already had the answer -- its
/// `matrix_fail` calls `matrix_dump_logs` -- so this is the repository's own
/// practice, applied to the jobs that had not adopted it.
#[test]
fn every_ci_job_that_launches_the_agent_cli_reads_its_evidence_into_the_log() {
    let agent_harnesses: Vec<String> = shell_scripts_ci_runs()
        .into_iter()
        .filter(|(_, body)| body.contains("--disable-stdin"))
        .map(|(path, _)| path)
        .collect();

    let mut checked = 0;
    for path in github_yaml_files() {
        let body = fs::read_to_string(&path)
            .expect("readable workflow")
            .replace("\r\n", "\n");
        for (job, job_body) in workflow_jobs(&body) {
            let launches_agent = experiment_script_references(&job_body)
                .iter()
                .any(|script| agent_harnesses.contains(script));
            if !launches_agent {
                continue;
            }
            checked += 1;
            assert!(
                job_body.contains("dump-agent-cli-evidence.sh"),
                "job `{job}` of {} runs an Agent CLI harness but never reads \
                 the harness's stream files back into the job log, so a \
                 failure there reports an exit status and no cause -- the \
                 shape of run 34061511110 (issue #1079)",
                path.display()
            );
            assert!(
                job_body.contains("if: failure()"),
                "job `{job}` of {} must read the evidence only when the run \
                 failed; a green run has nothing to explain (issue #1079)",
                path.display()
            );
        }
    }

    assert!(
        checked >= 3,
        "the sweep must find the jobs that run the Agent CLI, found {checked}"
    );
}

/// Issue #1079, D12. A diagnostic no test ever runs is the next silent
/// failure: it only executes on a red run, where nobody is in a position to
/// notice that it printed nothing. So run it here, against the evidence the
/// real failure left behind, and require that the unhandled rejection reaches
/// standard output.
///
/// Two properties matter beyond "it prints something". It must exit 0 even
/// when a path is missing -- a failing diagnostic would turn one failure into
/// two and bury the first -- and the stderr file must come first, because that
/// is where the cause lives and a reviewer meets the top of a folded log
/// before its middle.
#[test]
fn the_evidence_dump_reports_a_failure_without_becoming_one() {
    let script = format!(
        "{}/scripts/dump-agent-cli-evidence.sh",
        env!("CARGO_MANIFEST_DIR")
    );

    let directory = std::env::temp_dir().join(format!(
        "formal-ai-issue-1079-evidence-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock after 1970")
            .as_nanos()
    ));
    fs::create_dir_all(&directory).expect("a writable temporary directory");
    // The shape run 34061511110 actually left on disk, trimmed to its point.
    fs::write(
        directory.join("agent-stderr.log"),
        "{\"type\":\"error\",\"errorType\":\"UnhandledRejection\",\
         \"message\":\"Request is missing x-opencode-session\"}\n",
    )
    .expect("writable stderr fixture");
    fs::write(
        directory.join("agent-stream.raw.log"),
        "{\"type\":\"log\"}\n",
    )
    .expect("writable stream fixture");

    let output = std::process::Command::new("bash")
        .arg(&script)
        .arg(&directory)
        .arg(directory.join("no-such-path"))
        .output()
        .expect("the evidence dump must be runnable");

    fs::remove_dir_all(&directory).ok();

    assert!(
        output.status.success(),
        "the evidence dump must not fail even when a path is missing; it runs \
         on an already-failed job and a second error would bury the first \
         (issue #1079). status: {}",
        output.status
    );

    let printed = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        printed.contains("UnhandledRejection"),
        "the evidence dump must put the client's own diagnostic in the job \
         log; that line is the whole difference between `exit code 1` and a \
         root cause (issue #1079). printed:\n{printed}"
    );
    assert!(
        printed.contains("no Agent CLI evidence at"),
        "a missing path must be reported rather than passed over in silence \
         (issue #1079). printed:\n{printed}"
    );

    let stderr_at = printed
        .find("agent-stderr.log")
        .expect("the stderr file must appear");
    let stream_at = printed
        .find("agent-stream.raw.log")
        .expect("the stream file must appear");
    assert!(
        stderr_at < stream_at,
        "the stderr file carries the cause and must be printed before the \
         transcript it interrupted (issue #1079)"
    );
}
