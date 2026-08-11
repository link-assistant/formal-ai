//! Regression coverage for the failures reported in issue #989.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::issue_report::{ReportAttachment, ReportBody};
use formal_ai::memory::MemoryEvent;
use formal_ai::protocol::ToolCall;
use formal_ai::seed;
use formal_ai::skill_compiler::compile_natural_language_skill;
use formal_ai::{
    create_chat_completion_with_solver_and_memory, ChatCompletionRequest, ChatMessage,
    ConversationTurn, SolverConfig, UniversalSolver,
};

fn shell_command(prompt: &str) -> Option<String> {
    let plan = plan_chat_step(
        &[ChatMessage::user(prompt)],
        &["exec_command", "websearch", "write"],
    )?;
    let AgenticPlan::ToolCalls(calls) = plan else {
        return None;
    };
    let call = calls.first()?;
    if call.tool != "exec_command" {
        return None;
    }
    let arguments: serde_json::Value = serde_json::from_str(&call.arguments).ok()?;
    arguments["command"].as_str().map(str::to_owned)
}

#[test]
fn natural_location_questions_use_pwd_instead_of_web_search() {
    for prompt in [
        "Where we are?",
        "What is current location?",
        "What is current directory?",
    ] {
        assert_eq!(shell_command(prompt).as_deref(), Some("pwd"), "{prompt}");
    }
}

#[test]
fn ordinary_file_edits_are_not_mistaken_for_dialog_control() {
    let prompt = "update main.rs and change foo to bar";
    let plan = plan_chat_step(&[ChatMessage::user(prompt)], &["edit"]);
    let Some(AgenticPlan::ToolCalls(calls)) = plan else {
        panic!("ordinary file edit should remain an agentic tool request: {plan:?}");
    };
    assert_eq!(calls[0].tool, "edit");

    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "messages": [{"role": "user", "content": prompt}],
        "tools": [{
            "type": "function",
            "function": {"name": "edit", "parameters": {"type": "object"}}
        }]
    }))
    .expect("valid agent request");
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let completion = create_chat_completion_with_solver_and_memory(&request, &solver, &[]);
    assert_eq!(completion.choices[0].finish_reason, "tool_calls");
    assert_eq!(
        completion.choices[0].message.tool_calls[0].function.name,
        "edit"
    );
}

#[test]
fn links_notation_question_falls_through_to_the_symbolic_concept_answer() {
    let messages = [ChatMessage::user("What is Links Notation?")];
    assert!(
        plan_chat_step(&messages, &["websearch", "write", "exec_command"]).is_none(),
        "a definition must not start the document-formalization recipe"
    );
    let answer = UniversalSolver::default().solve("What is Links Notation?");
    assert_eq!(answer.intent, "concept_lookup");
    assert_eq!(
        answer.answer,
        "Links Notation (data-format): Links Notation is an indentation-based, untyped serialization format used by the Deep Theory project to represent links and link networks as portable text.\n\nSource: https://github.com/linksplatform/Documentation (project-docs)."
    );
    assert!(answer.answer.contains("Links Notation"));
}

#[test]
fn british_behaviour_rule_queries_match_the_rule_catalog() {
    let solver = UniversalSolver::default();
    let list = solver.solve("List behaviour rules");
    assert_eq!(list.intent, "behavior_rules_list", "{}", list.answer);

    let detail = solver.solve("Show behaviour rule unknown");
    assert_eq!(detail.intent, "behavior_rule_detail", "{}", detail.answer);
    assert!(detail.answer.contains("rule_unknown"));
}

#[test]
fn agentic_narration_does_not_reintroduce_the_rejected_subjective_wording() {
    assert_eq!(
        seed::localized_response("agentic_action_run", "en").as_deref(),
        Some("Let me run the requested command to get that for you.")
    );
    assert_eq!(
        seed::localized_response("agentic_action_ask_user", "en").as_deref(),
        Some("Let me ask you a few questions so I get this right.")
    );
}

#[test]
fn plain_text_trigger_and_answer_compile_and_replay() {
    for teaching in ["When I say # answer with 42.", "When I say # answer 42."] {
        let package = compile_natural_language_skill(teaching)
            .unwrap_or_else(|error| panic!("{teaching}: {error}"));
        assert_eq!(package.trigger, "#");
        assert_eq!(package.response, "42");

        let history = [
            ConversationTurn::user(teaching),
            ConversationTurn::assistant("Behavior rule recorded for this dialog."),
        ];
        let replay = UniversalSolver::default().solve_with_history("#", &history);
        assert_eq!(replay.answer, "42", "{teaching}");
    }
}

