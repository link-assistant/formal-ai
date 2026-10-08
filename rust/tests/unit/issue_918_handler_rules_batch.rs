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
//! A later batch moved `algorithm` into the rule document: the sort
//! operation (a new `operation` condition over the seeded operation
//! vocabulary) or the word "algorithm", the language a `table` block finds,
//! and the snippets as seeded responses read by a `response` value. Its
//! answers are byte-identical to the deleted `try_algorithm` in every prompt
//! language.
//!
//! The same batch moved the document-generation plan's vocabulary into the
//! `document_*` tables of the rule document, read by the native handler
//! through `rule_interpreter::handler_table_row`, and its four-language plan
//! and the conversion answer's lines into seeded responses.
//!
//! The English and Russian answers below are byte-identical to the ones the
//! deleted Rust produced (`tests/unit/specification/issue_146.rs` pins the
//! same two conversation-topic answers). The browser twin is
//! `rust/tests/web/issue-0918-handler-rules-batch.test.mjs`.

use formal_ai::event_log::EventLog;
use formal_ai::rule_interpreter::{
    handler_claims, handler_policy, handler_table_value, rules, run_handler,
};
use formal_ai::{ConversationTurn, FormalAiEngine, UniversalSolver};

const TOPIC_EN: &str = "We can talk about existence. I can start with a short definition, context, or a specific question; when web search is available, public facts can be checked against an external source.";
const TOPIC_RU: &str = "Можем. Тема: бытие. Я могу начать с краткого определения, контекста или конкретного вопроса; если веб-поиск доступен, публичные факты можно уточнить через внешний источник.";
const TOPIC_HI: &str = "हम बात कर सकते हैं. विषय: गणित. मैं छोटी परिभाषा, संदर्भ, या किसी ठोस प्रश्न से शुरू कर सकता हूँ; web search उपलब्ध हो तो public facts बाहरी स्रोत से जाँचे जा सकते हैं.";
const TOPIC_ZH: &str = "可以聊。主题: 音乐。我可以从简短定义、上下文或具体问题开始; 如果 web search 可用, 公开事实可以通过外部来源核对。";
const CONFLICT_EN: &str = "The sources you cite disagree: Wikipedia says X was born in 1880; Britannica says 1881. The disagreement is recorded as a conflict:source_disagreement link in the network rather than silently resolved.";
const CONFLICT_UNATTRIBUTED_EN: &str = "No source is named for either answer, so no disagreement between sources can be recorded. Name each source and what it states, for example: Wikipedia says 1880, but Britannica says 1881.";
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
        "algorithm",
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
    let unattributed = FormalAiEngine.answer("Was X born in 1880 or 1881?");
    assert_eq!(unattributed.intent, "source_conflict");
    assert_eq!(unattributed.answer, CONFLICT_UNATTRIBUTED_EN);
    // Issue #1175 R3: the conflict names the two alternatives it attributes
    // to sources; the disjunction above attributes neither and is refused.
    let prompt =
        "The sources conflict: Wikipedia says X was born in 1880, but Britannica says 1881.";
    let mut log = EventLog::default();
    let response = run_handler("source_conflict", prompt, &prompt.to_lowercase(), &mut log)
        .expect("an attributed conflict is answered");
    assert_eq!(response.intent, "source_conflict");
    assert_eq!(response.answer, CONFLICT_EN);
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "conflict:source_disagreement"),
        "the disagreement is recorded, not resolved: {:?}",
        log.events()
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

const BRAINSTORM_FIVE: &str = "1. A local Links Notation notebook with searchable traces.\n2. A deterministic code-review checklist generator.\n3. A multilingual prompt-variation test corpus.\n4. A CLI that converts issue requirements into traceable tests.\n5. A source-cache inspector for reproducible agent runs.";
const BRAINSTORM_TEN: &str = "1. A local Links Notation notebook with searchable traces.\n2. A deterministic code-review checklist generator.\n3. A multilingual prompt-variation test corpus.\n4. A CLI that converts issue requirements into traceable tests.\n5. A source-cache inspector for reproducible agent runs.\n6. A changelog-fragment consistency checker.\n7. A prompt-matrix generator for four-language smoke tests.\n8. A Wikidata anchor verifier for local seed records.\n9. A trace viewer that groups events by solver phase.\n10. A small offline issue-to-test planning tool.";

