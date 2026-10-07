//! Issue #918 (E71, R914-6): the minimal-core batch that moved three
//! specialized handlers out of Rust and into `data/seed/handler-rules.lino`.
//!
//! `conversation_topic`, `source_refresh` and `source_conflict` were Rust
//! functions whose cue words and four-language prose were literals. They are
//! now rule sets walked by the generic interpreter (`src/rule_interpreter.rs`),
//! with their wording in the seed's multilingual responses. The interpreter
//! gained three domain-free primitives for them: a `role_slot` capture (the
//! text filling the open slot of a role's prefix surface), a `stable_id`
//! value, and an `evidence` condition that asks the capability table's claim
//! reader, so a rule's refusal lane and its admission read one predicate.
//!
//! The second batch moved `execution_failure` into the same rule document and
//! the `incompatible_units` wording into the seeded `unit_incompatibility`
//! response (its unit-dimension walk over the meaning lexicon stays the
//! native primitive). Both had English-only wording, so their responses are
//! seeded in English only and every language renders it exactly as before.
//!
//! The third batch moved the research follow-ups onto their procedure seed:
//! `research_comparison_table` and `research_result_followup` read their
//! columns, statuses, whole-prompt follow-ups and wording from
//! `data/seed/research-table-procedure.lino` and the seeded
//! `research_result_followup_<status>` responses in both runtimes; the
//! browser twin stopped writing memorized topic facts into the table cells.
//!
//! The fourth batch moved `network_query` into the rule document too: the
//! snapshot, concept-introspection and user-filter branches are rules whose
//! values come from two more domain-free primitives (`quoted`, the first
//! quoted phrase, and `network_snapshot`, the runtime's own link network), and
//! a rule's intent may now name a captured value.
//!
//! The English and Russian answers below are byte-identical to the ones the
//! deleted Rust produced (`tests/unit/specification/issue_146.rs` pins the
//! same two conversation-topic answers). The browser twin is
//! `rust/tests/web/issue-0918-handler-rules-batch.test.mjs`.

use formal_ai::event_log::EventLog;
use formal_ai::rule_interpreter::{handler_claims, handler_policy, rules, run_handler};
use formal_ai::{ConversationTurn, FormalAiEngine, UniversalSolver};

const TOPIC_EN: &str = "We can talk about existence. I can start with a short definition, context, or a specific question; when web search is available, public facts can be checked against an external source.";
const TOPIC_RU: &str = "Можем. Тема: бытие. Я могу начать с краткого определения, контекста или конкретного вопроса; если веб-поиск доступен, публичные факты можно уточнить через внешний источник.";
const TOPIC_HI: &str = "हम बात कर सकते हैं. विषय: गणित. मैं छोटी परिभाषा, संदर्भ, या किसी ठोस प्रश्न से शुरू कर सकता हूँ; web search उपलब्ध हो तो public facts बाहरी स्रोत से जाँचे जा सकते हैं.";
const TOPIC_ZH: &str = "可以聊。主题: 音乐。我可以从简短定义、上下文或具体问题开始; 如果 web search 可用, 公开事实可以通过外部来源核对。";
const CONFLICT_EN: &str = "Sources disagree on this question. The disagreement is recorded as a conflict:source_disagreement link in the network rather than silently resolved.";
const REFRESH_EN: &str = "Cached source source_e2db54b48c90e140 has been queued for refresh against its origin URL. The refresh event is appended to the audit log and a fresh fetched_at timestamp will be recorded once the new copy is verified.";
const REFRESH_UNNAMED_EN: &str = "Cached source source_2a324f9681a3e3bd has been queued for refresh against its origin URL. The refresh event is appended to the audit log and a fresh fetched_at timestamp will be recorded once the new copy is verified.";

const EXECUTION_FAILURE: &str = "Execution status: failed in isolated sandbox.\n```python\nundefined_function()\n```\nTraceback (most recent call last):\n  File 'main.py', line 1, in <module>\nNameError: name 'undefined_function' is not defined.\nThe failure trace is appended to the action log; see the trace link.";
const UNITS_EN: &str = "meters measures length; kilogram measures mass. These are different physical dimensions and cannot be converted into each other. The incompatibility is recorded as a `unit_incompatibility` link in the network.";
const UNITS_RU: &str = "метр measures length; килограмм measures mass. These are different physical dimensions and cannot be converted into each other. The incompatibility is recorded as a `unit_incompatibility` link in the network.";

