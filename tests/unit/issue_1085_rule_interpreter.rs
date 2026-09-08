//! Issue #1085 (D1.3): specialized handlers as seed rules walked by one
//! interpreter, and the handlers that migrated into `data/seed/handler-rules.lino`.

use formal_ai::event_log::EventLog;
use formal_ai::rule_interpreter::{HandlerRules, handler_matches, rules};
use formal_ai::seed::{HANDLER_RULES_LINO, parse_lexicon_text};
use formal_ai::{FormalAiEngine, SymbolicAnswer};

/// The precedence names that are now data rather than Rust functions.
const MIGRATED_HANDLERS: [&str; 11] = [
    "conversation_control",
    "github_repository_traffic",
    "docs_method_explanation",
    "capabilities",
    "clarification",
    "punctuation_only_prompt",
    "ill_formed",
    "physical_action_question",
    "kupi_slona",
    "shell_refusal",
    "opinion_question",
];

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn the_embedded_rule_document_declares_every_migrated_handler() {
    let parsed = HandlerRules::parse(HANDLER_RULES_LINO).expect("embedded rules must parse");
    let names: Vec<&str> = parsed.handler_names().collect();
    assert_eq!(names, MIGRATED_HANDLERS);
    assert_eq!(parsed.rule_count(), 14);
    let precedence = formal_ai::seed::handler_precedence();
    for name in MIGRATED_HANDLERS {
        assert!(
            precedence.iter().any(|entry| entry == name),
            "{name} must keep its place in data/seed/handler-precedence.lino"
        );
        assert!(
            rules().handler(name).is_some(),
            "{name} must resolve at runtime"
        );
    }
}

/// D1's acceptance test: a rule for an intent no Rust knows routes in four
/// languages once its role surfaces and reply are links.
#[test]
fn an_injected_seed_rule_routes_an_unseen_intent_in_four_languages_without_rust() {
    const MEANINGS: &str = "meanings
  moon_phase_query
    defined-by inquiry
    role moon_phase_query
    lexeme en
      surface
        text \"phase of the moon\"
    lexeme ru
      surface
        text \"фаза луны\"
    lexeme hi
      surface
        text \"चंद्रमा की कला\"
    lexeme zh
      surface
        text 月相
";
    const RULES: &str = "handler_rules
  handler moon_phase
    rule moon_phase
      when
        role moon_phase_query raw
      respond moon_phase
        text en \"I cannot observe the sky, so I do not know tonight's moon phase.\"
        text ru \"Я не наблюдаю небо, поэтому не знаю сегодняшнюю фазу луны.\"
        text hi \"मैं आकाश नहीं देख सकता, इसलिए आज की चंद्रमा की कला नहीं जानता।\"
        text zh 我无法观测天空，所以不知道今晚的月相。
      link \"response:moon_phase\"
      confidence 1.0
";
    let lexicon = parse_lexicon_text(MEANINGS);
    let parsed = HandlerRules::parse(RULES).expect("injected rules must parse");
    let handler = parsed.handler("moon_phase").expect("handler must exist");
    let cases = [
        (
            "en",
            "Tell me the phase of the moon tonight",
            "I cannot observe the sky, so I do not know tonight's moon phase.",
        ),
        (
            "ru",
            "Какая сегодня фаза луны?",
            "Я не наблюдаю небо, поэтому не знаю сегодняшнюю фазу луны.",
        ),
        (
            "hi",
            "आज चंद्रमा की कला क्या है?",
            "मैं आकाश नहीं देख सकता, इसलिए आज की चंद्रमा की कला नहीं जानता।",
        ),
        (
            "zh",
            "今晚的月相是什么？",
            "我无法观测天空，所以不知道今晚的月相。",
        ),
    ];
    for (language, prompt, expected) in cases {
        let mut log = EventLog::default();
        let response = handler
            .run_with(&lexicon, prompt, &prompt.to_lowercase(), &mut log)
            .unwrap_or_else(|| panic!("{language}: the injected rule must answer"));
        assert_eq!(response.intent, "moon_phase", "{language}");
        assert_eq!(response.answer, expected, "{language}");
        assert!(
            log.events()
                .iter()
                .any(|event| event.kind == "rule_interpreter:rule" && event.payload == "moon_phase"),
            "{language}: the walked rule must be logged"
        );
    }
    let mut log = EventLog::default();
    assert!(
        handler
            .run_with(
                &lexicon,
                "What is the capital of France?",
                "what is the capital of france?",
                &mut log
            )
            .is_none(),
        "a prompt without the role surface must fall through"
    );
}

