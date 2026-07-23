//! End-to-end coverage for issue #819 pre-tool narration.
//!
//! The reported `OpenCode` session narrated each step by printing the raw shell
//! command that was about to run and a robotic "so I can verify the next step
//! before continuing" tail. Real assistants (Claude, Codex) instead say, in
//! plain words, *what* they are about to do. These tests drive the actual HTTP
//! API — the same path the wrapped TUI uses — and assert that the assistant's
//! visible message is natural and never leaks the command that `OpenCode` itself
//! prints when the step executes.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

/// Fragments that reveal the raw `find` command or the old robotic phrasing.
/// None of these belong in a natural, spoken explanation.
const COMMAND_LEAKS: [&str; 7] = [
    "-iname",
    "-type d",
    "-type f",
    "-print",
    "find \"",
    "verify the next step",
    "before continuing",
];

fn narration_for(prompt: &str, tools: &Value) -> String {
    let body = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}],
        "tools": tools,
    });
    let response = post("/v1/chat/completions", &body);
    let message = &response["choices"][0]["message"];
    assert_eq!(
        response["choices"][0]["finish_reason"], "tool_calls",
        "{prompt}: expected a tool call, got {response}"
    );
    message["content"]
        .as_str()
        .unwrap_or_else(|| panic!("{prompt}: narration must be a string, got {message}"))
        .to_owned()
}

fn assert_command_free(narration: &str, prompt: &str) {
    assert!(
        !narration.trim().is_empty(),
        "{prompt}: the user must see what is about to happen"
    );
    for leak in COMMAND_LEAKS {
        assert!(
            !narration.contains(leak),
            "{prompt}: narration leaked {leak:?}: {narration}"
        );
    }
}

fn bash_and_web() -> Value {
    json!([
        chat_tool("bash", &command_schema()),
        chat_tool("websearch", &web_search_schema()),
    ])
}

#[test]
fn desktop_find_is_explained_without_printing_the_command() {
    let narration = narration_for(
        "Find hive-mind-control center folder on my desktop",
        &bash_and_web(),
    );
    assert_command_free(&narration, "desktop find");
    assert!(
        narration.contains("Desktop"),
        "narration should name the location: {narration}"
    );
    assert!(
        narration.contains("hive"),
        "narration should name what is being looked for: {narration}"
    );
}

#[test]
fn desktop_find_narration_is_localized_and_command_free() {
    for (language, prompt, location) in [
        (
            "ru",
            "Найди папку hive-control-center на моём рабочем столе",
            "рабочем столе",
        ),
        ("hi", "मेरे डेस्कटॉप पर hive-control-center फ़ोल्डर खोजें", "डेस्कटॉप"),
        ("zh", "在我的桌面上查找 hive-control-center 文件夹", "桌面"),
    ] {
        let narration = narration_for(prompt, &bash_and_web());
        assert_command_free(&narration, language);
        assert!(
            narration.contains(location),
            "{language}: narration should name the location: {narration}"
        );
        assert!(
            narration.contains("hive"),
            "{language}: narration should name the subject: {narration}"
        );
    }
}

#[test]
fn opencode_run_shell_schema_still_gets_a_command_free_explanation() {
    let tools = json!([
        chat_tool(
            "run_shell_command",
            &json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["command", "description"],
                "additionalProperties": false
            })
        ),
        chat_tool("websearch", &web_search_schema()),
    ]);
    let narration = narration_for("Find hive-mind-control center folder on my desktop", &tools);
    assert_command_free(&narration, "OpenCode run_shell_command");
    assert!(narration.contains("Desktop"), "{narration}");
}

#[test]
fn open_web_search_is_explained_as_a_web_search() {
    let narration = narration_for("Search the web for hive control centers", &bash_and_web());
    assert_command_free(&narration, "web search");
    let lower = narration.to_lowercase();
    assert!(
        lower.contains("web") || lower.contains("search"),
        "narration should read as a web search: {narration}"
    );
}

#[test]
fn fetch_narration_names_the_source_url() {
    let body = json!({
        "model": "formal-ai",
        "messages": [
            {"role": "user", "content": "Find current evidence for this laptop charger?"},
            {
                "role": "assistant",
                "tool_calls": [{
                    "id": "search_1",
                    "type": "function",
                    "function": {"name": "websearch", "arguments": "{\"query\":\"charger\"}"}
                }]
            },
            {
                "role": "tool",
                "tool_call_id": "search_1",
                "name": "websearch",
                "content": "Result https://example.test/charger"
            }
        ],
        "tools": [
            chat_tool("websearch", &web_search_schema()),
            chat_tool(
                "webfetch",
                &json!({
                    "type": "object",
                    "properties": {"url": {"type": "string"}}
                })
            ),
        ]
    });
    let response = post("/v1/chat/completions", &body);
    let narration = response["choices"][0]["message"]["content"]
        .as_str()
        .expect("fetch narration string");
    assert!(
        narration.contains("https://example.test/charger"),
        "fetch narration should name the source it opens: {narration}"
    );
    assert_command_free(narration, "fetch");
}

fn chat_tool(name: &str, parameters: &Value) -> Value {
    json!({"type": "function", "function": {"name": name, "parameters": parameters}})
}

fn command_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"command": {"type": "string"}},
        "required": ["command"],
        "additionalProperties": false
    })
}

fn web_search_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"query": {"type": "string"}},
        "required": ["query"]
    })
}

fn post(path: &str, body: &Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let response = handle_api_request("POST", path, &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    serde_json::from_str(&response.body).expect("JSON response")
}