#[test]
fn the_migrated_handlers_are_seed_rule_sets() {
    for name in [
        "conversation_topic",
        "source_refresh",
        "source_conflict",
        "execution_failure",
        "network_query",
    ] {
        assert!(
            rules().handler(name).is_some(),
            "{name} must be a rule set of data/seed/handler-rules.lino"
        );
    }
    let refresh: Vec<&str> = rules()
        .handler("source_refresh")
        .expect("source_refresh rule set")
        .rule_names()
        .collect();
    assert_eq!(refresh, ["source_refresh", "source_refresh_unnamed"]);
}

#[test]
fn a_conversation_topic_answers_unchanged_through_the_engine() {
    for (prompt, expected) in [
        ("Let's talk about existence", TOPIC_EN),
        ("Поговорим о бытие", TOPIC_RU),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "conversation_topic", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
}

#[test]
fn the_topic_rule_captures_the_opener_slot_in_hindi_and_chinese() {
    for (prompt, expected, topic) in [
        ("चलो बात करें गणित", TOPIC_HI, "गणित"),
        ("聊聊音乐", TOPIC_ZH, "音乐"),
    ] {
        let mut log = EventLog::default();
        let response = run_handler(
            "conversation_topic",
            prompt,
            &prompt.to_lowercase(),
            &mut log,
        )
        .unwrap_or_else(|| panic!("{prompt}: the opener slot must be captured"));
        assert_eq!(response.answer, expected, "{prompt}");
        assert!(
            log.events()
                .iter()
                .any(|event| event.kind == "conversation_topic" && event.payload == topic),
            "{prompt}: the captured topic is logged"
        );
    }
}

#[test]
fn the_topic_claim_is_the_rule_and_an_empty_slot_is_no_topic() {
    assert!(handler_claims(
        "conversation_topic",
        "Let's talk about existence",
        "let's talk about existence"
    ));
    assert!(!handler_claims(
        "conversation_topic",
        "Let's talk about ?!",
        "let's talk about ?!"
    ));
    assert!(!handler_claims(
        "conversation_topic",
        "What is the capital of France?",
        "what is the capital of france?"
    ));
}

#[test]
fn a_source_conflict_answers_unchanged_through_the_engine() {
    let response = FormalAiEngine.answer("Was X born in 1880 or 1881?");
    assert_eq!(response.intent, "source_conflict");
    assert_eq!(response.answer, CONFLICT_EN);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("conflict:source_disagreement")),
        "the disagreement is recorded, not resolved: {:?}",
        response.evidence_links
    );
}

#[test]
fn a_source_refresh_answers_unchanged_through_the_engine() {
    let response = FormalAiEngine.answer("Refresh the cached page for example.com");
    assert_eq!(response.intent, "source_refresh");
    assert_eq!(response.answer, REFRESH_EN);
}

#[test]
fn an_unnamed_source_refresh_takes_the_refusal_lane() {
    let mut log = EventLog::default();
    let response = run_handler(
        "source_refresh",
        "Refresh the cache",
        "refresh the cache",
        &mut log,
    )
    .expect("a refresh cue is answered");
    assert_eq!(response.intent, "source_refresh");
    assert_eq!(response.answer, REFRESH_UNNAMED_EN);
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "source_refresh:refusal"),
        "no source named is the refusal lane"
    );

    let mut named = EventLog::default();
    run_handler(
        "source_refresh",
        "Refresh the cached page https://example.com/docs",
        "refresh the cached page https://example.com/docs",
        &mut named,
    )
    .expect("a named source is refreshed");
    assert!(
        !named
            .events()
            .iter()
            .any(|event| event.kind == "source_refresh:refusal"),
        "a URL is a named source"
    );
}

