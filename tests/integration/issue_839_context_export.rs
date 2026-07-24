//! Context export must name its source and never substitute one silently (#839).
//!
//! Issue #838 filed a report that claimed to carry a conversation and actually
//! carried 271 KB of base64 HTTP frames, because `--source harness` fell back to
//! the server proxy log without saying so. Each test below fails against the
//! code that produced that artifact.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::dialog_log::write_dialog_exchange;
use serde_json::{json, Value};

fn temporary_directory(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-issue-839-{}-{label}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("temporary directory");
    directory
}

/// Build an `OpenCode` database holding `sessions` as `(id, first user text)`.
fn create_opencode_fixture(path: &Path, sessions: &[(&str, &str)]) {
    let script = r"
import json, sqlite3, sys
db = sqlite3.connect(sys.argv[1])
db.execute('CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT, model TEXT, version TEXT, time_created INTEGER, time_updated INTEGER)')
db.execute('CREATE TABLE message (id TEXT PRIMARY KEY, session_id TEXT, time_created INTEGER, time_updated INTEGER, data TEXT)')
db.execute('CREATE TABLE part (id TEXT PRIMARY KEY, message_id TEXT, session_id TEXT, time_created INTEGER, time_updated INTEGER, data TEXT)')
for order, entry in enumerate(json.loads(sys.argv[2])):
    session, text = entry
    db.execute('INSERT INTO session VALUES (?, ?, ?, ?, ?, ?)', (session, '/workspace', 'formal-ai', '1.18.4', order + 1, order + 1))
    if text:
        db.execute('INSERT INTO message VALUES (?, ?, ?, ?, ?)', ('msg_' + session, session, 1, 1, json.dumps({'role':'user'})))
        db.execute('INSERT INTO part VALUES (?, ?, ?, ?, ?, ?)', ('part_' + session, 'msg_' + session, session, 1, 1, json.dumps({'type':'text','text':text})))
db.commit()
";
    let output = Command::new("python3")
        .args(["-c", script])
        .arg(path)
        .arg(serde_json::to_string(&sessions).expect("fixture plan"))
        .output()
        .expect("create SQLite fixture");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Record one server exchange for `dialog_id`, as the running server would.
fn record_server_exchange(directory: &Path, dialog_id: &str, prompt: &str, reply: &str) {
    let request = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}]
    })
    .to_string();
    let response = json!({
        "choices": [{"message": {"role": "assistant", "content": reply}}]
    })
    .to_string();
    write_dialog_exchange(
        directory,
        "POST",
        "/v1/chat/completions",
        &[("X-Formal-AI-Dialog-ID", dialog_id)],
        &request,
        200,
        "application/json",
        &response,
    )
    .expect("dialog fixture");
}

fn export(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["--silent", "context", "export"])
        .args(args)
        .env_remove("FORMAL_AI_DIALOG_ID")
        .output()
        .expect("run context export")
}

