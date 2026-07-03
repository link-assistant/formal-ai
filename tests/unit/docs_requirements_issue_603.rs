use std::fs;
use std::path::Path;

#[test]
fn issue_603_multi_protocol_gateway_docs_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "## Agentic AI Tools",
            "cargo run -- serve --host 127.0.0.1 --port 8080",
            "FORMAL_AI_API_BEARER_TOKEN",
            "~/.codex/config.toml",
            "wire_api = \"responses\"",
            "/api/openai/v1",
            "ANTHROPIC_BASE_URL",
            "/api/anthropic/v1/messages",
            "/v1/messages",
            "GOOGLE_GEMINI_BASE_URL",
            "/api/gemini/v1beta/models",
            "GOOGLE_VERTEX_BASE_URL",
            "/api/vertex/v1",
            "~/.config/opencode/opencode.json",
            "@ai-sdk/openai-compatible",
            "/v1/chat/completions",
            "~/.config/link-assistant-agent/opencode.json",
            "formal-ai serve --agent-mode --host 127.0.0.1 --port 8080",
            "agent --model formal-ai/formal-ai --permission-mode plan",
            "/v1/responses",
        ],
    );

    let server_api = read(root.join("docs/desktop/server-api.md"));
    assert_contains_all(
        "docs/desktop/server-api.md",
        &server_api,
        &[
            "POST /v1/responses",
            "POST /v1/chat/completions",
            "POST /v1/messages",
            "POST /api/openai/v1/responses",
            "POST /api/openai/v1/chat/completions",
            "POST /api/anthropic/v1/messages",
            "POST /api/gemini/v1beta/models/{model}:generateContent",
            "GOOGLE_GEMINI_BASE_URL",
            "GOOGLE_VERTEX_BASE_URL",
            "/api/vertex/v1",
            "~/.codex/config.toml",
            "wire_api = \"responses\"",
            "~/.config/opencode/opencode.json",
            "~/.config/link-assistant-agent/opencode.json",
            "@ai-sdk/openai-compatible",
            "FORMAL_AI_API_KEY",
            "ANTHROPIC_BASE_URL",
            "Codex configuration reference",
            "OpenCode provider documentation",
            "link-assistant/agent",
        ],
    );
    assert!(
        !server_api.contains("wire_api = \"chat\""),
        "Codex docs must stay on the Responses wire API"
    );
    assert!(
        !server_api.contains("export OPENAI_BASE_URL=\"http://127.0.0.1:8080/v1\""),
        "agent docs should use the supported OpenCode-style provider config"
    );
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
