//! Regression coverage for the grounded-action procedure in issue #840.

use std::process::Command;

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::seed::{
    self, ROLE_LOCAL_PATH_DIRECTORY_KIND, ROLE_LOCAL_PATH_SCOPE_DESKTOP,
    ROLE_LOCAL_PATH_SEARCH_ACTION,
};

fn one_call(messages: &[ChatMessage]) -> PlannedToolCall {
    let plan = plan_chat_step(
        messages,
        &["bash", "websearch", "webfetch", "request_user_input"],
    )
    .expect("the grounded task should be planned");
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("expected one tool call, got {plan:?}");
    };
    assert_eq!(calls.len(), 1, "{calls:?}");
    calls.into_iter().next().unwrap()
}

fn command(call: &PlannedToolCall) -> String {
    let arguments: serde_json::Value =
        serde_json::from_str(&call.arguments).expect("tool arguments");
    arguments["command"]
        .as_str()
        .expect("shell command")
        .to_owned()
}

fn add_result(messages: &mut Vec<ChatMessage>, call: PlannedToolCall, id: &str, output: &str) {
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.to_owned(),
        call.tool.clone(),
        call.arguments,
    )]));
    messages.push(ChatMessage::tool_result(id, call.tool, output));
}

#[test]
fn local_scope_dominates_search_verb_and_possessive_variations() {
    for prompt in [
        "Find hive-mind-control center folder on my desktop",
        "Search hive-mind-control-center on my desktop",
        "Find hive-mind-control center folder on desktop",
        "Look for hive-mind-control center directory on the desktop",
        "Найди папку hive-mind-control center на моём рабочем столе",
        "मेरे डेस्कटॉप पर hive-mind-control center फ़ोल्डर खोजें",
        "在桌面上搜索 hive-mind-control center 文件夹",
    ] {
        let call = one_call(&[ChatMessage::user(prompt)]);
        assert_eq!(call.tool, "bash", "{prompt}: {call:?}");
        let command = command(&call);
        assert!(command.starts_with("find "), "{prompt}: {command}");
        assert!(!command.contains("-print -quit"), "{prompt}: {command}");
        assert!(!command.contains(';'), "{prompt}: {command}");
        assert!(!command.contains("&&"), "{prompt}: {command}");
    }
}

#[test]
fn generated_local_routing_property_holds_for_twelve_prompts_per_language() {
    let lexicon = seed::lexicon();
    let supported = formal_ai::supported_languages();

    for language in supported {
        let language = language.as_str();
        let actions = lexicon
            .words_for_role_in_languages(ROLE_LOCAL_PATH_SEARCH_ACTION, &[language])
            .into_iter()
            .take(3)
            .collect::<Vec<_>>();
        let kinds = lexicon
            .words_for_role_in_languages(ROLE_LOCAL_PATH_DIRECTORY_KIND, &[language])
            .into_iter()
            .take(2)
            .collect::<Vec<_>>();
        let all_scopes =
            lexicon.words_for_role_in_languages(ROLE_LOCAL_PATH_SCOPE_DESKTOP, &[language]);
        let scopes = [
            all_scopes.first().expect("possessive desktop scope"),
            all_scopes.last().expect("plain desktop scope"),
        ];
        assert_eq!(actions.len(), 3, "{language}: search actions");
        assert_eq!(kinds.len(), 2, "{language}: directory kinds");

        let mut generated = 0;
        for action in &actions {
            for scope in scopes {
                for kind in &kinds {
                    let prompt = match language {
                        "hi" => format!("{scope} issue-840-target {kind} {action}"),
                        "zh" => format!("{scope}{action} issue-840-target {kind}"),
                        _ => format!("{action} issue-840-target {kind} {scope}"),
                    };
                    let call = one_call(&[ChatMessage::user(&prompt)]);
                    assert_eq!(call.tool, "bash", "{language}: {prompt}: {call:?}");
                    let command = command(&call);
                    assert!(
                        command.contains("FORMAL_AI_DESKTOP_DIR"),
                        "{language}: {prompt}: {command}"
                    );
                    assert!(
                        command.contains("-type d"),
                        "{language}: {prompt}: {command}"
                    );
                    assert!(
                        command.ends_with("-print"),
                        "{language}: {prompt}: {command}"
                    );
                    assert!(
                        !command.contains("-print -quit")
                            && !command.contains(';')
                            && !command.contains("&&"),
                        "{language}: {prompt}: {command}"
                    );
                    generated += 1;
                }
            }
        }
        assert_eq!(generated, 12, "{language}: generated property cases");
    }
}

