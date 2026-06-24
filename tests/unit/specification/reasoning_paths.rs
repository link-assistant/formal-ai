//! Reasoning-path tests (R85–R88).
//!
//! These tests pin down the universal solver's new specialized handlers and
//! prove that each interface (library and the convenience module-level
//! entry points) routes through the same loop, without any hardcoded
//! demo-style responses. Every test exercises the event-log projection so a
//! regression to memoized answers would break here first.

use formal_ai::{
    solve, solve_with_history, ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver,
};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn has_evidence(response: &SymbolicAnswer, expected: &str) -> bool {
    response
        .evidence_links
        .iter()
        .any(|link| link.starts_with(expected))
}

// ---------------------------------------------------------------------------
// R85: arithmetic — symbols, words, parentheses, errors.
// ---------------------------------------------------------------------------

#[test]
fn arithmetic_handles_basic_addition() {
    let response = answer("What is 2 + 2?");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains('4'));
    assert!((response.confidence - 1.0).abs() < f32::EPSILON);
}

#[test]
fn arithmetic_handles_parentheses_and_precedence() {
    let response = answer("Calculate 7 * (3 + 4)");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("49"));
}

#[test]
fn arithmetic_handles_word_operators() {
    let response = answer("What is 10 plus 20 times 3?");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("70"));
}

#[test]
fn arithmetic_handles_division_remainder() {
    let response = answer("Compute 100 - 25 % 7");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("96"));
}

#[test]
fn arithmetic_handles_decimals() {
    let response = answer("How much is 1.5 + 2.5?");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains('4'));
}

#[test]
fn arithmetic_reports_division_by_zero_without_panicking() {
    let response = answer("What is 5 / 0?");
    assert_eq!(response.intent, "calculation_error");
    assert!(response.answer.to_lowercase().contains("division by zero"));
    assert!(response.confidence < 1.0);
}

#[test]
fn arithmetic_records_calculation_event_in_evidence_log() {
    let response = answer("What is 6 * 7?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("calculation")),
        "evidence links should include the calculation event so the answer is a \
         projection of the log, not a memoized constant: {:?}",
        response.evidence_links,
    );
}

#[test]
fn selected_specialized_handler_is_recorded_as_a_meta_method() {
    let response = answer("What is 6 * 7?");
    assert_eq!(response.intent, "calculation");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("method:")),
        "registry-backed dispatch should record the selected method event: {:?}",
        response.evidence_links,
    );
    assert!(
        response.links_notation.contains("method \"arithmetic\""),
        "the method event should name the selected registry method:\n{}",
        response.links_notation,
    );
}

#[test]
fn arithmetic_handles_large_integer_multiplication_without_overflow() {
    // Multiplying large integers should yield an exact integer result,
    // not an overflow error. Regression for issue #55.
    // 9 pairs is enough to exceed f64's range (overflow happens at pair 8).
    let expr = "123123980921093128 * 2348023048230429324 * \
                123123980921093128 * 2348023048230429324 * \
                123123980921093128 * 2348023048230429324 * \
                123123980921093128 * 2348023048230429324 * \
                123123980921093128 * 2348023048230429324 * \
                123123980921093128 * 2348023048230429324 * \
                123123980921093128 * 2348023048230429324 * \
                123123980921093128 * 2348023048230429324 * \
                123123980921093128 * 2348023048230429324";
    let response = answer(expr);
    assert_eq!(
        response.intent, "calculation",
        "large integer multiplication must succeed, not overflow: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("overflow"),
        "answer must not mention overflow: {}",
        response.answer,
    );
}

#[test]
fn arithmetic_never_fires_on_plain_greetings() {
    let response = answer("Hi");
    assert_eq!(response.intent, "greeting");
}

#[test]
fn calendar_reasoning_answers_russian_weekday_successor() {
    let response = answer("какой день недели наступает после вторника");
    assert_eq!(response.intent, "calendar_weekday_relation");
    assert!(
        response.answer.to_lowercase().contains("среда"),
        "weekday successor should be computed as Wednesday, got: {}",
        response.answer,
    );
    assert!(
        has_evidence(&response, "calendar:operation:next"),
        "calendar reasoning must expose the successor operation in evidence: {:?}",
        response.evidence_links,
    );
}

