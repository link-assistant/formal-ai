use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir() -> std::path::PathBuf {
    // Compose a per-process, per-thread, per-call unique path. macOS clocks
    // can return the same nanos value for two near-simultaneous calls, so
    // we cannot rely on SystemTime alone for uniqueness across parallel
    // tests; the atomic counter guarantees no two callers in the same
    // process ever pick the same directory.
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let thread_id = format!("{:?}", std::thread::current().id())
        .replace(|c: char| !c.is_ascii_alphanumeric(), "");
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-cli-{}-{}-{seq}",
        std::process::id(),
        thread_id,
    ));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

#[test]
fn cli_version_flag_prints_crate_version() {
    // Issue #72: the CLI must advertise its own version so users can quote
    // the right release in bug reports. clap's `version` attribute reads
    // `CARGO_PKG_VERSION` so this test fails if anyone accidentally pins a
    // stale literal in `#[command(...)]`.
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["--version"])
        .output()
        .expect("failed to execute binary");
    assert!(output.status.success(), "exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = format!("formal-ai {}", env!("CARGO_PKG_VERSION"));
    assert!(
        stdout.trim() == expected,
        "expected `{expected}`, got `{}`",
        stdout.trim()
    );
}

#[test]
fn cli_chat_command_prints_text_response() {
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["chat", "--prompt", "Hi"])
        .output()
        .expect("failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "Hi, how may I help you?");
}

#[test]
fn cli_chat_command_can_emit_chat_completion_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "chat",
            "--prompt",
            "Write me hello world program in Rust",
            "--format",
            "chat",
        ])
        .output()
        .expect("failed to execute binary");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(json["object"], "chat.completion");
    assert!(json["choices"][0]["message"]["content"]
        .as_str()
        .expect("assistant content should be a string")
        .contains("```rust"));
}

#[test]
fn cli_environments_command_lists_every_supported_surface() {
    // R106: the seed declares every environment, and the CLI must surface
    // it so users can see where they can run the agent and how to migrate.
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["environments"])
        .output()
        .expect("failed to execute binary");
    assert!(output.status.success(), "exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "# browser",
        "# rust_library",
        "# cli",
        "# http_server",
        "# telegram",
        "# docker_microservice",
    ] {
        assert!(
            stdout.contains(expected),
            "expected `{expected}` in environments output:\n{stdout}",
        );
    }
}

#[test]
fn cli_github_logs_plan_prints_reproducible_capture_commands() {
    // Issue #115: the project needs a reusable way to collect GitHub issue,
    // PR, review, and run-log evidence for hive-mind case studies without
    // relying on handwritten command lists.
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "github-logs",
            "plan",
            "--repo",
            "link-assistant/hive-mind",
            "--output-dir",
            "docs/case-studies/issue-115/raw-data/hive-mind",
            "--issue",
            "1814",
            "--pull",
            "1816",
            "--run",
            "26058054431",
            "--recent-issues",
            "10",
            "--recent-pulls",
            "10",
            "--recent-runs",
            "5",
            "--branch",
            "issue-1814-0f855d3671ac",
        ])
        .output()
        .expect("github-logs plan");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# GitHub log capture plan"));
    assert!(stdout.contains("repo: link-assistant/hive-mind"));
    assert!(stdout.contains("issue-1814-comments.json"));
    assert!(stdout.contains("pr-1816-review-comments.json"));
    assert!(stdout.contains("pr-1816-conversation-comments.json"));
    assert!(stdout.contains("run-26058054431.log"));
    assert!(stdout.contains("gh api repos/link-assistant/hive-mind/pulls/1816/comments --paginate"));
}

