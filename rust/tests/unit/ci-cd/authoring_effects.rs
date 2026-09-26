//! Seeded input is context, not evidence that the model authored a change.
//!
//! Plan 03 L13 moved the live authoring loop from
//! `scripts/author-change-with-formal-ai.sh` into `src/authoring_loop.rs`,
//! keeping the script's CLI for the workflow. The same fixture fakes the bash
//! harness used to test through the wrapper — a `serve` stand-in and an Agent
//! CLI stand-in — now drive the loop in process through the executable
//! override seams, which also makes the commit landing and the evidence
//! isolation assertable hermetically (and the crate's `unsafe_code = forbid`
//! rules out the env-var route).

#![cfg(unix)]

use std::fs;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::cli_solve::{SolveArgs, run_solve};

fn executable(path: &Path, source: &str) {
    fs::write(path, source).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

struct Sandbox(PathBuf);

impl Sandbox {
    fn fresh(name: String) -> Self {
        let path = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // The path is the exact private directory this test created.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn agent_fixture(mode: &str) -> String {
    let action = match mode {
        "unchanged" => String::new(),
        "identical" => String::from("printf 'original\\n' > leaf.txt\n"),
        "modified" | "already_landed" => String::from("printf 'modified\\n' > leaf.txt\n"),
        "new" => String::from("printf 'new artifact\\n' > created.txt\n"),
        _ => unreachable!("unknown fixture mode {mode}"),
    };
    format!(
        r#"#!/bin/sh
{action}echo "progress chatter from the CLI"
printf '{{"session_id":"ses_authoringfixture"}}\n'
"#
    )
}

fn write_fixtures(root: &Path, mode: &str) {
    for dir in ["bin", "scripts", "seed", "published"] {
        fs::create_dir(root.join(dir)).unwrap();
    }
    fs::write(root.join("seed/leaf.txt"), "original\n").unwrap();
    fs::write(root.join("seed/support.txt"), "support\n").unwrap();
    fs::write(root.join("published/leaf.txt"), "keep destination\n").unwrap();
    if mode == "already_landed" {
        fs::write(root.join("published/leaf.txt"), "modified\n").unwrap();
        fs::write(root.join("published/support.txt"), "support\n").unwrap();
    }
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    fs::copy(
        repository.join("scripts/classify-agent-cli-stderr.sh"),
        root.join("scripts/classify-agent-cli-stderr.sh"),
    )
    .unwrap();
    // The server stand-in only has to stay alive; readiness is the loop's
    // bounded TCP probe, which this test satisfies with a real listener.
    executable(
        &root.join("bin/formal-ai-server"),
        "#!/bin/sh\nexec sleep 30\n",
    );
    executable(&root.join("bin/agent"), &agent_fixture(mode));
}

fn solve_args(root: &Path, mode: &str, port: u16, commit: bool) -> SolveArgs {
    let produced = if mode == "new" {
        "created.txt"
    } else {
        "leaf.txt"
    };
    SolveArgs {
        issue: None,
        task: Some(String::from("author an isolated fixture")),
        repository: root.display().to_string(),
        base_commit: None,
        model: String::from("formal-ai"),
        evidence: PathBuf::from("evidence"),
        pull_request: Some(String::from(
            "https://github.com/link-assistant/formal-ai/pull/888",
        )),
        commit,
        produces: vec![String::from(produced), String::from("support.txt")],
        into: vec![
            String::from("published/leaf.txt"),
            String::from("published/support.txt"),
        ],
        seed: Some(String::from("seed")),
        contains: Vec::new(),
        port,
        message: Some(String::from("fixture")),
        server_executable: Some(root.join("bin/formal-ai-server")),
        agent_executable: Some(root.join("bin/agent")),
    }
}

fn run_case_commit(
    mode: &str,
    commit: bool,
) -> (Sandbox, Result<formal_ai::cli_solve::SolveOutcome, String>) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = Sandbox::fresh(format!("formal-ai-authoring-{mode}-{stamp}"));
    write_fixtures(root.path(), mode);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let outcome =
        run_solve(&solve_args(root.path(), mode, port, commit)).map_err(|error| error.to_string());
    drop(listener);
    (root, outcome)
}

fn run_case(mode: &str) -> (Sandbox, Result<formal_ai::cli_solve::SolveOutcome, String>) {
    run_case_commit(mode, false)
}

#[test]
fn read_only_or_identical_seed_replays_are_not_authorship() {
    for mode in ["unchanged", "identical"] {
        let (root, outcome) = run_case(mode);
        let error = outcome.expect_err("a seed replay must not count as authorship");
        assert!(
            error.contains("no produced artifact differs"),
            "{mode}: {error}"
        );
        assert_eq!(
            fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
            "keep destination\n"
        );
        assert!(!root.path().join("published/support.txt").exists());
    }
}

#[test]
fn a_changed_artifact_can_be_published_with_unchanged_support() {
    let (root, outcome) = run_case("modified");
    outcome.expect("a modified artifact must author");
    assert_eq!(
        fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
        "modified\n"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("published/support.txt")).unwrap(),
        "support\n"
    );
}