#[test]
fn calendar_reasoning_answers_current_day_questions_across_supported_languages() {
    let cases = [
        ("What day is today?", "Today is", "language:en"),
        ("Какой сегодня день?", "Сегодня", "language:ru"),
        ("आज कौन सा दिन है?", "आज", "language:hi"),
        ("今天是星期几?", "今天", "language:zh"),
    ];

    for (prompt, expected_fragment, language_tag) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "calendar_current_day",
            "today question {prompt:?} should use calendar reasoning, got: {}",
            response.answer,
        );
        assert!(
            response.answer.contains(expected_fragment),
            "current-day answer for {prompt:?} should be localized, got: {}",
            response.answer,
        );
        assert!(
            has_evidence(&response, "calendar:today"),
            "current-day reasoning must record the resolved date for {prompt:?}: {:?}",
            response.evidence_links,
        );
        assert!(
            has_evidence(&response, "calendar:weekday"),
            "current-day reasoning must record the resolved weekday for {prompt:?}: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == language_tag),
            "current-day reasoning must record {language_tag} for {prompt:?}: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn calendar_reasoning_answers_weekday_predecessor_and_successor_variations() {
    let cases = [
        ("What day of the week comes after Tuesday?", "Wednesday"),
        ("What day comes before Monday?", "Sunday"),
        ("какой день недели перед средой", "вторник"),
        ("следующий день после воскресенья", "понедельник"),
    ];

    for (prompt, expected) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "calendar_weekday_relation",
            "prompt {prompt:?} should route to calendar reasoning, got {}",
            response.intent,
        );
        assert!(
            response
                .answer
                .to_lowercase()
                .contains(&expected.to_lowercase()),
            "prompt {prompt:?} should mention {expected:?}, got: {}",
            response.answer,
        );
    }
}

#[test]
fn calendar_reasoning_answers_weekday_relations_in_hindi_and_chinese() {
    // Issue #386: the weekday names, directions, and day references now live as
    // self-describing meanings in every supported language, so calendar
    // reasoning works in Hindi and Chinese too — not just English and Russian.
    // Each answer must also be localized in the asking language.
    let cases = [
        ("सोमवार के बाद कौन सा दिन आता है", "मंगलवार"), // after Monday → Tuesday
        ("सोमवार से पहले कौन सा दिन आता है", "रविवार"), // before Monday → Sunday
        ("星期一之后是星期几", "星期二"),            // after Monday → Tuesday
        ("星期三之前是星期几", "星期二"),            // before Wednesday → Tuesday
    ];

    for (prompt, expected) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "calendar_weekday_relation",
            "prompt {prompt:?} should route to calendar reasoning, got {}",
            response.intent,
        );
        assert!(
            response.answer.contains(expected),
            "prompt {prompt:?} should mention {expected:?} (localized), got: {}",
            response.answer,
        );
    }
}

// ---------------------------------------------------------------------------
// R86: concept lookup against the offline seed.
// ---------------------------------------------------------------------------

#[test]
fn concept_lookup_answers_what_is_wikipedia() {
    let response = answer("What is Wikipedia?");
    assert_eq!(response.intent, "concept_lookup");
    assert!(response.answer.to_lowercase().contains("encyclopedia"));
    assert!(response
        .answer
        .contains("https://en.wikipedia.org/wiki/Wikipedia"));
}

#[test]
fn concept_lookup_handles_tell_me_about_links_notation() {
    let response = answer("Tell me about Links Notation");
    assert_eq!(response.intent, "concept_lookup");
    assert!(response.answer.contains("Links Notation"));
    assert!(response.answer.to_lowercase().contains("indentation"));
}

#[test]
fn concept_lookup_handles_what_does_x_mean() {
    let response = answer("What does Wikidata mean?");
    assert_eq!(response.intent, "concept_lookup");
    assert!(response.answer.contains("Wikidata"));
}

#[test]
fn concept_lookup_includes_source_citation() {
    let response = answer("What is WebAssembly?");
    assert!(response.answer.contains("Source:"));
}

#[test]
fn concept_lookup_does_not_fire_for_identity_questions() {
    let response = answer("What is formal-ai?");
    assert_eq!(
        response.intent, "identity",
        "identity rule must win over concept lookup for formal-ai questions"
    );
}

