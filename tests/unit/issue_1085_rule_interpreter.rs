//! Issue #1085 (D1.3): specialized handlers as seed rules walked by one
//! interpreter, and the handlers that migrated into `data/seed/handler-rules.lino`.

use formal_ai::event_log::EventLog;
use formal_ai::rule_interpreter::{
    ConditionSource, ConditionSurface, HandlerRules, handler_matches, rules,
};
use formal_ai::seed::{HANDLER_RULES_LINO, Lexicon, parse_lexicon_text};
use formal_ai::{FormalAiEngine, SymbolicAnswer};

/// A condition backend over an injected lexicon: the test-side stand-in for the
/// `SeedTables` struct plan 09 leaf 40 deleted from `src/`. Rules are probed
/// against fixture meanings without touching the boot projection.
struct FixtureTables<'a> {
    lexicon: &'a Lexicon,
}

impl ConditionSource for FixtureTables<'_> {
    fn role_surfaces(&self, role: &str) -> Vec<ConditionSurface> {
        self.lexicon
            .meanings
            .iter()
            .filter(|meaning| meaning.has_role(role))
            .flat_map(|meaning| meaning.lexemes.iter())
            .flat_map(|lexeme| {
                lexeme.words.iter().map(|form| ConditionSurface {
                    text: form.text.clone(),
                    language: lexeme.language.clone(),
                    slot: form.slot(),
                })
            })
            .collect()
    }

    fn exact_route_surfaces(&self, _slug: &str) -> Vec<String> {
        Vec::new()
    }
}

