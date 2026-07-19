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
            "formal-ai-opencode-vscode-{}-{seq}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("temp dir");
        path
    }

    fn fake_code(bin_dir: &Path) {
        let path = bin_dir.join("code");
        std::fs::write(
            &path,
            r#"#!/bin/sh
{
  i=0
  for arg in "$@"; do printf "arg[%s]=%s\n" "$i" "$arg"; i=$((i + 1)); done
  printf "FORMAL_AI_API_KEY=%s\n" "${FORMAL_AI_API_KEY-}"
  printf "OPENCODE_CONFIG=%s\n" "${OPENCODE_CONFIG-}"
  if [ -n "${OPENCODE_CONFIG-}" ] && [ -f "$OPENCODE_CONFIG" ]; then
    printf "%s\n" "---OPENCODE_CONFIG---"
    cat "$OPENCODE_CONFIG"
  fi
} > "$FORMAL_AI_CAPTURE"
"#,
        )
        .expect("fake code");
        let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("chmod");
    }

    fn run(home: &Path, bin_dir: &Path, capture: &Path, alias: &str) -> Output {
        let path = format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args([
                "with",
                "--no-start-server",
                "--base-url",
                "http://127.0.0.1:18080",
                alias,
            ])
            .env("HOME", home)
            .env("PATH", path)
            .env("FORMAL_AI_CAPTURE", capture)
            .env_remove("FORMAL_AI_API_KEY")
            .env_remove("OPENCODE_CONFIG")
            .output()
            .expect("run wrapper")
    }

    #[test]
    fn aliases_launch_vscode_with_isolated_opencode_provider() {
        for alias in ["opencode-vscode", "opencode-code"] {
            let root = temp_dir();
            let home = root.join("home");
            let bin_dir = root.join("bin");
            let capture = root.join("capture.txt");
            std::fs::create_dir_all(&home).expect("home");
            std::fs::create_dir_all(&bin_dir).expect("bin");
            fake_code(&bin_dir);

            let output = run(&home, &bin_dir, &capture, alias);
            assert!(
                output.status.success(),
                "{alias}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let captured = std::fs::read_to_string(&capture).expect("capture");
            assert!(captured.contains("arg[0]=--new-window"), "{captured}");
            assert!(captured.contains("arg[1]=--wait"), "{captured}");
            assert!(
                captured.contains("FORMAL_AI_API_KEY=formal-ai"),
                "{captured}"
            );
            assert!(captured.contains("OPENCODE_CONFIG="), "{captured}");
            assert!(captured.contains("---OPENCODE_CONFIG---"), "{captured}");
            assert!(captured.contains("\"formalai\""), "{captured}");
            assert!(
                captured.contains("\"baseURL\": \"http://127.0.0.1:18080/api/openai/v1\""),
                "{captured}"
            );
            assert!(
                captured.contains("\"model\": \"formalai/formal-ai\""),
                "{captured}"
            );
            assert!(!home.join(".config/opencode/opencode.json").exists());
            std::fs::remove_dir_all(root).expect("cleanup");
        }
    }

    #[test]
    fn global_setup_uses_shared_opencode_config_and_undo_restores_it() {
        let root = temp_dir();
        let home = root.join("home");
        let config_dir = home.join(".config/opencode");
        let config_path = config_dir.join("opencode.json");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        let original = "{\n  \"theme\": \"system\"\n}\n";
        std::fs::write(&config_path, original).expect("original config");

        let configured = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args([
                "with",
                "--global",
                "--base-url",
                "http://127.0.0.1:18080",
                "opencode-vscode",
            ])
            .env("HOME", &home)
            .output()
            .expect("global setup");
        assert!(
            configured.status.success(),
            "{}",
            String::from_utf8_lossy(&configured.stderr)
        );
        let config = std::fs::read_to_string(&config_path).expect("configured config");
        assert!(config.contains("\"theme\": \"system\""), "{config}");
        assert!(config.contains("\"formalai\""), "{config}");
        assert!(
            config.contains("\"baseURL\": \"http://127.0.0.1:18080/api/openai/v1\""),
            "{config}"
        );
        assert!(
            config.contains("\"model\": \"formalai/formal-ai\""),
            "{config}"
        );
        assert!(config_path.with_extension("json.formal-ai.bak").exists());

        let restored = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(["with", "--undo", "opencode-code"])
            .env("HOME", &home)
            .output()
            .expect("undo setup");
        assert!(
            restored.status.success(),
            "{}",
            String::from_utf8_lossy(&restored.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(&config_path).expect("restored config"),
            original
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn docs_and_matrix_cover_the_whole_extension_flow() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let seed = std::fs::read_to_string(root.join("data/seed/client-integrations.lino"))
            .expect("integration seed");
        let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
        let matrix = std::fs::read_to_string(root.join("docs/testing/agentic-cli-tools.md"))
            .expect("agentic client matrix");

        for expected in [
            "tool \"opencode-vscode\"",
            "aliases (\"opencode-code\")",
            "command \"code\"",
            "prepend_arg \"--new-window\"",
            "prepend_arg \"--wait\"",
            "config_env \"OPENCODE_CONFIG\"",
            "path \".config/opencode/opencode.json\"",
        ] {
            assert!(seed.contains(expected), "missing seed contract: {expected}");
        }
        for expected in [
            "sst-dev.opencode",
            "formal-ai with opencode-vscode",
            "opencode-code",
        ] {
            assert!(
                readme.contains(expected),
                "missing README contract: {expected}"
            );
        }
        for expected in [
            "OpenCode VS Code extension",
            "OPENCODE_CALLER=vscode",
            "at least one tool call/result round trip",
            "#671 matrix",
        ] {
            assert!(
                matrix.contains(expected),
                "missing matrix contract: {expected}"
            );
        }
        assert!(
            root.join("experiments/opencode_vscode_e2e/run.sh")
                .is_file(),
            "missing reproducible real-extension E2E harness"
        );
        assert!(
            root.join("experiments/opencode_vscode_e2e/driver/extension.cjs")
                .is_file(),
            "missing real extension command driver"
        );
    }
}
