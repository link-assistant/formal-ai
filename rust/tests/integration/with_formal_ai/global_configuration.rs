use std::process::Command;

use super::tmpdir;

pub(super) fn with_formal_ai_global_configures_idempotently_and_undo_restores_backups() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(home.join(".codex")).expect("codex dir");
    std::fs::write(
        home.join(".codex/config.toml"),
        "approval_policy = \"never\"\n",
    )
    .expect("seed codex config");
    std::fs::write(
        home.join(".codex/formal-ai-model-catalog.json"),
        "user-managed catalog\n",
    )
    .expect("seed codex catalog");

    let first = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--base-url",
            "http://127.0.0.1:18080",
            "--all",
        ])
        .env("HOME", &home)
        .output()
        .expect("global configure");
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let codex_config = std::fs::read_to_string(home.join(".codex/config.toml")).expect("codex");
    assert!(codex_config.contains("approval_policy = \"never\""));
    assert!(codex_config.contains("model_provider = \"formalai\""));
    assert!(codex_config.contains("model = \"formal-ai\""));
    assert!(codex_config.contains("model_catalog_json = "));
    assert!(codex_config.contains("[model_providers.formalai]"));
    assert!(codex_config.contains("base_url = \"http://127.0.0.1:18080/api/openai/v1\""));
    assert!(home.join(".codex/config.toml.formal-ai.bak").exists());
    let codex_catalog = std::fs::read_to_string(home.join(".codex/formal-ai-model-catalog.json"))
        .expect("codex catalog");
    assert!(codex_catalog.contains("\"slug\": \"formal-ai\""));
    assert!(codex_catalog.contains("\"context_window\":"));
    assert!(codex_catalog.contains("\"context_used_tokens\":"));
    assert!(codex_catalog.contains("\"disk_free_bytes\":"));
    assert!(!codex_catalog.contains("\"context_window\": 60000"));
    assert!(
        home.join(".codex/formal-ai-model-catalog.json.formal-ai.bak")
            .exists()
    );

    let opencode_config =
        std::fs::read_to_string(home.join(".config/opencode/opencode.json")).expect("opencode");
    assert!(opencode_config.contains("\"formalai\""));
    assert!(opencode_config.contains("\"model\": \"formalai/formal-ai\""));
    assert!(
        home.join(".config/opencode/opencode.json.formal-ai.bak")
            .exists()
    );

    let agent_config =
        std::fs::read_to_string(home.join(".config/link-assistant-agent/opencode.json"))
            .expect("agent");
    assert!(agent_config.contains("\"formalai\""));
    assert!(agent_config.contains("\"model\": \"formalai/formal-ai\""));
    assert!(
        home.join(".config/link-assistant-agent/opencode.json.formal-ai.bak")
            .exists()
    );

    let profile = std::fs::read_to_string(home.join(".profile")).expect("profile");
    assert_eq!(profile.matches("formal-ai gemini").count(), 2);
    assert!(profile.contains("GOOGLE_GEMINI_BASE_URL=\"http://127.0.0.1:18080/api/gemini\""));
    assert!(home.join(".profile.formal-ai.bak").exists());

    let second = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "-g",
            "--base-url",
            "http://127.0.0.1:18080",
            "--all",
        ])
        .env("HOME", &home)
        .output()
        .expect("global configure again");
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let profile_again = std::fs::read_to_string(home.join(".profile")).expect("profile again");
    assert_eq!(profile_again.matches("formal-ai gemini").count(), 2);

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
    assert_eq!(
        std::fs::read_to_string(home.join(".codex/config.toml")).expect("restored codex"),
        "approval_policy = \"never\"\n"
    );
    assert_eq!(
        std::fs::read_to_string(home.join(".codex/formal-ai-model-catalog.json"))
            .expect("restored codex catalog"),
        "user-managed catalog\n"
    );
    assert!(
        !home
            .join(".codex/formal-ai-model-catalog.json.formal-ai.bak")
            .exists()
    );
    assert!(!home.join(".config/opencode/opencode.json").exists());
    assert!(
        !home
            .join(".config/link-assistant-agent/opencode.json")
            .exists()
    );
    assert!(!home.join(".profile").exists());

    let _ = std::fs::remove_dir_all(&dir);
}