#[test]
fn explicit_location_question_answers_the_route_without_searching() {
    let plan = plan_chat_step(
        &[ChatMessage::user(
            "Is the request 'Find a folder on my desktop' a local filesystem search or a web search?",
        )],
        &["bash", "websearch"],
    )
    .expect("the explicit local scope should determine the route");
    let AgenticPlan::Final(answer) = plan else {
        panic!("a route question should not execute either search: {plan:?}");
    };
    assert!(answer.to_lowercase().contains("local"), "{answer}");
    assert!(!answer.to_lowercase().contains("web search"), "{answer}");
}

#[test]
fn empty_exact_local_lookup_widens_instead_of_claiming_absence() {
    let mut messages = vec![ChatMessage::user(
        "Find hive-mind-control center folder on my desktop",
    )];
    let exact = one_call(&messages);
    let exact_command = command(&exact);
    assert!(
        exact_command.contains("hive-mind-control"),
        "{exact_command}"
    );
    add_result(&mut messages, exact, "exact", "(no output)");

    let widened = one_call(&messages);
    assert_eq!(widened.tool, "bash");
    let widened_command = command(&widened);
    assert!(widened_command.contains("*hive*"), "{widened_command}");
    assert!(
        !widened_command.contains("-print -quit"),
        "{widened_command}"
    );
}

#[test]
fn quoted_local_name_excludes_trailing_answer_instructions() {
    for (prompt, expected) in [
        (
            "Is there a folder named exactly 'hive-mind-control-center' on my desktop? \
             Answer yes or no and say what the closest match is.",
            "hive-mind-control-center",
        ),
        (
            "What's inside the 'Archive' folder on my desktop?",
            "archive",
        ),
    ] {
        let call = one_call(&[ChatMessage::user(prompt)]);
        let command = command(&call);
        assert!(
            command.contains(&format!("-iname '{expected}'")),
            "{command}"
        );
        assert!(!command.contains("answer"), "{command}");
        assert!(!command.contains("closest"), "{command}");
    }
}

#[test]
fn transport_wrapped_local_request_does_not_treat_the_whole_prompt_as_a_name() {
    let call = one_call(&[ChatMessage::user(
        "\"Find hive-mind-control center folder on my desktop\"",
    )]);
    let command = command(&call);
    assert!(
        command.contains("-iname 'hive-mind-control-center'"),
        "{command}"
    );
}

#[test]
fn unnamed_openai_tool_result_widens_by_matching_its_call_id() {
    let mut messages = vec![ChatMessage::user(
        "Find hive-mind-control center folder on my desktop",
    )];
    let exact = one_call(&messages);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "exact".to_owned(),
        exact.tool.clone(),
        exact.arguments,
    )]));
    let mut result = ChatMessage::tool_result("exact", exact.tool, "(no output)");
    result.name = None;
    messages.push(result);

    let widened = one_call(&messages);
    assert_eq!(widened.tool, "bash");
    assert!(command(&widened).contains("*hive*"), "{widened:?}");
}

#[test]
fn absence_requires_exact_substring_and_bounded_inventory_observations() {
    let mut messages = vec![ChatMessage::user(
        "Find zzz-nonexistent folder on my desktop",
    )];

    let exact = one_call(&messages);
    assert!(command(&exact).contains("-iname 'zzz-nonexistent'"));
    add_result(&mut messages, exact, "exact", "(no output)");

    let substring = one_call(&messages);
    assert!(command(&substring).contains("-iname '*nonexistent*'"));
    add_result(&mut messages, substring, "substring", "(no output)");

    let inventory = one_call(&messages);
    let inventory_command = command(&inventory);
    assert!(
        inventory_command.contains("-mindepth 1 -maxdepth 3"),
        "{inventory_command}"
    );
    add_result(&mut messages, inventory, "inventory", "(no output)");

    let plan = plan_chat_step(&messages, &["bash", "websearch"]).expect("scoped absence");
    let AgenticPlan::Final(answer) = plan else {
        panic!("the exhausted bounded ladder should finish: {plan:?}");
    };
    assert!(
        answer.contains("exact, substring, and nearby-name"),
        "{answer}"
    );
    assert!(answer.contains("FORMAL_AI_DESKTOP_DIR"), "{answer}");
    assert!(
        answer.contains("No wider location was searched"),
        "{answer}"
    );
}