#[test]
fn harness_source_fails_loudly_instead_of_returning_the_server_log() {
    let directory = temporary_directory("harness-fallback");
    let session = "ses_harness_missing";
    record_server_exchange(&directory, session, "Что такое фуфломицин?", "A placebo.");
    let database = directory.join("opencode.db");
    create_opencode_fixture(&database, &[("ses_other", "unrelated")]);

    // The server log for this session is right there and readable, which is
    // exactly the situation that let #838 be filed: the harness export failed,
    // the proxy trace was substituted, and nothing said so.
    let output = export(&[
        "--session",
        session,
        "--source",
        "harness",
        "--db",
        database.to_str().expect("database path"),
        "--log-dir",
        directory.to_str().expect("log directory"),
        "--format",
        "json",
    ]);

    assert!(
        !output.status.success(),
        "an unresolvable harness session must fail, got:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostic.contains(session),
        "the failure must name the session it could not export:\n{diagnostic}"
    );
    assert!(
        output.stdout.is_empty(),
        "a failed harness export must emit no document:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    // `--source server` still exports that same log on request; only the
    // silent substitution is the defect.
    let server = export(&[
        "--session",
        session,
        "--source",
        "server",
        "--log-dir",
        directory.to_str().expect("log directory"),
        "--format",
        "json",
    ]);
    assert!(
        server.status.success(),
        "{}",
        String::from_utf8_lossy(&server.stderr)
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn server_export_carries_conversation_turns_not_only_proxy_frames() {
    let directory = temporary_directory("server-turns");
    let session = "ses_server_turns";
    record_server_exchange(&directory, session, "ФБС vs ФБО", "They differ in load.");

    let output = export(&[
        "--session",
        session,
        "--source",
        "server",
        "--log-dir",
        directory.to_str().expect("log directory"),
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let context: Value = serde_json::from_slice(&output.stdout).expect("server context JSON");

    assert_eq!(
        context["metadata"]["messages_source"], "conversation-record",
        "the conversation must come from the conversation record, not the trace"
    );
    let messages = context["messages"]
        .as_array()
        .expect("server export keeps the conversation");
    let transcript = serde_json::to_string(messages).expect("transcript JSON");
    assert!(transcript.contains("ФБС vs ФБО"), "{transcript}");
    assert!(transcript.contains("They differ in load."), "{transcript}");
    assert!(
        !context["server_logs"].is_null(),
        "the transport trace stays available alongside the conversation"
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn both_merges_harness_and_server_records_into_one_document() {
    let directory = temporary_directory("both");
    let session = "ses_merged";
    record_server_exchange(
        &directory,
        session,
        "server side prompt",
        "server side reply",
    );
    let database = directory.join("opencode.db");
    create_opencode_fixture(&database, &[(session, "harness side prompt")]);

    let output = export(&[
        "--session",
        session,
        "--source",
        "both",
        "--db",
        database.to_str().expect("database path"),
        "--log-dir",
        directory.to_str().expect("log directory"),
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let context: Value = serde_json::from_slice(&output.stdout).expect("merged context JSON");

    assert_eq!(context["metadata"]["source"], "formal-ai-merged-context");
    assert!(
        context["metadata"]["harness_message_count"]
            .as_u64()
            .expect("harness count")
            > 0,
        "the merge must include the harness side"
    );
    assert!(
        context["metadata"]["server_message_count"]
            .as_u64()
            .expect("server count")
            > 0,
        "the merge must include the server side"
    );
    let merged = serde_json::to_string(&context["messages"]).expect("merged transcript");
    assert!(merged.contains("harness side prompt"), "{merged}");
    assert!(merged.contains("server side reply"), "{merged}");
    assert!(
        context["harness"]["messages"].is_null() && context["server"]["messages"].is_null(),
        "turns live in the merged transcript, not duplicated in both sub-documents"
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn an_empty_export_fails_instead_of_reporting_success() {
    let directory = temporary_directory("empty");
    let session = "ses_empty";
    let database = directory.join("opencode.db");
    create_opencode_fixture(&database, &[(session, "")]);

    let output = export(&[
        "--session",
        session,
        "--source",
        "harness",
        "--db",
        database.to_str().expect("database path"),
        "--format",
        "json",
    ]);

    assert!(
        !output.status.success(),
        "a session with no messages must not export as a success:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains(session), "{diagnostic}");
    assert!(output.stdout.is_empty(), "{diagnostic}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn the_session_subcommand_resolves_the_session_this_shell_is_inside() {
    let directory = temporary_directory("resolve");
    let database = directory.join("opencode.db");
    create_opencode_fixture(&database, &[("ses_older", "first"), ("ses_newest", "last")]);

    let resolved = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["--silent", "context", "session", "--db"])
        .arg(&database)
        .env_remove("FORMAL_AI_DIALOG_ID")
        .output()
        .expect("resolve session");
    assert!(
        resolved.status.success(),
        "{}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&resolved.stdout).trim(),
        "ses_newest",
        "`latest` must resolve to the most recent harness session"
    );

    let declared = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["--silent", "context", "session", "--db"])
        .arg(&database)
        .env("FORMAL_AI_DIALOG_ID", "ses_declared_by_the_caller")
        .output()
        .expect("resolve declared session");
    assert_eq!(
        String::from_utf8_lossy(&declared.stdout).trim(),
        "ses_declared_by_the_caller",
        "an explicitly declared session id wins over the database guess"
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn conversations_sharing_a_first_message_do_not_collide() {
    let directory = temporary_directory("collision");
    let prompt = "Что такое фуфломицин?";
    record_server_exchange(&directory, "ses_first_caller", prompt, "First answer.");
    record_server_exchange(&directory, "ses_second_caller", prompt, "Second answer.");

    let read = |session: &str| {
        let output = export(&[
            "--session",
            session,
            "--source",
            "server",
            "--log-dir",
            directory.to_str().expect("log directory"),
            "--format",
            "json",
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("context JSON")
    };

    let first = read("ses_first_caller");
    let second = read("ses_second_caller");
    assert!(first.contains("First answer."), "{first}");
    assert!(
        !first.contains("Second answer."),
        "two sessions sharing a first message must stay separate:\n{first}"
    );
    assert!(second.contains("Second answer."), "{second}");
    assert!(!second.contains("First answer."), "{second}");

    fs::remove_dir_all(directory).unwrap();
}