#[test]
fn an_execution_failure_answers_unchanged_in_chat_and_agent_mode() {
    let chat = FormalAiEngine.answer("Write a Python script that calls undefined_function()");
    assert_eq!(chat.intent, "execution_failure");
    assert_eq!(chat.answer, EXECUTION_FAILURE);
    assert!(
        chat.evidence_links
            .iter()
            .any(|link| link.starts_with("trace:execution_failure")),
        "the failure exposes its trace link"
    );

    let mut log = EventLog::default();
    let agent = run_handler(
        "execution_failure",
        "[agent] Run a Python script that calls undefined_function()",
        "[agent] run a python script that calls undefined_function()",
        &mut log,
    )
    .expect("the agent failure is answered");
    assert_eq!(agent.intent, "execution_failure");
    assert_eq!(agent.answer, EXECUTION_FAILURE);
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "agent_mode:opted_in"),
        "the agent opt-in is recorded"
    );
}

#[test]
fn an_incompatible_unit_pair_answers_unchanged_from_the_seeded_wording() {
    for (prompt, expected) in [
        ("How many meters are in a kilogram?", UNITS_EN),
        ("Сколько метров в килограмме?", UNITS_RU),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "unit_incompatibility", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
}

const RESEARCH_SEARCH: &str = "Search for information about:\n1. Machine learning algorithms\n2. Deep learning vs traditional ML\n3. Neural networks basics";
const RESEARCH_TABLE: &str = "Research comparison table (draft; verify claims against the source links from the preceding retrieval).\n\n| Topic | Key differences | Use cases | Advantages | Disadvantages |\n| --- | --- | --- | --- | --- |\n| Machine learning algorithms | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |\n| Deep learning vs traditional ML | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |\n| Neural networks basics | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |";
const RESEARCH_TASK: &str = "Research task: What would be the economic impact if Rust replaced C++ in all major open-source projects by 2030?\nSteps required:\n1. Search for current C++ vs Rust usage statistics in open-source projects.";
const RESEARCH_NO_RESULTS: &str = "The result of the previous research step is: no CORS-readable web search results were returned. I do not have verified source data to complete the requested analysis, calculation, table, or sources list yet.\n\nPrior research task: `Research task: What would be the economic impact if Rust replaced C++ in all major open-source projects by 2030? Steps required: 1. Search for current C++ vs Rust usage statistics in open-source projects.`\n\nNext step: rerun the search with narrower queries or provide source links; then I can calculate the requested impact from those sources.";
const RESEARCH_OPEN: &str = "There is no verified final research result in the conversation yet. The prior turn was a research request, but I do not see a completed source-backed answer to report.\n\nPrior research task: `Research task: What would be the economic impact if Rust replaced C++ in all major open-source projects by 2030? Steps required: 1. Search for current C++ vs Rust usage statistics in open-source projects.`\n\nNext step: run the search or provide source links; then I can produce the requested result.";

#[test]
fn a_research_comparison_table_renders_the_seeded_procedure() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user(RESEARCH_SEARCH),
        ConversationTurn::assistant(solver.solve(RESEARCH_SEARCH).answer),
    ];
    let response = solver.solve_with_history(
        "create a comparison table showing:\n- Key differences\n- Use cases for each\n- Advantages and disadvantages",
        &history,
    );
    assert_eq!(response.intent, "research_comparison_table");
    assert_eq!(response.answer, RESEARCH_TABLE);
}

#[test]
fn a_research_result_followup_reads_its_status_and_wording_from_the_seed() {
    let solver = UniversalSolver::default();
    for (prior_answer, expected, status) in [
        (
            "No CORS-enabled web search results were returned for `x`.\n\nProviders tried: DuckDuckGo.",
            RESEARCH_NO_RESULTS,
            "no_results",
        ),
        ("Here is a summary I wrote.", RESEARCH_OPEN, "open_research"),
    ] {
        let history = [
            ConversationTurn::user(RESEARCH_TASK),
            ConversationTurn::assistant(prior_answer),
        ];
        let response = solver.solve_with_history("What is the result?", &history);
        assert_eq!(response.intent, "research_result_followup", "{status}");
        assert_eq!(response.answer, expected, "{status}");
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("research_result_followup:status:{status}")),
            "{status}: {:?}",
            response.evidence_links
        );
    }
}

