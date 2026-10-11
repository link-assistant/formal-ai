//! Issue #1138 R1138-3-5: SWE-bench, the coding ladder and self-authoring
//! consume the same protocol document.
//!
//! Each caller runs here against one small local repository: the SWE-bench
//! entry point (`external_benchmarks::solve_repository_case`), the coding
//! ladder's entry point (`formal-ai solve --task`, which
//! `experiments/issue_847_coding_ladder/run_coding_ladder.sh` invokes), and
//! the self-authoring loop (`formal-ai solve --produces`) against fake `serve`
//! and Agent CLI executables, as in `authoring_effects.rs`. Each writes
//! `repository-protocol.lino`; the stage names they record are compared with
//! each other and with the stages the document declares.

#![cfg(unix)]

use std::fs;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::authoring_loop::run_authoring_with;
use formal_ai::cli_solve::{SolveArgs, run_solve};
use formal_ai::external_benchmarks::{BenchmarkCase, Expectation, solve_repository_case};
use formal_ai::repository_workspace::WorkspaceProtocol;
use formal_ai::repository_workspace::clone::WorkspaceSpec;
use formal_ai::repository_workspace::trace::{
    EDITOR_AGENT_SESSION, EDITOR_STRUCTURAL, ProtocolTrace, StageStatus, stage_ids, stage_statuses,
};

/// A requirement that names a file of the fixture, so every structural caller
/// locates and reads it and then stops at the structural editor, which derives
/// no edit for a plain text file.
const REQUIREMENT: &str = "Rewrite notes/leaf.txt so that it says modified.";

struct Sandbox(PathBuf);

