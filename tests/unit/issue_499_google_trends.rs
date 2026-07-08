//! Issue #499 — convert Google Trends into Formal AI request cases.
//!
//! The issue asks for an automated path from the current Google Trends top ten into
//! reviewable Formal AI test requests, with variations in every supported language.
//! These tests pin that path at three levels: the captured Google Trends RSS snapshot,
//! the generated benchmark fixture, and the agentic recipe that regenerates it.

use std::collections::{BTreeMap, BTreeSet};

use formal_ai::agentic_coding::{
    is_trend_prompt_catalog_task, plan_chat_step, run_agentic_task, trend_prompt_catalog as recipe,
    AgenticPlan, PlannedToolCall, DRIVER_TOOLS, TREND_PROMPT_CATALOG_PATH,
    TREND_PROMPT_CATALOG_TASK,
};
use formal_ai::google_trends::{
    build_prompt_suite, parse_trending_rss, render_prompt_suite_from_rss, TrendPromptSuiteConfig,
    SUPPORTED_TREND_PROMPT_LANGUAGES,
};
use formal_ai::{ChatMessage, ExecutionSurface, SolverConfig, ToolCall, UniversalSolver};
use lino_objects_codec::format::parse_indented;

const RSS_SNAPSHOT: &str =
    include_str!("../../docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml");
const COMMITTED_SUITE: &str = include_str!("../../data/benchmarks/google-trends-top10-suite.lino");

fn fixture_config() -> TrendPromptSuiteConfig {
    TrendPromptSuiteConfig {
        geo: "US".to_owned(),
        captured_at: "2026-07-08T20:29:32Z".to_owned(),
        source_url: "https://trends.google.com/trending/rss?geo=US".to_owned(),
        source_snapshot: "docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml".to_owned(),
        top_n: 10,
    }
}

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
fn google_trends_rss_snapshot_parses_top_ten() {
    let trends = parse_trending_rss(RSS_SNAPSHOT).expect("snapshot should parse");

    assert!(
        trends.len() >= 10,
        "the captured feed should include enough rows for the issue's top-ten requirement",
    );
    let top_ten = &trends[..10];
    assert_eq!(top_ten[0].title, "julián andrés quiñones");
    assert_eq!(top_ten[1].title, "blue jays");
    assert_eq!(top_ten[3].title, "grok 4.5");
    assert!(
        top_ten.iter().all(|trend| !trend.title.trim().is_empty()),
        "every top-ten trend needs a request topic",
    );
    assert!(
        top_ten
            .iter()
            .all(|trend| trend.approx_traffic.ends_with('+')),
        "traffic labels should be decoded from the Google Trends namespace",
    );
    assert!(
        top_ten.iter().any(|trend| !trend.news.is_empty()),
        "the RSS news metadata is part of the traceable raw data",
    );
}

#[test]
fn committed_google_trends_suite_is_generated_from_the_snapshot() {
    let generated = render_prompt_suite_from_rss(RSS_SNAPSHOT, &fixture_config())
        .expect("snapshot should render");

    assert_eq!(
        COMMITTED_SUITE, generated,
        "regenerate with `cargo run --bin formal-ai -- google-trends --input \
         docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml --output \
         data/benchmarks/google-trends-top10-suite.lino --captured-at 2026-07-08T20:29:32Z`",
    );
    parse_indented(COMMITTED_SUITE).expect("generated suite should parse as Links Notation");
    assert!(COMMITTED_SUITE.contains("record_type \"google_trends_prompt_suite\""));
    assert!(COMMITTED_SUITE.contains("minimum_prompt_count \"40\""));
    assert!(COMMITTED_SUITE.contains(
        "source_snapshot \"docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml\""
    ));
}