#[test]
fn concept_lookup_records_concept_event_in_evidence_log() {
    let response = answer("What is Rust?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("concept_lookup")),
        "evidence links should include the concept_lookup event: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// R87: multi-turn conversation memory via solve_with_history.
// ---------------------------------------------------------------------------

#[test]
fn solve_with_history_recalls_name_across_turns() {
    let history = [ConversationTurn::user("Hi, my name is Ada Lovelace.")];
    let response = solve_with_history("What is my name?", &history);
    assert_eq!(response.intent, "recall_name");
    assert!(response.answer.contains("Ada"));
}

#[test]
fn solve_with_history_recalls_last_question() {
    let history = [
        ConversationTurn::user("What is 2 + 2?"),
        ConversationTurn::assistant("2 + 2 = 4"),
    ];
    let response = solve_with_history("What was my previous question?", &history);
    assert_eq!(response.intent, "recall_last_question");
    assert!(response.answer.contains("2 + 2"));
}

#[test]
fn solve_with_history_summarizes_conversation() {
    let history = [
        ConversationTurn::user("Hi"),
        ConversationTurn::assistant("Hi, how may I help you?"),
        ConversationTurn::user("What is 2 + 2?"),
        ConversationTurn::assistant("2 + 2 = 4"),
    ];
    let response = solve_with_history("Summarize the conversation so far.", &history);
    assert_eq!(response.intent, "summarize_conversation");
    assert!(response.answer.contains("Hi"));
    assert!(response.answer.contains("2 + 2"));
}

#[test]
fn solve_with_history_summarizes_conversation_in_russian() {
    let history = [
        ConversationTurn::user("Что такое яблоко?"),
        ConversationTurn::assistant("Яблоко: Я́блоко — сочный плод яблони."),
    ];
    let response = solve_with_history("О чём мы разговаривали?", &history);
    assert_eq!(response.intent, "summarize_conversation");
    assert!(response.answer.contains("яблоко"));
}

#[test]
fn solve_with_history_summarizes_conversation_rezyume_besedy() {
    let history = [
        ConversationTurn::user("Привет"),
        ConversationTurn::assistant("Здравствуйте! Чем могу помочь?"),
    ];
    let response = solve_with_history("Резюме беседы", &history);
    assert_eq!(response.intent, "summarize_conversation");
    assert!(response.answer.contains("Привет"));
}

#[test]
fn solve_with_history_summarizes_conversation_single_word_summarize() {
    let history = [
        ConversationTurn::user("Hi"),
        ConversationTurn::assistant("Hi, how may I help you?"),
    ];
    let response = solve_with_history("Summarize", &history);
    assert_eq!(response.intent, "summarize_conversation");
    assert!(response.answer.contains("Hi"));
}

#[test]
fn solve_with_history_summarizes_conversation_in_chinese() {
    let history = [
        ConversationTurn::user("你好"),
        ConversationTurn::assistant("你好!请问有什么可以帮您的?"),
    ];
    let response = solve_with_history("总结", &history);
    assert_eq!(response.intent, "summarize_conversation");
    assert!(response.answer.contains("你好"));
}

#[test]
fn solve_with_history_falls_through_for_unrelated_prompts() {
    let history = [ConversationTurn::user("My name is Ada.")];
    let response = solve_with_history("Hi", &history);
    assert_eq!(response.intent, "greeting");
}

#[test]
fn solve_without_history_matches_legacy_entry_point() {
    let a = solve("Hi");
    let b = FormalAiEngine.answer("Hi");
    assert_eq!(a, b);
}

#[test]
fn prior_turns_appear_in_evidence_log() {
    let history = [ConversationTurn::user("My name is Ada.")];
    let response = UniversalSolver::default().solve_with_history("Hi", &history);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("prior_turn:user")),
        "prior turns must be recorded as events so memory recall is a projection \
         of the append-only log: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// R88: JavaScript execution — explicit declaration, no silent failure.
// ---------------------------------------------------------------------------

#[test]
fn javascript_request_returns_explicit_unavailability() {
    let prompt = "Please run this javascript:\n```js\nconsole.log(1 + 2);\n```";
    let response = answer(prompt);
    assert_eq!(response.intent, "tool_call_refused");
    assert!(response.answer.contains("Execution status: refused"));
    assert!(response.answer.contains("agent mode"));
    assert!(response.answer.contains("console.log(1 + 2);"));
}

#[test]
fn javascript_request_records_execution_status_event() {
    let prompt = "Please execute this javascript:\n```js\nlet x = 5;\n```";
    let response = answer(prompt);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("execution_status")),
        "the JS handler must emit an execution_status event so refusal is \
         auditable: {:?}",
        response.evidence_links,
    );
}

