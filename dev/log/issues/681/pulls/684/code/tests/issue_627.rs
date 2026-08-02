use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::{
    create_chat_completion_with_solver, ChatCompletionRequest, ChatMessage, SolverConfig, ToolCall,
    UniversalSolver,
};

const TOOLS: [&str; 2] = ["read", "bash"];

fn expect_single_call(messages: &[ChatMessage]) -> PlannedToolCall {
    match plan_chat_step(messages, &TOOLS) {
        Some(AgenticPlan::ToolCalls(mut calls)) => {
            assert_eq!(calls.len(), 1, "planner should emit one tool call");
            calls.remove(0)
        }
        other => panic!("expected one tool call, got {other:?}"),
    }
}

fn expect_calls(messages: &[ChatMessage], count: usize) -> Vec<PlannedToolCall> {
    match plan_chat_step(messages, &TOOLS) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), count, "unexpected tool call count");
            calls
        }
        other => panic!("expected {count} tool calls, got {other:?}"),
    }
}

fn answer_tool_call(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

fn arguments(call: &PlannedToolCall) -> serde_json::Value {
    serde_json::from_str(&call.arguments).expect("tool arguments should be JSON")
}

#[test]
fn direct_file_read_prompts_emit_read_tool_calls() {
    for (prompt, expected_path) in [
        ("read the file alpha.txt", "alpha.txt"),
        ("show me the contents of beta.md", "beta.md"),
        ("open alpha.txt and tell me what's inside", "alpha.txt"),
        ("what does beta.md say?", "beta.md"),
        ("please read gamma.json for me", "gamma.json"),
        (
            "what is the value of gamma_marker in gamma.json?",
            "gamma.json",
        ),
        ("print the first line of alpha.txt", "alpha.txt"),
    ] {
        let messages = vec![ChatMessage::user(prompt)];
        let call = expect_single_call(&messages);
        assert_eq!(call.tool, "read", "{prompt:?} should use read");
        assert_eq!(arguments(&call)["filePath"], expected_path);
    }
}

#[test]
fn cat_file_prompt_uses_bash_in_agent_mode() {
    let messages = vec![ChatMessage::user("cat gamma.json")];
    let call = expect_single_call(&messages);

    assert_eq!(call.tool, "bash");
    assert_eq!(arguments(&call)["command"], "cat gamma.json");
}

#[test]
fn chat_completion_does_not_fall_back_to_url_answer_for_local_filename() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "show me the contents of beta.md"}],
        "tools": [{
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read a file",
                "parameters": {
                    "type": "object",
                    "properties": {"filePath": {"type": "string"}},
                    "required": ["filePath"]
                }
            }
        }],
        "stream": false
    }))
    .unwrap();
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });

    let response = create_chat_completion_with_solver(&request, &solver);
    let choice = &response.choices[0];
    assert_eq!(choice.finish_reason, "tool_calls");
    assert_eq!(choice.message.tool_calls.len(), 1);
    assert_eq!(choice.message.tool_calls[0].function.name, "read");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&choice.message.tool_calls[0].function.arguments)
            .unwrap()["filePath"],
        "beta.md"
    );
    assert!(
        !choice
            .message
            .content
            .plain_text()
            .contains("https://beta.md"),
        "local filename must not be converted into a URL answer"
    );
}

#[test]
fn list_then_read_first_file_walks_tool_loop_to_final_content() {
    let mut messages = vec![ChatMessage::user(
        "list the files then read the first one alphabetically",
    )];

    let list = expect_single_call(&messages);
    assert_eq!(list.tool, "bash");
    assert_eq!(
        arguments(&list)["command"],
        "find . -maxdepth 1 -type f | sed 's#^./##' | sort"
    );
    answer_tool_call(&mut messages, &list, "alpha.txt\nbeta.md\ngamma.json\n");

    let read = expect_single_call(&messages);
    assert_eq!(read.tool, "read");
    assert_eq!(arguments(&read)["filePath"], "alpha.txt");
    answer_tool_call(&mut messages, &read, "ALPHA_MARKER_11111\nsecond line\n");

    match plan_chat_step(&messages, &TOOLS) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("ALPHA_MARKER_11111"));
        }
        other => panic!("expected final answer with read content, got {other:?}"),
    }
}

#[test]
fn read_file_in_named_folder_lists_then_reads_nested_file() {
    let mut messages = vec![ChatMessage::user("read the file in the subdir folder")];

    let list = expect_single_call(&messages);
    assert_eq!(list.tool, "bash");
    assert_eq!(
        arguments(&list)["command"],
        "find subdir -maxdepth 1 -type f | sed 's#^.*/##' | sort"
    );
    answer_tool_call(&mut messages, &list, "nested.md\n");

    let read = expect_single_call(&messages);
    assert_eq!(read.tool, "read");
    assert_eq!(arguments(&read)["filePath"], "subdir/nested.md");
    answer_tool_call(&mut messages, &read, "NESTED_MARKER_44444\n");

    match plan_chat_step(&messages, &TOOLS) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("NESTED_MARKER_44444"));
        }
        other => panic!("expected final answer with nested content, got {other:?}"),
    }
}

#[test]
fn ls_folder_prompt_reads_last_file_from_listing() {
    let mut messages = vec![ChatMessage::user(
        "ls the folder and show me the contents of the last file",
    )];

    let list = expect_single_call(&messages);
    assert_eq!(list.tool, "bash");
    assert_eq!(
        arguments(&list)["command"],
        "find . -maxdepth 1 -type f | sed 's#^./##' | sort"
    );
    answer_tool_call(&mut messages, &list, "alpha.txt\nbeta.md\ngamma.json\n");

    let read = expect_single_call(&messages);
    assert_eq!(read.tool, "read");
    assert_eq!(arguments(&read)["filePath"], "gamma.json");
    answer_tool_call(
        &mut messages,
        &read,
        "{\"gamma_marker\":\"GAMMA_33333\",\"n\":42}\n",
    );

    match plan_chat_step(&messages, &TOOLS) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("GAMMA_33333"));
        }
        other => panic!("expected final answer with last file content, got {other:?}"),
    }
}

#[test]
fn read_every_file_lists_then_reads_each_file() {
    let mut messages = vec![ChatMessage::user("read every file here and summarize them")];

    let list = expect_single_call(&messages);
    assert_eq!(list.tool, "bash");
    answer_tool_call(&mut messages, &list, "alpha.txt\nbeta.md\ngamma.json\n");

    let reads = expect_calls(&messages, 3);
    let paths: Vec<String> = reads
        .iter()
        .map(|call| {
            assert_eq!(call.tool, "read");
            arguments(call)["filePath"].as_str().unwrap().to_owned()
        })
        .collect();
    assert_eq!(paths, ["alpha.txt", "beta.md", "gamma.json"]);

    for (call, result) in reads.iter().zip([
        "ALPHA_MARKER_11111\n",
        "BETA_MARKER_22222\n",
        "{\"gamma_marker\":\"GAMMA_33333\",\"n\":42}\n",
    ]) {
        answer_tool_call(&mut messages, call, result);
    }

    match plan_chat_step(&messages, &TOOLS) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("ALPHA_MARKER_11111"));
            assert!(answer.contains("BETA_MARKER_22222"));
            assert!(answer.contains("GAMMA_33333"));
        }
        other => panic!("expected final summary, got {other:?}"),
    }
}
