use std::fs;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::http_server::{
    http_request, reserve_loopback_port, spawn_formal_ai_server_agent_mode, FormalAiServer,
};

struct FormalAiProxy {
    child: Child,
}

impl Drop for FormalAiProxy {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn proxy_forwards_streaming_chat_and_logs_tool_call() {
    let (_server, _proxy, log_path, proxy_port) = spawn_proxy_stack();

    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{
            "role": "user",
            "content": "Formalize the fisherman tale into links notation."
        }],
        "stream": true,
        "tools": [{
            "type": "function",
            "function": {
                "name": "web_search",
                "parameters": {"type": "object"}
            }
        }]
    })
    .to_string();

    let response = http_request(
        "POST",
        proxy_port,
        "/v1/chat/completions",
        Some("sk-local-agentic-tools"),
        Some(&body),
    )
    .expect("proxy POST should complete");

    assert_eq!(
        response.status_code, 200,
        "proxy should forward upstream status and body: {}",
        response.body
    );
    assert!(
        response.content_type.starts_with("text/event-stream"),
        "streaming content-type should be preserved, got {}",
        response.content_type
    );
    assert!(
        response.body.contains("tool_calls"),
        "streamed response should pass tool call chunks through: {}",
        response.body
    );

    let entry = wait_for_proxy_log_entry(&log_path, "/v1/chat/completions");

    assert_eq!(entry["method"], "POST");
    assert_eq!(entry["path"], "/v1/chat/completions");
    assert_eq!(entry["status"], 200);
    assert_eq!(entry["request_model"], "formal-ai");
    assert_eq!(entry["request_tools"], serde_json::json!(["web_search"]));
    assert_eq!(entry["response_model"], "formal-ai");
    assert_eq!(entry["response_tool_calls"][0]["name"], "web_search");
    assert!(
        entry["response_tool_calls"][0]["arguments"]
            .to_string()
            .contains("Сказка о рыбаке и рыбке"),
        "logged arguments should preserve the planned search query: {entry}"
    );

    let _ = fs::remove_file(log_path);
}

#[test]
fn proxy_forwards_responses_and_logs_function_call() {
    let (_server, _proxy, log_path, proxy_port) = spawn_proxy_stack();

    let body = serde_json::json!({
        "model": "formal-ai",
        "input": "Formalize the fisherman tale into links notation.",
        "tools": [{
            "type": "function",
            "name": "web_search",
            "parameters": {"type": "object"}
        }]
    })
    .to_string();

    let response = http_request(
        "POST",
        proxy_port,
        "/v1/responses",
        Some("sk-local-agentic-tools"),
        Some(&body),
    )
    .expect("proxy Responses POST should complete");

    assert_eq!(
        response.status_code, 200,
        "proxy should forward Responses status and body: {}",
        response.body
    );
    assert!(
        response.body.contains("function_call"),
        "Responses body should include the function call: {}",
        response.body
    );

    let entry = wait_for_proxy_log_entry(&log_path, "/v1/responses");
    assert_eq!(entry["method"], "POST");
    assert_eq!(entry["request_model"], "formal-ai");
    assert_eq!(entry["request_tools"], serde_json::json!(["web_search"]));
    assert_eq!(entry["response_model"], "formal-ai");
    assert_eq!(entry["response_tool_calls"][0]["name"], "web_search");
    assert!(
        entry["response_tool_calls"][0]["arguments"]
            .to_string()
            .contains("Сказка о рыбаке и рыбке"),
        "Responses log should preserve the planned search query: {entry}"
    );

    let _ = fs::remove_file(log_path);
}

#[test]
fn proxy_forwards_gemini_generate_content_and_logs_function_call() {
    let (_server, _proxy, log_path, proxy_port) = spawn_proxy_stack();

    let body = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{"text": "Formalize the fisherman tale into links notation."}]
        }],
        "tools": [{
            "functionDeclarations": [{
                "name": "web_search",
                "parameters": {"type": "object"}
            }]
        }]
    })
    .to_string();

    let path = "/api/gemini/v1beta/models/formal-ai:generateContent";
    let response = http_request(
        "POST",
        proxy_port,
        path,
        Some("sk-local-agentic-tools"),
        Some(&body),
    )
    .expect("proxy Gemini POST should complete");

    assert_eq!(
        response.status_code, 200,
        "proxy should forward Gemini status and body: {}",
        response.body
    );
    assert!(
        response.body.contains("functionCall"),
        "Gemini body should include the function call: {}",
        response.body
    );

    let entry = wait_for_proxy_log_entry(&log_path, path);
    assert_eq!(entry["method"], "POST");
    assert_eq!(entry["request_tools"], serde_json::json!(["web_search"]));
    assert_eq!(entry["response_model"], "formal-ai");
    assert_eq!(entry["response_tool_calls"][0]["name"], "web_search");
    assert!(
        entry["response_tool_calls"][0]["arguments"]
            .to_string()
            .contains("Сказка о рыбаке и рыбке"),
        "Gemini log should preserve the planned search query: {entry}"
    );

    let _ = fs::remove_file(log_path);
}

