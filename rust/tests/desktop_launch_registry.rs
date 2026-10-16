//! Issue #762: The desktop registry has cross-platform launch metadata and an isolated shared provider configuration.

#[test]
fn desktop_registry_has_cross_platform_launch_metadata() {
    let integrations = formal_ai::seed::client_integrations();
    let desktop = integrations
        .iter()
        .find(|integration| integration.id == "opencode-desktop")
        .expect("OpenCode Desktop integration");
    assert_eq!(desktop.command_env, "FORMAL_AI_OPENCODE_DESKTOP_BIN");
    assert_eq!(desktop.command, "opencode-desktop");
    assert!(
        desktop
            .platform_commands
            .iter()
            .any(|candidate| candidate.key == "linux"
                && candidate.value == "/opt/OpenCode/ai.opencode.desktop")
    );
    assert!(
        desktop
            .platform_commands
            .iter()
            .any(|candidate| candidate.key == "macos"
                && candidate.value == "/Applications/OpenCode.app/Contents/MacOS/OpenCode")
    );
    assert!(
        desktop
            .platform_commands
            .iter()
            .any(|candidate| candidate.key == "windows"
                && candidate.value == "{local_app_data}/Programs/OpenCode/OpenCode.exe")
    );
}

#[cfg(unix)]
mod unix {
    use serde_json::Value;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn sandbox() -> PathBuf {
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "formal-ai-opencode-desktop-{}-{seq}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("sandbox");
        path
    }