impl Sandbox {
    fn fresh(tag: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "formal-ai-protocol-callers-{tag}-{}-{stamp}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("sandbox directory");
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

fn executable(path: &Path, source: &str) {
    fs::write(path, source).expect("fixture script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("fixture mode");
}

fn git(root: &Path, argv: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(argv)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {argv:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// A one-commit repository holding `notes/leaf.txt`, the stderr classifier
/// the authoring loop runs, and the fake executables. Returns its HEAD.
fn write_fixture(root: &Path) -> String {
    let repository = root.join("repository");
    for dir in ["notes", "scripts"] {
        fs::create_dir_all(repository.join(dir)).expect("fixture directory");
    }
    fs::write(repository.join("notes/leaf.txt"), "original\n").expect("fixture file");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    fs::copy(
        source.join("scripts/classify-agent-cli-stderr.sh"),
        repository.join("scripts/classify-agent-cli-stderr.sh"),
    )
    .expect("stderr classifier");
    git(&repository, &["init", "-q"]);
    git(&repository, &["config", "user.name", "fixture"]);
    git(
        &repository,
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(&repository, &["add", "."]);
    git(
        &repository,
        &[
            "-c",
            "user.name=Formal AI",
            "-c",
            "user.email=formal-ai@localhost",
            "commit",
            "-q",
            "-m",
            "fixture",
        ],
    );

    let bin = root.join("bin");
    fs::create_dir_all(&bin).expect("bin directory");
    // The server stand-in only has to stay alive; readiness is the loop's
    // bounded TCP probe, which a real listener satisfies.
    executable(&bin.join("formal-ai-server"), "#!/bin/sh\nexec sleep 30\n");
    executable(
        &bin.join("agent"),
        "#!/bin/sh\nmkdir -p notes\nprintf 'modified\\n' > notes/leaf.txt\necho \"progress chatter from the CLI\"\nprintf '{\"session_id\":\"ses_protocolcallers\"}\\n'\n",
    );
    git(&repository, &["rev-parse", "HEAD"])
}

fn solve_args(root: &Path, evidence: PathBuf) -> SolveArgs {
    SolveArgs {
        task: Some(String::from(REQUIREMENT)),
        repository: root.join("repository").display().to_string(),
        evidence,
        pull_request: Some(String::from(
            "https://github.com/link-assistant/formal-ai/pull/1188",
        )),
        ..SolveArgs::default()
    }
}

/// The SWE-bench caller: one repository case through the benchmark entry point.
fn benchmark_trace(root: &Path, base_commit: &str) -> String {
    let case = BenchmarkCase {
        id: String::from("fixture__leaf-1"),
        prompt: String::from(REQUIREMENT),
        expectation: Expectation::SweBench {
            record: String::from("{}"),
        },
        repository: Some(WorkspaceSpec {
            origin: root.join("repository").display().to_string(),
            base_commit: base_commit.to_owned(),
            sparse_paths: Vec::new(),
        }),
        tests: None,
    };
    let run_root = root.join("benchmark-run");
    let answer = solve_repository_case(&case, &run_root);
    let evidence_root = run_root.join("repository-evidence");
    let written = fs::read_dir(&evidence_root)
        .expect("the benchmark writes its evidence directory")
        .map(|entry| entry.expect("evidence entry").path())
        .map(|case_dir| {
            fs::read_to_string(case_dir.join("repository-protocol.lino"))
                .expect("the benchmark writes repository-protocol.lino")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        written,
        vec![answer.links_notation.clone()],
        "the benchmark's evidence file is its answer's trace"
    );
    answer.links_notation
}

/// The coding ladder's caller: `formal-ai solve --task --evidence`.
fn ladder_trace(root: &Path) -> String {
    let evidence = root.join("ladder-evidence");
    let outcome = run_solve(&solve_args(root, evidence.clone())).expect("solve runs");
    assert!(!outcome.committed, "solve refuses to commit by default");
    fs::read_to_string(evidence.join("repository-protocol.lino"))
        .expect("solve writes repository-protocol.lino")
}

/// The self-authoring caller: `formal-ai solve --produces` against the fakes.
fn authoring_args(root: &Path, port: u16) -> SolveArgs {
    SolveArgs {
        produces: vec![String::from("notes/leaf.txt")],
        port,
        message: Some(String::from("fixture")),
        server_executable: Some(root.join("bin/formal-ai-server")),
        agent_executable: Some(root.join("bin/agent")),
        ..solve_args(root, root.join("authoring-evidence"))
    }
}

fn authoring_trace(root: &Path, protocol: &WorkspaceProtocol) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("listener address").port();
    let outcome = run_authoring_with(protocol, &authoring_args(root, port))
        .expect("the fake session authors the leaf");
    drop(listener);
    assert_eq!(outcome.session_id, "ses_protocolcallers");
    assert!(!outcome.committed, "the loop refuses to commit by default");
    fs::read_to_string(root.join("authoring-evidence/repository-protocol.lino"))
        .expect("the authoring loop writes repository-protocol.lino")
}

fn declared_ids(protocol: &WorkspaceProtocol) -> Vec<String> {
    protocol
        .steps()
        .iter()
        .map(|step| step.id.clone())
        .collect()
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(id, status)| ((*id).to_owned(), (*status).to_owned()))
        .collect()
}

#[test]
fn every_caller_records_the_stages_the_protocol_document_declares() {
    let sandbox = Sandbox::fresh("all");
    let base_commit = write_fixture(sandbox.path());
    let protocol = WorkspaceProtocol::load();
    let declared = declared_ids(&protocol);
    assert_eq!(
        declared,
        [
            "clone", "locate", "read", "serve", "edit", "session", "verify", "diff", "commit"
        ],
        "the Agent CLI session stages are declared in the document, between the shared ones"
    );

    let benchmark = benchmark_trace(sandbox.path(), &base_commit);
    let ladder = ladder_trace(sandbox.path());
    let authoring = authoring_trace(sandbox.path(), &protocol);

    for (caller, trace) in [
        ("benchmark", &benchmark),
        ("solve", &ladder),
        ("authoring", &authoring),
    ] {
        assert!(
            trace.starts_with("repository_protocol_trace\n"),
            "{caller}: {trace}"
        );
        assert!(
            trace.contains(&format!("  caller \"{caller}\"\n")),
            "{caller}: {trace}"
        );
        assert_eq!(stage_ids(trace), declared, "{caller}: {trace}");
    }
    assert_eq!(stage_ids(&benchmark), stage_ids(&ladder));
    assert_eq!(stage_ids(&ladder), stage_ids(&authoring));

    // Both structural callers locate and read the named file, own neither
    // Agent CLI session stage, and stop at the structural editor.
    let structural = pairs(&[
        ("clone", "observed"),
        ("locate", "observed"),
        ("read", "observed"),
        ("serve", "not_applicable"),
        ("edit", "stopped"),
        ("session", "not_applicable"),
        ("verify", "not_reached"),
        ("diff", "not_reached"),
        ("commit", "not_reached"),
    ]);
    assert_eq!(stage_statuses(&benchmark), structural, "{benchmark}");
    assert_eq!(stage_statuses(&ladder), structural, "{ladder}");
    for trace in [&benchmark, &ladder] {
        assert!(
            trace.contains(&format!("  editor \"{EDITOR_STRUCTURAL}\"\n")),
            "{trace}"
        );
    }

    // The authoring loop runs every stage, the Agent CLI session as its
    // editor, and leaves the commit gate shut without `--commit`.
    assert_eq!(
        stage_statuses(&authoring),
        pairs(&[
            ("clone", "observed"),
            ("locate", "observed"),
            ("read", "observed"),
            ("serve", "observed"),
            ("edit", "observed"),
            ("session", "observed"),
            ("verify", "observed"),
            ("diff", "observed"),
            ("commit", "refused"),
        ]),
        "{authoring}"
    );
    assert!(
        authoring.contains(&format!("  editor \"{EDITOR_AGENT_SESSION}\"\n")),
        "{authoring}"
    );
    assert!(
        authoring.contains("  session \"ses_protocolcallers\"\n"),
        "{authoring}"
    );
}

/// Spawning `serve` is a stage of the document, not a branch of the loop:
/// a document without it runs the same loop without spawning anything.
#[test]
fn the_serve_stage_is_data_the_authoring_loop_reads() {
    let sandbox = Sandbox::fresh("serve");
    write_fixture(sandbox.path());
    // The serve stage opens the server log before it spawns anything.
    let server_log = sandbox.path().join("authoring-evidence/formal-ai.log");

    let document = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("embedded/data/meta/repository-workspace-protocol.lino"),
    )
    .expect("protocol document");
    let start = document
        .find("repository_step_serve\n")
        .expect("the document declares the serve stage");
    let end = start
        + document[start..]
            .find("repository_step_edit\n")
            .expect("the edit stage follows the serve stage");
    let without_serve =
        WorkspaceProtocol::parse(&format!("{}{}", &document[..start], &document[end..]));
    assert!(
        !declared_ids(&without_serve).contains(&String::from("serve")),
        "the edited document no longer declares the serve stage"
    );

    let trace = authoring_trace(sandbox.path(), &without_serve);
    assert!(
        !server_log.exists(),
        "no serve stage in the document, no server spawned: {trace}"
    );
    assert_eq!(stage_ids(&trace), declared_ids(&without_serve), "{trace}");

    // The first run landed the leaf; restore it so the second run authors too.
    fs::remove_dir_all(sandbox.path().join("authoring-evidence")).expect("reset evidence");
    fs::write(
        sandbox.path().join("repository/notes/leaf.txt"),
        "original\n",
    )
    .expect("reset the landed leaf");
    let trace = authoring_trace(sandbox.path(), &WorkspaceProtocol::load());
    assert!(
        server_log.is_file(),
        "the committed document spawns the server: {trace}"
    );
}

/// Under `--commit` the commit carries the trace with the gate opened, next to
/// the source bytes and the other evidence.
#[test]
fn the_committed_trace_records_the_opened_commit_gate() {
    let sandbox = Sandbox::fresh("commit");
    write_fixture(sandbox.path());
    let repository = sandbox.path().join("repository");
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("listener address").port();
    let args = SolveArgs {
        commit: true,
        evidence: PathBuf::from("evidence"),
        ..authoring_args(sandbox.path(), port)
    };
    let outcome = run_solve(&args).expect("the fake session lands a commit");
    drop(listener);
    assert!(outcome.committed, "--commit lands the authored change");

    let committed = git(
        &repository,
        &["show", "HEAD:evidence/repository-protocol.lino"],
    );
    assert_eq!(
        stage_statuses(&committed),
        pairs(&[
            ("clone", "observed"),
            ("locate", "observed"),
            ("read", "observed"),
            ("serve", "observed"),
            ("edit", "observed"),
            ("session", "observed"),
            ("verify", "observed"),
            ("diff", "observed"),
            ("commit", "requested"),
        ]),
        "{committed}"
    );
    assert_eq!(
        git(&repository, &["show", "HEAD:notes/leaf.txt"]),
        "modified",
        "the source bytes land in the same commit as the trace"
    );
    assert_eq!(
        git(&repository, &["status", "--porcelain"]),
        "",
        "the trace written after the commit is the committed one"
    );
}

/// The Rust trace renders byte-for-byte what the JavaScript twin renders
/// (rust/tests/web/issue-1138-protocol-trace.test.mjs).
#[test]
fn the_trace_renders_what_the_javascript_twin_renders() {
    let protocol = WorkspaceProtocol::load();
    let mut trace = ProtocolTrace::new(&protocol, "solve", EDITOR_STRUCTURAL);
    trace.record("clone", StageStatus::Observed);
    trace.record("edit", StageStatus::Stopped);
    trace.record("undeclared", StageStatus::Observed);
    trace.set_field("model", "formal-ai/0");
    trace.set_field("model", "formal-ai/1");
    trace.open.push(String::from("a \"quoted\" gap"));
    assert_eq!(
        trace.render(),
        [
            "repository_protocol_trace",
            "  caller \"solve\"",
            "  editor \"structural\"",
            "  model \"formal-ai/1\"",
            "  stage clone \"observed\"",
            "  stage locate \"not_reached\"",
            "  stage read \"not_reached\"",
            "  stage serve \"not_applicable\"",
            "  stage edit \"stopped\"",
            "  stage session \"not_applicable\"",
            "  stage verify \"not_reached\"",
            "  stage diff \"not_reached\"",
            "  stage commit \"not_reached\"",
            "  open \"a \"\"quoted\"\" gap\"",
            "",
        ]
        .join("\n")
    );
}
