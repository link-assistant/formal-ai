//! Issue #656 (E37): `formal-ai improve --promote` end to end.
//!
//! The dry run prints the promotion plan without touching any files; `--apply`
//! without `--confirm` refuses; `--apply --confirm` materializes only the
//! promoted seed edit into the workspace.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

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
    Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .arg("improve")
        .args(args)
        .output()
        .expect("run improve")
}

#[test]
fn improve_promote_dry_run_touches_no_files() {
    let dir = tmpdir();
    let output = improve(&["--promote", "--seed-root", dir.to_str().unwrap()]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("promotion_run"), "plan output: {stdout}");
    assert!(stdout.contains("decision \"promoted\""));
    assert!(stdout.contains("decision \"rejected\""));

    // Dry run: no seed file was written under the workspace.
    let seed = dir.join("data/seed/learned-program-rules.lino");
    assert!(!seed.exists(), "dry run must not write {}", seed.display());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn improve_apply_without_confirm_refuses() {
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
    let dir = tmpdir();
    let output = improve(&[
        "--promote",
        "--apply",
        "--confirm",
        "--seed-root",
        dir.to_str().unwrap(),
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "apply stderr: {stderr}");

    let seed = dir.join("data/seed/learned-program-rules.lino");
    let body = std::fs::read_to_string(&seed).expect("materialized seed");
    assert!(body.contains("learned_reverse_sort"), "seed body: {body}");
    // The rejected proposal's rule must never be applied.
    assert!(
        !body.contains("learned_untested_rewrite"),
        "rejected edit must not be applied: {body}"
    );
    // The branch step is only a plan; nothing is pushed.
    assert!(stderr.contains("never executed"), "stderr: {stderr}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn improve_without_promote_prints_guidance() {
    let output = improve(&[]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--promote"), "guidance: {stdout}");
}