    fn fake_desktop(root: &Path) -> PathBuf {
        let executable = root.join("fake-opencode-desktop");
        std::fs::write(
            &executable,
            r#"#!/bin/sh
{
  printf 'arg_count=%s\n' "$#"
  printf 'FORMAL_AI_API_KEY=%s\n' "${FORMAL_AI_API_KEY-}"
  printf 'OPENCODE_CONFIG=%s\n' "${OPENCODE_CONFIG-}"
  printf 'OPENCODE_CONFIG_DIR=%s\n' "${OPENCODE_CONFIG_DIR-}"
  printf '%s\n' '---CONFIG---'
  if [ -n "${OPENCODE_CONFIG-}" ] && [ -f "$OPENCODE_CONFIG" ]; then
    cat "$OPENCODE_CONFIG"
  fi
} > "$FORMAL_AI_CAPTURE"
"#,
        )
        .expect("fake desktop");
        let mut permissions = std::fs::metadata(&executable)
            .expect("metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&executable, permissions).expect("chmod");
        executable
    }

    fn run(home: &Path, capture: &Path, desktop: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(args)
            .env("HOME", home)
            .env("FORMAL_AI_CAPTURE", capture)
            .env("FORMAL_AI_OPENCODE_DESKTOP_BIN", desktop)
            .env_remove("FORMAL_AI_API_KEY")
            .env_remove("OPENCODE_CONFIG")
            .env_remove("OPENCODE_CONFIG_DIR")
            .output()
            .expect("formal-ai with")
    }

    #[test]
    fn desktop_launch_uses_an_isolated_shared_provider_config() {
        let root = sandbox();
        let home = root.join("home");
        let capture = root.join("capture.txt");
        std::fs::create_dir_all(&home).expect("home");
        let desktop = fake_desktop(&root);

        let output = run(
            &home,
            &capture,
            &desktop,
            &[
                "with",
                "--no-start-server",
                "--base-url",
                "http://127.0.0.1:18080",
                "opencode-desktop",
            ],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let captured = std::fs::read_to_string(&capture).expect("capture");
        assert!(captured.contains("arg_count=0"), "{captured}");
        assert!(
            captured.contains("FORMAL_AI_API_KEY=formal-ai-local"),
            "{captured}"
        );
        let config_path = captured
            .lines()
            .find_map(|line| line.strip_prefix("OPENCODE_CONFIG="))
            .expect("OPENCODE_CONFIG");
        let config_dir = captured
            .lines()
            .find_map(|line| line.strip_prefix("OPENCODE_CONFIG_DIR="))
            .expect("OPENCODE_CONFIG_DIR");
        assert!(!config_path.is_empty(), "{captured}");
        assert!(!config_dir.is_empty(), "{captured}");
        assert!(Path::new(config_path).starts_with(config_dir), "{captured}");

        let json: Value = serde_json::from_str(
            captured
                .split_once("---CONFIG---\n")
                .expect("config marker")
                .1,
        )
        .expect("provider JSON");
        assert_eq!(json["model"], "formalai/formal-ai");
        assert_eq!(
            json["provider"]["formalai"]["npm"],
            "@ai-sdk/openai-compatible"
        );
        assert_eq!(
            json["provider"]["formalai"]["options"]["baseURL"],
            "http://127.0.0.1:18080/api/openai/v1"
        );
        assert_eq!(
            json["provider"]["formalai"]["options"]["apiKey"],
            "{env:FORMAL_AI_API_KEY}"
        );
        assert_eq!(
            json["provider"]["formalai"]["models"]["formal-ai"]["name"],
            "formal-ai"
        );
        assert!(!home.join(".config/opencode/opencode.json").exists());
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn permanent_desktop_setup_participates_in_all_and_undoes_exactly() {
        let root = sandbox();
        let home = root.join("home");
        let capture = root.join("capture.txt");
        let config = home.join(".config/opencode/opencode.json");
        std::fs::create_dir_all(config.parent().expect("parent")).expect("config dir");
        let original = "{\"theme\":\"dark\"}\n";
        std::fs::write(&config, original).expect("original");
        let desktop = fake_desktop(&root);

        let configured = run(
            &home,
            &capture,
            &desktop,
            &["with", "--global", "opencode-desktop"],
        );
        assert!(
            configured.status.success(),
            "{}",
            String::from_utf8_lossy(&configured.stderr)
        );
        let json: Value =
            serde_json::from_str(&std::fs::read_to_string(&config).expect("configured JSON"))
                .expect("configured object");
        assert_eq!(json["theme"], "dark");
        assert_eq!(json["model"], "formalai/formal-ai");
        assert_eq!(
            json["provider"]["formalai"]["options"]["apiKey"],
            "{env:FORMAL_AI_API_KEY}"
        );
        assert_eq!(
            std::fs::read_to_string(config.with_extension("json.formal-ai.bak")).expect("backup"),
            original
        );

        let all = run(&home, &capture, &desktop, &["with", "--global", "--all"]);
        assert!(
            all.status.success(),
            "{}",
            String::from_utf8_lossy(&all.stderr)
        );
        assert!(
            String::from_utf8_lossy(&all.stdout).contains("opencode-desktop already configured"),
            "{}",
            String::from_utf8_lossy(&all.stdout)
        );

        assert!(
            String::from_utf8_lossy(&all.stdout).contains("opencode-vscode already configured"),
            "shared OpenCode clients must reuse one limit snapshot: {}",
            String::from_utf8_lossy(&all.stdout)
        );
        assert_eq!(
            std::fs::read_to_string(config.with_extension("json.formal-ai.bak")).expect("backup"),
            original,
            "the shared batch must preserve the original backup"
        );

        let undone = run(
            &home,
            &capture,
            &desktop,
            &["with", "--undo", "opencode-desktop"],
        );
        assert!(
            undone.status.success(),
            "{}",
            String::from_utf8_lossy(&undone.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(&config).expect("restored"),
            original
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn one_batch_shares_served_limits_and_the_next_invocation_reprobes() {
        use std::io::{Read as _, Write as _};
        use std::net::TcpListener;
        use std::sync::Arc;
        use std::sync::atomic::AtomicUsize;
        use std::time::{Duration, Instant};

        let root = sandbox();
        let home = root.join("home");
        let config = home.join(".config/opencode/opencode.json");
        std::fs::create_dir_all(config.parent().expect("parent")).expect("config directory");
        let original = "{\"theme\":\"dark\"}\n";
        std::fs::write(&config, original).expect("original config");
        let capture = root.join("capture.txt");
        let desktop = fake_desktop(&root);
        let listener = TcpListener::bind("127.0.0.1:0").expect("served-limit fixture");
        listener.set_nonblocking(true).expect("bounded listener");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let requests = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&requests);
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(15);
            while observed.load(Ordering::SeqCst) < 2 && Instant::now() < deadline {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("limit fixture accept: {error}"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .expect("read bound");
                stream
                    .set_write_timeout(Some(Duration::from_secs(1)))
                    .expect("write bound");
                let mut request = [0_u8; 4096];
                let count = stream.read(&mut request).expect("models request");
                assert!(String::from_utf8_lossy(&request[..count]).starts_with("GET /v1/models "));
                let ordinal = observed.fetch_add(1, Ordering::SeqCst) + 1;
                let body = serde_json::json!({
                    "models": [{"context_window_tokens": ordinal * 1000, "max_output_tokens": 100}]
                })
                .to_string();
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .expect("models response");
            }
        });
        let all = run(
            &home,
            &capture,
            &desktop,
            &["with", "--global", "--all", "--base-url", &base],
        );
        let first_requests = requests.load(Ordering::SeqCst);
        let first: Value =
            serde_json::from_str(&std::fs::read_to_string(&config).expect("batch config"))
                .expect("batch JSON");
        let refreshed = run(
            &home,
            &capture,
            &desktop,
            &["with", "--global", "opencode-desktop", "--base-url", &base],
        );
        server.join().expect("bounded model server");
        assert!(
            all.status.success(),
            "{}",
            String::from_utf8_lossy(&all.stderr)
        );
        assert!(
            refreshed.status.success(),
            "{}",
            String::from_utf8_lossy(&refreshed.stderr)
        );
        assert_eq!(first_requests, 1, "one lazy served-limit probe per batch");
        assert_eq!(
            requests.load(Ordering::SeqCst),
            2,
            "a new invocation probes again"
        );
        let first_limit = &first["provider"]["formalai"]["models"]["formal-ai"]["limit"];
        assert_eq!(first_limit["context"], 1000);
        assert_eq!(first_limit["output"], 100);
        for client in ["opencode-vscode", "opencode-desktop"] {
            assert!(
                String::from_utf8_lossy(&all.stdout)
                    .contains(&format!("{client} already configured"))
            );
        }
        let next: Value =
            serde_json::from_str(&std::fs::read_to_string(&config).expect("fresh config"))
                .expect("fresh JSON");
        assert_eq!(
            next["provider"]["formalai"]["models"]["formal-ai"]["limit"]["context"],
            2000
        );
        assert_eq!(
            std::fs::read_to_string(config.with_extension("json.formal-ai.bak")).expect("backup"),
            original
        );
        let undone = run(
            &home,
            &capture,
            &desktop,
            &["with", "--undo", "opencode-desktop"],
        );
        assert!(
            undone.status.success(),
            "{}",
            String::from_utf8_lossy(&undone.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(&config).expect("restored config"),
            original
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
