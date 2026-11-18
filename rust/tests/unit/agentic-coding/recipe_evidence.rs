//! A successful tool result proves only the exact action that produced it.
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_symbolic_command_reroute};
use formal_ai::{ChatMessage, ToolCall, UniversalSolver};
use serde_json::json;

fn call(messages: &[ChatMessage]) -> PlannedToolCall {
    let answer = UniversalSolver::default().solve("Write a hello world program in Python.");
    match plan_symbolic_command_reroute(messages, &["write", "bash"], &answer).unwrap() {
        AgenticPlan::ToolCalls(mut calls) => calls.remove(0),
        other @ AgenticPlan::Final(_) => panic!("expected an unfinished step, got {other:?}"),
    }
}

fn record(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("step-{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

#[test]
fn prepared_source_probe_accepts_complete_semantics_without_inventing_an_exit_status() {
    let prompt = "Write a hello world program in Python.";
    let answer = UniversalSolver::default().solve(prompt);
    let recipe = answer.execution_recipe.as_ref().expect("program recipe");
    let request = format!(
        "{prompt}\nIssue to solve: https://github.com/fixture/project/issues/1\n\
         Your prepared Pull Request: https://github.com/fixture/project/pull/2"
    );
    let tools = ["write", "bash"];
    let mut messages = vec![ChatMessage::user(&request)];
    let Some(AgenticPlan::ToolCalls(mut probes)) =
        plan_symbolic_command_reroute(&messages, &tools, &answer)
    else {
        panic!("prepared work must start with a source probe");
    };
    let probe = probes.remove(0);
    assert_eq!(probe.tool, "bash");
    let payload = json!({"schema":"workspace-source-v1", "path":recipe.path,
        "success":true, "complete":true, "absent":true, "error_code":"ENOENT"});
    record(&mut messages, &probe, &payload.to_string());
    let Some(AgenticPlan::ToolCalls(writes)) =
        plan_symbolic_command_reroute(&messages, &tools, &answer)
    else {
        panic!("a bound absence receipt must permit the still-pending write");
    };
    assert_eq!(writes[0].tool, "write");
    let arguments: serde_json::Value = serde_json::from_str(&writes[0].arguments).unwrap();
    assert_eq!(arguments["path"], recipe.path);
    assert_eq!(arguments["content"], recipe.source);

    let mut failed = vec![ChatMessage::user(&request)];
    record(
        &mut failed,
        &probe,
        &json!({"exit_code":1,"stdout":payload.to_string()}).to_string(),
    );
    assert!(matches!(
        plan_symbolic_command_reroute(&failed, &tools, &answer),
        Some(AgenticPlan::Final(_))
    ));
}

#[test]
fn unrelated_or_wrong_content_write_does_not_satisfy_a_recipe_artifact() {
    for (path, content) in [("notes.txt", "unrelated"), ("main.py", "wrong bytes")] {
        let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
        let expected = call(&messages);
        record(
            &mut messages,
            &PlannedToolCall {
                tool: "write".to_owned(),
                arguments: json!({"path": path, "content": content}).to_string(),
            },
            "success",
        );
        assert_eq!(call(&messages), expected);
    }
}

#[test]
fn out_of_order_and_repeated_command_results_do_not_skip_verification() {
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let write = call(&messages);
    record(&mut messages, &write, "success");
    let compile = call(&messages);
    record(
        &mut messages,
        &PlannedToolCall {
            tool: "bash".to_owned(),
            arguments: json!({"command": "python3 main.py"}).to_string(),
        },
        "Exit Code: 0",
    );
    assert_eq!(
        call(&messages),
        compile,
        "run before compile is not compile evidence"
    );
    record(&mut messages, &compile, "Exit Code: 0");
    let verify = call(&messages);
    let duplicate = messages.last().unwrap().clone();
    messages.push(duplicate);
    assert_eq!(
        call(&messages),
        verify,
        "duplicate result cannot satisfy another step"
    );
}

#[test]
fn orphaned_success_result_is_not_evidence_of_a_requested_action() {
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let expected = call(&messages);
    messages.push(ChatMessage::tool_result("unknown", "write", "success"));
    assert_eq!(call(&messages), expected);
}

fn failed_answer(messages: &[ChatMessage]) -> String {
    let answer = UniversalSolver::default().solve("Write a hello world program in Python.");
    match plan_symbolic_command_reroute(messages, &["write", "bash"], &answer).unwrap() {
        AgenticPlan::Final(answer) => {
            assert!(!answer.contains("Created and verified"), "{answer}");
            answer
        }
        other @ AgenticPlan::ToolCalls(_) => panic!("a failure is still unresolved: {other:?}"),
    }
}

#[test]
fn a_bound_successful_retry_resumes_the_recipe_after_a_command_failure() {
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let write = call(&messages);
    record(&mut messages, &write, "success");
    let compile = call(&messages);
    record(
        &mut messages,
        &compile,
        "Exit Code: 127\ncompiler not found",
    );
    assert!(failed_answer(&messages).contains("127"));
    record(&mut messages, &compile, "Exit Code: 0");
    let verify = call(&messages);
    assert_ne!(
        verify, compile,
        "the successful retry should advance exactly one step"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.content.plain_text().contains("compiler not found")),
        "recovery must not erase the original failed observation"
    );
    assert_eq!(
        call(&messages.clone()),
        verify,
        "replay does not depend on hidden process state"
    );
}

#[test]
fn a_bound_successful_retry_resumes_after_a_failed_file_write() {
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let write = call(&messages);
    record(&mut messages, &write, "Error: permission denied");
    failed_answer(&messages);
    record(&mut messages, &write, "success");
    assert_eq!(call(&messages).tool, "bash");
}

#[test]
fn unrelated_setup_or_future_verification_cannot_clear_a_failed_step() {
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let write = call(&messages);
    record(&mut messages, &write, "success");
    let compile = call(&messages);
    let mut successful = messages.clone();
    record(&mut successful, &compile, "Exit Code: 0");
    let verify = call(&successful);
    record(
        &mut messages,
        &compile,
        "Exit Code: 127\ncompiler not found",
    );
    for unrelated in [
        PlannedToolCall {
            tool: "bash".to_owned(),
            arguments: json!({"command": "printf setup-completed"}).to_string(),
        },
        verify.clone(),
    ] {
        record(&mut messages, &unrelated, "Exit Code: 0");
        assert!(failed_answer(&messages).contains("127"));
    }
    messages.push(ChatMessage::tool_result(
        "orphan-retry",
        "bash",
        "Exit Code: 0",
    ));
    assert!(failed_answer(&messages).contains("127"));
    record(&mut messages, &compile, "Exit Code: 0");
    assert_eq!(
        call(&messages),
        verify,
        "out-of-order verification must be repeated after recovery"
    );
}

#[test]
fn replaying_a_success_for_the_old_failed_call_cannot_forge_a_retry() {
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let write = call(&messages);
    record(&mut messages, &write, "success");
    let compile = call(&messages);
    let id = format!("step-{}", messages.len());
    record(
        &mut messages,
        &compile,
        "Exit Code: 127\ncompiler not found",
    );
    messages.push(ChatMessage::tool_result(id, "bash", "Exit Code: 0"));
    assert!(failed_answer(&messages).contains("127"));
    record(
        &mut messages,
        &compile,
        "Exit Code: 2\nordinary compilation error",
    );
    let latest = failed_answer(&messages);
    assert!(
        latest.contains("ordinary compilation error"),
        "latest failed retry must remain visible: {latest}"
    );
}

#[test]
fn structured_probe_diagnostics_do_not_veto_a_bound_recipe_result() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/prerequisite-success-probe-stream.json"
    ))
    .unwrap();
    let observed = fixture["content"].as_str().unwrap();
    assert_eq!(
        formal_ai::agentic_coding::tool_result::step_outcome(observed),
        formal_ai::agentic_coding::tool_result::StepOutcome::Unreported
    );
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let write = call(&messages);
    record(&mut messages, &write, "success");
    let compile = call(&messages);
    record(&mut messages, &compile, observed);
    assert_ne!(
        call(&messages),
        compile,
        "the observed action advances exactly one pending step"
    );
}

