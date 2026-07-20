use formal_ai::agentic_coding::{
    google_trends_learning as recipe, is_google_trends_learning_task, plan_chat_step,
    run_agentic_task, AgenticPlan, PlannedToolCall, DRIVER_TOOLS, GOOGLE_TRENDS_LEARNING_PATH,
    GOOGLE_TRENDS_LEARNING_TASK,
};
use formal_ai::{
    recorded_google_trends_frontier, trending_learning_report, ChatMessage, ToolCall,
};
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
    // The honest coverage split. It used to be 20 routed / 60 on the frontier;
    // issue #701's learning cycle derived the missing request-opener surfaces
    // from that very frontier and promoted them through the human gate, so the
    // frontier is now empty. The pre-adoption verdicts are preserved in
    // `data/meta/learning-frontier-google-trends.lino`, and the per-prompt
    // capability deltas in `data/meta/learning-adoption-ledger.lino`.
    assert!(committed.contains("handled_by_engine \"80\""));
    assert!(committed.contains("learning_frontier \"0\""));
    // Still nothing is auto-adopted *here*: adoption went through issue #656.
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
    // Issue #701 closed the gap: the frontier is empty and every catalog prompt
    // routes. This is a ratchet — a regression that unroutes a prompt fails here.
    assert_eq!(report.frontier_count(), 0);
    assert_eq!(report.handled_by_engine, 80);

    // Every frontier prompt is genuinely unrouted, and each becomes a learning trace.
    assert!(report
        .frontier
        .iter()
        .all(|entry| entry.engine_intent == "unknown"));
    assert_eq!(report.run.trace_count, report.frontier_count());

    // The loop is still proposal-only and still adopts nothing on its own: the
    // surfaces that closed the gap were promoted through the human-gated
    // issue-#656 protocol, not auto-adopted here (issue #558's honest behaviour).
    assert!(report.is_proposal_only());
    assert_eq!(report.adopted_count(), 0);
    // With no frontier there is nothing to reject, so there is no uniform reason.
    assert_eq!(report.uniform_rejection_reason(), None);
    assert!(report.summary().contains("frontier is empty"));
}

#[test]
fn the_recorded_learning_frontier_spans_every_supported_language() {
    // Issue #498 requires the learning frontier to span *every* supported language,
    // not just one: English (en), Russian (ru), Hindi (hi), and Chinese (zh). A
    // regression that dropped a locale would silently narrow the engine's own map of
    // which trending prompts it could not yet answer. Each expected fragment is the
    // native-language request template the frontier prompt is built from, so this also
    // pins that the non-English prompts stay in their own script.
    //
    // The live frontier is empty since issue #701 closed the gap, so the check runs
    // against the frozen pre-adoption record — the durable frontier artifact that
    // keeps the failure visible instead of dropping it (R425).
    let recorded = recorded_google_trends_frontier();
    let expected: [(&str, &str); 4] = [
        ("en", "Give Google Trends context for"),
        ("ru", "Дай контекст Google Trends для"),
        ("hi", "के बारे में बताओ"),
        ("zh", "介绍一下"),
    ];
    for (language, request_fragment) in expected {
        assert!(
            recorded
                .iter()
                .any(|item| item.language == language && item.prompt.contains(request_fragment)),
            "the recorded frontier must include {language} prompts like {request_fragment:?}",
        );
    }

    // Coverage is derived from supported_languages(), so every language contributed at
    // least one frontier prompt — none was silently dropped.
    for language in ["en", "ru", "hi", "zh"] {
        assert!(
            recorded.iter().any(|item| item.language == language),
            "language {language} must appear on the recorded frontier",
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
