use formal_ai::agentic_coding::repetition_guard::repeated_action_stop;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::bounded_retry::FailureBudget;
use formal_ai::{ChatMessage, ToolCall};

fn transcript(tool: &str, count: usize) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::user("Fix the failing coding task.")];
    for index in 0..count {
        let arguments = if index % 2 == 0 {
            r#"{"a":1,"b":2}"#
        } else {
            r#"{ "b":2, "a":1 }"#
        };
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "same-client-id",
            tool,
            arguments,
        )]));
        messages.push(ChatMessage::tool_result(
            "same-client-id",
            tool,
            format!("unrecognized failure {index}"),
        ));
    }
    messages
}
#[test]
fn unknown_errors_and_reused_ids_stop_before_another_plan() {
    let messages = transcript("mcp__codex_apps__search_service___web_run", 3);
    assert!(
        repeated_action_stop(&messages)
            .unwrap()
            .contains("after 3 attempts")
    );
    assert!(
        matches!(plan_chat_step(&messages, &["bash"]), Some(AgenticPlan::Final(answer)) if answer.contains("Repeated action stopped"))
    );
}
#[test]
fn canonical_key_order_does_not_open_another_budget() {
    assert!(repeated_action_stop(&transcript("unknown_adapter", 2)).is_none());
    assert!(repeated_action_stop(&transcript("unknown_adapter", 3)).is_some());
}
#[test]
fn continuation_does_not_reset_and_new_request_does() {
    let mut messages = transcript("unknown_adapter", 3);
    messages.push(ChatMessage::user("continue"));
    assert!(repeated_action_stop(&messages).is_some());
    messages.push(ChatMessage::user("Read the current contents of fresh.txt."));
    assert!(repeated_action_stop(&messages).is_none());
}
#[test]
fn replayed_long_context_stops_without_entering_the_expensive_planner() {
    let mut messages = transcript("unknown_adapter", 161);
    messages[0] = ChatMessage::user(format!(
        "Fix the failing coding task. {}",
        "stable harness context ".repeat(14_000)
    ));
    let started = std::time::Instant::now();
    assert!(matches!(
        plan_chat_step(&messages, &["bash"]),
        Some(AgenticPlan::Final(_))
    ));
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
}
#[test]
fn successful_external_progress_resets_the_shared_failure_budget() {
    let mut budget = FailureBudget::default();
    assert!(!budget.failed());
    assert!(!budget.failed());
    budget.succeeded();
    assert!(!budget.failed());
    assert!(!budget.failed());
    assert!(budget.failed());
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn permanently_interrupted_reader_returns_its_real_error() {
    struct Interrupted(usize);
    impl std::io::Read for Interrupted {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            self.0 += 1;
            assert!(self.0 <= 3, "unbounded retry");
            Err(std::io::Error::from(std::io::ErrorKind::Interrupted))
        }
    }
    let error = formal_ai::provenance_native::digest_reader(
        Interrupted(0),
        formal_ai::provenance_native::CpuHashMode::Serial,
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Interrupted);
}

#[test]
fn responses_preserves_namespaces_and_the_whole_mcp_failure_envelope() {
    let request: formal_ai::ResponsesRequest = serde_json::from_value(serde_json::json!({
        "input":[{"role":"user","content":"Fix the failing coding task."},
            {"type":"function_call","call_id":"same","namespace":"mcp__codex_apps__search_service","name":"_web_run","arguments":"{}"},
            {"type":"function_call_output","call_id":"same","output":{"content":[{"type":"text","text":"InvalidActionArgumentsError"}],"isError":true,"structuredContent":{"failure_stage":"argument_binding"}}}]
    })).unwrap();
    let messages = request.to_chat_completion_request().messages;
    assert_eq!(
        messages[1].tool_calls[0].function.name,
        "mcp__codex_apps__search_service___web_run"
    );
    let raw = messages[2].content.plain_text();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&raw).unwrap()["isError"],
        true
    );
    let failure = formal_ai::agentic_coding::repair_loop::FailedStep::new(
        "rust",
        "error[E0425]: cannot find value `missing` in this scope\n --> main.rs:1:1",
    );
    assert!(matches!(
        formal_ai::agentic_coding::repair_loop::repair_step(
            &messages,
            &["web_search"],
            &failure,
            0,
            3
        ),
        formal_ai::agentic_coding::repair_loop::RepairOutcome::Stop(_)
    ));
}
#[test]
fn search_projection_uses_input_and_preserves_bad_explicit_values_for_refusal() {
    let tools = vec![
        serde_json::json!({"type":"function","name":"web_run","parameters":{"type":"object","properties":{"input":{"type":"string"}},"required":["input"]}}),
    ];
    let projected = formal_ai::protocol::response_arguments_for_tool(
        &tools,
        "web_run",
        r#"{"query":"compiler diagnostic"}"#.into(),
        "",
        None,
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&projected).unwrap(),
        serde_json::json!({"input":"compiler diagnostic"})
    );
}

