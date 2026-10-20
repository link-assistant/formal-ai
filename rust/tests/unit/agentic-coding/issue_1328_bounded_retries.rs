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
        assert!(plan.content.is_empty());
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
