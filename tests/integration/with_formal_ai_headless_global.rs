//! Issue #909: `--global` must write every file a headless start needs, and
//! must not report success when the client still cannot start.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

const BASE_URL: &str = "http://127.0.0.1:18080";

fn tmpdir() -> PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-headless-global-{}-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

fn configure_globally(home: &Path, tool: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--global", "--base-url", BASE_URL, tool])
        .env("HOME", home)
        .output()
        .expect("global configure")
}

/// Write a `gemini` stand-in that prints `output` and place it first on `PATH`.
fn fake_cli(bin_dir: &Path, name: &str, output: &str) -> String {
    std::fs::create_dir_all(bin_dir).expect("bin dir");
    let path = bin_dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\necho \"{output}\"\n")).expect("write fake cli");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

#[test]
fn global_gemini_writes_the_settings_file_that_selects_an_auth_type() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home");

    let output = configure_globally(&home, "gemini");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The exports alone never made gemini start: the auth type counts as
    // *selected* only when a settings file says so.
    let profile = std::fs::read_to_string(home.join(".profile")).expect("profile");
    assert!(profile.contains("export GEMINI_API_KEY="));
    assert!(
        profile.contains("export GOOGLE_GEMINI_BASE_URL=\"http://127.0.0.1:18080/api/gemini\"")
    );

    let settings =
        std::fs::read_to_string(home.join(".gemini/settings.json")).expect("gemini settings");
    let parsed: serde_json::Value = serde_json::from_str(&settings).expect("settings json");
    assert_eq!(
        parsed["security"]["auth"]["selectedType"].as_str(),
        Some("gemini-api-key")
    );
    assert!(home.join(".gemini/settings.json.formal-ai.bak").exists());

    // Configuring twice must not report a change the second time.
    let again = configure_globally(&home, "gemini");
    assert!(again.status.success());
    let stdout = String::from_utf8_lossy(&again.stdout).into_owned();
    assert!(
        stdout.contains("gemini already configured at") && stdout.contains("settings.json"),
        "stdout: {stdout}"
    );

    let undo = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--undo", "gemini"])
        .env("HOME", &home)
        .output()
        .expect("global undo");
    assert!(
        undo.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&undo.stderr)
    );
    // The settings file did not exist before `--global`, so an exact undo
    // removes it rather than leaving formal-ai's version behind.
    assert!(!home.join(".gemini/settings.json").exists());
    assert!(!home.join(".gemini/settings.json.formal-ai.bak").exists());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn global_gemini_preserves_and_restores_an_existing_settings_file() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(home.join(".gemini")).expect("gemini dir");
    std::fs::write(
        home.join(".gemini/settings.json"),
        "{\n  \"ui\": {\n    \"theme\": \"ansi\"\n  }\n}\n",
    )
    .expect("seed settings");

    assert!(configure_globally(&home, "gemini").status.success());
    let merged: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(home.join(".gemini/settings.json")).expect("settings"),
    )
    .expect("settings json");
    assert_eq!(merged["ui"]["theme"].as_str(), Some("ansi"));
    assert_eq!(
        merged["security"]["auth"]["selectedType"].as_str(),
        Some("gemini-api-key")
    );

    let undo = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--undo", "gemini"])
        .env("HOME", &home)
        .output()
        .expect("global undo");
    assert!(undo.status.success());
    assert_eq!(
        std::fs::read_to_string(home.join(".gemini/settings.json")).expect("restored settings"),
        "{\n  \"ui\": {\n    \"theme\": \"ansi\"\n  }\n}\n"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// The slice of a shell profile that `tool`'s managed block owns.
fn managed_block<'a>(profile: &'a str, tool: &str) -> &'a str {
    let start = profile
        .find(&format!("# >>> formal-ai {tool}\n"))
        .unwrap_or_else(|| panic!("{tool} managed block start in:\n{profile}"));
    let end = profile
        .find(&format!("# <<< formal-ai {tool}\n"))
        .unwrap_or_else(|| panic!("{tool} managed block end in:\n{profile}"));
    &profile[start..end]
}

#[test]
fn global_qwen_writes_the_complete_openai_triple() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home");

    // An export the operator already owns must not stand in for a managed one:
    // the readback only trusts what it finds inside formal-ai's own block.
    std::fs::write(home.join(".profile"), "export OPENAI_MODEL=\"operator\"\n").expect("profile");

    assert!(configure_globally(&home, "qwen").status.success());

    // qwen selects its OpenAI auth path only when all three are present.
    let profile = std::fs::read_to_string(home.join(".profile")).expect("profile");
    let managed = managed_block(&profile, "qwen");
    assert!(managed.contains("export OPENAI_API_KEY="));
    assert!(managed.contains("export OPENAI_BASE_URL=\"http://127.0.0.1:18080/api/openai/v1\""));
    assert!(managed.contains("export OPENAI_MODEL=\"formal-ai\""));
    // The operator's line is left untouched outside the block.
    assert!(profile.starts_with("export OPENAI_MODEL=\"operator\"\n"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_fails_when_the_configured_client_refuses_to_start() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home");
    let path = fake_cli(&dir.join("bin"), "gemini", "Invalid auth method selected.");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--verify",
            "--base-url",
            BASE_URL,
            "gemini",
        ])
        .env("HOME", &home)
        .env("PATH", &path)
        .output()
        .expect("global configure with verify");

    assert!(
        !output.status.success(),
        "an auth refusal must fail the run, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid auth method selected"),
        "stderr: {stderr}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_succeeds_when_the_configured_client_starts() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home");
    let path = fake_cli(&dir.join("bin"), "gemini", "pong");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--verify",
            "--base-url",
            BASE_URL,
            "gemini",
        ])
        .env("HOME", &home)
        .env("PATH", &path)
        .output()
        .expect("global configure with verify");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Registry requirements are templates; a test may spell out what the renderer