/// Held-out paraphrases for the migrated handlers, through the whole engine.
#[test]
fn migrated_handlers_answer_held_out_paraphrases_in_english_russian_hindi_and_chinese() {
    let cases = [
        ("en", "Hey, buy an elephant!", "kupi_slona"),
        ("ru", "Ну купи слона, пожалуйста", "kupi_slona"),
        ("hi", "चलो हाथी खरीदो ना", "kupi_slona"),
        ("zh", "快买大象吧", "kupi_slona"),
        (
            "en",
            "Sorry, I don't understand this at all",
            "clarification",
        ),
        ("ru", "Я не понимаю тебя", "clarification"),
        ("hi", "मुझे समझ नहीं आया", "clarification"),
        ("en", "So tell me, what can you do for me?", "capabilities"),
        ("ru", "А что ты умеешь делать?", "capabilities"),
        ("hi", "और क्या कर सकते हो?", "capabilities"),
        ("zh", "你还能做什么呢？", "capabilities"),
        (
            "en",
            "Please don't use `foo` in your replies",
            "conversation_preference",
        ),
        (
            "ru",
            "Не используй `bar` в ответах",
            "conversation_preference",
        ),
        ("hi", "`baz` का इस्तेमाल मत करो", "conversation_preference"),
        ("zh", "不要使用 `qux`", "conversation_preference"),
        (
            "en",
            "I didn't ask to update that file",
            "action_correction",
        ),
        (
            "en",
            "how does pandas DataFrame.join work?",
            "docs_method_explanation",
        ),
        (
            "ru",
            "объясни как работает pandas DataFrame.join",
            "docs_method_explanation",
        ),
        (
            "hi",
            "pandas DataFrame.join कैसे काम करता है?",
            "docs_method_explanation",
        ),
        (
            "zh",
            "pandas DataFrame.join 如何工作？",
            "docs_method_explanation",
        ),
    ];
    for (language, prompt, intent) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, intent,
            "{language}: {prompt:?} answered {:?}",
            response.answer
        );
    }
}

#[test]
fn english_only_policy_rules_keep_their_intents_and_wording() {
    let opinion = answer("What do you think about pineapple on pizza?");
    assert_eq!(opinion.intent, "opinion_question");
    assert_eq!(
        opinion.answer,
        "I am a deterministic symbolic AI. I do not hold opinions, beliefs, or feelings — every answer I give is derived from an explicit Links Notation rule. If you are looking for factual information on this topic, try asking \"what is <topic>\" and I will look it up in my knowledge base."
    );
    let refusal = answer("Please run `rm -rf /tmp/cache` on my behalf");
    assert_eq!(refusal.intent, "policy_bounded_autonomy");
    assert_eq!(
        refusal.answer,
        "I can only respond with a chat reply. Running shell commands on your behalf is not allowed without explicit agent mode opt-in, and even then only inside an isolated sandbox."
    );
    let punctuation = answer("???");
    assert_eq!(punctuation.intent, "clarification");
    assert_eq!(
        punctuation.answer,
        "I received only punctuation (`???`). What would you like me to do next?"
    );
    let physical = answer("So, did you suck at it yesterday?");
    assert_eq!(physical.intent, "physical_action_question");
    assert_eq!(physical.answer, "No. I do not have a physical body.");
}

#[test]
fn conversation_control_recognition_is_the_rule_set_the_planner_consults() {
    assert!(handler_matches(
        "conversation_control",
        "Please don't use `foo` in your replies"
    ));
    assert!(handler_matches(
        "conversation_control",
        "I didn't ask to update that file"
    ));
    assert!(!handler_matches(
        "conversation_control",
        "Add wikiquote as a search provider"
    ));
}
