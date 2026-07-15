//! Regression matrix for issue #712: semantically equivalent tool requests must not be phrasing-gated.
//!
//! Every row below is copied from the live v0.289.0 report and failed before the fix.
use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::ChatMessage;
fn single_call(prompt: &str, tools: &[&str]) -> (String, serde_json::Value) {
    let messages = vec![ChatMessage::user(prompt)];
    match plan_chat_step(&messages, tools) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1, "expected one call for {prompt:?}");
            let call = &calls[0];
            let arguments = serde_json::from_str(&call.arguments).expect("valid tool arguments");
            (call.tool.clone(), arguments)
        }
        other => panic!("expected a tool call for {prompt:?}, got {other:?}"),
    }
}
#[test]
fn failed_web_fetch_phrasings_route_by_url_intent() {
    for prompt in [
        "get the contents of https://example.com",
        "retrieve https://example.com",
        "open https://example.com and tell me what it says",
        "load https://example.com",
        "visit https://example.com and summarize it",
    ] {
        let (tool, arguments) = single_call(prompt, &["web_fetch"]);
        assert_eq!(tool, "web_fetch", "{prompt}");
        assert_eq!(arguments["url"], "https://example.com", "{prompt}");
    }
}
#[test]
fn failed_web_search_phrasings_route_by_research_intent() {
    for prompt in [
        "google what is a monad",
        "what does the web say about serde",
        "I need current info from the internet on axum",
    ] {
        let (tool, arguments) = single_call(prompt, &["web_search"]);
        assert_eq!(tool, "web_search", "{prompt}");
        assert!(
            arguments["query"]
                .as_str()
                .is_some_and(|query| !query.trim().is_empty()),
            "{prompt}: {arguments}"
        );
    }
}

#[test]
fn unseen_web_search_paraphrases_route_by_semantic_frame() {
    let cases = [
        (
            "Google tokio scheduler fairness",
            "tokio scheduler fairness",
        ),
        ("What does the internet report regarding serde?", "serde"),
        (
            "I require current information online concerning axum",
            "axum",
        ),
    ];
    for (prompt, subject) in cases {
        let (tool, arguments) = single_call(prompt, &["web_search"]);
        assert_eq!(tool, "web_search", "{prompt}");
        assert_eq!(arguments["query"], subject, "{prompt}");
    }
}

#[test]
fn source_frames_require_whole_seeded_source_markers() {
    let messages = vec![ChatMessage::user("webhook status regarding serde")];
    assert_eq!(plan_chat_step(&messages, &["web_search"]), None);
}
#[test]
fn failed_edit_phrasings_route_by_replacement_shape() {
    for action in [
        "update",
        "modify",
        "patch",
        "rewrite",
        "substitute",
        "refactor",
    ] {
        let prompt = format!("{action} main.rs and change foo to bar");
        let (tool, arguments) = single_call(&prompt, &["edit"]);
        assert_eq!(tool, "edit", "{prompt}");
        assert_eq!(arguments["path"], "main.rs", "{prompt}");
        assert_eq!(arguments["old"], "foo", "{prompt}");
        assert_eq!(arguments["new"], "bar", "{prompt}");
    }
}
#[test]
fn declarative_new_file_routes_to_write_not_read() {
    let prompt = "new file: notes.txt, contents: hello";
    let (tool, arguments) = single_call(prompt, &["write", "read"]);
    assert_eq!(tool, "write");
    assert!(arguments.to_string().contains("notes.txt"));
}

#[test]
fn seeded_write_parser_is_the_single_write_routing_gate() {
    for prompt in [
        "save file report.md with content ready",
        "создай файл report.md с содержанием готово",
        "फ़ाइल report.md बनाओ सामग्री के साथ तैयार",
        "创建 文件 report.md 内容为 完成",
    ] {
        let plan = compose_general_change_plan(prompt)
            .unwrap_or_else(|| panic!("seeded write frame should parse: {prompt}"));
        assert_eq!(plan.target, "report.md", "{prompt}");
    }
}
