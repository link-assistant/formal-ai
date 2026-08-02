#[cfg(unix)]
mod unix {
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("formal-ai-t3code-{}-{seq}", std::process::id()));
        std::fs::create_dir_all(&path).expect("temp dir");
        path
    }

    fn fake_t3(bin_dir: &Path) {
        let path = bin_dir.join("t3");
        std::fs::write(
            &path,
            r#"#!/bin/sh
{
  i=0
  for arg in "$@"; do printf "arg[%s]=%s\n" "$i" "$arg"; i=$((i + 1)); done
  printf "FORMAL_AI_API_KEY=%s\n" "${FORMAL_AI_API_KEY-}"
  printf "ANTHROPIC_AUTH_TOKEN=%s\n" "${ANTHROPIC_AUTH_TOKEN-}"
  printf "ANTHROPIC_API_KEY=%s\n" "${ANTHROPIC_API_KEY-}"
  printf "ANTHROPIC_BASE_URL=%s\n" "${ANTHROPIC_BASE_URL-}"
  printf "CODEX_HOME=%s\n" "${CODEX_HOME-}"
  if [ -n "${CODEX_HOME-}" ] && [ -f "$CODEX_HOME/config.toml" ]; then
    printf "%s\n" "---CODEX_CONFIG---"
    cat "$CODEX_HOME/config.toml"
  fi
} > "$FORMAL_AI_CAPTURE"
"#,
        )
        .expect("fake t3");
        let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("chmod");
    }

    fn run(home: &Path, bin_dir: &Path, capture: &Path, args: &[&str]) -> Output {
        let path = format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(args)
            .env("HOME", home)
            .env("PATH", path)
            .env("FORMAL_AI_CAPTURE", capture)
            .env_remove("FORMAL_AI_API_KEY")
            .env_remove("ANTHROPIC_AUTH_TOKEN")
            .env_remove("ANTHROPIC_API_KEY")
            .env_remove("ANTHROPIC_BASE_URL")
            .env_remove("CODEX_HOME")
            .output()
            .expect("run wrapper")
    }

    #[test]
    fn aliases_launch_t3_with_isolated_openai_provider() {
        for alias in ["t3code", "t3"] {
            let root = temp_dir();
            let home = root.join("home");
            let bin_dir = root.join("bin");
            let capture = root.join("capture.txt");
            std::fs::create_dir_all(&home).expect("home");
            std::fs::create_dir_all(&bin_dir).expect("bin");
            fake_t3(&bin_dir);
            let output = run(
                &home,
                &bin_dir,
                &capture,
                &[
                    "with",
                    "--no-start-server",
                    "--base-url",
                    "http://127.0.0.1:18080",
                    alias,
                ],
            );
            assert!(
                output.status.success(),
                "{alias}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let captured = std::fs::read_to_string(&capture).expect("capture");
            assert!(
                captured.contains("FORMAL_AI_API_KEY=formal-ai"),
                "{captured}"
            );
            assert!(captured.contains("CODEX_HOME="), "{captured}");
            assert!(captured.contains("---CODEX_CONFIG---"), "{captured}");
            assert!(
                captured.contains("model_provider = \"formalai\""),
                "{captured}"
            );
            assert!(captured.contains("model = \"formal-ai\""), "{captured}");
            assert!(
                captured.contains("base_url = \"http://127.0.0.1:18080/api/openai/v1\""),
                "{captured}"
            );
            assert!(
                captured.contains("env_key = \"FORMAL_AI_API_KEY\""),
                "{captured}"
            );
            assert!(!home.join(".codex/config.toml").exists());
            std::fs::remove_dir_all(root).expect("cleanup");
        }
    }

    #[test]
    fn anthropic_protocol_supplies_local_claude_auth() {
        let root = temp_dir();
        let home = root.join("home");
        let bin_dir = root.join("bin");
        let capture = root.join("capture.txt");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        fake_t3(&bin_dir);
        let output = run(
            &home,
            &bin_dir,
            &capture,
            &[
                "with",
                "--no-start-server",
                "--protocol",
                "anthropic",
                "t3code",
            ],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let captured = std::fs::read_to_string(&capture).expect("capture");
        assert!(
            captured.contains("ANTHROPIC_AUTH_TOKEN=formal-ai"),
            "{captured}"
        );
        assert!(captured.contains("ANTHROPIC_API_KEY="), "{captured}");
        assert!(
            captured.contains("ANTHROPIC_BASE_URL=http://127.0.0.1:8080/api/anthropic"),
            "{captured}"
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn launch_mode_controls_browser_opening() {
        let root = temp_dir();
        let home = root.join("home");
        let bin_dir = root.join("bin");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        fake_t3(&bin_dir);
        let interactive_capture = root.join("interactive.txt");
        let interactive = run(
            &home,
            &bin_dir,
            &interactive_capture,
            &["with", "--no-start-server", "--interactive", "t3"],
        );
        assert!(
            interactive.status.success(),
            "{}",
            String::from_utf8_lossy(&interactive.stderr)
        );
        assert!(!std::fs::read_to_string(&interactive_capture)
            .expect("interactive capture")
            .contains("--no-browser"));
        let headless_capture = root.join("headless.txt");
        let headless = run(
            &home,
            &bin_dir,
            &headless_capture,
            &["with", "--no-start-server", "--non-interactive", "t3code"],
        );
        assert!(
            headless.status.success(),
            "{}",
            String::from_utf8_lossy(&headless.stderr)
        );
        assert!(std::fs::read_to_string(&headless_capture)
            .expect("headless capture")
            .contains("arg[0]=--no-browser"));
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn global_and_all_setup_include_t3code_for_both_protocols() {
        let root = temp_dir();
        let t3_home = root.join("t3-home");
        std::fs::create_dir_all(t3_home.join(".codex")).expect("t3 home");
        let original = "approval_policy = \"never\"\n";
        std::fs::write(t3_home.join(".codex/config.toml"), original).expect("original config");
        let global = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(["with", "--global", "t3code"])
            .env("HOME", &t3_home)
            .output()
            .expect("global t3");
        assert!(
            global.status.success(),
            "{}",
            String::from_utf8_lossy(&global.stderr)
        );
        let config = std::fs::read_to_string(t3_home.join(".codex/config.toml")).expect("config");
        assert!(config.contains("approval_policy = \"never\""), "{config}");
        assert!(config.contains("model_provider = \"formalai\""), "{config}");
        assert!(
            config.contains("base_url = \"http://127.0.0.1:8080/api/openai/v1\""),
            "{config}"
        );
        assert!(t3_home.join(".codex/config.toml.formal-ai.bak").exists());
        let undo = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(["with", "--undo", "t3"])
            .env("HOME", &t3_home)
            .output()
            .expect("undo t3");
        assert!(
            undo.status.success(),
            "{}",
            String::from_utf8_lossy(&undo.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(t3_home.join(".codex/config.toml")).expect("restored config"),
            original
        );

        let home = root.join("home");
        std::fs::create_dir_all(&home).expect("home");

        let all = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(["with", "--global", "--all"])
            .env("HOME", &home)
            .output()
            .expect("global all");
        assert!(
            all.status.success(),
            "{}",
            String::from_utf8_lossy(&all.stderr)
        );
        assert!(
            String::from_utf8_lossy(&all.stdout).contains("t3code already configured"),
            "{}",
            String::from_utf8_lossy(&all.stdout)
        );

        let anthropic = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(["with", "--global", "--protocol", "anthropic", "t3"])
            .env("HOME", &home)
            .output()
            .expect("global anthropic");
        assert!(
            anthropic.status.success(),
            "{}",
            String::from_utf8_lossy(&anthropic.stderr)
        );
        let profile = std::fs::read_to_string(home.join(".profile")).expect("profile");
        assert!(profile.contains("# >>> formal-ai t3code"), "{profile}");
        assert!(
            profile.contains("ANTHROPIC_BASE_URL=\"http://127.0.0.1:8080/api/anthropic\""),
            "{profile}"
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
