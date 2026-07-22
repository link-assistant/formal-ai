//! End-to-end context export coverage for issue #822.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

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
    let script = r#"
import json, sqlite3, sys
db = sqlite3.connect(sys.argv[1])
db.execute('CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT, model TEXT, version TEXT, time_created INTEGER, time_updated INTEGER)')
db.execute('CREATE TABLE message (id TEXT PRIMARY KEY, session_id TEXT, time_created INTEGER, time_updated INTEGER, data TEXT)')
db.execute('CREATE TABLE part (id TEXT PRIMARY KEY, message_id TEXT, session_id TEXT, time_created INTEGER, time_updated INTEGER, data TEXT)')
db.execute('INSERT INTO session VALUES (?, ?, ?, ?, ?, ?)', ('ses_fixture', '/workspace/a:b', json.dumps({'providerID':'formalai','id':'formal-ai'}), '1.18.4', 1, 9))
db.execute('INSERT INTO message VALUES (?, ?, ?, ?, ?)', ('msg_b', 'ses_fixture', 2, 4, json.dumps({'role':'assistant','tokens':{'input':31},'cost':0.01})))
db.execute('INSERT INTO message VALUES (?, ?, ?, ?, ?)', ('msg_a', 'ses_fixture', 1, 1, json.dumps({'role':'user'})))
db.execute('INSERT INTO part VALUES (?, ?, ?, ?, ?, ?)', ('part_b', 'msg_b', 'ses_fixture', 4, 4, json.dumps({'type':'tool','tool':'websearch','state':{'status':'completed','output':'result','input':{'unsafe:key':'preserved'}}})))
db.execute('INSERT INTO part VALUES (?, ?, ?, ?, ?, ?)', ('part_a', 'msg_a', 'ses_fixture', 1, 1, json.dumps({'type':'text','text':'find a:b'})))
db.commit()
"#;
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

    let run = || {
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
            .output()
            .expect("run OpenCode context export")
    };
    let first = run();
    let second = run();
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
