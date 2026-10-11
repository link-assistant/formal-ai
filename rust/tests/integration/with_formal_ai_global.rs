//! Rung `R916-08` of the issue #916 write-effect ladder: what `formal-ai with
//! --global` leaves on disk, and whether `--undo` takes all of it back.
//!
//! These live apart from `with_formal_ai.rs` only because that file is at the
//! repository's 1000-line ceiling for Rust sources.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir() -> PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-with-global-{}-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

/// Rung `R916-08` of the issue #916 write-effect ladder: `--global` has to leave
/// a configuration the client can actually start from, and `--undo` has to take
/// all of it back.
///
/// Issue #909 recorded both halves failing. gemini-cli reads
/// `GEMINI_DEFAULT_AUTH_TYPE` as a *default* and still exited with `Invalid auth
/// method selected.` because the selection was never recorded in its settings
/// file; qwen-code prompted for a provider because `OPENAI_MODEL` was missing
/// from the triple it checks. `--global` reported success in both cases.
#[test]
fn with_formal_ai_global_writes_a_configuration_the_client_can_start_from() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home dir");

    for tool in ["gemini", "qwen"] {
        let configure = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            // `--global` has to precede the tool name: everything after it is
            // forwarded to the client verbatim (`trailing_var_arg`).
            .args([
                "with",
                "--global",
                "--base-url",
                "http://127.0.0.1:18080",
                tool,
            ])
            .env("HOME", &home)
            .output()
            .expect("global configure");
        assert!(
            configure.status.success(),
            "{tool} stderr: {}",
            String::from_utf8_lossy(&configure.stderr)
        );
    }

    // gemini: the auth type is selected in the settings file, not merely
    // defaulted in the environment.
    let settings =
        std::fs::read_to_string(home.join(".gemini/settings.json")).expect("gemini settings");
    let settings: serde_json::Value =
        serde_json::from_str(&settings).expect("gemini settings json");
    assert_eq!(
        settings["security"]["auth"]["selectedType"],
        serde_json::json!("gemini-api-key"),
        "gemini-cli starts headlessly only from a recorded selection: {settings}"
    );
    assert!(home.join(".gemini/settings.json.formal-ai.bak").exists());

    // qwen: the OpenAI auth path is selected only from the complete triple.
    let profile = std::fs::read_to_string(home.join(".profile")).expect("profile");
    for expected in [
        "OPENAI_API_KEY=",
        "OPENAI_BASE_URL=\"http://127.0.0.1:18080/api/openai/v1\"",
        "OPENAI_MODEL=\"formal-ai\"",
        "GEMINI_DEFAULT_AUTH_TYPE=\"gemini-api-key\"",
    ] {
        assert!(
            profile.contains(expected),
            "missing {expected} in {profile}"
        );
    }

    let undo = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--global", "--undo", "--all"])
        .env("HOME", &home)
        .output()
        .expect("global undo");
    assert!(
        undo.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&undo.stderr)
    );
    assert!(
        !home.join(".gemini/settings.json").exists(),
        "a companion file `--global` created must be taken back by `--undo`"
    );
    assert!(!home.join(".profile").exists());

    let _ = std::fs::remove_dir_all(&dir);
}

/// PR #1188 dogfooding: the Agent CLI compacted every session at its built-in
/// 60000-token guess for `formal-ai`, losing the work in progress, because no
/// client config stated the window Formal AI serves on `/v1/models`. Every
/// opencode-shaped client config `formal-ai with` writes now states the model's
/// limits: the served window (here, with no server listening, this host's
/// capacity, which is what a server here would serve) and the served output
/// cap.
#[test]
fn with_formal_ai_global_states_the_served_model_limits() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home dir");
    let configure = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--base-url",
            "http://127.0.0.1:18081",
            "agent",
        ])
        .env("HOME", &home)
        .env("FORMAL_AI_MEMORY_PATH", dir.join("memory.lino"))
        .output()
        .expect("global configure");
    assert!(
        configure.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&configure.stderr)
    );
    let config = std::fs::read_to_string(home.join(".config/link-assistant-agent/opencode.json"))
        .expect("agent config");
    let config: serde_json::Value = serde_json::from_str(&config).expect("agent config json");
    let limit = &config["provider"]["formalai"]["models"]["formal-ai"]["limit"];
    assert_eq!(limit["output"], serde_json::json!(8192), "{config}");
    assert!(
        limit["context"]
            .as_u64()
            .is_some_and(|window| window > 60_000),
        "the served window, not the client's 60000-token guess: {config}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