#[test]
fn javascript_handler_does_not_intercept_unrelated_code_blocks() {
    // No "run this" cue, so the handler must not steal the prompt from the
    // generic algorithm/code-fence flow.
    let prompt = "Here is some javascript:\n```js\nconsole.log(1);\n```";
    let response = answer(prompt);
    assert_ne!(response.intent, "javascript_execution_unavailable");
}

// ---------------------------------------------------------------------------
// Issue #52: "how it works?" follow-up after a Wikipedia lookup must not
// return intent: unknown.
// ---------------------------------------------------------------------------

#[test]
fn how_it_works_bare_is_not_unknown() {
    // The simplest bare-form follow-up must never resolve to `unknown`.
    let response = answer("how it works?");
    assert_ne!(
        response.intent, "unknown",
        "\"how it works?\" must not return unknown; got intent={}, answer={}",
        response.intent, response.answer,
    );
}

#[test]
fn how_does_it_work_is_not_unknown() {
    let response = answer("how does it work?");
    assert_ne!(
        response.intent, "unknown",
        "\"how does it work?\" must not return unknown; got intent={}, answer={}",
        response.intent, response.answer,
    );
}

#[test]
fn how_it_works_after_concept_lookup_looks_up_the_concept() {
    // Issue #52: after looking up a concept, "how it works?" should elaborate
    // on it. The prior assistant reply starts with "Wikipedia (encyclopedia): ..."
    // so the solver should re-run a concept lookup for Wikipedia.
    let history = [
        ConversationTurn::user("what is 25519"),
        ConversationTurn::assistant(
            "Curve25519 (cryptography): An elliptic curve used in ECC. \
             Source: https://en.wikipedia.org/wiki/Curve25519 (wikipedia).",
        ),
    ];
    let response = solve_with_history("how it works?", &history);
    assert_ne!(
        response.intent, "unknown",
        "\"how it works?\" after a concept reply must not return unknown; \
         got intent={}, answer={}",
        response.intent, response.answer,
    );
}

#[test]
fn how_does_wikipedia_work_resolves_concept_lookup() {
    // Explicit subject in "how does X work?" form.
    let response = answer("how does Wikipedia work?");
    assert_ne!(
        response.intent, "unknown",
        "\"how does Wikipedia work?\" must not return unknown; got intent={}, answer={}",
        response.intent, response.answer,
    );
    // Since Wikipedia is in the concept seed it should resolve as concept_lookup.
    assert!(
        response.intent.starts_with("concept_lookup")
            || response.intent == "meta_explanation"
            || response.intent == "how_it_works",
        "unexpected intent {} for \"how does Wikipedia work?\"",
        response.intent,
    );
}

#[test]
fn how_it_works_followup_records_followup_event_in_evidence() {
    let history = [
        ConversationTurn::user("what is Wikipedia"),
        ConversationTurn::assistant(
            "Wikipedia (encyclopedia): A free online encyclopedia.\n\nSource: \
             https://en.wikipedia.org/wiki/Wikipedia (wikipedia).",
        ),
    ];
    let response = solve_with_history("how it works?", &history);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("followup:")),
        "\"how it works?\" handler must emit a followup: evidence event: {:?}",
        response.evidence_links,
    );
}

#[test]
fn how_it_works_prior_reply_fallback_skips_function_words_case_insensitively() {
    // When the prior reply has no "Term (category):" header, topic extraction
    // falls back to the first capitalised token that is not a
    // `topic_scan_stop_word` (data/seed/meanings-how.lino). The skip list is
    // matched case-insensitively, so the all-caps leading "THE" is skipped and
    // the real subject "Widgetronics" is taken. A hardcoded title-case array
    // (the pre-conversion behaviour) would have failed to match "THE" and
    // mis-read the function word itself as the topic — this test guards that
    // generalization.
    let history = [
        ConversationTurn::user("tell me about the gadget"),
        ConversationTurn::assistant(
            "THE Widgetronics core spins quietly without a category header.",
        ),
    ];
    let response = solve_with_history("how it works?", &history);
    assert!(
        response.answer.to_lowercase().contains("widgetronics"),
        "fallback topic scan must skip the all-caps function word and name \
         'Widgetronics'; got intent={}, answer={}",
        response.intent,
        response.answer,
    );
    assert!(
        !response.answer.to_lowercase().contains("how the works"),
        "fallback topic scan must not treat the function word 'the' as the \
         topic; got answer={}",
        response.answer,
    );
}