#[test]
fn cli_memory_export_import_show_round_trips_events() {
    // R107: a `demo_memory` Links Notation file written by the CLI must
    // round-trip back through `memory import` and `memory show` without
    // losing events. This is the exact wire-format the browser writes via
    // its `Export memory` button.
    let dir = tmpdir();
    let source = dir.join("inbound.lino");
    let into = dir.join("memory.lino");
    std::fs::write(
        &source,
        "demo_memory\n\
         \x20\x20event \"1\"\n\
         \x20\x20\x20\x20role \"user\"\n\
         \x20\x20\x20\x20content \"Привет\"\n\
         \x20\x20event \"2\"\n\
         \x20\x20\x20\x20role \"assistant\"\n\
         \x20\x20\x20\x20intent \"greeting\"\n\
         \x20\x20\x20\x20content \"Hi, how may I help you?\"\n",
    )
    .expect("seed memory file");

    let import = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "memory",
            "import",
            "--path",
            source.to_str().unwrap(),
            "--into",
            into.to_str().unwrap(),
        ])
        .output()
        .expect("memory import");
    assert!(
        import.status.success(),
        "import stderr: {}",
        String::from_utf8_lossy(&import.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["memory", "show", "--path", into.to_str().unwrap()])
        .output()
        .expect("memory show");
    assert!(show.status.success());
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(stdout.contains("Привет"), "show output: {stdout}");
    assert!(stdout.contains("[assistant]"));
    assert!(stdout.contains("greeting"));

    // Default `memory export` now emits the full self-contained
    // `formal_ai_bundle` (R109): seed + events + metadata in one file.
    let exported = dir.join("exported.lino");
    let export = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "memory",
            "export",
            "--from",
            into.to_str().unwrap(),
            "--path",
            exported.to_str().unwrap(),
        ])
        .output()
        .expect("memory export");
    assert!(export.status.success());
    let text = std::fs::read_to_string(&exported).expect("read exported");
    assert!(text.starts_with("formal_ai_bundle\n"));
    assert!(text.contains("data/seed/agent-info.lino"));
    assert!(text.contains("demo_memory"));
    assert!(text.contains("Привет"));

    // `--events-only` preserves the legacy `demo_memory` shape so older
    // scripts that pipe the export to a parser keep working.
    let exported_legacy = dir.join("exported-legacy.lino");
    let export_legacy = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "memory",
            "export",
            "--from",
            into.to_str().unwrap(),
            "--path",
            exported_legacy.to_str().unwrap(),
            "--events-only",
        ])
        .output()
        .expect("memory export --events-only");
    assert!(export_legacy.status.success());
    let legacy = std::fs::read_to_string(&exported_legacy).expect("read legacy");
    assert!(legacy.starts_with("demo_memory\n"));
    assert!(legacy.contains("Привет"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_bundle_export_includes_seed_and_memory() {
    // R107: `bundle export` writes a `formal_ai_bundle` document the
    // browser can re-import. The exported file must list every seed file
    // and embed every memory event the CLI knows about.
    let dir = tmpdir();
    let memory_path = dir.join("memory.lino");
    std::fs::write(
        &memory_path,
        "demo_memory\n\
         \x20\x20event \"a\"\n\
         \x20\x20\x20\x20role \"user\"\n\
         \x20\x20\x20\x20content \"What is X?\"\n",
    )
    .expect("seed memory");
    let bundle_path = dir.join("bundle.lino");
    let export = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "bundle",
            "export",
            "--path",
            bundle_path.to_str().unwrap(),
            "--memory",
            memory_path.to_str().unwrap(),
        ])
        .output()
        .expect("bundle export");
    assert!(
        export.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&export.stderr)
    );
    let text = std::fs::read_to_string(&bundle_path).expect("read bundle");
    assert!(text.starts_with("formal_ai_bundle\n"));
    assert!(text.contains("data/seed/environments.lino"));
    assert!(text.contains("demo_memory"));
    assert!(text.contains("What is X?"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_bundle_import_recovers_memory_into_local_log() {
    // R107: a `formal_ai_bundle` produced anywhere (CLI or browser) must
    // import cleanly into the CLI memory log via `bundle import`.
    let dir = tmpdir();
    let memory_path = dir.join("memory.lino");
    std::fs::write(
        &memory_path,
        "demo_memory\n\
         \x20\x20event \"a\"\n\
         \x20\x20\x20\x20role \"user\"\n\
         \x20\x20\x20\x20content \"hello\"\n",
    )
    .expect("seed memory");
    let bundle_path = dir.join("bundle.lino");
    let export = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "bundle",
            "export",
            "--path",
            bundle_path.to_str().unwrap(),
            "--memory",
            memory_path.to_str().unwrap(),
        ])
        .output()
        .expect("bundle export");
    assert!(export.status.success());

    let restored = dir.join("restored.lino");
    let import = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "bundle",
            "import",
            "--path",
            bundle_path.to_str().unwrap(),
            "--into",
            restored.to_str().unwrap(),
        ])
        .output()
        .expect("bundle import");
    assert!(
        import.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&import.stderr)
    );
    let text = std::fs::read_to_string(&restored).expect("read restored");
    assert!(text.starts_with("demo_memory\n"));
    assert!(text.contains("hello"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_memory_export_to_stdout_returns_links_notation() {
    // R107: `-` selects stdout/stdin so flows like
    // `formal-ai memory export --path - | ssh other formal-ai memory import --path -`
    // work without temp files.
    let dir = tmpdir();
    let memory_path = dir.join("memory.lino");
    std::fs::write(
        &memory_path,
        "demo_memory\n\
         \x20\x20event \"1\"\n\
         \x20\x20\x20\x20role \"user\"\n\
         \x20\x20\x20\x20content \"pipe me\"\n",
    )
    .expect("seed memory");
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "memory",
            "export",
            "--from",
            memory_path.to_str().unwrap(),
            "--path",
            "-",
        ])
        .output()
        .expect("export");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Default export is the full self-contained bundle (R109); `pipe me`
    // remains embedded inside the nested `demo_memory` section so a pipe
    // into `memory import --path -` still recovers it.
    assert!(stdout.starts_with("formal_ai_bundle\n"));
    assert!(stdout.contains("demo_memory"));
    assert!(stdout.contains("pipe me"));

    // Pipe the stdout back into `memory import --path -` and assert the
    // event lands in a new file. This is the load-bearing flow R107
    // promises for cross-surface migration.
    let into = dir.join("piped.lino");
    let mut child = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "memory",
            "import",
            "--path",
            "-",
            "--into",
            into.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn import");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdout.as_bytes())
        .expect("write");
    let status = child.wait().expect("wait");
    assert!(status.success());
    let saved = std::fs::read_to_string(&into).expect("read piped");
    assert!(saved.contains("pipe me"));
    let _ = std::fs::remove_dir_all(&dir);
}