#[test]
fn differently_typed_near_match_is_named_instead_of_reported_as_absent() {
    let mut messages = vec![ChatMessage::user(
        "Find the folder named 'only-a-file' on my desktop",
    )];

    for id in ["exact", "substring"] {
        let call = one_call(&messages);
        assert!(command(&call).contains("-type d"), "{call:?}");
        add_result(&mut messages, call, id, "(no output)");
    }

    let inventory = one_call(&messages);
    let inventory_command = command(&inventory);
    assert!(
        inventory_command.contains("-mindepth 1 -maxdepth 3"),
        "{inventory_command}"
    );
    assert!(
        !inventory_command.contains("-type d"),
        "the bounded candidate inventory must expose type mismatches: {inventory_command}"
    );
    add_result(
        &mut messages,
        inventory,
        "inventory",
        "/tmp/Desktop/only-a-file",
    );

    let metadata = one_call(&messages);
    assert!(command(&metadata).starts_with("ls -ld -- "), "{metadata:?}");
    add_result(
        &mut messages,
        metadata,
        "metadata",
        "-rw------- 1 user user 0 Jul 24 00:00 /tmp/Desktop/only-a-file",
    );

    let plan = plan_chat_step(&messages, &["bash", "websearch"]).expect("type mismatch answer");
    let AgenticPlan::Final(answer) = plan else {
        panic!("a verified type mismatch should finish: {plan:?}");
    };
    assert!(answer.contains("No folder matched only-a-file"), "{answer}");
    assert!(answer.contains("non-directory"), "{answer}");
    assert!(answer.contains("/tmp/Desktop/only-a-file"), "{answer}");
}

#[test]
fn failed_local_lookup_is_reported_instead_of_treated_as_empty() {
    let mut messages = vec![ChatMessage::user(
        "Find hive-mind-control center folder on my desktop",
    )];
    let exact = one_call(&messages);
    add_result(
        &mut messages,
        exact,
        "failed",
        r#"{"exit_code":1,"stderr":"desktop is unavailable"}"#,
    );
    let plan = plan_chat_step(&messages, &["bash", "websearch"]).expect("failure explanation");
    let AgenticPlan::Final(answer) = plan else {
        panic!("a failed observation must not trigger widening: {plan:?}");
    };
    assert!(answer.contains("desktop is unavailable"), "{answer}");
}

#[test]
fn report_intent_is_meanings_driven_for_the_reported_russian_phrase() {
    for prompt in ["Зарепорти баг", "Сообщи об ошибке", "Report a bug"] {
        let call = one_call(&[ChatMessage::user(prompt)]);
        assert_eq!(call.tool, "request_user_input", "{prompt}: {call:?}");
    }
}

#[test]
fn bare_definition_followup_asks_for_its_antecedent() {
    let plan = plan_chat_step(
        &[ChatMessage::user("Так что это такое то?")],
        &["websearch", "webfetch"],
    )
    .expect("follow-up should have a deterministic clarification");
    let AgenticPlan::Final(answer) = plan else {
        panic!("bare follow-up must not search or show a capability menu: {plan:?}");
    };
    assert!(answer.to_lowercase().contains("имеете в виду"), "{answer}");
    assert!(!answer.contains("Hello World"), "{answer}");
    assert!(!answer.contains("Приветствия"), "{answer}");
}

#[test]
fn same_turn_definition_followup_reuses_only_the_antecedent_topic() {
    let call = one_call(&[ChatMessage::user(
        "Что такое фуфломицин? Затем: так что это такое то?",
    )]);
    assert_eq!(call.tool, "websearch", "{call:?}");
    let arguments: serde_json::Value =
        serde_json::from_str(&call.arguments).expect("search arguments");
    let query = arguments["query"].as_str().expect("search query");
    assert!(query.contains("фуфломицин"), "{query}");
    assert!(!query.contains("затем"), "{query}");
    assert!(!query.contains("такое"), "{query}");
}

