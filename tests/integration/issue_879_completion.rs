use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use formal_ai::seed::software_authoring_completion_contract;
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

/// Durable learning state root for one fixture, kept out of the workspace so a
/// test can never write into the caller's tree and never inherits real history.
fn state_dir(directory: &Path) -> PathBuf {
    let path = directory.join("state");
    std::fs::create_dir_all(&path).expect("create fixture state directory");
    path
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
printf '  "rawMetadata": "diagnostic preface\\n{\\"usage\\":{\\"inputTokens\\":17,\\"outputTokens\\":9}}\\n{\\"tokensBreakdown\\":{\\"input\\":21,\\"output\\":11}}"\n'
printf '}\n'
printf '{"type":"concatenated"}'
printf 'plain text\n'
"#,
    )
    .expect("write fake Agent CLI");
    let mut permissions = std::fs::metadata(&path)
        .expect("fake Agent CLI metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make fake Agent CLI executable");
}

fn write_effect_client(bin_dir: &Path, name: &str, artifact_attempt: usize) {
    let path = bin_dir.join(name);
    let script = format!(
        r#"#!/bin/sh
attempt=0
if [ -f "$FORMAL_AI_ATTEMPTS" ]; then
  attempt=$(cat "$FORMAL_AI_ATTEMPTS")
fi
attempt=$((attempt + 1))
printf '%s\n' "$attempt" > "$FORMAL_AI_ATTEMPTS"
{{
  printf 'command=%s attempt=%s\n' "$(basename "$0")" "$attempt"
  printf 'args='
  printf ' <%s>' "$@"
  printf '\n'
  printf 'OPENAI_BASE_URL=%s\n' "$OPENAI_BASE_URL"
  printf 'ANTHROPIC_BASE_URL=%s\n' "$ANTHROPIC_BASE_URL"
  printf 'GOOGLE_GEMINI_BASE_URL=%s\n' "$GOOGLE_GEMINI_BASE_URL"
  printf 'LINK_ASSISTANT_AGENT_CONFIG_CONTENT=%s\n' "$LINK_ASSISTANT_AGENT_CONFIG_CONTENT"
  if [ -n "$OPENCODE_CONFIG" ] && [ -f "$OPENCODE_CONFIG" ]; then
    printf 'opencode_config='
    tr -d '\n' < "$OPENCODE_CONFIG"
    printf '\n'
  fi
  if [ -n "$HOME" ] && [ -f "$HOME/.qwen/settings.json" ]; then
    printf 'qwen_config='
    tr -d '\n' < "$HOME/.qwen/settings.json"
    printf '\n'
  fi
}} >> "$FORMAL_AI_CAPTURE"
if [ -f "$FORMAL_AI_CREATE_SESSION" ]; then
  session_dir="$HOME/.local/share/link-assistant-agent/storage/session"
  mkdir -p "$session_dir"
  printf '{{"sessionId":"ses_issue879","attempt":%s}}\n' "$attempt" \
    > "$session_dir/ses_issue879.json"
fi
if [ "$attempt" -ge "{artifact_attempt}" ]; then
  printf 'object HelloWorld extends App {{ println("Hello, World!") }}\n' > Hello.scala
fi
printf '{{\n'
printf '  "type": "result",\n'
printf '  "subtype": "success",\n'
printf '  "usage": {{ "input_tokens": 21, "output_tokens": 8 }}\n'
printf '}}\n'
"#
    );
    std::fs::write(&path, script).expect("write fake coding client");
    let mut permissions = std::fs::metadata(&path)
        .expect("fake coding client metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make fake coding client executable");
}