#[test]
fn preference_and_mutation_correction_receive_explicit_acknowledgements() {
    assert!(seed::lexicon().mentions_role(
        seed::ROLE_CONVERSATION_PREFERENCE_AVOID,
        "quick is subjective opinion please don t use these anymore"
    ));
    let solver = UniversalSolver::default();
    let prompt = "`quick` is subjective opinion, please don't use these anymore.";
    let formalization = formal_ai::intent_formalization::formalize_intent(prompt, "en", None);
    assert!(
        formalization.has_relevant_handler("conversation_control"),
        "{:?}",
        formalization.relevants
    );
    let preference = solver.solve(prompt);
    assert_eq!(preference.intent, "conversation_preference");
    assert_eq!(
        preference.answer,
        "Understood. I'll avoid `quick` in this dialog."
    );
    assert!(preference.answer.contains("`quick`"));

    let correction = solver.solve("I didn't ask to update anything.");
    assert_eq!(correction.intent, "action_correction");
    assert_eq!(
        correction.answer,
        "I'm sorry. You asked for information, not an update. I won't make further changes unless you explicitly request them."
    );
    assert!(correction.answer.to_lowercase().contains("sorry"));
}

fn memory_answer(prompt: &str, prior_user: Option<&str>) -> String {
    let mut messages = Vec::new();
    if let Some(previous) = prior_user {
        messages.push(ChatMessage::user(previous));
        messages.push(ChatMessage::assistant(
            "That looked like document generation.",
        ));
    }
    messages.push(ChatMessage::user(prompt));
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "messages": messages,
        "tools": [{
            "type": "function",
            "function": {"name": "websearch", "parameters": {"type": "object"}}
        }]
    }))
    .expect("valid request");
    let events = [MemoryEvent {
        id: String::from("memory-1"),
        kind: Some(String::from("message")),
        role: Some(String::from("user")),
        content: Some(String::from("remembered fact")),
        conversation_id: Some(String::from("conversation-1")),
        ..MemoryEvent::default()
    }];
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let completion = create_chat_completion_with_solver_and_memory(&request, &solver, &events);
    assert_eq!(completion.choices[0].finish_reason, "stop");
    assert!(completion.choices[0].message.tool_calls.is_empty());
    completion.choices[0].message.content.plain_text()
}

#[test]
fn associative_memory_introspection_preempts_web_and_document_planning() {
    let count = memory_answer("How many links are in your memory?", None);
    assert!(count.contains("link"), "{count}");
    assert!(count.contains("records: 1"), "{count}");

    let inventory = memory_answer("What is available in your local memory?", None);
    assert!(inventory.contains("message"), "{inventory}");
    assert!(inventory.contains("conversation-1"), "{inventory}");

    let roots = memory_answer("Give me root links you have in your memory", None);
    assert!(roots.contains("memory-1"), "{roots}");

    let corrected = memory_answer(
        "No that is not about document generation, it is about associative memory data retrieval.",
        Some("Give me root links you have in your memory"),
    );
    assert!(corrected.contains("memory-1"), "{corrected}");
}

fn github_report_command() -> String {
    let messages = vec![
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "target",
            "request_user_input",
            r#"{"report_target":["harness_log","server_log","github_issue","formal_ai"]}"#,
        ),
    ];
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&messages, &["request_user_input", "exec_command"])
            .expect("report export plan")
    else {
        panic!("expected a report command");
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    let command = arguments["command"].as_str().unwrap();
    assert!(command.contains("--source harness"), "{command}");

    let mut progressed = messages;
    progressed.push(ChatMessage::tool_result(
        "run-1",
        "exec_command",
        "/tmp/harness.lino",
    ));
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&progressed, &["request_user_input", "exec_command"])
            .expect("server export plan")
    else {
        panic!("expected a server export command");
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    assert!(arguments["command"]
        .as_str()
        .unwrap()
        .contains("--source server"));

    progressed.push(ChatMessage::tool_result(
        "run-2",
        "exec_command",
        "/tmp/server.lino",
    ));
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&progressed, &["request_user_input", "exec_command"])
            .expect("learning plan")
    else {
        panic!("expected a learning command");
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    assert!(arguments["command"]
        .as_str()
        .unwrap()
        .contains("context learn"));

    progressed.push(ChatMessage::tool_result("run-3", "exec_command", "learned"));
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&progressed, &["request_user_input", "exec_command"])
            .expect("github filing plan")
    else {
        panic!("expected a github filing command");
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    arguments["command"].as_str().unwrap().to_owned()
}