const CONCEPT_INTROSPECTION: &str = "Here is what I know about 'greeting':\n\nintent: greeting\nrole: the network records 'greeting' as a concept with rules and example links.";
const FILTER_USER: &str = "No facts have been recorded under your user filter yet. Submit a 'teach this fact' request to start your personal contribution list.";

#[test]
fn the_network_query_rules_answer_unchanged_through_the_engine() {
    let snapshot = FormalAiEngine.answer("Export the network");
    assert_eq!(snapshot.intent, "network_snapshot");
    let expected_snapshot = format!(
        "Here is the current link network as a links-notation snapshot:\n\n```links\n{}\n```",
        formal_ai::knowledge_links_notation()
    );
    assert_eq!(snapshot.answer, expected_snapshot);

    let introspection = UniversalSolver::default().solve("What do you know about 'greeting'?");
    assert_eq!(introspection.answer, CONCEPT_INTROSPECTION);
    assert!(
        introspection
            .evidence_links
            .iter()
            .any(|link| link == "intent:concept_introspection_greeting"),
        "the rule's intent names the captured concept: {:?}",
        introspection.evidence_links
    );

    let filter = FormalAiEngine.answer("List the facts I have contributed");
    assert_eq!(filter.intent, "filter_user");
    assert_eq!(filter.answer, FILTER_USER);
    assert!(
        filter
            .evidence_links
            .iter()
            .any(|link| link.starts_with("filter:user")),
        "personal queries declare a user filter"
    );
}

/// The shell-command rewrite reads its loop and session templates, joiners,
/// prompt markers, command heads and prose leads from
/// `data/seed/code-task-cues.lino`; the four-language replay of
/// `tests/unit/specification/shared_dialog_replay.rs` pins the same answers.
#[test]
fn the_shell_rewrites_answer_unchanged_from_the_seeded_syntax() {
    let solver = UniversalSolver::default();
    let looped = solver.solve(
        "box@87ffc301f5eb:~$ sleep 30m && hive-cleanup -f\n\nmake a loop of that (infinite), answer with only single line",
    );
    assert_eq!(looped.intent, "shell_command_transform");
    assert_eq!(
        looped.answer,
        "while true; do sleep 30m && hive-cleanup -f; done"
    );

    let screen = solver.solve_with_history(
        "Use `screen -R auto-cleanup` to execute that line inside, answer in one line.",
        &[ConversationTurn::assistant(
            "while true; do sleep 30m && hive-cleanup -f; done",
        )],
    );
    assert_eq!(screen.intent, "shell_command_transform");
    assert_eq!(
        screen.answer,
        "screen -dmS auto-cleanup bash -c 'while true; do sleep 30m && hive-cleanup -f; done'"
    );
}

const RETARGET_ZH: &str = "我是 formal-ai —— 一个确定性的符号化 AI 系统,根据本地的 Links Notation 规则和兼容 OpenAI 的 API 形式作答。本演示不进行任何神经网络推理。";

/// The response-language follow-up replays the prior request through the whole
/// solver with the requested language forced; how short a bare language switch
/// may be is the `terse_word_limit` policy of the rule document, not a constant
/// in either runtime.
#[test]
fn a_terse_language_switch_replays_the_prior_answer_under_the_seeded_limit() {
    assert_eq!(
        handler_policy("response_language_followup", "terse_word_limit").as_deref(),
        Some("4")
    );
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("что ты такое"),
        ConversationTurn::assistant(
            "Я formal-ai — детерминированный символьный ИИ, отвечающий по локальным правилам Links Notation.",
        ),
    ];
    let response = solver.solve_with_history("用中文", &history);
    assert_eq!(response.intent, "identity");
    assert_eq!(response.answer, RETARGET_ZH);
    assert!(
        response
            .evidence_links
            .contains(&"response_language_followup:target:zh".to_owned()),
        "{:?}",
        response.evidence_links
    );
}