fn run_client(
    directory: &Path,
    workspace: &Path,
    home: &Path,
    bin_dir: &Path,
    tool: &str,
    task: &str,
    extra_args: &[&str],
) -> std::process::Output {
    let attempts = directory.join(format!("{tool}-attempts.txt"));
    let capture = directory.join(format!("{tool}-capture.txt"));
    let existing_path = std::env::var_os("PATH").unwrap_or_default();
    let path = format!("{}:{}", bin_dir.display(), existing_path.to_string_lossy());
    let mut args = vec!["with", "--no-start-server", "--non-interactive"];
    args.extend_from_slice(extra_args);
    args.push(tool);
    args.push(task);
    Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(args)
        .current_dir(workspace)
        .env("HOME", home)
        .env("PATH", path)
        .env("FORMAL_AI_ATTEMPTS", attempts)
        .env("FORMAL_AI_CAPTURE", capture)
        .env("FORMAL_AI_STATE_DIR", state_dir(directory))
        .env(
            "FORMAL_AI_CREATE_SESSION",
            directory.join("create-native-session"),
        )
        .output()
        .expect("run formal-ai with coding client")
}

fn json_records(output: &[u8]) -> Vec<Value> {
    String::from_utf8(output.to_vec())
        .expect("UTF-8 stdout")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).expect("one compact JSON value per line"))
        .collect()
}

#[test]
fn seeded_contract_is_complete_and_bounded() {
    let contract = software_authoring_completion_contract().expect("completion contract");
    assert_eq!(contract.observable_postcondition, "workspace_effect");
    assert_eq!(contract.max_attempts, 4);
    assert_eq!(
        contract.recovery_strategies,
        vec![
            "restate_postcondition".to_owned(),
            "name_target_artifact".to_owned(),
            "decompose_into_leaf".to_owned(),
        ],
        "the recovery ladder must be declared as data, one distinct strategy per retry"
    );
    assert_eq!(
        contract.max_attempts,
        contract.recovery_strategies.len() + 1,
        "the attempt budget must be exactly the initial attempt plus one try per strategy"
    );
    assert_eq!(
        contract.diverted_endpoints,
        vec![
            "https://api.openai.com".to_owned(),
            "https://api.anthropic.com".to_owned(),
            "https://generativelanguage.googleapis.com".to_owned(),
        ],
        "the public endpoints a local run must never reach are seed data, not Rust constants"
    );
    assert_eq!(
        contract.incomplete_reason,
        "required_workspace_effect_missing"
    );
    assert_eq!(contract.scratch_directory, ".formal-ai");
    assert_eq!(
        contract.incomplete_error,
        "{command} did not satisfy the software-authoring completion contract"
    );
    assert_eq!(
        contract.process_error,
        "{command} exited with status {status}"
    );
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
        .env("FORMAL_AI_STATE_DIR", state_dir(&directory))
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
        "4\n",
        "the incomplete run must spend its whole bounded recovery ladder and stop there"
    );

    let records = json_records(&output.stdout);
    let completion = records.last().expect("completion record");
    assert_eq!(completion["type"], "formal_ai_completion");
    assert_eq!(completion["completion_state"], "incomplete");
    assert_eq!(completion["reason"], "required_workspace_effect_missing");
    assert_eq!(completion["attempts"], 4);
    assert_eq!(
        completion["recovery"]["strategies_spent"],
        serde_json::json!([
            "restate_postcondition",
            "name_target_artifact",
            "decompose_into_leaf",
        ]),
        "each retry must spend a different recovery strategy, never repeat one"
    );
    assert_eq!(completion["rawMetadata"]["formalai"]["model"], "formal-ai");
    assert_eq!(
        completion["rawMetadata"]["formalai"]["endpoint"],
        "http://127.0.0.1:8080/api/openai/v1"
    );
    assert_eq!(completion["rawMetadata"]["formalai"]["input_tokens"], 21);
    assert_eq!(completion["rawMetadata"]["formalai"]["output_tokens"], 11);
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "concatenated")
            .count(),
        4,
        "each attempt must preserve concatenated JSON output"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record["type"] == "formal_ai_client_output" && record["text"] == "plain text"
            })
            .count(),
        4,
        "each attempt must wrap plain stdout in a machine-readable record"
    );

    let invocation = std::fs::read_to_string(&capture).expect("captured invocation");
    for marker in [
        "observable postcondition",
        "one concrete relative path",
        "smallest leaf",
    ] {
        assert!(
            invocation.contains(marker),
            "the recovery ladder did not escalate through a distinct correction ({marker}):\n{invocation}"
        );
    }
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