#[test]
fn definition_imperatives_route_to_research_in_every_supported_language() {
    for (prompt, subject) in [
        ("Define flarb in one sentence", "flarb"),
        (
            "Дай определение слова фуфломицин одним предложением",
            "фуфломицин",
        ),
        ("परिभाषित करें फ्लार्ब", "फ्लार्ब"),
        ("定义弗拉布", "弗拉布"),
    ] {
        let call = one_call(&[ChatMessage::user(prompt)]);
        assert_eq!(call.tool, "websearch", "{prompt}: {call:?}");
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("search arguments");
        let query = arguments["query"].as_str().expect("search query");
        assert!(query.contains(subject), "{prompt}: {query}");
    }
}

#[test]
fn unresolved_definition_question_with_output_instruction_routes_to_research() {
    let prompt = "What is a fufloмицин (фуфломицин)? Answer in English.";
    let call = one_call(&[ChatMessage::user(prompt)]);
    assert_eq!(call.tool, "websearch", "{call:?}");
    let arguments: serde_json::Value =
        serde_json::from_str(&call.arguments).expect("search arguments");
    let query = arguments["query"].as_str().expect("search query");
    assert!(query.contains("фуфломицин"), "{query}");
}

#[test]
fn later_definition_followup_reuses_the_prior_user_topic() {
    let call = one_call(&[
        ChatMessage::user("Что такое фуфломицин?"),
        ChatMessage::assistant("Я проверю определение."),
        ChatMessage::user("Так что это такое то?"),
    ]);
    assert_eq!(call.tool, "websearch", "{call:?}");
    let arguments: serde_json::Value =
        serde_json::from_str(&call.arguments).expect("search arguments");
    let query = arguments["query"].as_str().expect("search query");
    assert!(query.contains("фуфломицин"), "{query}");
    assert!(!query.contains("такое"), "{query}");
}

#[test]
fn comparison_is_decomposed_before_open_web_research() {
    let mut messages = vec![ChatMessage::user("ФБС vs ФБО")];
    let first = one_call(&messages);
    assert_eq!(first.tool, "websearch", "{first:?}");
    let arguments: serde_json::Value =
        serde_json::from_str(&first.arguments).expect("search arguments");
    let query = arguments["query"].as_str().expect("search query");
    let query = query.to_lowercase();
    assert!(
        query.contains("фбс") && !query.contains("фбо"),
        "the first lookup should cover only one side: {query}"
    );
    add_result(
        &mut messages,
        first,
        "left",
        "ФБС evidence: seller warehouse",
    );

    let second = one_call(&messages);
    assert_eq!(second.tool, "websearch", "{second:?}");
    let arguments: serde_json::Value =
        serde_json::from_str(&second.arguments).expect("search arguments");
    let query = arguments["query"].as_str().expect("search query");
    let query = query.to_lowercase();
    assert!(
        query.contains("фбо") && !query.contains("фбс"),
        "the second lookup should cover only the other side: {query}"
    );
    add_result(
        &mut messages,
        second,
        "right",
        "ФБО evidence: marketplace warehouse",
    );

    let plan = plan_chat_step(&messages, &["websearch", "webfetch"]).expect("comparison synthesis");
    let AgenticPlan::Final(answer) = plan else {
        panic!("two observed sides should complete the comparison: {plan:?}");
    };
    assert!(answer.contains("ФБС evidence"), "{answer}");
    assert!(answer.contains("ФБО evidence"), "{answer}");
}

#[test]
fn comparison_retries_the_missing_side_after_a_failed_search_result() {
    let mut messages = vec![ChatMessage::user("ФБС vs ФБО")];
    let first = one_call(&messages);
    add_result(
        &mut messages,
        first,
        "left",
        "ФБС evidence: seller warehouse",
    );

    let second = one_call(&messages);
    add_result(
        &mut messages,
        second,
        "right-timeout",
        "Error: MCP tool timed out before returning evidence",
    );

    let retry = one_call(&messages);
    assert_eq!(retry.tool, "websearch", "{retry:?}");
    let arguments: serde_json::Value =
        serde_json::from_str(&retry.arguments).expect("retry search arguments");
    assert_eq!(arguments["query"], "фбо", "{retry:?}");

    add_result(
        &mut messages,
        retry,
        "right-retry",
        "ФБО evidence: marketplace warehouse",
    );
    let plan = plan_chat_step(&messages, &["websearch", "webfetch"])
        .expect("comparison synthesis after retry");
    let AgenticPlan::Final(answer) = plan else {
        panic!("successful retry should complete the comparison: {plan:?}");
    };
    assert!(answer.contains("ФБС evidence"), "{answer}");
    assert!(answer.contains("ФБО evidence"), "{answer}");
    assert!(!answer.contains("timed out"), "{answer}");
}

