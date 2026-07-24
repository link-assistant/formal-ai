//! End-to-end context export coverage for issue #822.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::conversation_context::conversation_context_to_lino;
use formal_ai::dialog_log::write_dialog_exchange;
use serde_json::{json, Value};

fn temporary_directory(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-issue-822-{}-{label}-{nonce}",
        std::process::id()
    ))
}

fn create_opencode_fixture(path: &std::path::Path) {
    let script = r"
import json, sqlite3, sys
db = sqlite3.connect(sys.argv[1])
db.execute('CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT, model TEXT, version TEXT, time_created INTEGER, time_updated INTEGER)')
db.execute('CREATE TABLE message (id TEXT PRIMARY KEY, session_id TEXT, time_created INTEGER, time_updated INTEGER, data TEXT)')
db.execute('CREATE TABLE part (id TEXT PRIMARY KEY, message_id TEXT, session_id TEXT, time_created INTEGER, time_updated INTEGER, data TEXT)')
db.execute('INSERT INTO session VALUES (?, ?, ?, ?, ?, ?)', ('ses_fixture', '/workspace/a:b', json.dumps({'providerID':'formalai','id':'formal-ai'}), '1.18.4', 1, 9))
db.execute('INSERT INTO message VALUES (?, ?, ?, ?, ?)', ('msg_b', 'ses_fixture', 2, 4, json.dumps({'role':'assistant','tokens':{'input':31},'cost':0.01})))
db.execute('INSERT INTO message VALUES (?, ?, ?, ?, ?)', ('msg_a', 'ses_fixture', 1, 1, json.dumps({'role':'user'})))
db.execute('INSERT INTO part VALUES (?, ?, ?, ?, ?, ?)', ('part_b', 'msg_b', 'ses_fixture', 4, 4, json.dumps({'type':'tool','tool':'websearch','state':{'status':'completed','output':'result','input':{'unsafe:key':'preserved'}}})))
db.execute('INSERT INTO part VALUES (?, ?, ?, ?, ?, ?)', ('part_a', 'msg_a', 'ses_fixture', 1, 1, json.dumps({'type':'text','text':'true'})))
db.commit()
";
    let output = Command::new("python3")
        .args(["-c", script])
        .arg(path)
        .output()
        .expect("create SQLite fixture");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn opencode_export_is_complete_native_read_only_and_deterministic() {
    let directory = temporary_directory("opencode");
    fs::create_dir_all(&directory).unwrap();
    let database = directory.join("opencode.db");
    create_opencode_fixture(&database);
    let before = fs::read(&database).unwrap();

    let run = |format: Option<&str>| {
        Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args([
                "--silent",
                "context",
                "export",
                "--session",
                "ses_fixture",
                "--source",
                "opencode",
                "--db",
            ])
            .arg(&database)
            .args(
                format
                    .map(|value| ["--format", value])
                    .into_iter()
                    .flatten(),
            )
            .output()
            .expect("run OpenCode context export")
    };
    let first = run(None);
    let second = run(None);
    let json = run(Some("json"));
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, second.stdout, "export must be deterministic");
    assert_eq!(
        before,
        fs::read(&database).unwrap(),
        "database was modified"
    );

    let lino = String::from_utf8(first.stdout).unwrap();
    assert!(
        json.status.success(),
        "{}",
        String::from_utf8_lossy(&json.stderr)
    );
    let tree: Value = serde_json::from_slice(&json.stdout).expect("OpenCode JSON export");
    assert_eq!(
        lino,
        conversation_context_to_lino("ses_fixture", &tree),
        "OpenCode and arbitrary JSON exports must use one shared LiNo serializer"
    );
    for expected in [
        "conversation ses_fixture",
        "directory \"/workspace/a:b\"",
        "providerID formalai",
        "role user",
        "role assistant",
        "tool websearch",
        "tokens",
        "cost 0.01",
        "output result",
        "text \"true\"",
        "name \"unsafe:key\"",
        "value preserved",
    ] {
        assert!(lino.contains(expected), "missing {expected:?}:\n{lino}");
    }
    assert_eq!(lino.matches("    message\n").count(), 2, "{lino}");
    assert_eq!(lino.matches("      part\n").count(), 2, "{lino}");
    assert!(!lino.contains("message_0"), "{lino}");
    assert!(lino.find("id msg_a") < lino.find("id msg_b"), "{lino}");
    links_notation::parse_lino(&lino).expect("OpenCode output must satisfy canonical grammar");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn general_json_converter_defaults_to_links_notation() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["--silent", "context", "json-to-lino"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start converter");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"messages":[{"role":"user","content":"a:b"}]}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let lino = String::from_utf8(output.stdout).unwrap();
    assert!(lino.contains("messages\n  message\n"), "{lino}");
    assert!(lino.contains("content \"a:b\""), "{lino}");
    assert!(!lino.contains("message_0"), "{lino}");
}

#[test]
fn local_context_learning_uses_the_dialog_log_without_a_server() {
    let directory = temporary_directory("learn");
    fs::create_dir_all(&directory).unwrap();
    let dialog_id = "issue-832-local-learning";
    let request = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Report this conversation"}]
    })
    .to_string();
    write_dialog_exchange(
        &directory,
        "POST",
        "/v1/chat/completions",
        &[("X-Formal-AI-Dialog-ID", dialog_id)],
        &request,
        200,
        "application/json",
        r#"{"choices":[{"message":{"role":"assistant","content":"Recorded."}}]}"#,
    )
    .expect("dialog fixture");
    let memory = directory.join("memory.lino");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "--silent",
            "context",
            "learn",
            "--session",
            dialog_id,
            "--log-dir",
        ])
        .arg(&directory)
        .env("FORMAL_AI_MEMORY_PATH", &memory)
        .output()
        .expect("run local context learning");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).expect("learning JSON");
    assert_eq!(result["dialog_id"], dialog_id);
    assert_eq!(result["learned"], true);
    assert!(memory.is_file(), "reported context was not stored");
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn verbose_is_global_default_and_silent_is_available() {
    let help = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .arg("--help")
        .output()
        .unwrap();
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("--verbose"), "{text}");
    assert!(text.contains("enabled by default"), "{text}");
    assert!(text.contains("--silent"), "{text}");
}
