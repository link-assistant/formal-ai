//! Issue #656 (E37): `formal-ai improve --promote` end to end.
//!
//! The dry run prints the promotion plan without touching any files; `--apply`
//! without `--confirm` refuses; `--apply --confirm` materializes only the
//! promoted seed edit into the workspace.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use formal_ai::{render_promotion_proposals, PromotionProposal, SeedEdit};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);
static PROMOTION_CLI_LOCK: Mutex<()> = Mutex::new(());

fn tmpdir() -> std::path::PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let thread_id = format!("{:?}", std::thread::current().id())
        .replace(|c: char| !c.is_ascii_alphanumeric(), "");
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-improve-{}-{thread_id}-{seq}",
        std::process::id(),
    ));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

fn improve(args: &[&str]) -> std::process::Output {
    let gate_bin = fake_gate_bin();
    let path = format!(
        "{}:{}",
        gate_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .arg("improve")
        .args(args)
        .env("PATH", path)
        .output()
        .expect("run improve")
}

fn fake_gate_bin() -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!(
        "formal-ai-promotion-gate-bin-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("gate bin");
    let cargo = dir.join("cargo");
    std::fs::write(
        &cargo,
        r#"#!/bin/sh
case "$*" in
  *issue_362*) echo "coding-modification benchmark pass/fail counts: passed=4 failed=0 total=4 minimum_pass_count=4" ;;
  *issue_304*) echo "benchmark pass/fail counts: passed=13 failed=0 total=13 minimum_pass_count=12" ;;
  *) echo "test result: ok. 1653 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out" ;;
esac
"#,
    )
    .expect("gate executable");
    let mut permissions = std::fs::metadata(&cargo)
        .expect("gate metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&cargo, permissions).expect("gate permissions");
    dir
}

fn proposal_file(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.with_extension("promotion-proposals.lino");
    let proposal = PromotionProposal::new(
        "learned_rule:cli_exact",
        "Promote a CLI-observed learning rule.",
        SeedEdit::new(
            "data/seed/learned-program-rules.lino",
            "substitution_rules\n  id \"cli_learned_rules\"\n  rule \"cli_exact\"",
        ),
        vec![],
    );
    std::fs::write(&path, render_promotion_proposals(&[proposal])).expect("proposal document");
    path
}

fn init_git(dir: &std::path::Path) {
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .status()
        .expect("git init")
        .success());
}

#[test]
fn improve_promote_dry_run_touches_no_files() {
    let _guard = PROMOTION_CLI_LOCK.lock().expect("promotion CLI lock");
    let dir = tmpdir();
    let proposals = proposal_file(&dir);
    let output = improve(&[
        "--promote",
        "--proposals",
        proposals.to_str().unwrap(),
        "--seed-root",
        dir.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("promotion_run"), "plan output: {stdout}");
    assert!(stdout.contains("decision \"promoted\""));
    assert!(stdout.contains("formal_ai_unit_specifications"));

    // Dry run: no seed file was written under the workspace.
    let seed = dir.join("data/seed/learned-program-rules.lino");
    assert!(!seed.exists(), "dry run must not write {}", seed.display());

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&proposals);
}

#[test]
fn improve_apply_without_confirm_refuses() {
    let _guard = PROMOTION_CLI_LOCK.lock().expect("promotion CLI lock");
    let dir = tmpdir();
    let output = improve(&["--promote", "--apply", "--seed-root", dir.to_str().unwrap()]);
    assert!(
        !output.status.success(),
        "apply without --confirm must fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--confirm"),
        "refusal should mention --confirm: {stderr}"
    );
    let seed = dir.join("data/seed/learned-program-rules.lino");
    assert!(!seed.exists(), "refused apply must not write files");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn improve_apply_with_confirm_materializes_only_promoted_edit() {
    let _guard = PROMOTION_CLI_LOCK.lock().expect("promotion CLI lock");
    let dir = tmpdir();
    init_git(&dir);
    let proposals = proposal_file(&dir);
    let output = improve(&[
        "--promote",
        "--proposals",
        proposals.to_str().unwrap(),
        "--apply",
        "--confirm",
        "--seed-root",
        dir.to_str().unwrap(),
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "apply stderr: {stderr}");

    let seed = dir.join("data/seed/learned-program-rules.lino");
    let body = std::fs::read_to_string(&seed).expect("materialized seed");
    assert!(body.contains("rule \"cli_exact\""), "seed body: {body}");
    assert!(
        stderr.contains("Formal AI Agent session evidence"),
        "{stderr}"
    );
    assert!(stderr.contains("Created local review branch"), "{stderr}");
    assert!(
        !stderr.contains("git checkout -b"),
        "already-executed branch creation must not be presented as a remaining command: {stderr}"
    );
    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&dir)
        .output()
        .expect("current branch");
    assert!(String::from_utf8_lossy(&branch.stdout).starts_with("promotion/"));

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&proposals);
}

#[test]
fn improve_without_promote_prints_guidance() {
    let output = improve(&[]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--promote"), "guidance: {stdout}");
}