#[test]
fn structured_events_cannot_hide_an_outer_process_or_unstructured_failure() {
    let stream = [
        json!({"phase":"probe","exit_code":1,"output":"error: English Русский हिन्दी 中文 Español"})
            .to_string(),
        json!({"phase":"probe","exit_code":0,"output":"ready"}).to_string(),
    ]
    .join("\n");
    for suffix in [
        "Exit Code: 2",
        "Error: permission denied",
        "Failed: compiler stopped",
        "{broken error: invalid argument",
    ] {
        let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
        let write = call(&messages);
        record(&mut messages, &write, "success");
        let compile = call(&messages);
        record(&mut messages, &compile, &format!("{stream}\n{suffix}"));
        let failure = failed_answer(&messages);
        assert!(failure.contains(suffix), "{failure}");
    }
}

#[test]
fn agent_result_preserves_the_captured_nested_exit_status() {
    use formal_ai::agentic_coding::tool_result::{StepOutcome, step_outcome};
    let render = |label: &str, raw: &str, prompt: &str| {
        let request: formal_ai::ChatCompletionRequest = serde_json::from_value(json!({
            "model":"formal-ai",
            "messages":[
                {"role":"user","content":prompt},
                {"role":"assistant","content":"","tool_calls":[{"id":"observed-render","type":"function",
                    "function":{"name":"bash","arguments":json!({"command":label}).to_string()}}]},
                {"role":"tool","name":"bash","tool_call_id":"observed-render","content":raw}
            ],
            "tools":[{"type":"function","function":{"name":"bash","parameters":{"type":"object",
                "properties":{"command":{"type":"string"}},"required":["command"],"additionalProperties":false}}}]
        })).expect("the observed provider result has a declared public protocol frame");
        let solver = UniversalSolver::new(formal_ai::SolverConfig {
            agent_mode: true,
            ..Default::default()
        });
        formal_ai::create_chat_completion_with_solver(&request, &solver)
            .choices
            .into_iter()
            .next()
            .expect("the public protocol returns an observed-result answer")
            .message
            .content
            .plain_text()
    };
    let packet: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/issue-1364-agent-tool-use.json"
    ))
    .expect("captured redacted provider fixture");
    let frame = &packet["fixture"];
    let raw = frame.to_string();
    assert_eq!(step_outcome(&raw), StepOutcome::Failed);
    let message = render(
        "scalac Main.scala",
        &raw,
        "Explain the observed result in English.",
    );
    assert!(message.contains("127"));
    assert!(!message.to_lowercase().contains("no exit code"));
    for code in [1, 13, 42, 127, -1] {
        let mut changed = frame.clone();
        changed["part"]["state"]["error"] = "The compiler is unavailable.".into();
        changed["part"]["state"]["metadata"]["exit"] = code.into();
        let raw = changed.to_string();
        assert_eq!(step_outcome(&raw), StepOutcome::Failed);
        let message = render("exec", &raw, "Explain the observed result in English.");
        assert!(message.contains(&code.to_string()));
        assert!(!message.to_lowercase().contains("no exit code"));
    }
}

