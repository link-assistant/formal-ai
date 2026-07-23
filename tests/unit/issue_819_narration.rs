//! Narration-quality regressions for issue #819.
//!
//! The user asked that every step be explained in natural, useful language —
//! never by echoing the command that is about to run, and never with the
//! robotic "so I can verify the next step before continuing" tail. These tests
//! exercise the typed solver directly so the assistant's visible message can be
//! asserted precisely.

use formal_ai::protocol::ChatCompletionRequest;
use formal_ai::seed::response_for;
use formal_ai::{create_chat_completion_with_solver, SolverConfig, UniversalSolver};

fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

fn narration(prompt: &str, tools: &serde_json::Value) -> String {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}],
        "tools": tools,
    }))
    .unwrap();
    let completion = create_chat_completion_with_solver(&request, &agent_solver());
    let choice = &completion.choices[0];
    assert_eq!(choice.finish_reason, "tool_calls", "{prompt}");
    choice.message.content.plain_text().trim().to_owned()
}

fn function_tool(name: &str, parameters: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "type": "function",
        "function": {"name": name, "description": name, "parameters": parameters},
    })
}

fn assert_natural_and_command_free(narration: &str) {
    assert!(!narration.is_empty(), "narration must not be blank");
    for leak in [
        "-iname",
        "-type d",
        "-print",
        "find \"",
        "verify the next step",
    ] {
        assert!(
            !narration.contains(leak),
            "narration leaked {leak:?}: {narration}"
        );
    }
}

#[test]
fn desktop_find_reads_as_a_spoken_sentence() {
    let narration = narration(
        "Find hive-mind-control center folder on my desktop",
        &serde_json::json!([
            function_tool("bash", &serde_json::json!({"type": "object"})),
            function_tool("websearch", &serde_json::json!({"type": "object"})),
        ]),
    );
    assert_natural_and_command_free(&narration);
    assert!(narration.contains("Desktop"), "{narration}");
    assert!(narration.contains("hive"), "{narration}");
    // A spoken sentence, not a bare label or a command echo.
    assert!(narration.ends_with('.'), "{narration}");
}

#[test]
fn agentic_action_narration_is_seeded_for_every_supported_language() {
    // The step narration must read naturally in every supported language, not
    // just English — a fix that only pins one language leaves the others free
    // to regress. Each language is asserted with a phrase a native speaker
    // would recognise, and none of them echoes the raw command.
    // English (en): the Desktop find is spoken, not printed.
    let english = response_for("agentic_action_find_desktop", "en").expect("English narration");
    assert!(english.contains("Desktop"), "{english}");
    assert!(!english.contains("find \""), "{english}");
    // Russian (русский): «рабочем столе» is the Desktop.
    let russian = response_for("agentic_action_find_desktop", "ru").expect("Russian narration");
    assert!(russian.contains("столе"), "{russian}");
    // Hindi (हिंदी): डेस्कटॉप is the Desktop.
    let hindi = response_for("agentic_action_find_desktop", "hi").expect("Hindi narration");
    assert!(hindi.contains("डेस्कटॉप"), "{hindi}");
    // Chinese (中文): 桌面 is the Desktop.
    let chinese = response_for("agentic_action_find_desktop", "zh").expect("Chinese narration");
    assert!(chinese.contains("桌面"), "{chinese}");

    // The web-search step, too, is worded distinctly per language so the user
    // can always tell a local search from an internet search.
    assert!(response_for("agentic_action_search", "en") // english
        .expect("English web-search narration")
        .to_lowercase()
        .contains("web"));
    assert!(response_for("agentic_action_search", "ru")
        .expect("Russian web-search narration")
        .contains("интернете"));
    assert!(response_for("agentic_action_search", "hi")
        .expect("Hindi web-search narration")
        .contains("इंटरनेट"));
    assert!(response_for("agentic_action_search", "zh")
        .expect("Chinese web-search narration")
        .contains("上网"));
}

#[test]
fn report_flow_explains_that_it_will_ask_questions() {
    let narration = narration(
        "Report this problem",
        &serde_json::json!([
            function_tool("request_user_input", &serde_json::json!({"type": "object"})),
            function_tool("bash", &serde_json::json!({"type": "object"})),
        ]),
    );
    assert_natural_and_command_free(&narration);
    let lower = narration.to_lowercase();
    assert!(
        lower.contains("ask") || lower.contains("question"),
        "the report step should say it will ask the user: {narration}"
    );
}
