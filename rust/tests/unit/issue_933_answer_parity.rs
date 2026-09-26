//! Every language keeps its whole conversational answer (issues #123/#933).
//!
//! Issue #933 records the answer each wording produces next to the wording, so
//! the corpus reads as documentation. Recording them surfaced sixteen prompts
//! that answered with an empty string: Chinese `谢谢`, Chinese `你好吗` and
//! Hindi `धन्यवाद`. The seed holds the right text for all of them
//! (`cargo run --example issue_933_response_probe`), and the routing is right
//! too -- the answer is lost afterwards, in the question-necessity pass.
//!
//! Two seed-blind defects combined there:
//!
//! 1. a sentence boundary was only recognized when whitespace followed the
//!    stop, and the Devanagari danda was not a stop at all, so a whole Chinese
//!    or Hindi answer counted as one question; and
//! 2. the requirement cues covered the English follow-ups only, so the seed's
//!    own Russian, Hindi and Chinese follow-ups were refused as factual
//!    unknowns and cut out -- together with the statement in front of them.
//!
//! The English answer was never touched, which is exactly the language parity
//! issue #123 asked for: "All features should be supported in all 4 languages".

use formal_ai::event_log::EventLog;
use formal_ai::question_necessity::{QuestionClass, classify_question, enforce_questions};
use formal_ai::{FormalAiEngine, seed};

const LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

/// The conversational cases of the issue #933 corpus that answer from a seed
/// response, plus the clarification fallback they share a voice with.
/// `calculation` is computed rather than looked up, so it has no seed text.
const CONVERSATIONAL_INTENTS: [&str; 10] = [
    "greeting",
    "wellbeing",
    "farewell",
    "courtesy_response",
    "identity",
    "capabilities",
    "test_status",
    "assistant_free_time",
    "assistant_name",
    "clarification",
];

#[test]
fn issue_933_conversational_seed_answers_survive_the_question_pass() {
    for intent in CONVERSATIONAL_INTENTS {
        for language in LANGUAGES {
            let text = seed::localized_response(intent, language)
                .unwrap_or_else(|| panic!("seed response for {intent}/{language}"));
            let enforced = enforce_questions(&text, &mut EventLog::new());
            assert_eq!(
                enforced, text,
                "{intent}/{language}: the question pass rewrote a seed-authored answer"
            );
        }
    }
}

#[test]
fn issue_933_seed_follow_up_questions_are_requirements_in_every_language() {
    for (language, question) in [
        ("en", "What would you like to do next?"),
        ("ru", "Готов помочь — с чего начнём?"),
        ("ru", "Есть ли еще тема, с которой помочь?"),
        ("ru", "Продолжим с другим вопросом?"),
        ("hi", "अब आप क्या करना चाहेंगे?"),
        ("hi", "मैं कैसे मदद कर सकता हूँ?"),
        ("hi", "क्या किसी और चीज़ में मदद चाहिए?"),
        ("zh", "接下来您想做什么?"),
        ("zh", "您想讨论别的事情吗?"),
        ("zh", "我能帮您做些什么?"),
    ] {
        assert_eq!(
            classify_question(question).class,
            QuestionClass::Requirement,
            "{language}: {question} asks the user for a requirement, not for a fact"
        );
    }
}

#[test]
fn issue_933_a_refused_question_leaves_the_statement_in_front_of_it() {
    // "Which one source or missing fact" is the seed's own factual cue, so
    // these questions are refused by policy. Chinese runs its sentences
    // together and Hindi ends them with a danda; before issue #933 neither
    // counted as a sentence boundary, so the refusal took the statement with
    // the question and the answer came back empty.
    for (language, body, statement) in [
        (
            "zh",
            "很高兴听到。应该使用哪一个来源或缺失事实?",
            "很高兴听到。",
        ),
        (
            "hi",
            "यह सुनकर अच्छा लगा। कौन सा एक source या missing fact इस्तेमाल करूँ?",
            "यह सुनकर अच्छा लगा।",
        ),
        (
            "zh",
            "我很好,谢谢关心!应该使用哪一个来源或缺失事实?",
            "我很好,谢谢关心!",
        ),
    ] {
        let enforced = enforce_questions(body, &mut EventLog::new());
        assert_eq!(
            enforced, statement,
            "{language}: refusing the question should keep the statement"
        );
    }
}

#[test]
fn issue_933_a_question_quoted_as_an_example_is_not_a_question_the_answer_asks() {
    // The capability listing and the clarification fallback both show the user
    // what a question looks like. Chinese quotes with corner brackets and every
    // language uses parentheses, neither of which used to shield the example,
    // so the bullet that offered it was deleted.
    for (language, body) in [
        (
            "en",
            "I can answer concept lookups (what is X?), arithmetic, and greetings.",
        ),
        ("zh", "- **概念查找**：解释术语，例如「什么是维基百科？」"),
        ("zh", "我能回答概念查询（什么是X？）、算术以及问候。"),
    ] {
        assert_eq!(
            enforce_questions(body, &mut EventLog::new()),
            body,
            "{language}: an example question is part of the answer, not a question"
        );
    }
}

#[test]
fn issue_933_the_spanish_seed_answers_survive_the_question_pass_too() {
    // Spanish is registered `status partial` in `data/seed/languages.lino`, so
    // it is outside the four languages issue #123 set the floor over and has no
    // corpus records. The question pass runs over its answers all the same, and
    // both Spanish conversational responses end in a question -- the exact
    // shape that emptied the Chinese ones -- so they are pinned here.
    for (intent, expected) in [
        ("greeting", "¡Hola! ¿Cómo puedo ayudarte?"),
        (
            "identity",
            "Soy formal-ai, una implementación de IA simbólica determinista que responde mediante reglas locales en Links Notation.",
        ),
        (
            "language_gap",
            "Este significado aún no está cubierto en español; no se sustituyó por una respuesta en inglés.",
        ),
    ] {
        let text = seed::localized_response(intent, "es")
            .unwrap_or_else(|| panic!("Spanish seed response for {intent}"));
        assert_eq!(text, expected, "the Spanish seed text for {intent}");
        assert_eq!(
            enforce_questions(&text, &mut EventLog::new()),
            expected,
            "es/{intent}: the question pass rewrote a seed-authored Spanish answer"
        );
    }
}

#[test]
fn issue_933_thanks_and_wellbeing_answer_in_full_in_every_language() {
    for (prompt, expected) in [
        ("thanks", "Glad to hear it. What would you like to do next?"),
        ("спасибо", "Рад это слышать. Что вы хотите сделать дальше?"),
        ("धन्यवाद", "यह सुनकर अच्छा लगा। अब आप क्या करना चाहेंगे?"),
        ("谢谢", "很高兴听到。接下来您想做什么?"),
        (
            "how are you?",
            "I'm doing great, thanks for asking! I'm ready to help — what would you like to do?",
        ),
        (
            "как дела?",
            "У меня всё хорошо, спасибо, что спросили! Готов помочь — с чего начнём?",
        ),
        (
            "आप कैसे हैं?",
            "मैं बढ़िया हूँ, पूछने के लिए धन्यवाद! मैं मदद के लिए तैयार हूँ — आप क्या करना चाहेंगे?",
        ),
        ("你好吗?", "我很好,谢谢关心!我随时可以帮忙,您想做点什么?"),
    ] {
        assert_eq!(
            FormalAiEngine.answer(prompt).answer,
            expected,
            "prompt {prompt:?} should answer with the whole seed response"
        );
    }
}
