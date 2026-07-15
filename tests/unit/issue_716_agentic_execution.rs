use formal_ai::protocol::{
    create_chat_completion_with_solver, ChatCompletionRequest, ChatMessage, ToolCall,
};
use formal_ai::solver::{ExecutionSurface, SolverConfig, UniversalSolver};
use serde_json::{json, Value};

struct LocalizedAgentCase {
    language: &'static str,
    prompt: &'static str,
}

fn function_tool(name: &str) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": format!("issue-716 test tool: {name}"),
            "parameters": {"type": "object"}
        }
    })
}

fn solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

fn request(messages: Vec<ChatMessage>, tools: &[&str]) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: Some(String::from("formal-ai")),
        messages,
        temperature: None,
        stream: false,
        tools: tools.iter().map(|name| function_tool(name)).collect(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    }
}

fn only_call(request: &ChatCompletionRequest) -> ToolCall {
    let completion = create_chat_completion_with_solver(request, &solver());
    assert_eq!(completion.choices[0].finish_reason, "tool_calls");
    assert_eq!(completion.choices[0].message.tool_calls.len(), 1);
    completion.choices[0].message.tool_calls[0].clone()
}

#[test]
fn code_generation_writes_source_then_runs_every_catalog_command_in_cli_harness() {
    let mut messages = vec![ChatMessage::user("Give me hello world program in Rust")];

    let write = only_call(&request(messages.clone(), &["write", "bash"]));
    assert_eq!(write.function.name, "write");
    let args: Value = serde_json::from_str(&write.function.arguments).unwrap();
    assert_eq!(args["filePath"], "main.rs");
    assert!(args["content"].as_str().unwrap().contains("Hello, world!"));
    messages.push(ChatMessage::assistant_tool_calls(vec![write.clone()]));
    messages.push(ChatMessage::tool_result(
        &write.id,
        "write",
        "Wrote main.rs",
    ));

    let check = only_call(&request(messages.clone(), &["write", "bash"]));
    assert_eq!(check.function.name, "bash");
    assert_eq!(
        serde_json::from_str::<Value>(&check.function.arguments).unwrap()["command"],
        "rustc main.rs -o main"
    );
    messages.push(ChatMessage::assistant_tool_calls(vec![check.clone()]));
    messages.push(ChatMessage::tool_result(&check.id, "bash", ""));

    let run = only_call(&request(messages.clone(), &["write", "bash"]));
    assert_eq!(run.function.name, "bash");
    assert_eq!(
        serde_json::from_str::<Value>(&run.function.arguments).unwrap()["command"],
        "./main"
    );
    messages.push(ChatMessage::assistant_tool_calls(vec![run.clone()]));
    messages.push(ChatMessage::tool_result(&run.id, "bash", "Hello, world!\n"));

    let completion =
        create_chat_completion_with_solver(&request(messages, &["write", "bash"]), &solver());
    assert_eq!(completion.choices[0].finish_reason, "stop");
    let answer = completion.choices[0].message.content.plain_text();
    assert!(answer.contains("Hello, world!"));
    assert!(answer.contains("agentic CLI harness"));
    assert!(!answer.contains("issue-8 local verification harness"));
}

#[test]
fn follow_up_change_writes_the_updated_program_instead_of_repeating_old_code() {
    let messages = vec![
        ChatMessage::user("Give me hello world program in Rust"),
        ChatMessage::assistant("Here is the program."),
        ChatMessage::user("Change the output message to `Hello 2`."),
    ];
    let write = only_call(&request(messages, &["write_file", "run_command"]));
    assert_eq!(write.function.name, "write_file");
    let args: Value = serde_json::from_str(&write.function.arguments).unwrap();
    assert_eq!(args["path"], "main.rs");
    assert!(
        args["content"].as_str().unwrap().contains("Hello 2"),
        "unexpected write arguments: {args}"
    );
    assert!(!args["content"].as_str().unwrap().contains("Hello, world!"));
}

#[test]
fn follow_up_output_change_preserves_non_output_literals_in_other_languages() {
    let messages = vec![
        ChatMessage::user("Give me hello world program in Go"),
        ChatMessage::assistant("Here is the program."),
        ChatMessage::user("Change the output message to `Hello 2`."),
    ];
    let write = only_call(&request(messages, &["write", "shell"]));
    let args: Value = serde_json::from_str(&write.function.arguments).unwrap();
    let source = args["content"].as_str().unwrap();
    assert!(source.contains("import \"fmt\""));
    assert!(source.contains("fmt.Println(\"Hello 2\")"));
}

#[test]
fn code_generation_does_not_fabricate_calls_when_required_harness_tools_are_absent() {
    let request = request(
        vec![ChatMessage::user("Create a Python hello world program")],
        &["web_search"],
    );
    let completion = create_chat_completion_with_solver(&request, &solver());
    assert_eq!(completion.choices[0].finish_reason, "stop");
    assert!(completion.choices[0].message.tool_calls.is_empty());
}

#[test]
fn failed_compile_is_reported_and_the_run_command_is_not_attempted() {
    let mut messages = vec![ChatMessage::user(
        "Produce a Rust hello world program for me",
    )];
    let write = only_call(&request(messages.clone(), &["write", "bash"]));
    messages.push(ChatMessage::assistant_tool_calls(vec![write.clone()]));
    messages.push(ChatMessage::tool_result(
        &write.id,
        "write",
        "Wrote main.rs",
    ));
    let check = only_call(&request(messages.clone(), &["write", "bash"]));
    messages.push(ChatMessage::assistant_tool_calls(vec![check.clone()]));
    messages.push(ChatMessage::tool_result(
        &check.id,
        "bash",
        "command exited with status 1\nstderr:\nerror: compile failed",
    ));

    let completion =
        create_chat_completion_with_solver(&request(messages, &["write", "bash"]), &solver());
    assert_eq!(completion.choices[0].finish_reason, "stop");
    assert!(completion.choices[0].message.tool_calls.is_empty());
    let answer = completion.choices[0].message.content.plain_text();
    assert!(answer.contains("could not complete"));
    assert!(answer.contains("compile failed"));
}

#[test]
fn http_api_never_executes_agent_actions_in_the_servers_embedded_workspace() {
    let prompt = "[agent] run terminal command `ls`";
    let local = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        execution_surface: ExecutionSurface::Cli,
        ..SolverConfig::default()
    })
    .solve(prompt);
    assert_eq!(local.intent, "agent_workspace_task");
    assert!(local.answer.contains("Workspace isolation:"));

    let cases = [
        LocalizedAgentCase {
            language: "en",
            prompt,
        },
        LocalizedAgentCase {
            language: "ru",
            prompt: "[agent] выполнить терминальную команду `ls`",
        },
        LocalizedAgentCase {
            language: "hi",
            prompt: "[agent] टर्मिनल कमांड `ls` चलाएँ",
        },
        LocalizedAgentCase {
            language: "zh",
            prompt: "[agent] 运行终端命令 `ls`",
        },
    ];

    for case in cases {
        let api = UniversalSolver::new(SolverConfig {
            agent_mode: true,
            execution_surface: ExecutionSurface::HttpServer,
            ..SolverConfig::default()
        })
        .solve(case.prompt);

        assert_ne!(api.intent, "agent_workspace_task", "{}", case.language);
        assert!(
            !api.answer.contains("Workspace isolation:"),
            "{}",
            case.language
        );
        assert!(
            !api.answer.contains("Execution status: completed"),
            "{}",
            case.language
        );
    }
}