#[test]
fn completed_provider_parts_retain_outer_failure_signals() {
    use formal_ai::agentic_coding::tool_result::{StepOutcome, step_outcome};
    let state =
        serde_json::json!({"status":"completed","metadata":{"exit":0,"output":"tool output"}});
    let frame =
        serde_json::json!({"type":"tool_use","part":{"type":"tool","tool":"exec","state":state}});
    assert_eq!(step_outcome(&frame.to_string()), StepOutcome::Succeeded);
    for extra in [
        serde_json::json!({"status":"error"}),
        serde_json::json!({"outcome":"failed"}),
        serde_json::json!({"status_code":500}),
        serde_json::json!({"success":false}),
        serde_json::json!({"is_error":true}),
        serde_json::json!({"timed_out":true}),
        serde_json::json!({"signal":"SIGTERM"}),
    ] {
        let mut changed = frame.clone();
        changed
            .as_object_mut()
            .expect("frame")
            .extend(extra.as_object().expect("signals").clone());
        assert_eq!(step_outcome(&changed.to_string()), StepOutcome::Failed);
    }
}

#[test]
fn typed_git_status_stdout_preserves_leading_columns_before_restart_hygiene() {
    let task = "Issue to solve: https://github.com/example/fixture/issues/1\nYour prepared Pull Request: https://github.com/example/fixture/pull/2\nUNCOMMITTED CHANGES DETECTED\n```\n M main\n```\nContinue.";
    let status = "git status --porcelain --untracked-files=all";
    let frame = serde_json::json!({"type":"tool_use","part":{"type":"tool","tool":"bash",
        "state":{"status":"completed","metadata":{"exit":0,"output":" M main\n"}}}});
    let mut messages = vec![
        formal_ai::ChatMessage::user(task),
        formal_ai::ChatMessage::assistant_tool_calls(vec![formal_ai::ToolCall::function(
            "status",
            "bash",
            serde_json::json!({"command":status}).to_string(),
        )]),
        formal_ai::ChatMessage::tool_result("status", "bash", frame.to_string()),
    ];
    let Some(formal_ai::agentic_coding::AgenticPlan::ToolCalls(calls)) =
        formal_ai::agentic_coding::plan_chat_step(&messages, &["bash"])
    else {
        panic!("exact typed status must request owned hygiene instead of a scope refusal");
    };
    assert_eq!(calls.len(), 1);
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    let command = arguments["command"].as_str().unwrap();
    assert!(command.contains("python3"));
    assert!(!command.contains("gh pr ready"));
    let mut raw = messages.clone();
    raw[2] = formal_ai::ChatMessage::tool_result("status", "bash", " M main\n");
    assert!(matches!(
        formal_ai::agentic_coding::plan_chat_step(&raw, &["bash"]),
        Some(formal_ai::agentic_coding::AgenticPlan::ToolCalls(_))
    ));
    let mut failed = frame;
    failed["isError"] = true.into();
    messages[2] = formal_ai::ChatMessage::tool_result("status", "bash", failed.to_string());
    assert!(matches!(
        formal_ai::agentic_coding::plan_chat_step(&messages, &["bash"]),
        Some(formal_ai::agentic_coding::AgenticPlan::Final(_))
    ));
}

#[test]
fn original_three_program_recipes_probe_external_tools_before_compilation() {
    for language in ["Rust", "Kotlin", "Scala"] {
        let prompt = format!("Write a program in {language} that prints \"Hello, World!\".");
        let answer = UniversalSolver::default().solve(&prompt);
        let recipe = answer.execution_recipe.expect("original language recipe");
        let verifier = recipe
            .supporting_files
            .iter()
            .find(|file| file.path == "tests/verify-output.sh")
            .expect("owned verifier");
        let probe = verifier.source.find("command -v ").expect("tool preflight");
        let workspace = verifier
            .source
            .find("verification_dir=$(mktemp")
            .expect("isolated workspace");
        assert!(
            probe < workspace,
            "all prerequisites precede compiler outputs: {language}"
        );
        assert!(
            verifier
                .source
                .contains("__formal_ai_owned_process_failed_0__")
        );
    }
}