#[test]
fn corrective_retry_reuses_the_native_session_and_can_complete() {
    let directory = tmpdir();
    let workspace = directory.join("workspace");
    let home = directory.join("home");
    let orchestration_home = directory.join("orchestration-home");
    let bin_dir = directory.join("bin");
    for path in [&workspace, &home, &orchestration_home, &bin_dir] {
        std::fs::create_dir_all(path).expect("fixture directory");
    }
    assert!(Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&workspace)
        .status()
        .expect("initialize fixture repository")
        .success());
    write_effect_client(&bin_dir, "agent", 2);

    std::fs::write(directory.join("create-native-session"), "").expect("session sentinel");
    let output = run_client(
        &directory,
        &workspace,
        &home,
        &bin_dir,
        "agent",
        "Implement Hello World in Scala",
        &[
            "--orchestration",
            "--orchestration-home",
            orchestration_home.to_str().expect("UTF-8 path"),
        ],
    );
    assert!(
        output.status.success(),
        "corrective retry did not complete:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let completion = json_records(&output.stdout)
        .into_iter()
        .last()
        .expect("completion record");
    assert_eq!(completion["completion_state"], "complete");
    assert_eq!(completion["reason"], "workspace_effect_observed");
    assert_eq!(completion["attempts"], 2);
    assert_eq!(completion["rawMetadata"]["formalai"]["input_tokens"], 21);
    assert_eq!(completion["rawMetadata"]["formalai"]["output_tokens"], 8);
    assert!(workspace.join("Hello.scala").is_file());

    let capture =
        std::fs::read_to_string(directory.join("agent-capture.txt")).expect("captured invocations");
    assert!(
        capture.contains("<--resume> <ses_issue879> <--no-fork>"),
        "retry did not resume the native session:\n{capture}"
    );
    assert!(
        capture.contains("workspace_effect_present=true")
            && capture.contains("workspace_effect_count=0"),
        "retry did not include evidence disproving completion:\n{capture}"
    );
    let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn six_supported_coding_clients_share_the_completion_contract() {
    let clients = [
        (
            "agent",
            "--permission-mode",
            "http://127.0.0.1:8080/api/openai/v1",
        ),
        (
            "claude",
            "acceptEdits",
            "http://127.0.0.1:8080/api/anthropic",
        ),
        ("opencode", "--auto", "http://127.0.0.1:8080/api/openai/v1"),
        (
            "codex",
            "workspace-write",
            "http://127.0.0.1:8080/api/openai/v1",
        ),
        ("qwen", "auto-edit", "http://127.0.0.1:8080/api/openai/v1"),
        ("gemini", "auto_edit", "http://127.0.0.1:8080/api/gemini"),
    ];
    for (tool, editing_marker, endpoint) in clients {
        let directory = tmpdir();
        let workspace = directory.join("workspace");
        let home = directory.join("home");
        let bin_dir = directory.join("bin");
        for path in [&workspace, &home, &bin_dir] {
            std::fs::create_dir_all(path).expect("fixture directory");
        }
        assert!(Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&workspace)
            .status()
            .expect("initialize fixture repository")
            .success());
        write_effect_client(&bin_dir, tool, 1);

        let output = run_client(
            &directory,
            &workspace,
            &home,
            &bin_dir,
            tool,
            "Implement Hello World in Scala",
            &[],
        );
        assert!(
            output.status.success(),
            "{tool} did not satisfy the shared contract:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let completion = json_records(&output.stdout)
            .into_iter()
            .last()
            .expect("completion record");
        assert_eq!(completion["completion_state"], "complete", "{tool}");
        assert_eq!(completion["attempts"], 1, "{tool}");
        assert_eq!(
            completion["rawMetadata"]["formalai"]["endpoint"], endpoint,
            "{tool}"
        );
        assert_eq!(
            completion["rawMetadata"]["formalai"]["input_tokens"], 21,
            "{tool}"
        );
        assert_eq!(
            completion["rawMetadata"]["formalai"]["output_tokens"], 8,
            "{tool}"
        );
        let capture = std::fs::read_to_string(directory.join(format!("{tool}-capture.txt")))
            .expect("captured invocation");
        assert!(
            capture.contains(editing_marker),
            "{tool} did not enable its editing profile ({editing_marker}):\n{capture}"
        );
        assert!(
            capture.contains(endpoint),
            "{tool} did not receive the local endpoint ({endpoint}):\n{capture}"
        );
        assert!(
            workspace.join("Hello.scala").is_file(),
            "{tool} did not produce an artifact"
        );
        let status = Command::new("git")
            .args(["status", "--porcelain=v1", "--untracked-files=all"])
            .current_dir(&workspace)
            .output()
            .expect("inspect fixture repository");
        assert_eq!(
            String::from_utf8(status.stdout).expect("UTF-8 status"),
            "?? Hello.scala\n",
            "{tool} left wrapper scratch files"
        );
        let _ = std::fs::remove_dir_all(directory);
    }
}

#[test]
fn authoring_prompts_in_every_supported_language_require_workspace_effects() {
    let tasks = [
        ("en", "English", "Implement Hello World in Scala"),
        ("ru", "Russian", "Реализуй Hello World на Scala"),
        ("hi", "Hindi", "Scala में Hello World लागू करो"),
        ("zh", "Chinese", "用 Scala 实现 Hello World"),
    ];
    for (language, language_name, task) in tasks {
        let directory = tmpdir();
        let workspace = directory.join("workspace");
        let home = directory.join("home");
        let bin_dir = directory.join("bin");
        for path in [&workspace, &home, &bin_dir] {
            std::fs::create_dir_all(path).expect("fixture directory");
        }
        assert!(Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&workspace)
            .status()
            .expect("initialize fixture repository")
            .success());
        write_effect_client(&bin_dir, "agent", 1);

        let output = run_client(&directory, &workspace, &home, &bin_dir, "agent", task, &[]);
        assert!(
            output.status.success(),
            "{language_name} ({language}) authoring did not complete:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let completion = json_records(&output.stdout)
            .into_iter()
            .last()
            .expect("completion record");
        assert_eq!(
            completion["completion_state"], "complete",
            "{language_name} ({language})"
        );
        assert_eq!(
            completion["reason"], "workspace_effect_observed",
            "{language_name} ({language})"
        );
        assert_eq!(completion["attempts"], 1, "{language_name} ({language})");
        assert!(
            workspace.join("Hello.scala").is_file(),
            "{language_name} ({language}) authoring produced no artifact"
        );
        let _ = std::fs::remove_dir_all(directory);
    }
}

/// The self-learning half of the loop: a strategy that produced an artifact for
/// one client is tried first next time, and one that never did is tried last.
#[test]
fn recovery_order_is_learned_from_what_actually_produced_artifacts() {
    let directory = tmpdir();
    let bin_dir = directory.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("bin directory");
    let home = directory.join("home");
    std::fs::create_dir_all(&home).expect("home");

    let teaching_workspace = directory.join("teaching-workspace");
    std::fs::create_dir_all(&teaching_workspace).expect("teaching workspace");
    assert!(Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&teaching_workspace)
        .status()
        .expect("initialize fixture repository")
        .success());
    // Only the third attempt writes a file: the initial request and the first
    // recovery strategy fail, the second recovery strategy succeeds.
    write_effect_client(&bin_dir, "agent", 3);
    let teaching = run_client(
        &directory,
        &teaching_workspace,
        &home,
        &bin_dir,
        "agent",
        "Implement Hello World in Scala",
        &[],
    );
    assert!(
        teaching.status.success(),
        "the teaching run did not recover:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&teaching.stdout),
        String::from_utf8_lossy(&teaching.stderr)
    );
    let taught = json_records(&teaching.stdout)
        .into_iter()
        .last()
        .expect("completion record");
    assert_eq!(taught["completion_state"], "complete");
    assert_eq!(
        taught["recovery"]["strategies_spent"],
        serde_json::json!(["restate_postcondition", "name_target_artifact"]),
        "the teaching run must stop as soon as a strategy produces an effect"
    );

    let ledger = state_dir(&directory).join("formal-ai/completion-recovery.lino");
    let recorded = std::fs::read_to_string(&ledger).expect("durable recovery ledger");
    assert!(
        recorded.contains("restate_postcondition") && recorded.contains("name_target_artifact"),
        "the ledger did not record the (client, strategy, outcome) triples:\n{recorded}"
    );
    assert_eq!(
        taught["recovery"]["ledger"],
        Value::String(ledger.display().to_string()),
        "the completion record must name where the learning was persisted"
    );

    // A second run of the same client, with nothing left in the workspace to
    // find, must now start from what worked and end with what never has.
    let learning_workspace = directory.join("learning-workspace");
    std::fs::create_dir_all(&learning_workspace).expect("learning workspace");
    assert!(Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&learning_workspace)
        .status()
        .expect("initialize fixture repository")
        .success());
    std::fs::remove_file(directory.join("agent-attempts.txt")).expect("reset attempt counter");
    write_no_effect_agent(&bin_dir);
    let learned_run = run_client(
        &directory,
        &learning_workspace,
        &home,
        &bin_dir,
        "agent",
        "Implement Hello World in Scala",
        &[],
    );
    assert!(
        !learned_run.status.success(),
        "a run that produced nothing was still accepted"
    );
    let learned = json_records(&learned_run.stdout)
        .into_iter()
        .last()
        .expect("completion record");
    assert_eq!(
        learned["recovery"]["strategies_spent"],
        serde_json::json!([
            "name_target_artifact",
            "decompose_into_leaf",
            "restate_postcondition",
        ]),
        "learning must promote the strategy that worked, keep untried ones next, \
         and demote the one that never produced an effect"
    );
    assert_eq!(
        learned["recovery"]["strategies_available"],
        serde_json::json!([
            "restate_postcondition",
            "name_target_artifact",
            "decompose_into_leaf",
        ]),
        "learning may only reorder the seeded ladder, never add or drop a strategy"
    );

    // Defect 4: the durable learning state is never written into the caller's tree.
    for workspace in [&teaching_workspace, &learning_workspace] {
        let status = Command::new("git")
            .args(["status", "--porcelain=v1", "--untracked-files=all"])
            .current_dir(workspace)
            .output()
            .expect("inspect fixture repository");
        let status = String::from_utf8(status.stdout).expect("UTF-8 status");
        assert!(
            !status.contains("completion-recovery"),
            "learning state leaked into the workspace:\n{status}"
        );
    }
    let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn public_vendor_endpoint_diversion_fails_closed() {
    let directory = tmpdir();
    let workspace = directory.join("workspace");
    let home = directory.join("home");
    let bin_dir = directory.join("bin");
    for path in [&workspace, &home, &bin_dir] {
        std::fs::create_dir_all(path).expect("fixture directory");
    }
    assert!(Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&workspace)
        .status()
        .expect("initialize fixture repository")
        .success());
    let path = bin_dir.join("codex");
    std::fs::write(
        &path,
        r#"#!/bin/sh
printf 'object HelloWorld extends App {}\n' > Hello.scala
printf '{"type":"result","subtype":"success"}\n'
printf 'request failed: https://api.openai.com/v1/responses\n' >&2
"#,
    )
    .expect("write endpoint-diverting client");
    let mut permissions = std::fs::metadata(&path)
        .expect("client metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("make client executable");

    let output = run_client(
        &directory,
        &workspace,
        &home,
        &bin_dir,
        "codex",
        "Implement Hello World in Scala",
        &[],
    );
    assert!(!output.status.success(), "vendor endpoint diversion passed");
    let completion = json_records(&output.stdout)
        .into_iter()
        .last()
        .expect("completion record");
    assert_eq!(completion["completion_state"], "failed");
    assert_eq!(completion["reason"], "endpoint_mismatch");
    assert_eq!(completion["attempts"], 1);
    assert_eq!(
        completion["rawMetadata"]["formalai"]["actual_endpoint"],
        "https://api.openai.com"
    );
    let _ = std::fs::remove_dir_all(directory);
}