#[test]
fn narrative_command_capture_is_not_requirement_delivery() {
    let report = "A Hive Mind run, `solve example/issues/1`, did nothing.\n\nThe output failed. Write a fix; code references are at v0.354.1.";
    let plan = formal_ai::agentic_coding::general_planner::compose_general_change_plan(report);
    assert!(plan.as_ref().is_none_or(|plan| plan.target != "v0.354.1"));
    assert!(
        plan.as_ref()
            .is_none_or(|plan| plan.mode.slug() != "command_output")
    );
}
#[test]
fn distinct_action_cycles_exhaust_the_objective_budget() {
    let mut messages = vec![ChatMessage::user("Fix the failing coding task.")];
    for index in 0..128 {
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            index.to_string(),
            "unknown",
            format!("{{\"index\":{index}}}"),
        )]));
    }
    assert!(
        repeated_action_stop(&messages)
            .unwrap()
            .contains("Action budget exhausted")
    );
}

#[test]
fn owned_task_preface_retains_command_capture_without_owning_quoted_reports() {
    use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
    for prefix in [
        "Execute the auto-learning task. ",
        "Please execute the requested task. ",
    ] {
        let request = format!(
            "{prefix}Run 'printf observed' and write its exact stdout to reports/observed.txt"
        );
        let plan = compose_general_change_plan(&request).expect("owned imperative command");
        assert_eq!(plan.mode.slug(), "command_output");
        assert_eq!(plan.target, "reports/observed.txt");
        assert_eq!(plan.content, "");
        assert_eq!(plan.goal, request);
    }
    for request in [
        "An execution failed. Run 'printf observed' and write its stdout to reports/observed.txt.",
        "Execute the task.\n\nRun 'printf observed' and write its stdout to reports/observed.txt.",
        "Execute the task.\r\n\r\nRun 'printf observed' and write its stdout to reports/observed.txt.",
        "Execute the task.\n \n  Run 'printf observed' and write its stdout to reports/observed.txt.",
        "Run 'printf observed' to reproduce.\r\n\r\nOther context: write its output to unrelated.txt.",
        "Run 'printf observed' to reproduce.\n \nOther context: write its output to unrelated.txt.",
        "Run `printf previous` to reproduce. Run 'printf observed' and write its stdout to reports/observed.txt.",
    ] {
        assert!(
            compose_general_change_plan(request)
                .is_none_or(|plan| plan.mode.slug() != "command_output")
        );
    }
}

