//! Real repository observations preserve patch and attribution boundaries (PR #1188).
use formal_ai::meta_frame::NeedStatus;
use formal_ai::repository_workspace::operation::{RepositoryOperation, classify};
use formal_ai::repository_workspace::{RepositoryTask, RepositoryWorkspace, WorkspaceProtocol};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new(broken: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "formal-ai-observation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("checks")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("Cargo.toml"),
            "[package]\nname=\"observation-heldout\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[[test]]\nname=\"boundary\"\npath=\"checks/observed.rs\"\n").unwrap();
        std::fs::write(
            root.join("src/lib.rs"),
            if broken {
                "pub fn invalid( {"
            } else {
                "pub fn identity(value: u64) -> u64 { value }\n"
            },
        )
        .unwrap();
        std::fs::write(
            root.join("checks/observed.rs"),
            "#[test]\nfn heldout() { assert_eq!(2 + 3, 5); }\n",
        )
        .unwrap();
        for argv in [
            vec!["init"],
            vec!["add", "."],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-m",
                "Observed",
            ],
        ] {
            let output = Command::new("git")
                .args(argv)
                .current_dir(&root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Self { root }
    }
    fn status(&self) -> Vec<u8> {
        Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.root)
            .output()
            .unwrap()
            .stdout
    }
    fn run(&self, request: &str) -> formal_ai::repository_workspace::ProtocolOutcome {
        let mut workspace = RepositoryWorkspace::adopt(&self.root).unwrap();
        let task = RepositoryTask {
            requirement: request.to_owned(),
            clone: workspace.spec().clone(),
            tests: None,
        };
        WorkspaceProtocol::load().execute(&mut workspace, &task)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
#[test]
fn original_question_reports_actual_heldout_manifest_target_without_mutation() {
    let fixture = Fixture::new(false);
    let before = fixture.status();
    let outcome = fixture.run("Where do unit tests live in this repository?");
    assert!(outcome.open.is_empty(), "{:?}", outcome.open);
    assert_eq!(outcome.diff.is_empty() && outcome.edited.len(), 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.report).unwrap();
    assert_eq!(report["schema"], "repository-query/v1");
    let sources = report["sources"].as_array().unwrap();
    assert!(
        sources
            .iter()
            .any(|source| source["path"] == "checks/observed.rs")
    );
    for source in sources {
        let path = source["path"].as_str().unwrap();
        let bytes = std::fs::read(fixture.root.join(path)).unwrap();
        assert_eq!(
            source["sha256"],
            formal_ai::source_fetch::sha256_hex(&bytes)
        );
    }
    assert_eq!(before, fixture.status());
    assert!(
        outcome
            .need_ledger
            .rows
            .iter()
            .all(|row| matches!(row.route.as_deref(), Some("clone" | "locate" | "read")))
    );
}
#[test]
fn original_cargo_check_reports_real_exit_and_complete_output() {
    let fixture = Fixture::new(false);
    let outcome =
        fixture.run("Run cargo check on this repository and tell me whether it succeeds.");
    assert!(outcome.open.is_empty(), "{:?}", outcome.open);
    let report: serde_json::Value = serde_json::from_str(&outcome.report).unwrap();
    assert_eq!(report["schema"], "repository-command/v1");
    assert_eq!(report["exit_code"], 0);
    assert_eq!(report["complete"], true);
    assert!(
        report["command"]
            .as_str()
            .unwrap()
            .contains("--manifest-path")
    );
    let output = report["combined_output"].as_str().unwrap();
    assert_eq!(
        report["observed_output_sha256"],
        formal_ai::source_fetch::sha256_hex(output.as_bytes())
    );
    assert_ne!(output.len(), 0);
    assert_eq!(outcome.edited.is_empty() && outcome.diff.len(), 0);
    assert_eq!(
        Command::new("git")
            .args(["diff", "--name-only"])
            .current_dir(&fixture.root)
            .output()
            .unwrap()
            .stdout,
        b""
    );
}
#[test]
fn failed_command_keeps_actual_diagnostics_and_cannot_satisfy_run_need() {
    let fixture = Fixture::new(true);
    let outcome = fixture.run("Run cargo check.");
    let report: serde_json::Value = serde_json::from_str(&outcome.report).unwrap();
    assert_ne!(report["exit_code"], 0);
    assert_eq!(report["complete"], false);
    assert_ne!(report["combined_output"].as_str().unwrap().len(), 0);
    assert_ne!(outcome.open.len(), 0);
    assert!(
        outcome.need_ledger.rows.iter().any(
            |row| row.route.as_deref() == Some("verify") && row.status != NeedStatus::Satisfied
        )
    );
    assert_eq!(outcome.edited.is_empty() && outcome.diff.len(), 0);
}
#[test]
fn compound_goals_do_not_silently_finish_one_clause() {
    assert_eq!(
        classify("Inspect tests and implement a new function."),
        RepositoryOperation::Unsupported
    );
    let fixture = Fixture::new(false);
    let before = fixture.status();
    let outcome = fixture.run("Run cargo check and write a new function.");
    assert_ne!(outcome.open.len(), 0);
    assert_eq!(
        outcome.report.is_empty() && outcome.edited.is_empty() && outcome.diff.len(),
        0
    );
    assert_eq!(fixture.status(), before);
}
#[test]
fn wrong_source_head_refuses_before_observation_or_edit() {
    let fixture = Fixture::new(false);
    let mut workspace = RepositoryWorkspace::adopt(&fixture.root).unwrap();
    let mut spec = workspace.spec().clone();
    spec.base_commit = "0".repeat(40);
    let outcome = WorkspaceProtocol::load().execute(
        &mut workspace,
        &RepositoryTask {
            requirement: "Where are tests?".to_owned(),
            clone: spec,
            tests: None,
        },
    );
    assert_eq!(outcome.report.is_empty() && !outcome.open.len(), 0);
    assert_eq!(outcome.observations.len(), 0);
}

#[test]
fn source_context_options_preserve_exact_and_prefix_refusals() {
    use formal_ai::repository_workspace::operation::command_context_excluded;
    for (program, options) in [
        ("git", vec!["-C", "-c", "--git-dir", "--work-tree"]),
        ("cargo", vec!["--manifest-path", "--target-dir", "--config"]),
    ] {
        for option in options {
            assert!(command_context_excluded(program, &[option.to_owned()]));
            if option.len() > 2 {
                for suffix in ["=outside", "-suffix"] {
                    assert!(command_context_excluded(
                        program,
                        &[format!("{option}{suffix}")]
                    ));
                }
            }
        }
    }
    for (program, arguments) in [
        ("git", vec!["status", "--porcelain", "-Cinside"]),
        ("cargo", vec!["check", "--workspace", "--configuration"]),
    ] {
        // Cargo's --configuration is intentionally covered by the original --config prefix.
        let expected = program == "cargo";
        assert_eq!(
            command_context_excluded(
                program,
                &arguments.into_iter().map(str::to_owned).collect::<Vec<_>>()
            ),
            expected
        );
    }
    assert!(command_context_excluded("missing-program", &[]));
}

#[test]
fn forbidden_context_options_are_refused_by_the_actual_command_binding() {
    use formal_ai::repository_workspace::operation::bind_command;
    use formal_ai::repository_workspace::verify::RunCommand;
    let fixture = Fixture::new(false);
    let workspace = RepositoryWorkspace::adopt(&fixture.root).unwrap();
    for line in [
        "git status -C outside",
        "git status -c setting=value",
        "git status --git-dir=outside",
        "git status --work-tree=outside",
        "cargo check --manifest-path=outside",
        "cargo check --target-dir=outside",
        "cargo check --config=outside",
    ] {
        assert!(
            bind_command(
                &workspace,
                &RunCommand {
                    line: line.to_owned(),
                    names: Vec::new()
                }
            )
            .is_err(),
            "{line}"
        );
    }
    for line in ["git status --porcelain", "cargo check --workspace"] {
        assert!(
            bind_command(
                &workspace,
                &RunCommand {
                    line: line.to_owned(),
                    names: Vec::new()
                }
            )
            .is_ok(),
            "{line}"
        );
    }
}