// ---------------------------------------------------------------------------
// Issue #183: "how X works?" must support explicit subjects across languages.
// ---------------------------------------------------------------------------

#[test]
fn multilingual_how_x_works_prompts_use_mechanism_discovery() {
    for prompt in [
        "как устроен AUR?",
        "как работает AUR?",
        "how does AUR work?",
        "AUR कैसे काम करता है?",
        "AUR 如何工作?",
    ] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "how_it_works",
            "{prompt:?} must route to mechanism discovery; got intent={}, answer={}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.to_lowercase().contains("aur"),
            "{prompt:?} answer should name the requested subject; answer={}",
            response.answer,
        );

        for expected in [
            "followup:how_it_works",
            "followup:subject:inline:aur",
            "mechanism_query:stage:wikipedia",
            "mechanism_query:stage:wikidata",
            "mechanism_query:stage:web_search",
        ] {
            assert!(
                has_evidence(&response, expected),
                "{prompt:?} missing evidence prefix {expected:?}: {:?}",
                response.evidence_links,
            );
        }
    }
}

#[test]
fn russian_how_known_concept_works_resolves_concept_lookup() {
    let response = answer("как устроена Википедия?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "known Russian subject should still resolve through concept lookup; \
         got intent={}, answer={}",
        response.intent,
        response.answer,
    );
    assert!(
        has_evidence(&response, "followup:how_it_works"),
        "handler should record the how-it-works follow-up event: {:?}",
        response.evidence_links,
    );
    assert!(
        has_evidence(&response, "concept_lookup:hit:concept_wikipedia"),
        "known subject should hit the Wikipedia concept seed: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// Issue #404: natural language calendar create/schedule ("забей 18 число...").
// The prompt must not fall to unknown; it must produce a structured
// calendar_create_event with rich parsed_* evidence and a confirmation body.
// ---------------------------------------------------------------------------

#[test]
fn calendar_create_event_russian_day_number_with_time_and_tz() {
    // Exact prompt from https://github.com/link-assistant/formal-ai/issues/404
    let response = answer("Забей мне 18 число в 17:00 по грузии на встречу с Леваном");
    assert_ne!(
        response.intent, "unknown",
        "calendar scheduling prompt must not return unknown; got intent={}, answer={}",
        response.intent, response.answer
    );
    assert!(
        response.intent == "calendar_create_event" || response.intent.contains("calendar"),
        "expected calendar_create_event intent, got {}",
        response.intent
    );
    // Rich trace evidence for the parsed fields (the heart of the feature).
    // Evidence links use generated ids after the key (e.g. calendar:parsed_date:calendar:parsed_date_xxx).
    // We assert the presence of the parsed keys (recorded by the handler) + correct intent.
    let parsed_keys = response
        .evidence_links
        .iter()
        .filter(|l| l.starts_with("calendar:parsed_"))
        .count();
    assert!(
        parsed_keys >= 4,
        "must emit multiple calendar:parsed_* evidence keys; links={:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|l| l.contains("parsed_time_zone")
                || l.contains("Asia/Tbilisi")
                || l.contains("грузии")),
        "must capture timezone in evidence; links={:?}",
        response.evidence_links
    );
    // Confirmation-style answer (the handler proposes; it does not auto-create).
    let a = response.answer.to_lowercase();
    assert!(
        a.contains("создать") || a.contains("событие") || a.contains("да"),
        "answer should propose the event and invite confirmation; got: {}",
        response.answer
    );
    // Real, portable calendar artifacts: an RFC 5545 VEVENT plus a no-login
    // Google Calendar render URL, with the "по грузии" alias resolved to IANA.
    assert!(
        response.answer.contains("BEGIN:VCALENDAR") && response.answer.contains("BEGIN:VEVENT"),
        "answer must embed an importable .ics VEVENT; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("TZID=Asia/Tbilisi"),
        "Russian timezone alias must resolve to IANA Asia/Tbilisi; got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("calendar.google.com/calendar/render"),
        "answer must offer a no-login Google Calendar render URL; got: {}",
        response.answer
    );
}

#[test]
fn calendar_create_event_fallback_english() {
    let response = answer("schedule meeting with Levan on the 18th at 5pm Georgia time");
    assert_ne!(response.intent, "unknown");
    assert!(
        response.intent.contains("calendar"),
        "english scheduling must also hit calendar path; intent={}",
        response.intent
    );
    assert!(
        response.answer.contains("BEGIN:VCALENDAR")
            && response.answer.contains("TZID=Asia/Tbilisi")
            && response
                .answer
                .contains("calendar.google.com/calendar/render"),
        "english scheduling must also export a .ics + Google Calendar URL; got: {}",
        response.answer
    );
}

#[test]
fn calendar_create_event_hindi() {
    // No timezone in the prompt → defaults to UTC; the schedule verb is
    // stripped from the .ics SUMMARY so the title reads as the event noun.
    let response = answer("18 तारीख को शाम 5 बजे लेवान के साथ मीटिंग शेड्यूल करें");
    assert_ne!(response.intent, "unknown");
    assert!(
        response.intent.contains("calendar"),
        "hindi scheduling must hit calendar path; intent={}",
        response.intent
    );
    assert!(
        response.answer.contains("BEGIN:VEVENT")
            && response
                .answer
                .contains("calendar.google.com/calendar/render"),
        "hindi scheduling must export a .ics + Google Calendar URL; got: {}",
        response.answer
    );
}

#[test]
fn calendar_create_event_chinese() {
    let response = answer("18号下午5点和Levan安排会议");
    assert_ne!(response.intent, "unknown");
    assert!(
        response.intent.contains("calendar"),
        "chinese scheduling must hit calendar path; intent={}",
        response.intent
    );
    assert!(
        response.answer.contains("BEGIN:VEVENT")
            && response
                .answer
                .contains("calendar.google.com/calendar/render"),
        "chinese scheduling must export a .ics + Google Calendar URL; got: {}",
        response.answer
    );
}

// ---------------------------------------------------------------------------
// Cross-handler sanity: every reasoning path projects from a non-empty event
// log, so the answer is never memoized.
// ---------------------------------------------------------------------------

#[test]
fn every_specialized_handler_emits_a_trace_link() {
    let prompts = [
        "Hi",
        "What is 2 + 2?",
        "What is Wikipedia?",
        "Please run this javascript:\n```js\n1+1;\n```",
        "Write me hello world program in Rust",
    ];
    for prompt in prompts {
        let response = answer(prompt);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("trace:")),
            "prompt {prompt:?} must emit a trace link: {:?}",
            response.evidence_links,
        );
    }
}