#[test]
fn github_report_requests_three_separate_context_links() {
    let command = github_report_command();
    assert!(command.contains("--source both"), "{command}");
    assert!(command.contains("--separate-context-links"), "{command}");
}

#[test]
fn link_only_report_attachments_do_not_emit_empty_code_fences() {
    let report = ReportBody {
        attachments: vec![ReportAttachment {
            heading: String::from("### Harness context"),
            note: String::from("Full context: https://example.test/harness"),
            ..ReportAttachment::default()
        }],
        ..ReportBody::default()
    }
    .render();
    assert!(report.contains("### Harness context"));
    assert!(report.contains("https://example.test/harness"));
    assert!(!report.contains("```"), "{report}");
}

fn failed_web_transport_answer() -> String {
    let mut messages = vec![ChatMessage::user("Research the ZXQ protocol")];
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&messages, &["websearch", "webfetch"]).expect("research plan")
    else {
        panic!("expected web search");
    };
    let call = &calls[0];
    assert_eq!(call.tool, "websearch");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "search-1",
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    let mut failure = ChatMessage::tool_result(
        "search-1",
        "websearch",
        "Transport error (POST https://mcp.exa.ai/mcp)",
    );
    failure.is_error = true;
    messages.push(failure);

    let AgenticPlan::Final(answer) =
        plan_chat_step(&messages, &["websearch", "webfetch"]).expect("failure answer")
    else {
        panic!("a failed search must stop instead of claiming completion");
    };
    answer
}

#[test]
fn failed_web_transport_is_reported_as_failure_not_empty_success() {
    let answer = failed_web_transport_answer();
    assert!(answer.contains("Transport error"), "{answer}");
    assert!(!answer.contains("Research completed"), "{answer}");
    assert!(!answer.contains("no content"), "{answer}");
}

#[test]
fn whole_reported_dialog_stays_local_and_preserves_diagnostics() {
    let solver = UniversalSolver::default();
    assert_eq!(shell_command("Where we are?").as_deref(), Some("pwd"));
    assert_eq!(
        solver.solve("What is links notation?").intent,
        "concept_lookup"
    );
    assert_eq!(
        solver.solve("List behaviour rules").intent,
        "behavior_rules_list"
    );
    assert_eq!(
        solver
            .solve("`quick` is subjective opinion, please don't use these anymore.")
            .answer,
        "Understood. I'll avoid `quick` in this dialog."
    );
    assert_eq!(
        compile_natural_language_skill("When I say # answer with 42.")
            .expect("unquoted teaching rule")
            .response,
        "42"
    );
    assert!(memory_answer("How many links are in your memory?", None).contains("records: 1"));
    assert!(memory_answer(
        "No that is not about document generation, it is about associative memory data retrieval.",
        Some("Give me root links you have in your memory"),
    )
    .contains("memory-1"));
    assert_eq!(
        seed::localized_response("agentic_action_ask_user", "en").as_deref(),
        Some("Let me ask you a few questions so I get this right.")
    );
    assert!(github_report_command().contains("--separate-context-links"));
    assert!(failed_web_transport_answer().contains("Transport error"));
}

#[test]
fn same_task_agent_cli_authorship_is_preserved() {
    const SESSION: &str = "ses_00fc002c9ffetId67J25NEeudn";
    const COMMITTED: &str =
        include_str!("../../docs/case-studies/issue-989/issue-989-task-decomposition.lino");
    const GENERATED: &str = include_str!(
        "../../docs/case-studies/issue-989/self-hosting-authorship/decomposition-session/issue-989-task-decomposition.lino"
    );
    const AGENT_LOG: &str = include_str!(
        "../../docs/case-studies/issue-989/self-hosting-authorship/decomposition-session/agent-cli.log"
    );
    const FORMAL_AI_LOG: &str = include_str!(
        "../../docs/case-studies/issue-989/self-hosting-authorship/decomposition-session/formal-ai.log"
    );

    assert_eq!(COMMITTED, GENERATED);
    assert_eq!(COMMITTED.matches("  leaf ").count(), 5, "{COMMITTED}");
    assert_eq!(
        COMMITTED.matches("author formal_ai").count(),
        1,
        "{COMMITTED}"
    );
    assert!(COMMITTED.contains("required_self_authored_leaves 1"));
    assert!(AGENT_LOG.contains(SESSION));
    for transition in [
        "planned ToolCalls",
        "tool=write",
        "tool: \"bash\"",
        "planned Final",
        "issue-989-task-decomposition.lino",
    ] {
        assert!(
            FORMAL_AI_LOG.contains(transition),
            "server trace is missing {transition:?}"
        );
    }
}
