//! Regression coverage for issue #819 local path discovery.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::ChatMessage;

fn first_tool_call(prompt: &str) -> (String, serde_json::Value) {
    let plan = plan_chat_step(
        &[ChatMessage::user(prompt)],
        &["bash", "websearch", "webfetch"],
    )
    .expect(prompt);
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("{prompt:?} did not produce a tool call: {plan:?}");
    };
    let call = calls.first().expect("one tool call");
    let arguments = serde_json::from_str(&call.arguments).expect("JSON tool arguments");
    (call.tool.clone(), arguments)
}

#[test]
fn reported_desktop_request_uses_find_instead_of_the_web() {
    let (tool, arguments) = first_tool_call("Find hive-mind-control center folder on my desktop");

    assert_eq!(tool, "bash");
    let command = arguments["command"].as_str().expect("shell command");
    assert!(command.starts_with("find "), "{command}");
    assert!(command.contains("Desktop"), "{command}");
    assert!(command.contains("-type d"), "{command}");
    assert!(command.contains("hive"), "{command}");
    assert!(command.contains("control"), "{command}");
    assert!(command.contains("center"), "{command}");
}

#[test]
fn local_path_discovery_generalizes_across_language_action_kind_and_scope() {
    for (language, prompt, expected_root, expected_kind) in [
        (
            "en",
            "Locate quarterly-report.pdf on this computer",
            "FORMAL_AI_HOME_DIR",
            "-type f",
        ),
        (
            "ru",
            "Найди папку hive-control-center на моём рабочем столе",
            "FORMAL_AI_DESKTOP_DIR",
            "-type d",
        ),
        (
            "hi",
            "मेरे डेस्कटॉप पर hive-control-center फ़ोल्डर खोजें",
            "FORMAL_AI_DESKTOP_DIR",
            "-type d",
        ),
        (
            "zh",
            "在我的桌面上查找 hive-control-center 文件夹",
            "FORMAL_AI_DESKTOP_DIR",
            "-type d",
        ),
    ] {
        let (tool, arguments) = first_tool_call(prompt);
        assert_eq!(tool, "bash", "{language}: {prompt}");
        let command = arguments["command"].as_str().expect("shell command");
        assert!(command.starts_with("find "), "{language}: {command}");
        assert!(command.contains(expected_root), "{language}: {command}");
        assert!(command.contains(expected_kind), "{language}: {command}");
    }
}

#[test]
fn fuzzy_find_command_locates_the_reported_folder_name() {
    let (_, arguments) = first_tool_call("Find hive-mind-control center folder on my desktop");
    let command = arguments["command"].as_str().expect("shell command");
    let fixture =
        std::env::temp_dir().join(format!("formal-ai-issue819-find-{}", std::process::id()));
    let expected = fixture.join("Archive/hive-control-center");
    std::fs::create_dir_all(&expected).expect("reported folder fixture");

    let output = std::process::Command::new("bash")
        .args(["-c", command])
        .env("FORMAL_AI_DESKTOP_DIR", &fixture)
        .output()
        .expect("execute generated find command");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        expected.to_string_lossy()
    );
    std::fs::remove_dir_all(&fixture).expect("remove isolated fixture");
}

#[test]
fn open_web_find_requests_still_use_web_search() {
    for prompt in [
        "Find information online about hive mind research",
        "Search the web for hive control centers",
    ] {
        let (tool, _) = first_tool_call(prompt);
        assert_eq!(tool, "websearch", "{prompt}");
    }
}
