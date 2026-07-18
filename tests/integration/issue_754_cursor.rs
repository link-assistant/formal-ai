use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use formal_ai::seed::client_integrations;
use formal_ai::{handle_api_request, handle_api_request_with_auth, ApiAuthConfig};

use crate::http_server::{http_request, reserve_loopback_port, spawn_formal_ai_server};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-754-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&path).expect("create temp directory");
    path
}

fn write_fake_cursor(bin_dir: &std::path::Path) {
    let path = bin_dir.join("cursor-agent");
    std::fs::write(
        &path,
        r#"#!/bin/sh
printf '%s\n' "$@" > "$FORMAL_AI_CAPTURE.args"
cp "$HOME/.cursor/mcp.json" "$FORMAL_AI_CAPTURE.mcp.json"
"#,
    )
    .expect("write fake cursor-agent");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make fake cursor executable");
}

#[test]
fn cursor_is_seeded_as_cursor_agent_with_mcp_configuration() {
    let cursor = client_integrations()
        .into_iter()
        .find(|integration| integration.id == "cursor")
        .expect("cursor integration");
    assert_eq!(cursor.command, "cursor-agent");
    assert_eq!(cursor.invocation.non_interactive_args, ["-p"]);
    assert_eq!(cursor.invocation.temp_home_env, "HOME");
    assert_eq!(cursor.invocation.temp_home_config_path, ".cursor/mcp.json");
    assert!(cursor
        .invocation
        .temp_home_json_settings
        .iter()
        .any(|(path, value)| path == "mcpServers.formal-ai.url" && value == "{base_url}/mcp"));
}

#[test]
fn mcp_endpoint_supports_initialize_list_and_formal_ai_chat() {
    let initialize = handle_api_request(
        "POST",
        "/mcp",
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
    );
    assert_eq!(initialize.status_code, 200);
    let initialized: serde_json::Value =
        serde_json::from_str(&initialize.body).expect("initialize JSON");
    assert_eq!(initialized["result"]["serverInfo"]["name"], "formal-ai");
    assert_eq!(initialized["result"]["protocolVersion"], "2025-06-18");

    let notification = handle_api_request(
        "POST",
        "/mcp",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    );
    assert_eq!(notification.status_code, 200);
    let notification: serde_json::Value =
        serde_json::from_str(&notification.body).expect("notification JSON");
    assert_eq!(notification["result"], serde_json::json!({}));

    let get = handle_api_request("GET", "/mcp", "");
    assert_eq!(get.status_code, 405);

    let listed = handle_api_request(
        "POST",
        "/mcp",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    );
    let listed: serde_json::Value = serde_json::from_str(&listed.body).expect("tools JSON");
    assert_eq!(listed["result"]["tools"][0]["name"], "formal_ai_chat");
    assert!(listed["result"]["tools"][0]["description"]
        .as_str()
        .is_some_and(|description| !description.is_empty()));

    let called = handle_api_request(
        "POST",
        "/mcp",
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"formal_ai_chat","arguments":{"prompt":"hi"}}}"#,
    );
    assert_eq!(called.status_code, 200);
    let called: serde_json::Value = serde_json::from_str(&called.body).expect("call JSON");
    assert_eq!(called["result"]["content"][0]["type"], "text");
    assert!(called["result"]["content"][0]["text"]
        .as_str()
        .is_some_and(|text| !text.is_empty()));
}

#[test]
fn mcp_endpoint_returns_json_rpc_errors_for_bad_requests() {
    let malformed = handle_api_request("POST", "/mcp", "{");
    let malformed: serde_json::Value =
        serde_json::from_str(&malformed.body).expect("parse error JSON");
    assert_eq!(malformed["error"]["code"], -32700);

    let unknown = handle_api_request(
        "POST",
        "/mcp",
        r#"{"jsonrpc":"2.0","id":7,"method":"unknown"}"#,
    );
    let unknown: serde_json::Value =
        serde_json::from_str(&unknown.body).expect("method error JSON");
    assert_eq!(unknown["id"], 7);
    assert_eq!(unknown["error"]["code"], -32601);
}

