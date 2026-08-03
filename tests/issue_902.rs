#[cfg(unix)]
mod unix {
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "formal-ai-codex-c-order-{}-{seq}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("temp dir");
        path
    }

    fn fake_codex(bin_dir: &Path) {
        let path = bin_dir.join("codex");
        std::fs::write(
            &path,
            r#"#!/bin/sh
{
  i=0
  for arg in "$@"; do printf "arg[%s]=%s\n" "$i" "$arg"; i=$((i + 1)); done
  printf "FORMAL_AI_API_KEY=%s\n" "${FORMAL_AI_API_KEY-}"
  printf "HOME=%s\n" "$HOME"
  if [ -f "$HOME/.codex/config.toml" ]; then
    printf "%s\n" "---CODEX_CONFIG---"
    cat "$HOME/.codex/config.toml"
  fi
  if [ -f "$HOME/.codex/formal-ai-model-catalog.json" ]; then
    printf "%s\n" "---CODEX_MODEL_CATALOG---"
    cat "$HOME/.codex/formal-ai-model-catalog.json"
  fi
} > "$FORMAL_AI_CAPTURE"
"#,
        )
        .expect("fake codex");
        let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("chmod");
    }

    fn run(home: &Path, bin_dir: &Path, capture: &Path) -> Output {
        let path = format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args([
                "with",
                "--no-start-server",
                "--non-interactive",
                "--base-url",
                "http://127.0.0.1:18080",
                "codex",
                "-c",
                "model_reasoning_effort=none",
                "-c",
                "model_reasoning_summary=auto",
                "hi",
            ])
            .env("HOME", home)
            .env("PATH", path)
            .env("FORMAL_AI_CAPTURE", capture)
            .env_remove("FORMAL_AI_API_KEY")
            .output()
            .expect("run wrapper")
    }

    fn captured_args(capture: &str) -> Vec<&str> {
        capture
            .lines()
            .filter_map(|line| {
                line.strip_prefix("arg[")?
                    .split_once("]=")
                    .map(|(_, arg)| arg)
            })
            .collect()
    }

    #[test]
    fn codex_provider_config_survives_caller_overrides() {
        let root = temp_dir();
        let home = root.join("home");
        let bin_dir = root.join("bin");
        let capture = root.join("capture.txt");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        fake_codex(&bin_dir);

        let output = run(&home, &bin_dir, &capture);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let captured = std::fs::read_to_string(&capture).expect("capture");
        assert_eq!(
            captured_args(&captured),
            [
                "exec",
                "--skip-git-repo-check",
                "--sandbox",
                "read-only",
                "-c",
                "model_reasoning_effort=none",
                "-c",
                "model_reasoning_summary=auto",
                "hi",
            ],
            "{captured}"
        );
        assert!(
            captured.contains("FORMAL_AI_API_KEY=formal-ai"),
            "{captured}"
        );
        assert!(captured.contains("---CODEX_CONFIG---"), "{captured}");
        assert!(
            captured.contains("model_provider = \"formalai\""),
            "{captured}"
        );
        assert!(captured.contains("model = \"formal-ai\""), "{captured}");
        assert!(captured.contains("model_catalog_json = "), "{captured}");
        assert!(
            captured.contains("[model_providers.formalai]"),
            "{captured}"
        );
        assert!(
            captured.contains("name = \"formal-ai server\""),
            "{captured}"
        );
        assert!(
            captured.contains("base_url = \"http://127.0.0.1:18080/api/openai/v1\""),
            "{captured}"
        );
        assert!(
            captured.contains("env_key = \"FORMAL_AI_API_KEY\""),
            "{captured}"
        );
        assert!(captured.contains("wire_api = \"responses\""), "{captured}");
        assert!(captured.contains("---CODEX_MODEL_CATALOG---"), "{captured}");
        assert!(captured.contains("\"slug\": \"formal-ai\""), "{captured}");
        assert!(!home.join(".codex/config.toml").exists());
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
