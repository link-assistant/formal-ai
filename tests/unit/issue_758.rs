//! Regression coverage for capability-first agentic tool routing (issue #758).

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::ChatMessage;

fn routed_tool(prompt: &str, tools: &[&str]) -> String {
    let plan = plan_chat_step(&[ChatMessage::user(prompt)], tools).expect(prompt);
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("expected a tool call for {prompt:?}");
    };
    assert_eq!(calls.len(), 1, "{prompt}");
    calls[0].tool.clone()
}

#[test]
fn local_code_search_uses_every_advertised_cli_alias() {
    for alias in ["grep", "grep_search", "search", "codesearch", "Grep"] {
        assert_eq!(
            routed_tool("search the code for TODO", &[alias, "web_search"]),
            alias,
            "{alias}"
        );
    }
    assert_eq!(
        routed_tool(
            "search the local code for CAPABILITY_SENTINEL",
            &["grep", "web_search"]
        ),
        "grep"
    );
}

#[test]
fn shared_navigation_planning_and_delegation_capabilities_route_by_intent() {
    for (prompt, aliases) in [
        (
            "find all Rust files matching **/*.rs",
            &["glob", "Glob"][..],
        ),
        (
            "show the contents of the src directory",
            &["list_directory", "list", "ls", "LS"][..],
        ),
        (
            "create a todo list for this change",
            &[
                "todo_write",
                "todowrite",
                "update_plan",
                "create_todo_list",
                "update_todo_list",
                "TodoWrite",
            ][..],
        ),
        (
            "delegate this investigation to a subagent",
            &["task", "multi_agent_v1", "invoke_agent", "Task"][..],
        ),
        (
            "read all of these files: Cargo.toml and README.md",
            &["read_many_files"][..],
        ),
        (
            "replace alpha with beta in a.txt and b.txt",
            &["multi_edit", "MultiEdit"][..],
        ),
    ] {
        for alias in aliases {
            assert_eq!(routed_tool(prompt, &[*alias]), *alias, "{prompt}: {alias}");
        }
    }
}

#[test]
fn specialized_navigation_tools_win_over_shell_fallback() {
    assert_eq!(
        routed_tool("search the repository for FIXME", &["bash", "grep"]),
        "grep"
    );
    assert_eq!(
        routed_tool("list files in this folder", &["bash", "list_directory"]),
        "list_directory"
    );
    assert_eq!(
        routed_tool("find files matching *.lino", &["bash", "glob"]),
        "glob"
    );
    assert_eq!(
        routed_tool("search the repository for FIXME", &["codesearch", "grep"]),
        "grep"
    );
}

#[test]
fn compatible_shell_fallback_quotes_each_file_path() {
    let plan = plan_chat_step(
        &[ChatMessage::user("read all of these files: a.txt and b.md")],
        &["bash"],
    )
    .expect("read-many shell fallback");
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("expected a shell tool call");
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "bash");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&calls[0].arguments).unwrap()["command"],
        "cat 'a.txt' 'b.md'"
    );
}

#[test]
fn capability_detection_has_supported_language_parity() {
    for (prompt, tool) in [
        ("найди файлы по шаблону *.rs", "glob"),
        ("इस निर्देशिका की सामग्री दिखाएँ", "list_directory"),
        ("创建这个更改的待办列表", "todo_write"),
        ("поручи эту задачу подагенту", "task"),
    ] {
        assert_eq!(routed_tool(prompt, &[tool]), tool, "{prompt}");
    }
}