#[test]
fn inflected_comparison_excludes_output_format_from_the_right_operand() {
    let mut messages = vec![ChatMessage::user(
        "Чем отличается ФБС от ФБО одним предложением?",
    )];
    let first = one_call(&messages);
    add_result(&mut messages, first, "left", "ФБС evidence");

    let second = one_call(&messages);
    let arguments: serde_json::Value =
        serde_json::from_str(&second.arguments).expect("search arguments");
    assert_eq!(arguments["query"], "фбо", "{second:?}");
}

#[test]
fn reference_differential_is_machine_checked_and_wired_into_release_ci() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let reference_dir = root.join("experiments/issue_840_reference_agents");
    let baseline_path = reference_dir.join("baseline.json");
    let checker_path = reference_dir.join("check_differential.py");
    let results_path = root.join("experiments/issue_840_task_ladder/results.json");

    let baseline: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&baseline_path).expect("machine-readable reference baseline"),
    )
    .expect("valid reference baseline JSON");
    assert_eq!(
        baseline["policy"]["quota_exhaustion"], "inconclusive",
        "provider quota exhaustion must not be scored as a model failure"
    );
    assert_eq!(baseline["best_reference"]["agent"], "laguna-s-2.1-free");
    assert_eq!(baseline["best_reference"]["command_count"], 5);

    let checked = Command::new("python3")
        .args([&checker_path, &baseline_path, &results_path])
        .output()
        .expect("run the issue #840 differential checker");
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let mut regressed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&results_path).unwrap()).unwrap();
    let local = regressed["results"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == "838.L1")
        .unwrap();
    local["assistant_output"] = serde_json::Value::String(
        "The answer is hive-mind-bot.2025-12-26.private-key.pem".to_owned(),
    );
    local["pass"] = serde_json::Value::Bool(true);
    let regression_path = std::env::temp_dir().join(format!(
        "formal-ai-issue-840-regression-{}.json",
        std::process::id()
    ));
    std::fs::write(
        &regression_path,
        serde_json::to_vec_pretty(&regressed).unwrap(),
    )
    .unwrap();
    let rejected = Command::new("python3")
        .args([&checker_path, &baseline_path, &regression_path])
        .output()
        .expect("run the issue #840 differential checker on a regression");
    let _ = std::fs::remove_file(regression_path);
    assert!(
        !rejected.status.success(),
        "a PEM-decoy regression must fail the differential gate"
    );

    let workflow = std::fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();
    assert!(
        workflow.contains("issue_840_reference_agents/run_differential_gate.sh"),
        "the differential comparison must execute in release CI"
    );
}

#[test]
fn issue_840_agent_cli_e2e_covers_every_seed_journey_including_russian_reporting() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let issue_harness =
        std::fs::read_to_string(root.join("experiments/agent_cli_e2e/run_issue_840.sh")).unwrap();
    let local_harness =
        std::fs::read_to_string(root.join("experiments/agent_cli_e2e/run_issue_819.sh")).unwrap();
    let all_issue_harnesses = format!("{issue_harness}\n{local_harness}");
    for prompt in [
        "Find hive-mind-control center folder on my desktop",
        "Что такое фуфломицин? Затем: так что это такое то?",
        "ФБС vs ФБО",
        "Зарепорти баг",
    ] {
        assert!(
            all_issue_harnesses.contains(prompt),
            "the real Agent CLI harness omits {prompt}"
        );
    }
    assert!(
        issue_harness.contains("issue_714_agentic_mode/run_report_e2e.sh"),
        "the Russian report request must reach the real report executor"
    );
    for timeout_setting in [
        r#""tool_call_timeout": 120000"#,
        r#""max_tool_call_timeout": 600000"#,
        "--mcp-default-tool-call-timeout 120000",
        "--mcp-max-tool-call-timeout 600000",
    ] {
        assert!(
            issue_harness.contains(timeout_setting),
            "the Agent CLI harness must set {timeout_setting} explicitly"
        );
    }

    let report_harness =
        std::fs::read_to_string(root.join("experiments/issue_714_agentic_mode/run_report_e2e.sh"))
            .unwrap();
    assert!(
        report_harness.contains(r#"REPORT_PROMPT="${REPORT_PROMPT:-Report issue}""#),
        "the reusable report harness must accept the issue #840 Russian prompt"
    );
}