/// produces for the protocols this issue covers.
fn render_requirement(target: &str, protocol: &str) -> String {
    let base_env = match protocol {
        "vertex" => "GOOGLE_VERTEX_BASE_URL",
        "gemini" => "GOOGLE_GEMINI_BASE_URL",
        "openai" => "OPENAI_BASE_URL",
        "anthropic" => "ANTHROPIC_BASE_URL",
        _ => "FORMAL_AI_BASE_URL",
    };
    let auth_type = match protocol {
        "vertex" => "vertex-ai",
        "gemini" => "gemini-api-key",
        _ => "",
    };
    target
        .replace("{protocol_base_env}", base_env)
        .replace("{google_auth_type}", auth_type)
}

/// Resolve one registry-declared headless requirement against the files
/// `--global` actually left behind, re-reading them the way a freshly started
/// client would rather than trusting what the writer intended.
fn requirement_is_materialised(home: &Path, kind: &str, target: &str) -> bool {
    match kind {
        "env" => std::fs::read_to_string(home.join(".profile"))
            .is_ok_and(|profile| profile.contains(&format!("export {target}="))),
        "json" => {
            let Some((path, dotted)) = target.split_once(':') else {
                return false;
            };
            let Ok(text) = std::fs::read_to_string(home.join(path)) else {
                return false;
            };
            let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) else {
                return false;
            };
            let mut cursor = &parsed;
            for key in dotted.split('.') {
                match cursor.get(key) {
                    Some(next) => cursor = next,
                    None => return false,
                }
            }
            !cursor.is_null()
        }
        "toml" => {
            let Some((path, dotted)) = target.split_once(':') else {
                return false;
            };
            let Ok(text) = std::fs::read_to_string(home.join(path)) else {
                return false;
            };
            let Ok(parsed) = text.parse::<toml_edit::DocumentMut>() else {
                return false;
            };
            let mut cursor = parsed.as_item();
            for key in dotted.split('.') {
                match cursor.get(key) {
                    Some(next) => cursor = next,
                    None => return false,
                }
            }
            !cursor.is_none()
        }
        _ => false,
    }
}

/// The whole task of issue #909 in one run: configuring every client globally
/// must leave each one able to start headlessly, and the live probe must accept
/// a client that answers while rejecting one that refuses to authenticate.
#[test]
fn global_all_leaves_every_client_headless_ready_and_probed() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home");
    // Both probed clients answer, so `--verify` must accept the whole sweep.
    // The search path deliberately excludes the operator's own client
    // installations: this test probes its own stand-ins, never a real CLI.
    let bin_dir = dir.join("bin");
    fake_cli(&bin_dir, "gemini", "pong");
    fake_cli(&bin_dir, "qwen", "pong");
    let path = format!("{}:/usr/bin:/bin", bin_dir.display());

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--all",
            "--verify",
            "--base-url",
            BASE_URL,
        ])
        .env("HOME", &home)
        .env("PATH", &path)
        .output()
        .expect("configure every client globally");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut checked = 0_usize;
    for integration in formal_ai::seed::client_integrations() {
        let config = &integration.global_config;
        let declarations = std::iter::once(&config.headless_requirements).chain(
            config
                .companion_files
                .iter()
                .map(|companion| &companion.headless_requirements),
        );
        for headless_requirements in declarations {
            for (kind, target) in headless_requirements {
                let target = render_requirement(target, &integration.default_protocol);
                assert!(
                    requirement_is_materialised(&home, kind, &target),
                    "{} left {kind}:{target} unwritten after --global --all",
                    integration.id
                );
                checked += 1;
            }
        }
    }
    // The registry, not this test, decides how many requirements exist; a seed
    // that declares none would otherwise let the assertion loop pass vacuously.
    assert!(
        checked >= 6,
        "expected the registry to declare the issue #909 startup contract, got {checked}"
    );

    // Same sweep, but gemini now refuses to authenticate: the run must fail.
    let refusing = tmpdir();
    let refusing_home = refusing.join("home");
    std::fs::create_dir_all(&refusing_home).expect("home");
    let refusing_bin = refusing.join("bin");
    fake_cli(&refusing_bin, "gemini", "Invalid auth method selected.");
    let refusing_path = format!("{}:/usr/bin:/bin", refusing_bin.display());
    let refused = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--all",
            "--verify",
            "--base-url",
            BASE_URL,
        ])
        .env("HOME", &refusing_home)
        .env("PATH", &refusing_path)
        .output()
        .expect("configure every client globally");
    assert!(
        !refused.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&refused.stdout)
    );

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&refusing);
}

#[test]
fn verify_is_rejected_without_global() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--verify", "--no-start-server", "gemini"])
        .env("HOME", &home)
        .output()
        .expect("run with verify only");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--verify"), "stderr: {stderr}");

    let _ = std::fs::remove_dir_all(&dir);
}
