//! Issue #1017: locate the one-time cost the integration harness pays inside a
//! server's *first* HTTP response.
//!
//! The macOS core slices failed at `tests/integration/http_server.rs:185` with a
//! 30-second `WouldBlock`, which is the harness `RESPONSE_TIMEOUT`. Measured with
//! `experiments/issue-1017-integration-http-latency.sh`, a freshly spawned server
//! answers its first `/api/openai/v1/chat/completions` in ~13 s and every later
//! one in ~0.7 s, and the 13 s is pure CPU. This binary times the same two calls
//! in-process so the cost can be attributed without a socket in the way.
//!
//! Run with: `cargo run --release --example issue_1017_cold_request_profile`
//! (or in `dev` to match the profile CI builds).

use std::time::Instant;

fn main() {
    if std::env::var("PROFILE_AGENT_MODE").as_deref() == Ok("1") {
        formal_ai::server::enable_http_agent_mode_for_current_process();
    }
    let messages: Vec<formal_ai::protocol::ChatMessage> =
        serde_json::from_value(serde_json::json!([{
            "role": "user",
            "content": "look up the latest news about renewable energy"
        }]))
        .expect("messages should deserialize");
    let tool_names = ["web_search", "web_fetch"];

    for attempt in 1..=3 {
        let started = Instant::now();
        let plan = formal_ai::agentic_coding::planner::plan_chat_step(&messages, &tool_names);
        println!(
            "plan_chat_step {attempt}: {} ms (planned {})",
            started.elapsed().as_millis(),
            plan.is_some()
        );
    }

    let body = serde_json::json!({
        "model": "formal-ai",
        "stream": false,
        "messages": [{
            "role": "user",
            "content": "look up the latest news about renewable energy"
        }],
        "tools": [{
            "type": "function",
            "function": {"name": "web_search", "parameters": {"type": "object"}}
        }]
    })
    .to_string();

    for attempt in 1..=3 {
        let started = Instant::now();
        let response =
            formal_ai::server::handle_api_request("POST", "/api/openai/v1/chat/completions", &body);
        println!(
            "request {attempt}: {} ms (status {})",
            started.elapsed().as_millis(),
            response.status_code
        );
    }
}