#[test]
fn prompt_suite_covers_each_top_ten_topic_in_every_supported_language() {
    let suite = build_prompt_suite(RSS_SNAPSHOT, &fixture_config()).expect("suite");
    let expected_languages: BTreeSet<&str> = SUPPORTED_TREND_PROMPT_LANGUAGES.into_iter().collect();

    assert_eq!(suite.trends.len(), 10);
    assert_eq!(suite.prompt_cases.len(), 40);
    assert_eq!(suite.minimum_prompt_count, 40);

    let mut languages_by_rank: BTreeMap<usize, BTreeSet<&str>> = BTreeMap::new();
    for case in &suite.prompt_cases {
        assert!(
            case.prompt.contains(&case.topic),
            "prompt should preserve the trend topic for reviewability: {case:?}",
        );
        assert_eq!(case.expected_use, "formal_ai_request");
        languages_by_rank
            .entry(case.trend_rank)
            .or_default()
            .insert(case.language.as_str());
    }
    for rank in 1..=10 {
        assert_eq!(
            languages_by_rank.get(&rank),
            Some(&expected_languages),
            "trend rank {rank} should have one prompt per supported language",
        );
    }

    let first_topic = &suite.trends[0].title;
    assert!(suite
        .prompt_cases
        .iter()
        .any(|case| { case.language == "en" && case.prompt == format!("What is {first_topic}?") }));
    assert!(suite.prompt_cases.iter().any(|case| {
        case.language == "ru" && case.prompt == format!("Что такое {first_topic}?")
    }));
    assert!(suite.prompt_cases.iter().any(|case| {
        case.language == "hi" && case.prompt == format!("{first_topic} क्या है?")
    }));
    assert!(suite.prompt_cases.iter().any(|case| {
        case.language == "zh" && case.prompt == format!("{first_topic} 是什么?")
    }));
}

#[test]
fn generated_prompts_are_usable_formal_ai_requests() {
    let suite = build_prompt_suite(RSS_SNAPSHOT, &fixture_config()).expect("suite");
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    });

    for case in &suite.prompt_cases {
        let response = solver.solve(&case.prompt);
        assert!(
            !response.answer.trim().is_empty(),
            "prompt case {} should produce a usable Formal AI response",
            case.id,
        );
        assert!(
            (0.0..=1.0).contains(&response.confidence),
            "confidence should remain a probability for {}",
            case.id,
        );
    }
}

#[test]
fn recognises_the_trend_prompt_catalog_task_without_colliding() {
    assert!(is_trend_prompt_catalog_task(TREND_PROMPT_CATALOG_TASK));
    assert!(is_trend_prompt_catalog_task(
        "Convert Google Trends top 10 into Formal AI requests in every language."
    ));
    assert!(is_trend_prompt_catalog_task(
        "Build the trending search prompt catalog from the RSS snapshot."
    ));

    assert!(!is_trend_prompt_catalog_task(
        "what files are in this folder?"
    ));
    assert!(!is_trend_prompt_catalog_task("what is formal ai?"));
    assert!(!formal_ai::agentic_coding::is_question_catalog_task(
        TREND_PROMPT_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_repair_strategy_task(
        TREND_PROMPT_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_self_heal_task(
        TREND_PROMPT_CATALOG_TASK
    ));
}

#[test]
fn planner_walks_the_trend_prompt_catalog_recipe() {
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(TREND_PROMPT_CATALOG_TASK)];

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "write_file");
    assert!(call.arguments.contains(TREND_PROMPT_CATALOG_PATH));
    let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    answer_tool_call(&mut messages, &call, "wrote google trends catalog");

    let call = expect_single_call(&messages, &tools);
    assert_eq!(call.tool, "run_command");
    assert!(call.arguments.contains(TREND_PROMPT_CATALOG_PATH));
    answer_tool_call(&mut messages, &call, &recipe::render_document());

    match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains(TREND_PROMPT_CATALOG_PATH));
            assert!(answer.contains("Google Trends"));
        }
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn committed_agent_cli_session_matches_a_fresh_run() {
    let committed =
        include_str!("../../docs/case-studies/issue-499/agent-cli-session-google-trends.json");
    let fresh = run_agentic_task(TREND_PROMPT_CATALOG_TASK).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed issue-499 Agent CLI session is stale; regenerate it with \
         `cargo run --bin formal-ai -- agent --task \"<TREND_PROMPT_CATALOG_TASK>\" \
         --session-json docs/case-studies/issue-499/agent-cli-session-google-trends.json`",
    );
}

#[test]
fn driver_drives_the_trend_prompt_catalog_recipe_to_a_write() {
    assert!(DRIVER_TOOLS.contains(&"write_file"));
    let outcome = run_agentic_task(TREND_PROMPT_CATALOG_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap, "the loop must finish, not run away");
    let write = outcome
        .steps
        .iter()
        .find(|step| step.tool == "write_file")
        .expect("a write step");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["content"], recipe::render_document());
    assert_eq!(written["path"], TREND_PROMPT_CATALOG_PATH);
    assert!(outcome.final_answer.contains(TREND_PROMPT_CATALOG_PATH));
}