/// The seeded brainstorm list: how many items a reply lists by default and
/// which cardinal a prompt names to ask for more are the seed's
/// `default_count` and `count_cardinal`, not constants in either runtime.
/// The list itself is seeded example data; topic-bearing requests are
/// composed by `brainstorm_composition` before this row is reached.
#[test]
fn the_seeded_brainstorm_list_reads_its_count_policy_from_the_seed() {
    for (prompt, expected) in [
        (
            "Give me five ideas for an open-source side project.",
            BRAINSTORM_FIVE,
        ),
        (
            "Suggest ten open-source utilities for developers.",
            BRAINSTORM_TEN,
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "brainstorm_project_ideas", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
}

/// The roleplay frame, personas, topics and fallback body are
/// `data/seed/personas.lino`; the browser twin no longer carries a bootstrap
/// copy of them.
#[test]
fn a_roleplay_frame_renders_the_seeded_persona_and_topic() {
    for (prompt, expected) in [
        (
            "Explain like you are Ada Lovelace teaching algorithms.",
            "Roleplay frame recorded for Ada Lovelace. I will keep the persona explicit and factual: an algorithm is a precise sequence of steps, so a reliable explanation names the inputs, the ordered operations, and the expected result.",
        ),
        (
            "Roleplay as a teacher explaining relativity.",
            "Roleplay frame recorded for teacher. I will keep the persona explicit and factual: relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent.",
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "roleplay_explanation", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
}

const SPIDER_EN: &str = "Released title-role Spider-Man films in release order: 1. Spider-Man (2002); 2. Spider-Man 2 (2004); 3. Spider-Man 3 (2007); 4. The Amazing Spider-Man (2012); 5. The Amazing Spider-Man 2 (2014); 6. Spider-Man: Homecoming (2017); 7. Spider-Man: Into the Spider-Verse (2018); 8. Spider-Man: Far From Home (2019); 9. Spider-Man: No Way Home (2021); 10. Spider-Man: Across the Spider-Verse (2023); 11. Spider-Man: Brand New Day (2026). Announced but not yet released: Spider-Man: Beyond the Spider-Verse (2027-06-18). Source: Wikidata Query Service, snapshot taken 2026-08-04.";
const SPIDER_RU: &str = "Вышедшие фильмы, где Человек-паук — главный герой в порядке выхода: 1. Человек-паук (2002); 2. Человек-паук 2 (2004); 3. Человек-паук 3: Враг в отражении (2007); 4. Новый Человек-паук (2012); 5. Новый Человек-паук. Высокое напряжение (2014); 6. Человек-паук: Возвращение домой (2017); 7. Человек-паук: Через вселенные (2018); 8. Человек-паук: Вдали от дома (2019); 9. Человек-паук: Нет пути домой (2021); 10. Человек-паук: Паутина вселенных (2023); 11. Человек-паук: Новый день (2026). Анонсированы, но ещё не вышли: Человек-паук: За пределами вселенных (2027-06-18). Источник: Wikidata Query Service, снимок данных от 2026-08-04.";

/// A fact backed by a release timeline is the snapshot rendered against the
/// day it is asked (`data/seed/release-timelines.lino`); the browser twin
/// rendered nothing for it (an empty answer) until it got the same renderer.
#[test]
fn a_release_timeline_fact_renders_the_same_snapshot_answer() {
    for (language, expected) in [("en", SPIDER_EN), ("ru", SPIDER_RU)] {
        let rendered = formal_ai::release_timeline::render(
            "spider_man_title_role_films",
            language,
            "2026-08-04",
        )
        .expect("the Spider-Man timeline renders");
        assert_eq!(rendered.text, expected, "{language}");
    }
    let response = UniversalSolver::default().solve("List Spider-Man films in release order.");
    let today = formal_ai::external_benchmarks::today_utc();
    let expected = formal_ai::release_timeline::render("spider_man_title_role_films", "en", &today)
        .expect("the Spider-Man timeline renders")
        .text;
    assert_eq!(response.intent, "fact_lookup");
    assert_eq!(response.answer, expected);
}

const CONCEPT_WIKIDATA: &str = "Wikidata (structured-knowledge): Wikidata is a collaboratively edited multilingual knowledge graph hosted by the Wikimedia Foundation. It stores structured data items that power Wikipedia infoboxes and external knowledge applications.\n\nSource: https://en.wikipedia.org/wiki/Wikidata (wikipedia).";

/// The concept-query reader takes its request leads, question starts,
/// inverted-who frame, meaning-question leads and tails, non-subject words,
/// idiom tails and articles from the `concept_lookup` cue records of
/// `data/seed/code-task-cues.lino` in both runtimes. A meaning question
/// ("What does X mean?") walks the meaning-tail and stem-lead lists.
#[test]
fn a_meaning_question_resolves_through_the_seeded_query_vocabulary() {
    let response = FormalAiEngine.answer("What does Wikidata mean?");
    assert_eq!(response.intent, "concept_lookup");
    assert_eq!(response.answer, CONCEPT_WIKIDATA);
}

const SUMMARY_RUST: &str = "Rust is a multi-paradigm, general-purpose programming language that emphasises performance, type safety, and concurrency. It enforces memory safety without using a garbage collector.";
const SUMMARY_FORMAL_AI: &str =
    "formal-ai is a deterministic symbolic AI that answers without any neural-network inference.";

/// A topic summary is the seeded topic's canonical concept summary or its
/// heaviest project statement (`data/seed/summary-topics.lino`); the browser
/// answered these requests with the unknown opener until it got the same
/// reader, run after web search as the native row is.
#[test]
fn a_topic_summary_is_the_seeded_record_in_both_runtimes() {
    for (prompt, expected) in [
        ("Can you summarize Rust?", SUMMARY_RUST),
        (
            "Please summarize formal-ai in one paragraph.",
            SUMMARY_FORMAL_AI,
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "summarize_topic", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
}

const SORT_RUST: &str = "Here is a reviewable sorting algorithm in rust:\n\n```rust\nfn sort(values: &mut Vec<i32>) {\n    values.sort();\n}\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";
const SORT_GO: &str = "Here is a reviewable sorting algorithm in go:\n\n```python\ndef sort(values):\n    return sorted(values)\n\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";
const SORT_TYPESCRIPT_TESTS: &str = "Here is a reviewable sorting algorithm in typescript with a test:\n\n```typescript\nfunction sort(values) {\n  return [...values].sort((a, b) => a - b);\n}\n```\n\nTests:\n```typescript\nfunction test_sort_ascending() {\n  assert.deepEqual(sort([3,1,2]), [1,2,3]);\n}\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";
const SORT_PYTHON_TESTS: &str = "Here is a reviewable sorting algorithm in python with a test:\n\n```python\ndef sort(values):\n    return sorted(values)\n\n```\n\nTests:\n```python\ndef test_sort_ascending():\n    assert sort([3, 1, 2]) == [1, 2, 3]\n\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";
const SORT_PYTHON: &str = "Here is a reviewable sorting algorithm in python:\n\n```python\ndef sort(values):\n    return sorted(values)\n\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";

#[test]
fn the_algorithm_rule_set_answers_unchanged_in_every_prompt_language() {
    let english = FormalAiEngine.answer("Write me a sorting algorithm in Rust");
    assert_eq!(english.intent, "algorithm_sort_rust");
    assert_eq!(english.answer, SORT_RUST);
    for (prompt, intent, expected) in [
        (
            "write a sorting algorithm in python with tests",
            "algorithm_sort_python",
            SORT_PYTHON_TESTS,
        ),
        (
            "Напиши сортировку списка на go с тестом",
            "algorithm_sort_go",
            SORT_GO,
        ),
        (
            "Rust में सॉर्टिंग एल्गोरिदम लिखो",
            "algorithm_sort_rust",
            SORT_RUST,
        ),
        (
            "写一个 typescript 排序 algorithm 带 test",
            "algorithm_sort_typescript",
            SORT_TYPESCRIPT_TESTS,
        ),
    ] {
        let mut log = EventLog::default();
        let answer = run_handler("algorithm", prompt, &prompt.to_lowercase(), &mut log)
            .unwrap_or_else(|| panic!("{prompt} is answered by the algorithm rules"));
        assert_eq!(answer.intent, intent, "{prompt}");
        assert_eq!(answer.answer, expected, "{prompt}");
        assert!(
            log.events()
                .iter()
                .any(|event| event.kind == "execution_status" && event.payload == "unavailable"),
            "{prompt} records the execution status"
        );
        assert!(
            !log.events()
                .iter()
                .any(|event| event.kind == "algorithm:refusal"),
            "{prompt} names the sort operation"
        );
    }
    let mut log = EventLog::default();
    let unnamed = run_handler(
        "algorithm",
        "explain the algorithm",
        "explain the algorithm",
        &mut log,
    )
    .expect("an algorithm request without an operation is still answered");
    assert_eq!(unnamed.answer, SORT_PYTHON);
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "algorithm:refusal" && event.payload == "no operation named")
    );
    let mut log = EventLog::default();
    assert!(
        run_handler(
            "algorithm",
            "What is the capital of France?",
            "what is the capital of france?",
            &mut log
        )
        .is_none()
    );
}

const PLAN_EN_PDF: &str = "This is a document-generation request in PDF format. I am a deterministic symbolic solver: I cannot research arbitrary live data on the web and I do not render binary files directly, so I decompose the task into the formal plan the universal algorithm produces (decompose → tests → drafts → composition). The document workflow uses link-foundation/meta-language for txt, Markdown, HTML, PDF, and DOCX representation/conversion, with concept profiles for headings, paragraphs, lists, strong/bold text, emphasis, and hyperlinks:\n\n1. Scope the document and its criteria: which items to include and which attributes distinguish them.\n2. Collect the list of items from verifiable sources and record links to those sources.\n3. Classify each item against the stated criteria.\n4. Assemble the document structure: title, sections, and a table or list.\n5. Export the finished structure to the requested format.\n\nConfirm the plan or refine the criteria and sources and I will continue with concrete steps. If you need facts that are not in the local Links Notation memory, name a source and I will add it as a links rule.";
const PLAN_RU_PDF: &str = "Это запрос на создание документа в формате PDF. Я детерминированный символьный решатель: у меня нет доступа к произвольным актуальным данным в вебе и я не рендерю бинарные файлы напрямую, поэтому я раскладываю задачу на формальный план по универсальному алгоритму (декомпозиция → проверки → черновики → композиция):\n\n1. Уточнить объём и критерии документа: какие элементы включать и по каким признакам их различать.\n2. Собрать список элементов из проверяемых источников и зафиксировать ссылки на эти источники.\n3. Классифицировать каждый элемент по заявленным критериям.\n4. Собрать структуру документа: заголовок, разделы и таблицу или список.\n5. Экспортировать готовую структуру в запрошенный формат.\n\nПодтвердите план или уточните критерии и источники — и я продолжу с конкретными шагами. Если нужны фактические данные, которых нет в локальной памяти Links Notation, укажите источник, и я добавлю его как правило связей.";
const PLAN_HI_PDF: &str = "यह एक दस्तावेज़ बनाने का अनुरोध है (PDF प्रारूप में). मैं एक नियतात्मक प्रतीकात्मक हल करने वाला हूँ: मैं वेब पर मनमाना सजीव डेटा नहीं खोज सकता और बाइनरी फ़ाइलें सीधे नहीं बनाता, इसलिए मैं इस कार्य को सार्वभौमिक एल्गोरिदम की औपचारिक योजना में विभाजित करता हूँ (विभाजन → जाँच → मसौदे → रचना):\n\n1. दस्तावेज़ और उसके मानदंड का दायरा तय करें: कौन-सी वस्तुएँ शामिल करनी हैं और कौन-से गुण उन्हें अलग करते हैं।\n2. सत्यापन योग्य स्रोतों से वस्तुओं की सूची एकत्र करें और उन स्रोतों के लिंक दर्ज करें।\n3. प्रत्येक वस्तु को बताए गए मानदंड के अनुसार वर्गीकृत करें।\n4. दस्तावेज़ की संरचना बनाएँ: शीर्षक, अनुभाग और एक तालिका या सूची।\n5. तैयार संरचना को अनुरोधित प्रारूप में निर्यात करें।\n\nयोजना की पुष्टि करें या मानदंड और स्रोत स्पष्ट करें, और मैं ठोस चरणों के साथ आगे बढ़ूँगा।";
const PLAN_ZH_PDF: &str = "这是一个生成文档的请求（PDF 格式）。我是一个确定性的符号求解器：我无法在网络上检索任意实时数据，也不会直接渲染二进制文件，因此我把任务分解为通用算法生成的形式化计划（分解 → 校验 → 草稿 → 组合）：\n\n1. 界定文档及其标准：包含哪些条目以及用哪些属性区分它们。\n2. 从可验证的来源收集条目清单，并记录这些来源的链接。\n3. 根据所述标准对每个条目进行分类。\n4. 组装文档结构：标题、章节以及表格或列表。\n5. 将完成的结构导出为所请求的格式。\n\n请确认计划或细化标准与来源，我将继续给出具体步骤。";
const PLAN_RU_DOCUMENT: &str = "Это запрос на создание документа. Я детерминированный символьный решатель: у меня нет доступа к произвольным актуальным данным в вебе и я не рендерю бинарные файлы напрямую, поэтому я раскладываю задачу на формальный план по универсальному алгоритму (декомпозиция → проверки → черновики → композиция):\n\n1. Уточнить объём и критерии документа: какие элементы включать и по каким признакам их различать.\n2. Собрать список элементов из проверяемых источников и зафиксировать ссылки на эти источники.\n3. Классифицировать каждый элемент по заявленным критериям.\n4. Собрать структуру документа: заголовок, разделы и таблицу или список.\n5. Экспортировать готовую структуру в запрошенный формат.\n\nПодтвердите план или уточните критерии и источники — и я продолжу с конкретными шагами. Если нужны фактические данные, которых нет в локальной памяти Links Notation, укажите источник, и я добавлю его как правило связей.";

#[test]
fn the_document_plan_renders_the_seeded_plan_in_every_prompt_language() {
    for (prompt, expected) in [
        (
            "Make me a PDF document listing countries with food subsidies for low-income people.",
            PLAN_EN_PDF,
        ),
        (
            "Сделай мне пдф файл со списком стран, где есть пособия/скидки на еду для малоимущих, как в виде прямых денежных дотаций, так и в косвенной форме, например, талоны.",
            PLAN_RU_PDF,
        ),
        (
            "गरीब लोगों के लिए खाद्य सब्सिडी वाले देशों की सूची के साथ एक PDF दस्तावेज़ बनाओ।",
            PLAN_HI_PDF,
        ),
        (
            "给我做一个包含为低收入者提供食品补贴的国家列表的PDF文档。",
            PLAN_ZH_PDF,
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "document_generation_plan", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
    assert!(PLAN_RU_DOCUMENT.starts_with("Это запрос на создание документа. "));
    assert_eq!(
        formal_ai::rule_interpreter::handler_table_row("document_format", "make me a pdf"),
        Some("PDF")
    );
    assert_eq!(
        formal_ai::rule_interpreter::handler_table_row("document_noun", "сделай отчёт"),
        Some("document")
    );
    assert_eq!(
        formal_ai::rule_interpreter::handler_table_value("document_output_fence", "TOML"),
        Some("")
    );
}

/// The calendar rows (issue #918): the weekday names with the Russian case a
/// direction phrase takes, the default event title and every cue list of the
/// event parser are `calendar_*` tables of the rule document; the relation
/// and confirmation sentences are seeded responses. The browser twin pins the
/// same strings for the same prompts.
const CALENDAR_RELATIONS: [(&str, &str); 9] = [
    (
        "What day of the week comes after Tuesday?",
        "The day after Tuesday is Wednesday. I move Tuesday by +1 in the seven-day calendar cycle.",
    ),
    (
        "What day comes before Monday?",
        "The day before Monday is Sunday. I move Monday by -1 in the seven-day calendar cycle.",
    ),
    (
        "какой день недели перед средой",
        "Перед средой идёт вторник. Я сдвинул среда на -1 в семидневном календарном цикле.",
    ),
    (
        "следующий день после воскресенья",
        "После воскресенья наступает понедельник. Я сдвинул воскресенье на +1 в семидневном календарном цикле.",
    ),
    (
        "सोमवार के बाद कौन सा दिन आता है",
        "सोमवार के बाद मंगलवार आता है। मैं सात दिनों के कैलेंडर चक्र में सोमवार को +1 दिन सरकाता हूँ।",
    ),
    (
        "सोमवार से पहले कौन सा दिन आता है",
        "सोमवार से पहले रविवार आता है। मैं सात दिनों के कैलेंडर चक्र में सोमवार को -1 दिन सरकाता हूँ।",
    ),
    (
        "星期一之后是星期几",
        "星期一之后是星期二。我在七天的日历循环中将星期一移动+1天。",
    ),
    (
        "星期三之前是星期几",
        "星期三之前是星期二。我在七天的日历循环中将星期三移动-1天。",
    ),
    (
        "какой день будет через 100 дней после понедельника?",
        "Через 100 дней после понедельника — среда. 100 дней = 14 недель + 2 дня; понедельник + 2 дня = среда в семидневном календарном цикле.",
    ),
];

#[test]
fn the_weekday_relations_render_the_seeded_names_and_sentences() {
    for (prompt, expected) in CALENDAR_RELATIONS {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "calendar_weekday_relation", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
    assert_eq!(
        handler_table_value("calendar_weekday_label", "monday.ru.next"),
        Some("понедельника")
    );
    assert_eq!(
        handler_table_value("calendar_default_title", "es"),
        Some("Event")
    );
}

/// A confirmation with its volatile parts (the ICS block, the Google Calendar
/// link, the event date and the Russian day number) replaced by named slots,
/// so the seeded sentence around them is pinned exactly.
fn confirmation_shape(answer: &str) -> String {
    const END: &str = "END:VCALENDAR\r\n";
    let start = answer.find("BEGIN:VCALENDAR").expect("an ICS block");
    let end = answer.find(END).expect("the ICS end") + END.len();
    let ics = &answer[start..end];
    let stamp = ics
        .split("DTSTART;TZID=")
        .nth(1)
        .and_then(|rest| rest.split_once(':'))
        .map(|(_, value)| value[..8].to_owned())
        .expect("a start stamp");
    let date = format!("{}-{}-{}", &stamp[..4], &stamp[4..6], &stamp[6..8]);
    let day = stamp[6..8].trim_start_matches('0');
    let link = answer.find("https://calendar.google.com").expect("a link");
    let url = answer[link..].split('\n').next().unwrap_or_default();
    answer
        .replace(ics, concat!("{", "ics}"))
        .replace(url, concat!("{", "url}"))
        .replace(&date, concat!("{", "date}"))
        .replace(&format!("на {day} число"), concat!("на {", "day} число"))
}

#[test]
fn an_event_confirmation_renders_the_seeded_sentence_in_every_prompt_language() {
    for (prompt, expected) in [
        (
            "schedule a call for tomorrow",
            "Create event «Call» on {date}. Time: 17:00, timezone: UTC. Duration 60 minutes.\nImport this .ics file into any calendar:\n{ics}\nOr open it in Google Calendar (no login required):\n{url}\nReply 'yes' to confirm.",
        ),
        (
            "поставь созвон на завтра",
            "Создать событие «Созвон» на {day} число ({date}). Время: 17:00, часовой пояс: UTC. Длительность 60 минут.\nИмпортируйте этот файл .ics в любой календарь:\n{ics}\nИли откройте в Google Календаре (вход не требуется):\n{url}\nОтветьте «да», чтобы подтвердить.",
        ),
        (
            "कल मीटिंग शेड्यूल करें",
            "{date} (17:00, समय क्षेत्र UTC) पर «मीटिंग» कार्यक्रम बनाएँ। अवधि 60 मिनट।\nइस .ics फ़ाइल को किसी भी कैलेंडर में आयात करें:\n{ics}\nया Google Calendar में खोलें (लॉगिन आवश्यक नहीं):\n{url}\nपुष्टि के लिए «हाँ» उत्तर दें।",
        ),
        (
            "明天安排一个通话",
            "在 {date}（17:00，时区 UTC）创建事件「通话」。时长 60 分钟。\n将此 .ics 文件导入任何日历：\n{ics}\n或在 Google 日历中打开（无需登录）：\n{url}\n回复「是」以确认。",
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "calendar_create_event", "{prompt}");
        assert_eq!(confirmation_shape(&response.answer), expected, "{prompt}");
    }
}