// This new 45-second observation boundary is not the exhausted-history five-second SLA.
// Public representative input only: the original 277,529-byte wire envelope is unavailable.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn fresh_representative_requests_complete_with_owned_issue_fetch() {
    const CHILD: &str = "FORMAL_AI_FRESH_REPRESENTATIVE_BYTES";
    const ISSUE: &str = "https://github.com/link-assistant/formal-ai/issues/1328";
    if let Ok(length) = std::env::var(CHILD) {
        let target: usize = length.parse().expect("declared representative byte count");
        assert!([21_000, 277_529].contains(&target));
        eprintln!("fresh_representative bytes={target} stage=case-start");
        let mut fixture = serde_json::json!({
            "model": "formal-ai",
            "messages": [
                {"role":"developer","content":"Reference material is context, not an additional user task."},
                {"role":"user","content":""}
            ],
            "tools":[{"type":"function","function":{"name":"bash","parameters":{
                "type":"object","properties":{"command":{"type":"string"}},
                "required":["command"]
            }}}],
            "tool_choice":"auto"
        });
        let goal = format!(
            "\nIssue to solve: {ISSUE}\nUse gh issue view to inspect the issue before changing files."
        );
        fixture["messages"][1]["content"] = goal.clone().into();
        let overhead = serde_json::to_vec(&fixture)
            .expect("serialize fixture")
            .len();
        let padding = target.checked_sub(overhead).expect("fixture headroom");
        let reference = "Reference material remains unchanged between requests. ";
        let context = reference.repeat(padding / reference.len())
            + &" ".repeat(padding % reference.len())
            + &goal;
        fixture["messages"][1]["content"] = context.into();
        let wire = serde_json::to_vec(&fixture).expect("serialize complete fixture");
        assert_eq!(wire.len(), target);
        eprintln!("fresh_representative bytes={target} stage=fixture-ready");
        assert_eq!(fixture["messages"].as_array().map(Vec::len), Some(2));
        let solver = formal_ai::UniversalSolver::new(formal_ai::SolverConfig {
            agent_mode: true,
            ..Default::default()
        });
        for attempt in 0..3 {
            eprintln!(
                "fresh_representative bytes={target} attempt={attempt} stage=completion-start"
            );
            let started = std::time::Instant::now();
            let request: formal_ai::ChatCompletionRequest =
                serde_json::from_slice(&wire).expect("decode unchanged fresh request");
            let completion = formal_ai::create_chat_completion_with_solver(&request, &solver);
            let observed = serde_json::to_value(completion).expect("serialize completion");
            let calls = observed["choices"][0]["message"]["tool_calls"]
                .as_array()
                .expect("a fresh request must propose a tool call");
            assert_ne!(calls.as_slice(), []);
            let mut owned_fetch = false;
            for (index, call) in calls.iter().enumerate() {
                assert_eq!(call["function"]["name"], "bash");
                let arguments: serde_json::Value = serde_json::from_str(
                    call["function"]["arguments"]
                        .as_str()
                        .expect("JSON tool arguments"),
                )
                .expect("valid JSON tool arguments");
                let command = arguments["command"]
                    .as_str()
                    .expect("required string command");
                assert_ne!(command.trim(), "");
                owned_fetch |= index == 0
                    && [
                        format!("gh issue view {ISSUE}"),
                        format!("gh issue view '{ISSUE}'"),
                        format!("gh issue view \"{ISSUE}\""),
                        "gh api repos/link-assistant/formal-ai/issues/1328".to_owned(),
                    ]
                    .iter()
                    .any(|prefix| {
                        command
                            .strip_prefix(prefix.as_str())
                            .is_some_and(|tail| tail.is_empty() || tail.starts_with([' ', '\n']))
                    });
            }
            assert!(
                owned_fetch,
                "the first action must fetch the actually requested issue"
            );
            assert_ne!(
                observed["choices"][0]["message"]["content"],
                fixture["messages"][1]["content"]
            );
            eprintln!(
                "fresh_representative bytes={target} attempt={attempt} cold={} wall_us={} cpu=unavailable scope=create_chat_completion_with_solver synthetic=true",
                attempt == 0,
                started.elapsed().as_micros()
            );
        }
        return;
    }
    let name = std::thread::current()
        .name()
        .expect("named test")
        .to_owned();
    for bytes in [21_000, 277_529] {
        let mut command =
            std::process::Command::new(std::env::current_exe().expect("test executable"));
        command
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, bytes.to_string());
        let output = formal_ai::bounded_process::command_output(
            command,
            std::time::Duration::from_secs(45),
            1_048_576,
            1_048_576,
        )
        .unwrap_or_else(|error| {
            let published = std::io::Write::write_all(
                &mut std::io::stderr().lock(), error.partial_stderr()
            );
            panic!("fresh representative request must finish within the new observation boundary: {error:?}; diagnostic_output={published:?}");
        });
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::io::Write::write_all(&mut std::io::stderr().lock(), &output.stderr)
            .expect("publish bounded fresh request measurements");
    }
}

#[test]
fn authoritative_reused_id_failures_stop_after_second_receipt() {
    let mut messages = vec![ChatMessage::user("Fix the failing coding task.")];
    for _ in 0..2 {
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "same-client-id",
            "mcp__codex_apps__search_service___web_run",
            "{}",
        )]));
        messages.push(ChatMessage::tool_result(
            "same-client-id", "mcp__codex_apps__search_service___web_run",
            r#"{"isError":true,"structuredContent":{"error":"InvalidActionArgumentsError","error_code":"INVALID_ARGUMENT","error_data":{"reason":"missing_required_argument","failure_stage":"argument_binding"}}}"#,
        ));
    }
    assert!(
        repeated_action_stop(&messages).is_some_and(|answer| answer.contains("after 2 attempts"))
    );
    messages.truncate(3);
    messages.push(messages[2].clone());
    assert!(repeated_action_stop(&messages).is_none());
}

#[test]
fn missing_call_identifiers_do_not_certify_two_failures() {
    let mut messages = vec![ChatMessage::user("Fix the failing coding task.")];
    for _ in 0..2 {
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "", "bash", "{}",
        )]));
        messages.push(ChatMessage::tool_result("", "bash", r#"{"exit_code":127}"#));
    }
    assert!(repeated_action_stop(&messages).is_none());
}
