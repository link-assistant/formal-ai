use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir() -> PathBuf {
    let sequence = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-issue-879-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("create temporary directory");
    directory
}

fn write_no_effect_agent(bin_dir: &Path) {
    let path = bin_dir.join("agent");
    std::fs::write(
        &path,
        r#"#!/bin/sh
attempt=0
if [ -f "$FORMAL_AI_ATTEMPTS" ]; then
  attempt=$(cat "$FORMAL_AI_ATTEMPTS")
fi
attempt=$((attempt + 1))
printf '%s\n' "$attempt" > "$FORMAL_AI_ATTEMPTS"
{
  printf 'attempt=%s\n' "$attempt"
  printf 'args='
  printf ' <%s>' "$@"
  printf '\n'
} >> "$FORMAL_AI_CAPTURE"
printf '{\n'
printf '  "type": "result",\n'
printf '  "subtype": "success",\n'
printf '  "rawMetadata": "{\\"formalai\\":{}}"\n'
printf '}\n'
"#,
    )
    .expect("write fake Agent CLI");
    let mut permissions = std::fs::metadata(&path)
        .expect("fake Agent CLI metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make fake Agent CLI executable");
}

#[test]
fn software_authoring_cannot_succeed_without_an_artifact() {
    let directory = tmpdir();
    let workspace = directory.join("workspace");
    let home = directory.join("home");
    let bin_dir = directory.join("bin");
    std::fs::create_dir_all(&workspace).expect("workspace");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin directory");
    assert!(Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&workspace)
        .status()
        .expect("initialize fixture repository")
        .success());
    write_no_effect_agent(&bin_dir);

    let attempts = directory.join("attempts.txt");
    let capture = directory.join("capture.txt");
    let existing_path = std::env::var_os("PATH").unwrap_or_default();
    let path = format!("{}:{}", bin_dir.display(), existing_path.to_string_lossy());
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--no-start-server",
            "--non-interactive",
            "agent",
            "Implement Hello World in Scala",
        ])
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("PATH", path)
        .env("FORMAL_AI_ATTEMPTS", &attempts)
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run formal-ai with Agent CLI");

    assert!(
        !output.status.success(),
        "a zero exit without an artifact was accepted; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&attempts).expect("attempt count"),
        "2\n",
        "the incomplete run must receive one bounded corrective retry"
    );

    let records = String::from_utf8(output.stdout)
        .expect("UTF-8 stdout")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).expect("one compact JSON value per line"))
        .collect::<Vec<_>>();
    let completion = records.last().expect("completion record");
    assert_eq!(completion["type"], "formal_ai_completion");
    assert_eq!(completion["completion_state"], "incomplete");
    assert_eq!(completion["reason"], "required_workspace_effect_missing");
    assert_eq!(completion["attempts"], 2);
    assert_eq!(completion["rawMetadata"]["formalai"]["model"], "formal-ai");
    assert_eq!(
        completion["rawMetadata"]["formalai"]["endpoint"],
        "http://127.0.0.1:8080/api/openai/v1"
    );
    assert!(completion["rawMetadata"]["formalai"]["input_tokens"].is_number());
    assert!(completion["rawMetadata"]["formalai"]["output_tokens"].is_number());

    let invocation = std::fs::read_to_string(&capture).expect("captured invocation");
    assert!(
        invocation.contains("--permission-mode") && invocation.contains("<auto>"),
        "software-authoring run did not enable the editing profile:\n{invocation}"
    );
    assert!(
        invocation.contains("--output-format") && invocation.contains("<stream-json>"),
        "software-authoring run did not request machine output:\n{invocation}"
    );
    let status = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .current_dir(&workspace)
        .output()
        .expect("inspect fixture repository");
    assert!(
        status.status.success(),
        "git status failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(
        status.stdout.is_empty(),
        "wrapper left scratch files in the repository:\n{}",
        String::from_utf8_lossy(&status.stdout)
    );

    let _ = std::fs::remove_dir_all(directory);
}
