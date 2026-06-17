//! Render Telegram webhook replies for issue #488 to show the concrete
//! "thinking" reasoning surfaced as a native expandable blockquote.
//!
//! The deep-thinking UX is not UI-only: every non-UI surface (CLI `--thinking`,
//! the OpenAI/Anthropic APIs, and — shown here — the Telegram bot) renders the
//! same concrete reasoning steps. On Telegram the steps ride inside the
//! platform's native `<blockquote expandable>`, which is collapsed by default
//! and expands on tap, so the answer leads and the reasoning stays one tap away.
//!
//! Run with: `cargo run --example issue_488_telegram_thinking`

use formal_ai::handle_api_request;

const PROMPTS: &[&str] = &[
    "Hi",
    "2 + 2",
    "What is 8% of 50?",
    "Write me hello world program in Rust",
];

fn webhook_reply(prompt: &str, id: i64) -> String {
    let body = serde_json::json!({
        "update_id": id,
        "message": {
            "message_id": id,
            "date": 0,
            "chat": {"id": 42, "type": "private"},
            "text": prompt
        }
    })
    .to_string();
    let response = handle_api_request("POST", "/telegram/webhook", &body);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("telegram reply should be JSON");
    json["text"].as_str().unwrap_or_default().to_owned()
}

fn main() {
    for (index, prompt) in PROMPTS.iter().enumerate() {
        let id = i64::try_from(index).expect("prompt index fits in i64") + 1;
        let reply = webhook_reply(prompt, id);
        println!("=== prompt: {prompt} ===");
        println!("{reply}");
        println!();
    }
}