#[test]
fn proxy_forwards_gemini_stream_generate_content_and_logs_function_call() {
    let (_server, _proxy, log_path, proxy_port) = spawn_proxy_stack();

    let body = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{"text": "Formalize the fisherman tale into links notation."}]
        }],
        "tools": [{
            "functionDeclarations": [{
                "name": "web_search",
                "parameters": {"type": "object"}
            }]
        }]
    })
    .to_string();

    let path = "/api/gemini/v1beta/models/formal-ai:streamGenerateContent";
    let response = http_request(
        "POST",
        proxy_port,
        path,
        Some("sk-local-agentic-tools"),
        Some(&body),
    )
    .expect("proxy Gemini stream POST should complete");

    assert_eq!(
        response.status_code, 200,
        "proxy should forward Gemini stream status and body: {}",
        response.body
    );
    assert!(
        response.content_type.starts_with("text/event-stream"),
        "Gemini stream content-type should be preserved, got {}",
        response.content_type
    );
    assert!(
        response.body.contains("functionCall"),
        "Gemini stream body should include the function call: {}",
        response.body
    );

    let entry = wait_for_proxy_log_entry(&log_path, path);
    assert_eq!(entry["method"], "POST");
    assert_eq!(entry["request_tools"], serde_json::json!(["web_search"]));
    assert_eq!(entry["response_model"], "formal-ai");
    assert_eq!(entry["response_tool_calls"][0]["name"], "web_search");
    assert!(
        entry["response_tool_calls"][0]["arguments"]
            .to_string()
            .contains("Сказка о рыбаке и рыбке"),
        "Gemini stream log should preserve the planned search query: {entry}"
    );

    let _ = fs::remove_file(log_path);
}

fn spawn_proxy_stack() -> (FormalAiServer, FormalAiProxy, std::path::PathBuf, u16) {
    let upstream_port = reserve_loopback_port();
    let proxy_port = reserve_loopback_port();
    let server = spawn_formal_ai_server_agent_mode(upstream_port);
    let log_path = proxy_log_path();
    let proxy = spawn_formal_ai_proxy(proxy_port, upstream_port, &log_path);
    (server, proxy, log_path, proxy_port)
}

fn spawn_formal_ai_proxy(
    proxy_port: u16,
    upstream_port: u16,
    log_path: &std::path::Path,
) -> FormalAiProxy {
    let child = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "proxy",
            "--listen",
            &format!("127.0.0.1:{proxy_port}"),
            "--upstream",
            &format!("http://127.0.0.1:{upstream_port}"),
            "--log",
            &log_path.display().to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn formal-ai proxy");
    let mut proxy = FormalAiProxy { child };
    wait_for_proxy_health(proxy_port, &mut proxy.child);
    proxy
}

/// How long the proxy helpers wait for a slow-but-correct proxy.
///
/// Both waits poll every 50 ms and return the instant the condition holds, so a generous
/// ceiling costs nothing on a healthy run and only decides how much scheduling delay is
/// tolerated before the test calls the proxy broken. The old 5 s ceiling was too tight
/// for the `Code Coverage` job, where `cargo llvm-cov` runs the proxy *and* the upstream
/// server instrumented on a shared runner: `proxy_forwards_streaming_chat_and_logs_tool_call`
/// received the streamed body, then failed because the matching log row had not been
/// flushed within 5 s — while the same test passed in the uninstrumented `Test` job.
const PROXY_WAIT_TIMEOUT: Duration = Duration::from_secs(45);

fn wait_for_proxy_health(port: u16, child: &mut Child) {
    let deadline = Instant::now() + PROXY_WAIT_TIMEOUT;
    let mut last_error = String::from("proxy was not probed");
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("check proxy process") {
            panic!("formal-ai proxy exited before becoming healthy: {status}");
        }
        match http_request("GET", port, "/health", None, None) {
            Ok(response) if response.status_code == 200 => return,
            Ok(response) => {
                last_error = format!(
                    "GET /health through proxy returned HTTP {} with body {}",
                    response.status_code, response.body
                );
            }
            Err(error) => last_error = error.to_string(),
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("formal-ai proxy did not become healthy on port {port}: {last_error}");
}

fn proxy_log_path() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-proxy-{}-{stamp}.jsonl",
        std::process::id()
    ))
}

fn wait_for_proxy_log_entry(path: &std::path::Path, request_path: &str) -> serde_json::Value {
    let deadline = Instant::now() + PROXY_WAIT_TIMEOUT;
    while Instant::now() < deadline {
        if let Ok(log) = fs::read_to_string(path) {
            for line in log.lines() {
                let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                if entry["path"] == request_path {
                    return entry;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "proxy log {} did not contain a row for {request_path}",
        path.display()
    );
}
