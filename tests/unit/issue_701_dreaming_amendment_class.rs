//! Issue #701 §4 — dreaming amendments must change *solving*, class-wide.
//!
//! The pre-existing coverage
//! (`learned_amendment_changes_a_new_task_answer_without_repeating_requirement`)
//! pinned one topic, one language, one held-out prompt, and asserted only that
//! the compliance line appeared. That assertion is satisfied by a pure string
//! append, so it could not distinguish learning from decoration.
//!
//! This suite raises it to a class: multiple topics × every supported language ×
//! held-out paraphrases, driven through the production protocol surfaces, and it
//! asserts the *non-decorative* delta — an answer covered by a retained standing
//! requirement is never reported as unresolved.

use formal_ai::dreaming_application::STANDING_REQUIREMENT_INTENT;
use formal_ai::{
    create_chat_completion_with_solver_and_memory, create_response_with_solver_and_memory,
    solve_with_amendment_records, solve_with_standing_requirements, ChatCompletionRequest,
    ChatMessage, MemoryEvent, ResponsesRequest, RetainedAmendment, UniversalSolver,
};

/// A learned topic, its standing rule, and the held-out paraphrases — one per
/// supported language — that the engine has never seen before.
struct AmendedTopic {
    topic: &'static str,
    rule: &'static str,
    /// `(language, held-out paraphrase)`; the topic token is embedded in each
    /// prompt exactly as a real multilingual request would carry it.
    held_out: [(&'static str, &'static str); 4],
}

const SUPPORTED_LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

fn amended_topics() -> Vec<AmendedTopic> {
    vec![
        AmendedTopic {
            topic: "latex",
            rule: "Always include a LaTeX verification step in proof solutions.",
            held_out: [
                ("en", "latex: solve a new recurrence proof"),
                ("ru", "latex: разобрать новое рекуррентное доказательство"),
                ("hi", "latex: एक नया पुनरावर्ती प्रमाण हल करो"),
                ("zh", "latex: 求解一个新的递推证明"),
            ],
        },
        AmendedTopic {
            topic: "benchmark",
            rule: "Always report the sample size next to a benchmark result.",
            held_out: [
                ("en", "benchmark: compare two sorting routines"),
                ("ru", "benchmark: сравнить две процедуры сортировки"),
                ("hi", "benchmark: दो सॉर्टिंग रूटीन की तुलना करो"),
                ("zh", "benchmark: 比较两个排序例程"),
            ],
        },
        AmendedTopic {
            topic: "unicode",
            rule: "Always normalize unicode text to NFC before comparing it.",
            held_out: [
                ("en", "unicode: fold these two names together"),
                ("ru", "unicode: свести эти два имени вместе"),
                ("hi", "unicode: इन दो नामों को एक साथ मिलाओ"),
                ("zh", "unicode: 把这两个名字合并起来"),
            ],
        },
    ]
}

fn amendment_events(topics: &[AmendedTopic]) -> Vec<MemoryEvent> {
    topics
        .iter()
        .enumerate()
        .map(|(index, topic)| MemoryEvent {
            id: format!("amendment-{index}"),
            kind: Some(String::from("meta_algorithm_amendment")),
            role: Some(String::from("system")),
            intent: Some(String::from("generalize")),
            content: Some(String::from(topic.rule)),
            inputs: Some(format!("topic={}", topic.topic)),
            outputs: Some(format!("rule={}", topic.rule)),
            demo_label: Some(String::from(topic.topic)),
            ..MemoryEvent::default()
        })
        .collect()
}

fn chat_answer(prompt: &str, events: &[MemoryEvent]) -> String {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage::user(prompt)],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };
    let completion =
        create_chat_completion_with_solver_and_memory(&request, &UniversalSolver::default(), events);
    completion.choices[0].message.content.plain_text()
}

fn responses_answer(prompt: &str, events: &[MemoryEvent]) -> String {
    let request = ResponsesRequest {
        input: serde_json::Value::String(String::from(prompt)),
        ..ResponsesRequest::default()
    };
    let response =
        create_response_with_solver_and_memory(&request, &UniversalSolver::default(), events);
    response.output_messages()[0].content[0].text.clone()
}