#[test]
fn mcp_endpoint_enforces_authentication_and_origin_checks() {
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
    let auth = ApiAuthConfig::bearer_token("local-test-token");
    let missing = handle_api_request_with_auth("POST", "/mcp", &[], body, &auth);
    assert_eq!(missing.status_code, 401);

    let hostile = handle_api_request_with_auth(
        "POST",
        "/mcp",
        &[
            ("authorization", "Bearer local-test-token"),
            ("origin", "https://attacker.example"),
            ("host", "127.0.0.1:8080"),
        ],
        body,
        &auth,
    );
    assert_eq!(hostile.status_code, 403);

    let loopback = handle_api_request_with_auth(
        "POST",
        "/mcp",
        &[
            ("authorization", "Bearer local-test-token"),
            ("origin", "http://127.0.0.1:8080"),
            ("host", "127.0.0.1:8080"),
        ],
        body,
        &auth,
    );
    assert_eq!(loopback.status_code, 200);
}

#[test]
fn mcp_method_errors_keep_their_status_on_the_http_wire() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server(port);
    let response = http_request("GET", port, "/mcp", Some("sk-local-agentic-tools"), None)
        .expect("GET /mcp should complete");
    assert_eq!(response.status_code, 405);
}

#[test]
fn formal_ai_with_cursor_runs_both_modes_through_launch_scoped_mcp_config() {
    let dir = temp_dir();
    let home = dir.join("home");
    let bin = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin).expect("bin");
    write_fake_cursor(&bin);
    let capture = dir.join("capture");
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--no-start-server",
            "--base-url",
            "http://127.0.0.1:18090",
            "cursor",
            "-p",
            "hi",
        ])
        .env("HOME", &home)
        .env("PATH", &path)
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run cursor wrapper");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(format!("{}.args", capture.display())).expect("captured args"),
        "-p\nhi\n"
    );
    let config = std::fs::read_to_string(format!("{}.mcp.json", capture.display()))
        .expect("captured MCP config");
    let config: serde_json::Value = serde_json::from_str(&config).expect("MCP config JSON");
    assert_eq!(
        config["mcpServers"]["formal-ai"]["url"],
        "http://127.0.0.1:18090/mcp"
    );
    assert_eq!(config["mcpServers"]["formal-ai"]["type"], "http");
    assert_eq!(
        config["mcpServers"]["formal-ai"]["headers"]["Authorization"],
        "Bearer formal-ai"
    );
    assert!(!home.join(".cursor/mcp.json").exists());

    let interactive = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--no-start-server",
            "--base-url",
            "http://127.0.0.1:18090",
            "--interactive",
            "cursor",
        ])
        .env("HOME", &home)
        .env("PATH", &path)
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run interactive cursor wrapper");
    assert!(
        interactive.status.success(),
        "{}",
        String::from_utf8_lossy(&interactive.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(format!("{}.args", capture.display())).expect("interactive args"),
        "\n"
    );
    assert!(!home.join(".cursor/mcp.json").exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn formal_ai_with_cursor_global_config_is_merge_preserving_and_reversible() {
    let dir = temp_dir();
    let home = dir.join("home");
    std::fs::create_dir_all(home.join(".cursor")).expect("cursor config dir");
    let original =
        "{\n  \"mcpServers\": {\n    \"user-server\": {\"command\": \"user-command\"}\n  }\n}\n";
    std::fs::write(home.join(".cursor/mcp.json"), original).expect("seed cursor config");

    let configured = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--base-url",
            "http://127.0.0.1:18090",
            "cursor",
        ])
        .env("HOME", &home)
        .output()
        .expect("configure cursor globally");
    assert!(configured.status.success());
    let config = std::fs::read_to_string(home.join(".cursor/mcp.json")).expect("cursor config");
    let config: serde_json::Value = serde_json::from_str(&config).expect("cursor config JSON");
    assert_eq!(
        config["mcpServers"]["user-server"]["command"],
        "user-command"
    );
    assert_eq!(
        config["mcpServers"]["formal-ai"]["url"],
        "http://127.0.0.1:18090/mcp"
    );
    assert!(home.join(".cursor/mcp.json.formal-ai.bak").exists());

    let undone = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--global", "--undo", "cursor"])
        .env("HOME", &home)
        .output()
        .expect("undo cursor config");
    assert!(undone.status.success());
    assert_eq!(
        std::fs::read_to_string(home.join(".cursor/mcp.json")).expect("restored config"),
        original
    );
    let _ = std::fs::remove_dir_all(dir);
}
