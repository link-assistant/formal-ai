use formal_ai::agentic_coding::{
    google_trends_learning as recipe, is_google_trends_learning_task, plan_chat_step,
    run_agentic_task, AgenticPlan, PlannedToolCall, DRIVER_TOOLS, GOOGLE_TRENDS_LEARNING_PATH,
    GOOGLE_TRENDS_LEARNING_TASK,
};
use formal_ai::{trending_learning_report, ChatMessage, ToolCall};
use lino_objects_codec::format::parse_indented;

fn expect_single_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(mut calls)) => {
            assert_eq!(calls.len(), 1, "the planner emits one call per step");
            calls.remove(0)
        }
        other => panic!("expected a single tool call, got {other:?}"),
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

#[test]
fn committed_google_trends_learning_report_is_generated_by_the_recipe() {
    let committed = include_str!("../../data/meta/google-trends-learning.lino");
    assert_eq!(
        committed,
        recipe::render_document(),
        "the committed Trends learning report is stale; regenerate it from the issue-498 recipe",
    );
    parse_indented(committed).expect("the learning report should parse as Links Notation");
    assert!(committed.contains("google_trends_learning"));
    assert!(committed.contains("auto_learning_loop \"issue_558_self_improvement\""));
    assert!(committed.contains("human_gated \"true\""));
    // The honest coverage split: the engine already routes 20 prompts, and 60 land on
    // the frontier handed to the gated learner — which adopts nothing.
    assert!(committed.contains("handled_by_engine \"20\""));
    assert!(committed.contains("learning_frontier \"60\""));
    assert!(committed.contains("learning_run_adopted \"0\""));
    // A frontier entry is only an *unrouted* prompt; no routed intent leaks in.
    assert!(!committed.contains("engine_intent \"web_search\""));
}

#[test]
fn the_learning_report_is_a_faithful_proposal_only_run() {
    let report = trending_learning_report();
    // The catalog holds 80 prompts (10 topics × 4 languages × 2 variations); the split
    // is derived from the engine, not hardcoded: routed + frontier == total.
    assert_eq!(report.total_prompts, 80);
    assert_eq!(
        report.handled_by_engine + report.frontier_count(),
        report.total_prompts,
        "every prompt is either routed or on the frontier",
    );
    assert_eq!(report.frontier_count(), 60);
    assert_eq!(report.handled_by_engine, 20);

    // Every frontier prompt is genuinely unrouted, and each becomes a learning trace.
    assert!(report
        .frontier
        .iter()
        .all(|entry| entry.engine_intent == "unknown"));
    assert_eq!(report.run.trace_count, report.frontier_count());

    // Open-domain trending questions produce no adoptable rule: the loop stays
    // proposal-only and nothing is auto-adopted (issue #558's honest behaviour).
    assert!(report.is_proposal_only());
    assert_eq!(report.adopted_count(), 0);
    assert_eq!(
        report.uniform_rejection_reason(),
        Some("no rule_synthesis_candidate event"),
    );
}

#[test]
fn the_learning_frontier_spans_every_supported_language() {
    // Issue #498 requires the learning frontier to span *every* supported language,
    // not just one: English (en), Russian (ru), Hindi (hi), and Chinese (zh). A
    // regression that dropped a locale would silently narrow the engine's own map of
    // which trending prompts it cannot yet answer. Each expected fragment is the
    // native-language request template the frontier prompt is built from, so this also
    // pins that the non-English prompts stay in their own script.
    let report = trending_learning_report();
    let expected: [(&str, &str); 4] = [
        ("en", "Give Google Trends context for"),
        ("ru", "Дай контекст Google Trends для"),
        ("hi", "के बारे में बताओ"),
        ("zh", "介绍一下"),
    ];
    for (language, request_fragment) in expected {
        assert!(
            report
                .frontier
                .iter()
                .any(|entry| entry.language == language && entry.prompt.contains(request_fragment)),
            "the learning frontier must include {language} prompts like {request_fragment:?}",
        );
    }

    // Coverage is derived from supported_languages(), so every language contributes at
    // least one frontier prompt — none is silently dropped.
    for language in ["en", "ru", "hi", "zh"] {
        assert!(
            report
                .frontier
                .iter()
                .any(|entry| entry.language == language),
            "language {language} must appear on the frontier",
        );
    }
}

#[test]
fn recognises_the_google_trends_learning_task_without_colliding_with_siblings() {
    assert!(is_google_trends_learning_task(GOOGLE_TRENDS_LEARNING_TASK));
    assert!(is_google_trends_learning_task(
        "Map the Google Trends learning frontier and hand it to the self-improvement loop."
    ));
    assert!(is_google_trends_learning_task(
        "Which trending searches can Formal AI not yet resolve? Route them to the gated learner."
    ));

    // Disjoint from the sibling catalog recipe, in both directions.
    assert!(!is_google_trends_learning_task(
        formal_ai::agentic_coding::GOOGLE_TRENDS_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_google_trends_catalog_task(
        GOOGLE_TRENDS_LEARNING_TASK
    ));
    // And it does not steal the self-healing / ledger auto-learning recipes.
    assert!(!formal_ai::agentic_coding::is_self_heal_task(
        GOOGLE_TRENDS_LEARNING_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_ledger_task(
        GOOGLE_TRENDS_LEARNING_TASK
    ));
    assert!(!is_google_trends_learning_task("what is formal ai?"));
}

#[test]
fn planner_walks_the_google_trends_learning_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(GOOGLE_TRENDS_LEARNING_TASK)];

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(GOOGLE_TRENDS_LEARNING_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    answer_tool_call(&mut messages, &call, "wrote google-trends-learning.lino");

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(GOOGLE_TRENDS_LEARNING_PATH));
    answer_tool_call(&mut messages, &call, &recipe::render_document());

    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(GOOGLE_TRENDS_LEARNING_PATH));
            assert!(answer.contains("learning frontier"));
            assert!(answer.contains("adopted 0"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn committed_agent_cli_session_matches_a_fresh_google_trends_learning_run() {
    let committed = include_str!(
        "../../docs/case-studies/issue-498/agent-cli-session-google-trends-learning.json"
    );
    let fresh = run_agentic_task(GOOGLE_TRENDS_LEARNING_TASK).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed Google Trends learning Agent CLI session is stale",
    );
}

#[test]
fn driver_drives_the_google_trends_learning_recipe_to_a_write() {
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(GOOGLE_TRENDS_LEARNING_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish");

    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    assert_eq!(written["path"], GOOGLE_TRENDS_LEARNING_PATH);
}
