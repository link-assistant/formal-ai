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
//! The English and Russian answers below are byte-identical to the ones the
//! deleted Rust produced (`tests/unit/specification/issue_146.rs` pins the
//! same two conversation-topic answers). The browser twin is
//! `rust/tests/web/issue-0918-handler-rules-batch.test.mjs`.

use formal_ai::FormalAiEngine;
use formal_ai::event_log::EventLog;
use formal_ai::rule_interpreter::{handler_claims, rules, run_handler};

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