/// The precedence names that are now data rather than Rust functions.
const MIGRATED_HANDLERS: [&str; 12] = [
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
    "agentic_continuation",
];

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn the_embedded_rule_document_declares_every_migrated_handler() {
    let parsed = HandlerRules::parse(HANDLER_RULES_LINO).expect("embedded rules must parse");
    let names: Vec<&str> = parsed.handler_names().collect();
    assert_eq!(names, MIGRATED_HANDLERS);
    assert_eq!(parsed.rule_count(), 15);
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
        let tables = FixtureTables { lexicon: &lexicon };
        let response = handler
            .run_with_source(&tables, prompt, &prompt.to_lowercase(), &mut log)
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
    let tables = FixtureTables { lexicon: &lexicon };
    assert!(
        handler
            .run_with_source(
                &tables,
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
            "समझाओ pandas DataFrame.join कैसे काम करता है",
            "docs_method_explanation",
        ),
        (
            "zh",
            "解释 pandas DataFrame.join 如何工作",
            "docs_method_explanation",
        ),
    ];
    // The Russian, Hindi and Chinese paraphrases open with the seeded explain
    // verb. The bare interrogatives in those three languages (`как работает …?`,
    // `… कैसे काम करता है?`, `… 如何工作？`) used to be claimed by the web-search
    // handler and answered with its offline-fetch notice while the same
    // question in English was not; that asymmetry was issue #1101 and is fixed.
    // `tests/unit/issue_1101_documentation_question_parity.rs` asserts the bare
    // forms directly, so this case no longer needs the explain verb to stand in
    // for them.
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

/// Issue #1138 B9, plan 09 leaf 9: the two grammar primitives the promotion
/// migration needs, and the only ones it adds.
///
/// `of padded` is the space-padded subject `role_padded` built privately for
/// one condition; as a subject, every condition can ask for it, which is what
/// lets `contains("в ")` in `src/intent_formalization/prompt_relevants.rs`
/// become `role temporal_preposition of padded` — seed data in every language
/// rather than one Russian preposition compiled into Rust. `shape` states a
/// structural property of the input that carries no natural language, so
/// `contains(':')` becomes `shape time_separator` instead of hiding a
/// structural test inside a lexical one.
const SHAPE_RULES: &str = "handler_rules
  handler padded_probe
    rule padded_probe
      when
        substring \" qed \" of padded
      respond padded_probe
        text en \"padded\"
  handler normalized_probe
    rule normalized_probe
      when
        substring \" qed \" of normalized
      respond normalized_probe
        text en \"normalized\"
  handler digit_probe
    rule digit_probe
      when
        shape digit of trimmed
      respond digit_probe
        text en \"digit\"
  handler time_probe
    rule time_probe
      when
        shape time_separator of trimmed
      respond time_probe
        text en \"time\"
  handler url_probe
    rule url_probe
      when
        shape url of prompt
      respond url_probe
        text en \"url\"
  handler path_probe
    rule path_probe
      when
        shape path of prompt
      respond path_probe
        text en \"path\"
  handler quoted_probe
    rule quoted_probe
      when
        shape quoted of prompt
      respond quoted_probe
        text en \"quoted\"
";

fn probe(parsed: &HandlerRules, handler: &str, prompt: &str) -> bool {
    let lexicon = parse_lexicon_text("meanings\n");
    let tables = FixtureTables { lexicon: &lexicon };
    parsed
        .handler(handler)
        .unwrap_or_else(|| panic!("{handler} must exist"))
        .matches_with_source(&tables, prompt, &prompt.to_lowercase())
}

#[test]
fn of_padded_matches_a_boundary_surface_at_the_edges_of_the_input() {
    let parsed = HandlerRules::parse(SHAPE_RULES).expect("the probe rules must parse");

    // The whole prompt is the word: padded sees " qed ", normalized sees "qed".
    assert!(
        probe(&parsed, "padded_probe", "qed"),
        "`of padded` must match a boundary surface at the start and end of the input"
    );
    assert!(
        !probe(&parsed, "normalized_probe", "qed"),
        "without padding the same condition cannot match at the edges, which is \
         the reason the subject exists"
    );

    // Inside the sentence both agree, so padding widens and never narrows.
    assert!(probe(&parsed, "padded_probe", "show the qed line"));
    assert!(probe(&parsed, "normalized_probe", "show the qed line"));

    // A longer word that merely contains the surface is still not a match.
    assert!(
        !probe(&parsed, "padded_probe", "qedx"),
        "padding adds a boundary; it does not remove one"
    );
}

#[test]
fn every_shape_is_a_structural_property_and_carries_no_natural_language() {
    let parsed = HandlerRules::parse(SHAPE_RULES).expect("the probe rules must parse");

    // digit — any Unicode decimal digit, in any script.
    assert!(probe(&parsed, "digit_probe", "meet at 7"));
    assert!(probe(&parsed, "digit_probe", "встреча в ７"));
    assert!(!probe(&parsed, "digit_probe", "meet at noon"));

    // time_separator — the literal replacement for `contains(':')`.
    assert!(probe(&parsed, "time_probe", "20:00"));
    assert!(probe(&parsed, "time_probe", "20：00"));
    assert!(!probe(&parsed, "time_probe", "2000"));

    // url — an absolute web address or a bare host.
    assert!(probe(&parsed, "url_probe", "open https://example.org/docs"));
    assert!(probe(&parsed, "url_probe", "open www.example.org"));
    assert!(!probe(&parsed, "url_probe", "open the docs"));

    // path — a separator-carrying token that is not a URL.
    assert!(probe(&parsed, "path_probe", "read src/lib.rs"));
    assert!(probe(&parsed, "path_probe", "read C:\\\\src\\\\lib.rs"));
    assert!(
        !probe(&parsed, "path_probe", "open https://example.org/docs"),
        "a URL is a URL and not also a path; the two shapes must stay distinguishable"
    );
    assert!(!probe(&parsed, "path_probe", "read the file"));

    // quoted — a matched delimiter pair, in any registered script.
    assert!(probe(&parsed, "quoted_probe", "find \"the qed line\""));
    assert!(probe(&parsed, "quoted_probe", "найди «строку»"));
    assert!(probe(&parsed, "quoted_probe", "找到「那一行」"));
    assert!(
        !probe(&parsed, "quoted_probe", "find the qed line"),
        "an unquoted span is not quoted, and one lone mark does not pair"
    );
    assert!(!probe(&parsed, "quoted_probe", "find \"the qed line"));
}

#[test]
fn an_unknown_shape_or_subject_is_refused_rather_than_ignored() {
    let unknown_shape = "handler_rules\n  handler probe\n    rule probe\n      when\n        \
                         shape weather of trimmed\n      respond probe\n        text en \"x\"\n";
    let error = HandlerRules::parse(unknown_shape).expect_err("an unknown shape must not parse");
    assert!(error.contains("unknown_shape"), "{error}");

    let unknown_subject = "handler_rules\n  handler probe\n    rule probe\n      when\n        \
                           shape digit of sideways\n      respond probe\n        text en \"x\"\n";
    let error =
        HandlerRules::parse(unknown_subject).expect_err("an unknown subject must not parse");
    assert!(error.contains("unknown_subject"), "{error}");
}
