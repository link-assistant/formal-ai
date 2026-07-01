//! Generalized response-language follow-up tests (issue #556).
//!
//! The response-language follow-up must replay the *whole class* of prior
//! requests in a newly requested language — not just repository lookups. A
//! bare "answer me in <language>" turn replays the previous user request
//! through the entire solver with the target language forced onto every
//! localizable handler, so capabilities, identity, and project answers all
//! re-render in the requested language.
//!
//! These cases mirror the JS worker harness in
//! `experiments/js_worker_followup_harness.mjs`, proving Rust ↔ JS parity for
//! the generalization across intent families in every seeded language
//! (`en`/`ru`/`hi`/`zh`).

use formal_ai::{ConversationTurn, UniversalSolver};

/// A prior request, its English answer, and the localized fragment expected
/// when that request is replayed in each seeded language.
struct GeneralizedCase {
    prior_prompt: &'static str,
    prior_answer: &'static str,
    expected_intent: &'static str,
    /// (follow-up prompt, language slug, target-language fragment).
    followups: &'static [(&'static str, &'static str, &'static str)],
}

fn assert_generalized(case: &GeneralizedCase) {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user(case.prior_prompt),
        ConversationTurn::assistant(case.prior_answer),
    ];

    for (follow_up, language_slug, expected_fragment) in case.followups {
        let response = solver.solve_with_history(follow_up, &history);
        assert_eq!(
            response.intent, case.expected_intent,
            "follow-up {follow_up:?} should replay the prior {} answer, got {} -> {}",
            case.expected_intent, response.intent, response.answer,
        );
        assert!(
            response.answer.contains(expected_fragment),
            "follow-up {follow_up:?} should render the {language_slug} fragment \
             {expected_fragment:?}, got {}",
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .contains(&format!("language_to:{language_slug}")),
            "follow-up {follow_up:?} should record language_to:{language_slug}, got {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .contains(&format!("response_language_followup:target:{language_slug}")),
            "follow-up {follow_up:?} should carry response-language-followup provenance, got {:?}",
            response.evidence_links,
        );
        assert!(
            response.evidence_links.contains(&format!(
                "response_language_followup:handler:{}",
                case.expected_intent
            )),
            "follow-up {follow_up:?} should name the replayed handler, got {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn capabilities_answer_reanswers_in_every_seeded_language() {
    assert_generalized(&GeneralizedCase {
        prior_prompt: "what can you do",
        prior_answer: "I am formal-ai, a deterministic symbolic AI. Here is what I can do: \
                       greetings, Hello World programs, web search, concept lookups, \
                       arithmetic, translation, memory, and behavior rules.",
        expected_intent: "capabilities",
        followups: &[
            (
                "я не понимаю по английски, напиши по русски",
                "ru",
                "Вот что я умею",
            ),
            (
                "मुझे अंग्रेजी समझ नहीं आती, हिंदी में लिखो",
                "hi",
                "मैं यह कर सकता हूँ",
            ),
            ("我不懂英语，用中文", "zh", "以下是我的功能"),
        ],
    });
}

#[test]
fn identity_answer_reanswers_in_every_seeded_language() {
    assert_generalized(&GeneralizedCase {
        prior_prompt: "what are you",
        prior_answer: "I am formal-ai, a deterministic symbolic AI implementation that \
                       answers from local Links Notation rules and OpenAI-compatible API \
                       shapes. I do not perform neural inference in this demo.",
        expected_intent: "identity",
        followups: &[
            (
                "я не понимаю по английски, напиши по русски",
                "ru",
                "детерминированный символьный ИИ",
            ),
            (
                "मुझे अंग्रेजी समझ नहीं आती, हिंदी में लिखो",
                "hi",
                "नियतात्मक प्रतीकात्मक AI",
            ),
            ("我不懂英语，用中文", "zh", "确定性的符号化 AI"),
        ],
    });
}

/// The generalization must be reversible: an answer produced in one language
/// replays back into English when the user asks for English, closing the
/// round-trip (issue #526 spirit) across intent families.
#[test]
fn capabilities_follow_up_returns_to_english_on_request() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("что ты умеешь"),
        ConversationTurn::assistant(
            "Я formal-ai — детерминированный символьный ИИ. Вот что я умею: приветствия, \
             Hello World, веб-поиск, поиск понятий и арифметика.",
        ),
    ];

    let response =
        solver.solve_with_history("I do not understand Russian, write in English", &history);

    assert_eq!(
        response.intent, "capabilities",
        "English follow-up should replay the capabilities answer, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("Here is what I can do")
            || response.answer.contains("what I can do"),
        "English retarget should render capabilities in English, got {}",
        response.answer,
    );
    assert!(
        response.evidence_links.contains(&"language_to:en".to_owned()),
        "English retarget should record language_to:en, got {:?}",
        response.evidence_links,
    );
}