#[test]
fn retained_amendments_change_held_out_answers_across_topics_and_languages() {
    let topics = amended_topics();
    let events = amendment_events(&topics);

    let mut covered_languages = Vec::new();
    for topic in &topics {
        for (language, prompt) in topic.held_out {
            let before = chat_answer(prompt, &[]);
            let after = chat_answer(prompt, &events);
            assert_ne!(
                before, after,
                "[{}/{language}] a retained amendment must change the held-out answer",
                topic.topic,
            );
            assert!(
                after.contains(topic.rule),
                "[{}/{language}] the standing rule must reach the answer: {after}",
                topic.topic,
            );
            // The requirement is *not* repeated back at the user as their own
            // words: it arrives as learned knowledge the engine now holds.
            assert!(
                after.contains("Learned standing requirement"),
                "[{}/{language}] {after}",
                topic.topic,
            );

            // The same class holds on the responses surface, not only on chat.
            let responses_after = responses_answer(prompt, &events);
            assert!(
                responses_after.contains(topic.rule),
                "[{}/{language}] responses surface must carry the rule too",
                topic.topic,
            );

            if !covered_languages.contains(&language) {
                covered_languages.push(language);
            }
        }
    }

    assert_eq!(topics.len(), 3, "the class spans at least three topics");
    for language in SUPPORTED_LANGUAGES {
        assert!(
            covered_languages.contains(&language),
            "language {language} must be exercised by the class-level suite",
        );
    }
}

#[test]
fn a_covered_task_is_never_reported_unresolved() {
    // The non-decorative delta. A string append cannot change how the answer is
    // *classified*; this can, and it is the check that would fail if the
    // amendment path regressed back into decoration.
    let topics = amended_topics();
    let amendments = topics
        .iter()
        .map(|topic| RetainedAmendment {
            id: format!("amendment:{}", topic.topic),
            topic: String::from(topic.topic),
            rule: String::from(topic.rule),
        })
        .collect::<Vec<_>>();

    let solver = UniversalSolver::default();
    let mut reclassified = 0;
    for topic in &topics {
        for (language, prompt) in topic.held_out {
            let plain = solver.solve(prompt);
            let amended = solve_with_amendment_records(&solver, prompt, &[], &amendments);
            assert_ne!(
                amended.intent, "unknown",
                "[{}/{language}] a task covered by a standing requirement must not stay unresolved",
                topic.topic,
            );
            if plain.intent == "unknown" {
                assert_eq!(
                    amended.intent, STANDING_REQUIREMENT_INTENT,
                    "[{}/{language}] an otherwise-unroutable task is resolved by the amendment",
                    topic.topic,
                );
                reclassified += 1;
            }
            assert!(
                amended
                    .evidence_links
                    .iter()
                    .any(|link| link.starts_with("meta_algorithm_amendment:")),
                "[{}/{language}] the amendment must be cited as evidence",
                topic.topic,
            );
        }
    }
    assert!(
        reclassified > 0,
        "the suite must exercise at least one genuinely unroutable task",
    );
}

#[test]
fn an_unrelated_task_is_left_byte_identical() {
    // The delta is scoped, not global: a task no retained amendment covers must
    // come back exactly as it would with an empty memory. Without this, "the
    // answer changed" would be evidence of nothing.
    let events = amendment_events(&amended_topics());
    for prompt in [
        "what is the capital of France?",
        "какая столица Франции?",
        "2 + 2",
    ] {
        assert_eq!(
            chat_answer(prompt, &[]),
            chat_answer(prompt, &events),
            "an uncovered task must not be touched by retained amendments",
        );
    }
}

#[test]
fn the_production_entry_point_and_the_replay_core_agree() {
    // "Covered by amendment" must mean "the production path re-derives it";
    // the two entry points may not drift into separate behaviours.
    let topics = amended_topics();
    let events = amendment_events(&topics);
    let amendments = topics
        .iter()
        .enumerate()
        .map(|(index, topic)| RetainedAmendment {
            id: format!("amendment-{index}"),
            topic: String::from(topic.topic),
            rule: String::from(topic.rule),
        })
        .collect::<Vec<_>>();

    let solver = UniversalSolver::default();
    for topic in &topics {
        for (language, prompt) in topic.held_out {
            let production = solve_with_standing_requirements(&solver, prompt, &[], &events);
            let core = solve_with_amendment_records(&solver, prompt, &[], &amendments);
            assert_eq!(
                production.answer, core.answer,
                "[{}/{language}] production and replay must derive the same answer",
                topic.topic,
            );
            assert_eq!(production.intent, core.intent);
        }
    }
}