#[test]
fn a_new_artifact_is_an_authored_effect() {
    let (root, outcome) = run_case("new");
    outcome.expect("a new artifact must author");
    assert_eq!(
        fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
        "new artifact\n"
    );
}

#[test]
fn already_published_bytes_do_not_become_new_authorship_through_fresh_logs() {
    let (root, outcome) = run_case("already_landed");
    let error = outcome.expect_err("already-landed bytes must not count as authorship");
    assert!(error.contains("no produced artifact differs"), "{error}");
    assert_eq!(
        fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
        "modified\n"
    );
}

#[test]
fn the_evidence_keeps_the_framed_events_and_never_the_raw_stream() {
    let (root, outcome) = run_case("modified");
    outcome.expect("a modified artifact must author");
    let evidence = root.path().join("evidence");
    // Only the framed events are kept: the raw stream repeats them verbatim
    // around the CLI's own progress chatter, and committing both would double
    // the evidence for no extra proof.
    let jsonl = fs::read_to_string(evidence.join("agent-stream.jsonl")).unwrap();
    assert!(jsonl.contains("ses_authoringfixture"), "{jsonl}");
    assert!(
        !jsonl.contains("progress chatter"),
        "the raw stream leaked into the evidence directory"
    );
    for name in [
        "task.txt",
        "session-id.txt",
        "agent-stderr.log",
        "formal-ai.log",
    ] {
        assert!(
            evidence.join(name).is_file(),
            "{name} missing from evidence"
        );
    }
    assert_eq!(
        fs::read_to_string(evidence.join("task.txt")).unwrap(),
        "author an isolated fixture\n"
    );
    // One evidence file carries both markers the metric looks for.
    let session_id = fs::read_to_string(evidence.join("session-id.txt")).unwrap();
    assert!(
        session_id.contains("formal-ai session ses_authoringfixture"),
        "{session_id}"
    );
    assert!(
        session_id.contains("formal-ai model formal-ai/"),
        "{session_id}"
    );
    for entry in fs::read_dir(&evidence).unwrap() {
        let name = entry.unwrap().file_name();
        assert!(
            !name.to_string_lossy().contains("raw"),
            "the raw stream must not land in the evidence directory: {name:?}"
        );
    }
}

#[test]
fn the_loop_refuses_a_contains_miss() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = Sandbox::fresh(format!("formal-ai-authoring-contains-{stamp}"));
    write_fixtures(root.path(), "modified");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let mut args = solve_args(root.path(), "modified", port, false);
    args.contains = vec![String::from("a line no fixture writes")];
    let error = run_solve(&args)
        .map_err(|error| error.to_string())
        .unwrap_err();
    assert!(error.contains("no artifact contains"), "{error}");
}

#[test]
fn a_malformed_pull_request_is_rejected_before_anything_runs() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = Sandbox::fresh(format!("formal-ai-authoring-pr-{stamp}"));
    write_fixtures(root.path(), "modified");
    let mut args = solve_args(root.path(), "modified", 8899, false);
    args.pull_request = Some(String::from(
        "https://github.com/link-assistant/formal-ai/pull/0888",
    ));
    let error = run_solve(&args)
        .map_err(|error| error.to_string())
        .unwrap_err();
    assert!(
        error.contains("canonical GitHub pull-request URL"),
        "{error}"
    );
    // Nothing was authored, copied, or measured: the fixture never ran.
    assert!(!root.path().join("evidence").exists());
    assert_eq!(
        fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
        "keep destination\n"
    );
}