// ---------------------------------------------------------------------------
// R89: incompatible-unit queries — explicit symbolic refusal (issue #43).
//
// "Сколько метров в килобайте?" mixes length (meters) with data-storage
// (kilobytes). The solver must recognise the dimensional mismatch and emit
// `intent:unit_incompatibility` with a clear explanation rather than falling
// through to `intent:unknown`.
// ---------------------------------------------------------------------------

#[test]
fn russian_meters_in_kilobyte_returns_unit_incompatibility() {
    let response = answer("Сколько метров в килобайте?");
    assert_eq!(
        response.intent, "unit_incompatibility",
        "mixing length and data-storage units must not fall through to unknown: {:?}",
        response.answer,
    );
    assert!(
        response.answer.contains("length") || response.answer.contains("длин"),
        "answer should mention the length dimension: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("data storage") || response.answer.contains("данн"),
        "answer should mention the data storage dimension: {}",
        response.answer,
    );
    assert!(
        (response.confidence - 1.0).abs() < f32::EPSILON,
        "incompatibility is a known fact, confidence must be 1.0",
    );
}

#[test]
fn english_meters_in_kilobyte_returns_unit_incompatibility() {
    let response = answer("How many meters in a kilobyte?");
    assert_eq!(response.intent, "unit_incompatibility");
    assert!(response.answer.contains("length"));
    assert!(response.answer.contains("data storage"));
}

#[test]
fn incompatible_unit_answer_records_evidence_link() {
    let response = answer("How many meters in a kilobyte?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("unit_incompatibility")),
        "must emit a unit_incompatibility event: {:?}",
        response.evidence_links,
    );
}

#[test]
fn compatible_unit_query_does_not_trigger_incompatibility_handler() {
    // km to meters: both are length — must not fire the incompatibility handler.
    let response = answer("What is 2 + 2?");
    assert_ne!(
        response.intent, "unit_incompatibility",
        "arithmetic prompt must not trigger unit_incompatibility",
    );
}

#[test]
fn greeting_is_not_intercepted_by_incompatibility_handler() {
    let response = answer("Hi");
    assert_eq!(response.intent, "greeting");
}
