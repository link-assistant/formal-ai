//! Regression coverage for the task-ladder false greens found by issue #842.

use std::{fs, path::Path};

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::{ChatMessage, ToolCall};

fn one_call(prompt: &str) -> PlannedToolCall {
    let plan = plan_chat_step(
        &[ChatMessage::user(prompt)],
        &["bash", "websearch", "request_user_input"],
    )
    .expect("the task should be planned");
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("expected one tool call, got {plan:?}");
    };
    assert_eq!(calls.len(), 1, "{calls:?}");
    calls.into_iter().next().unwrap()
}

#[test]
fn definition_example_requests_route_to_research_in_every_supported_language() {
    for (language, prompt, subject) in [
        ("English", "Give one example of a flarb", "flarb"),
        (
            "Russian",
            "Приведи один пример препарата, который называют фуфломицином",
            "фуфломицин",
        ),
        ("Hindi", "फ्लार्ब का एक उदाहरण दें", "फ्लार्ब"),
        ("Chinese", "举一个弗拉布的例子", "弗拉布"),
    ] {
        let call = one_call(prompt);
        assert_eq!(call.tool, "websearch", "{language}: {prompt}: {call:?}");
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("search arguments");
        let query = arguments["query"].as_str().expect("search query");
        assert!(query.contains(subject), "{language}: {prompt}: {query}");
    }
}

#[test]
fn isolated_contextual_pronoun_question_does_not_search_the_web() {
    let prompt = "Что означает слово это в предложении: так что это такое то?";
    let plan = plan_chat_step(
        &[ChatMessage::user(prompt)],
        &["bash", "websearch", "request_user_input"],
    )
    .expect("the contextual question should be handled");
    let AgenticPlan::Final(answer) = plan else {
        panic!("an unresolved contextual pronoun should ask for context: {plan:?}");
    };
    assert!(answer.to_lowercase().contains("имеете в виду"), "{answer}");
}

#[test]
fn web_synthesis_discards_short_page_furniture_without_a_phrase_blocklist() {
    let prompt = "Что такое фуфломицин?";
    let mut messages = vec![ChatMessage::user(prompt)];
    let call = one_call(prompt);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "search",
        call.tool.clone(),
        call.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "search",
        call.tool,
        "Фуфломицин — развернуть\nЧто такое рок — развернуть\n\n\
         Фуфломицин — название препаратов с недоказанной эффективностью: \
         их эффективность не подтверждена качественными исследованиями.\n\n\
         Читайте также — развернуть",
    ));

    let plan = plan_chat_step(&messages, &["websearch"]).expect("research synthesis");
    let AgenticPlan::Final(answer) = plan else {
        panic!("search evidence should complete without a fetch tool: {plan:?}");
    };
    assert!(answer.contains("недоказанной эффективностью"), "{answer}");
    for furniture in ["развернуть", "Что такое рок", "Читайте также"]
    {
        assert!(!answer.contains(furniture), "{furniture}: {answer}");
    }
}

#[test]
fn task_ladder_ratchet_preserves_real_formal_ai_authorship_evidence() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let stream_path = root.join("docs/case-studies/issue-842/self-hosting/agent-stream.jsonl");
    let stream = fs::read_to_string(&stream_path)
        .unwrap_or_else(|error| panic!("{}: {error}", stream_path.display()));
    assert!(stream.contains("ses_052c4f066ffecNc3GC3Dk13wJ9"));
    assert!(stream.contains("task-ladder-authored-invariant.lino"));

    let generated = fs::read(
        root.join("docs/case-studies/issue-842/self-hosting/task-ladder-authored-invariant.lino"),
    )
    .expect("Agent CLI generated invariant");
    let canonical = fs::read(root.join("data/meta/task-ladder-authored-invariant.lino"))
        .expect("canonical task-ladder invariant");
    assert_eq!(
        canonical, generated,
        "the promoted bytes must be Agent-authored"
    );

    let decomposition = fs::read_to_string(
        root.join("docs/case-studies/issue-842/self-hosting/decomposition.lino"),
    )
    .expect("reviewed issue #842 decomposition");
    assert_eq!(decomposition.matches("issue_842_smallest_leaf_").count(), 5);
    assert_eq!(
        decomposition
            .matches("authorship formal_ai_agent_cli")
            .count(),
        1
    );
    assert!(decomposition.contains("formal_ai_authored_percent 20"));

    let census_stream = fs::read_to_string(
        root.join("docs/case-studies/issue-842/self-hosting-census/agent-stream-self-ast.jsonl"),
    )
    .expect("real Agent CLI self-AST stream");
    assert!(census_stream.contains("ses_052baa382ffe4lQ4ABBJC6806D"));
    assert!(census_stream.contains("formal-ai"));

    let refresh_stream = fs::read_to_string(root.join(
        "docs/case-studies/issue-842/self-hosting-census-refresh/agent-stream-self-ast.jsonl",
    ))
    .expect("final-source Agent CLI self-AST stream");
    assert!(refresh_stream.contains("ses_03fe8c9d2ffeY5HOH1TaPmaML7"));
    assert!(refresh_stream.contains("formal-ai"));

    let self_heal_stream = fs::read_to_string(root.join(
        "docs/case-studies/issue-842/self-hosting-census-refresh/agent-stream-self-heal.jsonl",
    ))
    .expect("final-source Agent CLI self-healing stream");
    assert!(self_heal_stream.contains("ses_03fe3b40effenlEuRw7U21BjsA"));
    assert!(self_heal_stream.contains("formal-ai"));
    assert_eq!(
        fs::read(root.join("data/meta/self-healing-case.lino")).expect("canonical repair case"),
        fs::read(root.join(
            "docs/case-studies/issue-842/self-hosting-census-refresh/self-healing-case.lino",
        ),)
        .expect("Agent-authored repair case"),
        "the committed repair case must match the real Agent CLI artifact"
    );

    let snapshot_root =
        root.join("docs/case-studies/issue-842/self-hosting-census/workspace-census");
    let snapshot_index =
        fs::read_to_string(snapshot_root.join("index.lino")).expect("historical census index");
    assert!(snapshot_index.starts_with("self_ast_census_index\n"));
    assert!(snapshot_index.contains("  engine meta_language\n"));

    let snapshot_module =
        fs::read_to_string(snapshot_root.join("src/agentic_coding/web_research.lino"))
            .expect("historical census module");
    let target = snapshot_module
        .lines()
        .find_map(|line| line.strip_prefix("  target "))
        .expect("historical module target");
    let content_id = snapshot_module
        .lines()
        .find_map(|line| line.strip_prefix("  content_id "))
        .expect("historical module content id");
    let indexed = snapshot_index
        .lines()
        .find(|line| line.trim_start().starts_with(target))
        .expect("historical module is referenced by its index");
    assert!(indexed.contains(" full_ast "), "{indexed}");
    assert!(
        indexed.ends_with(content_id),
        "historical index and module content ids disagree: {indexed}"
    );
}
