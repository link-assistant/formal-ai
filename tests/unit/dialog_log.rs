use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::dialog_log::{write_dialog_exchange, DialogExchangeLog};

fn isolated_directory(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-dialog-log-{}-{test_name}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn full_exchanges_from_one_dialog_append_to_one_file() {
    let directory = isolated_directory("full-exchanges");
    let header = [("X-Formal-AI-Dialog-ID", "issue-781-reproduction")];
    let request =
        r#"{"model":"formal-ai","messages":[{"role":"user","content":"Find a charger"}]}"#;
    let response = r#"{"choices":[{"message":{"content":"I will search","tool_calls":[]}}]}"#;

    let first = write_dialog_exchange(
        &directory,
        "POST",
        "/v1/chat/completions",
        &header,
        request,
        200,
        "application/json",
        response,
    )
    .expect("first dialog log row");
    let second = write_dialog_exchange(
        &directory,
        "POST",
        "/v1/chat/completions",
        &header,
        request,
        200,
        "application/json",
        response,
    )
    .expect("second dialog log row");

    assert_eq!(first, second);
    let rows = fs::read_to_string(&first).expect("dialog log file");
    assert_eq!(rows.lines().count(), 2);
    for row in rows.lines() {
        let record: DialogExchangeLog = serde_json::from_str(row).expect("valid dialog JSONL row");
        assert_eq!(record.exchange.request_body.as_deref(), Some(request));
        assert_eq!(record.exchange.response_body.as_deref(), Some(response));
    }
    fs::remove_dir_all(directory).expect("remove isolated test directory");
}

#[test]
fn expanded_histories_share_the_first_user_prompt_id() {
    let directory = isolated_directory("expanded-history");
    let first_request = r#"{"messages":[{"role":"user","content":"Find a charger"}]}"#;
    let next_request = r#"{"messages":[{"role":"user","content":"Find a charger"},{"role":"assistant","content":"Searching"},{"role":"tool","content":"result"}]}"#;

    let first = write_dialog_exchange(
        &directory,
        "POST",
        "/v1/chat/completions",
        &[],
        first_request,
        200,
        "application/json",
        "{}",
    )
    .expect("first dialog log row");
    let next = write_dialog_exchange(
        &directory,
        "POST",
        "/v1/chat/completions",
        &[],
        next_request,
        200,
        "application/json",
        "{}",
    )
    .expect("expanded-history dialog log row");

    assert_eq!(first, next);
    fs::remove_dir_all(directory).expect("remove isolated test directory");
}
