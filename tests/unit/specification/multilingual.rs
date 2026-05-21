//! Multilingual chat surface tests.
//!
//! `VISION.md` asks for chat in English, Russian, Hindi, Chinese, and later
//! other languages. These tests pin down the user-visible expectations.

use formal_ai::{humanize_url, ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectation: implementation English greeting.
// ---------------------------------------------------------------------------

#[test]
fn english_greeting_is_handled_today() {
    assert_eq!(answer("Hi").intent, "greeting");
    assert_eq!(answer("Hello").intent, "greeting");
    assert_eq!(answer("Hey").intent, "greeting");
}

// ---------------------------------------------------------------------------
// full-scope expectations: Russian, Hindi, Chinese baseline greetings and identity.
// ---------------------------------------------------------------------------

#[test]
fn russian_greeting_returns_greeting_intent() {
    let response = answer("Привет");
    assert_eq!(response.intent, "greeting");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:ru"),
        "Russian answers should tag the detected language"
    );
}

#[test]
fn russian_greeting_reply_is_in_russian() {
    let response = answer("Привет");
    assert!(
        response.answer.contains("Здравствуйте") || response.answer.contains("Привет"),
        "Russian greeting should be answered in Russian, got: {}",
        response.answer
    );
}

#[test]
fn hindi_greeting_returns_greeting_intent() {
    let response = answer("नमस्ते");
    assert_eq!(response.intent, "greeting");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "language:hi"));
}

#[test]
fn chinese_greeting_returns_greeting_intent() {
    let response = answer("你好");
    assert_eq!(response.intent, "greeting");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "language:zh"));
}

#[test]
fn russian_identity_question_returns_identity_intent() {
    let response = answer("Кто ты?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn russian_combined_greeting_and_identity_question_returns_identity_intent() {
    let response = answer("Привет. ты кто?");
    assert_eq!(response.intent, "identity");
    assert!(
        response.answer.contains("formal-ai"),
        "combined greeting and identity prompt should answer identity, got: {}",
        response.answer
    );
}

#[test]
fn hindi_identity_question_returns_identity_intent() {
    let response = answer("तुम कौन हो?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn chinese_identity_question_returns_identity_intent() {
    let response = answer("你是谁?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn every_multilingual_answer_declares_detected_language_link() {
    for prompt in ["Hi", "Привет", "你好", "नमस्ते"] {
        let response = answer(prompt);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("language:")),
            "missing language tag for prompt {prompt:?}"
        );
    }
}

#[test]
fn unknown_language_prompts_fall_back_to_english_with_unknown_language_link() {
    let response = answer("لطفاً سلام بگو");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:unknown"),
        "answers in unsupported languages should record an unknown-language link"
    );
    assert!(response.answer.contains("English"));
}

// ---------------------------------------------------------------------------
// Issue #16: "What is X?" style prompts must work in Russian, Hindi, Chinese.
// ---------------------------------------------------------------------------

#[test]
fn russian_concept_question_returns_concept_lookup_intent() {
    let response = answer("Что такое Википедия?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Russian concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
    assert!(
        response.answer.to_lowercase().contains("wikipedia")
            || response.answer.to_lowercase().contains("encyclopedia")
            || response.answer.to_lowercase().contains("википед"),
        "Russian Wikipedia answer should reference the concept, got: {}",
        response.answer
    );
}

// Issue #161: graph prompts should be answered from the local concept seed in
// every supported language. With associative project promotion enabled by
// default, each localized answer explains graphs through the Link Foundation
// meta-theory / Links Notation lens.
#[test]
fn graph_questions_promote_links_notation_context_across_supported_languages() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "what is graph",
            "language:en",
            &["Graph", "vertices", "edges", "Links Notation"],
        ),
        (
            "что такое граф",
            "language:ru",
            &["Граф", "вершин", "ребер", "Links Notation", "сеть связей"],
        ),
        (
            "ग्राफ क्या है",
            "language:hi",
            &["ग्राफ", "शीर्ष", "किनार", "Links Notation", "links network"],
        ),
        (
            "图是什么",
            "language:zh",
            &["图", "顶点", "边", "Links Notation", "链接网络"],
        ),
    ];

    for (prompt, language_link, fragments) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "concept_lookup",
            "graph question {prompt:?} should resolve as concept_lookup, got {} -> {}",
            response.intent, response.answer
        );
        assert_ne!(
            response.intent, "unknown",
            "graph question {prompt:?} must not fall through to unknown"
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == language_link),
            "graph question {prompt:?} should record {language_link}, got {:?}",
            response.evidence_links
        );
        for fragment in *fragments {
            assert!(
                response.answer.contains(fragment),
                "graph answer for {prompt:?} should contain {fragment:?}, got: {}",
                response.answer
            );
        }
        assert!(
            response
                .answer
                .contains("https://github.com/link-foundation/meta-theory"),
            "graph answer for {prompt:?} should cite the Link Foundation meta-theory repository, got: {}",
            response.answer
        );
    }
}

#[test]
fn hindi_concept_question_returns_concept_lookup_intent() {
    let response = answer("विकिपीडिया क्या है?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Hindi concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
}

#[test]
fn chinese_concept_question_returns_concept_lookup_intent() {
    let response = answer("维基百科是什么?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Chinese concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
}

// Issue #20: "что такое X в Y" — (concept, context) disambiguation across
// English, Russian, Hindi, and Chinese, matching every typical phrasing.
// The reporter's exact prompt is in the Russian test.
// ---------------------------------------------------------------------------

#[test]
fn russian_iir_in_ml_returns_context_aware_concept_lookup() {
    let response = answer("что такое iir в ml");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Russian (concept,context) prompt should map to concept_lookup_in_context, got: {}",
        response.intent
    );
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("iir") && lower.contains("ml"),
        "Russian (concept,context) answer should reference both halves, got: {}",
        response.answer
    );
}

#[test]
fn english_what_is_iir_in_ml_returns_context_aware_concept_lookup() {
    let response = answer("what is IIR in ML?");
    assert_eq!(response.intent, "concept_lookup_in_context");
    let lower = response.answer.to_lowercase();
    assert!(lower.contains("iir"));
    assert!(lower.contains("ml") || lower.contains("machine learning"));
}

#[test]
fn hindi_iir_in_ml_returns_context_aware_concept_lookup() {
    // Hindi places the context before the concept ("ML में IIR क्या है?").
    let response = answer("ML में IIR क्या है?");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Hindi context-first prompt should map to concept_lookup_in_context, got: {}",
        response.intent
    );
}

#[test]
fn chinese_iir_in_ml_returns_context_aware_concept_lookup() {
    // Chinese also places the context before the concept ("ML 中的 IIR 是什么?").
    let response = answer("ML中的IIR是什么?");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Chinese context-first prompt should map to concept_lookup_in_context, got: {}",
        response.intent
    );
}

#[test]
fn bare_iir_without_context_still_resolves() {
    // Without a context clause the solver should still find the term and
    // return the plain concept_lookup intent (not the in-context variant).
    let response = answer("what is IIR?");
    assert_eq!(
        response.intent, "concept_lookup",
        "Bare term should map to plain concept_lookup, got: {}",
        response.intent
    );
}

#[test]
fn concept_lookup_evidence_records_context_match_event() {
    // Verbose/debug trail: an in-context hit must leave a
    // `concept_lookup:context-match:*` evidence link so we can root-cause
    // future regressions from the trace alone (maintainer requirement #5).
    let response = answer("что такое iir в ml");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("concept_lookup:context-match")),
        "expected a concept_lookup:context-match evidence link, got: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("concept_lookup:request")),
        "expected a concept_lookup:request evidence link, got: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// Issue #20 (maintainer follow-up): native-language body and full disambiguated
// context name. The maintainer asked for:
//   - В контексте «ml» (Машинное обучение) IIR ... [R8]
//   - Russian term: "Фильтр с бесконечной импульсной характеристикой ... или
//     IIR-фильтр" [R9]
//   - Russian summary verbatim from ru.wikipedia.org [R10]
//   - Prefer the user's prevailing language [R11]
// ---------------------------------------------------------------------------

#[test]
fn russian_iir_in_ml_body_uses_native_term_and_context_label() {
    // R8 + R9 + R11: when the prevailing language is Russian, the body must
    // (a) name the resolved context in Russian ("Машинное обучение") and
    // (b) use the Russian term ("Фильтр с бесконечной импульсной...").
    let response = answer("что такое iir в ml");
    let answer_text = &response.answer;
    assert!(
        answer_text.contains("«ml»"),
        "Russian answer should quote the user's literal context phrase, got: {answer_text}"
    );
    assert!(
        answer_text.contains("Машинное обучение"),
        "Russian answer should append the registry label «Машинное обучение», got: {answer_text}"
    );
    assert!(
        answer_text.contains("Фильтр с бесконечной импульсной характеристикой"),
        "Russian answer should use the native term, got: {answer_text}"
    );
    assert!(
        answer_text.contains("IIR-фильтр"),
        "Russian answer should reference the Russian-language alias \"IIR-фильтр\", got: {answer_text}"
    );
}

#[test]
fn russian_iir_in_ml_source_points_at_russian_wikipedia() {
    // R10: the cited source must be the Russian Wikipedia article body the
    // maintainer linked, not the English fallback.
    let response = answer("что такое iir в ml");
    assert!(
        response.answer.contains("ru.wikipedia.org"),
        "Russian answer should cite ru.wikipedia.org, got: {}",
        response.answer
    );
}

#[test]
fn russian_iir_when_context_is_typed_natively_drops_redundant_parens() {
    // R8 corollary: if the user types the localized label themselves, the
    // response should not duplicate it as `«Машинное обучение» (Машинное
    // обучение)`. The `concept_lookup_in_context_no_alias` template handles
    // this without committing to per-language Rust code.
    let response = answer("что такое iir в машинное обучение");
    assert_eq!(response.intent, "concept_lookup_in_context");
    let answer_text = &response.answer;
    assert!(
        answer_text.contains("Машинное обучение"),
        "answer should mention the localized context label, got: {answer_text}"
    );
    assert!(
        !answer_text.contains("«машинное обучение» (Машинное обучение)"),
        "no_alias template should not render «label» (label) duplication, got: {answer_text}"
    );
}

#[test]
fn english_iir_in_ml_body_uses_english_native_term() {
    // R11: prevailing-language routing for English. The localized "en" block
    // expands "IIR" to "infinite impulse response (IIR)" for the long form.
    let response = answer("what is IIR in ML?");
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("infinite impulse response"),
        "English answer should expand the acronym, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("machine learning"),
        "English answer should mention the resolved context label, got: {}",
        response.answer
    );
}

#[test]
fn chinese_iir_in_ml_body_uses_chinese_context_label() {
    // R8 in Chinese: the resolved label «机器学习» (machine learning) must
    // appear in the response body.
    let response = answer("ML中的IIR是什么?");
    assert!(
        response.answer.contains("机器学习"),
        "Chinese answer should append the localized context label, got: {}",
        response.answer
    );
}

#[test]
fn hindi_iir_in_ml_body_uses_hindi_context_label() {
    // R8 in Hindi: the resolved label «मशीन लर्निंग» must appear.
    let response = answer("ML में IIR क्या है?");
    assert!(
        response.answer.contains("मशीन लर्निंग"),
        "Hindi answer should append the localized context label, got: {}",
        response.answer
    );
}

#[test]
fn russian_iir_evidence_includes_wikidata_anchor() {
    // R13: the link network must carry the Wikidata Q-ID anchor so callers
    // can use it as a cross-language join key (this is how human-language and
    // meta-expression translate across the four target languages).
    let response = answer("что такое iir в ml");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.contains("Q740073") || link.contains("wikidata")),
        "expected a wikidata-anchored evidence link for cross-language joins, got: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// Issue #49: "что за дичь?" and "что ты умеешь?" must not fall through to
// intent: unknown. The user expressed frustration after the agent failed to
// handle a capabilities question in Russian. Both the capability query and
// the confusion/frustration expression must produce a meaningful intent.
// ---------------------------------------------------------------------------

#[test]
fn russian_capabilities_question_does_not_return_unknown() {
    let response = answer("что ты умеешь?");
    assert_ne!(
        response.intent, "unknown",
        "Russian capability question should not fall through to unknown, got: {}",
        response.answer,
    );
}

#[test]
fn russian_capabilities_question_returns_capabilities_intent() {
    let response = answer("что ты умеешь?");
    assert_eq!(
        response.intent, "capabilities",
        "Russian capability question should map to capabilities intent, got: {}",
        response.answer,
    );
}

#[test]
fn russian_confusion_phrase_does_not_return_unknown() {
    let response = answer("что за дичь?");
    assert_ne!(
        response.intent, "unknown",
        "Russian slang confusion phrase should not fall through to unknown, got: {}",
        response.answer,
    );
}

#[test]
fn russian_confusion_phrase_returns_capabilities_intent() {
    // "что за дичь?" literally means "what is this nonsense?" — a frustrated
    // reaction to getting an unhelpful answer. The agent should respond with a
    // capabilities overview so the user understands what the agent can handle.
    let response = answer("что за дичь?");
    assert_eq!(
        response.intent, "capabilities",
        "Russian confusion phrase should map to capabilities intent, got: {}",
        response.answer,
    );
}

#[test]
fn russian_capabilities_answer_is_in_russian() {
    let response = answer("что ты умеешь?");
    assert!(
        response
            .answer
            .chars()
            .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)),
        "Russian capabilities answer should contain Cyrillic text, got: {}",
        response.answer,
    );
}

#[test]
fn russian_more_capabilities_follow_up_uses_history_without_repeating_web_search() {
    let history = [
        ConversationTurn::user("Ты можешь искать в интернете?"),
        ConversationTurn::assistant(
            "Да. В этой конфигурации веб-поиск включен: я могу использовать DuckDuckGo.",
        ),
    ];
    let response = UniversalSolver::default().solve_with_history("Что ещё ты умеешь?", &history);
    assert_eq!(
        response.intent, "capabilities",
        "Russian follow-up capabilities question should map to capabilities, got {}: {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("Арифметика") && response.answer.contains("Перевод"),
        "follow-up should list additional capabilities, got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("Веб-поиск")
            && !response.answer.to_lowercase().contains("интернет")
            && !response.answer.contains("DuckDuckGo"),
        "follow-up should not repeat the already discussed web-search capability, got: {}",
        response.answer,
    );
}

#[test]
fn english_capabilities_question_returns_capabilities_intent() {
    let response = answer("what can you do?");
    assert_eq!(
        response.intent, "capabilities",
        "English capability question should map to capabilities intent, got: {}",
        response.answer,
    );
}

// ---------------------------------------------------------------------------
// Issue #21: URLs with non-ASCII characters must be displayed in human-readable
// IRI form across every surface, while remaining functional (the encoded URI
// must still resolve when clicked). These tests pin down the helper that every
// formal-ai surface uses to render Wikipedia and concept-lookup sources.
// ---------------------------------------------------------------------------

#[test]
fn humanize_url_renders_cyrillic_wikipedia_link_readably() {
    // The exact URL pattern from issue #21.
    let encoded = "https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4";
    assert_eq!(
        humanize_url(encoded),
        "https://ru.wikipedia.org/wiki/Изумруд",
        "Cyrillic Wikipedia URL must display as readable IRI",
    );
}

#[test]
fn humanize_url_handles_every_supported_language() {
    let cases = [
        (
            "https://hi.wikipedia.org/wiki/%E0%A4%A8%E0%A4%AE%E0%A4%B8%E0%A5%8D%E0%A4%A4%E0%A5%87",
            "https://hi.wikipedia.org/wiki/नमस्ते",
        ),
        (
            "https://zh.wikipedia.org/wiki/%E4%BD%A0%E5%A5%BD",
            "https://zh.wikipedia.org/wiki/你好",
        ),
        (
            "https://ja.wikipedia.org/wiki/%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF",
            "https://ja.wikipedia.org/wiki/こんにちは",
        ),
        (
            "https://ar.wikipedia.org/wiki/%D9%85%D8%B1%D8%AD%D8%A8%D8%A7",
            "https://ar.wikipedia.org/wiki/مرحبا",
        ),
    ];
    for (encoded, expected) in cases {
        assert_eq!(
            humanize_url(encoded),
            expected,
            "humanize_url failed for {encoded}",
        );
    }
}

#[test]
fn humanize_url_preserves_functional_link_target() {
    // The encoded form must round-trip cleanly: encode(humanize(x)) ≈ x for
    // every URL we ship. We approximate the cycle by asserting that the
    // humanized form, when fed through Rust's standard percent-encoding via
    // the path crate (or by ensuring it contains the original Unicode chars),
    // does not lose information.
    let encoded = "https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4";
    let humanized = humanize_url(encoded);
    assert!(humanized.contains("Изумруд"));
    assert!(humanized.starts_with("https://ru.wikipedia.org/wiki/"));
    // ASCII URLs must round-trip untouched.
    let ascii = "https://en.wikipedia.org/wiki/Albert_Einstein";
    assert_eq!(humanize_url(ascii), ascii);
}

// ---------------------------------------------------------------------------
// Issue #44: Russian prompts with no matching rule return unknown + Russian
// reply.
// ---------------------------------------------------------------------------

#[test]
fn russian_nonsensical_question_returns_unknown_intent() {
    let response = answer("куда плешивый спрятал сахар?");
    assert_eq!(response.intent, "unknown");
}

#[test]
fn russian_mixed_units_question_returns_unit_incompatibility_intent() {
    let response = answer("Сколько метров в килобайте?");
    assert_eq!(response.intent, "unit_incompatibility");
}

#[test]
fn russian_trick_riddle_returns_unknown_intent() {
    let response = answer(
        "Стоит четырёхэтажный дом, в каждом этаже по восьми окон, \
         на крыше — два слуховых окна и две трубы, в каждом этаже \
         по два квартиранта. А теперь скажите, господа, в каком году \
         умерла у швейцара его бабушка?",
    );
    assert_eq!(response.intent, "unknown");
}

#[test]
fn russian_unknown_reply_is_in_russian() {
    let response = answer("куда плешивый спрятал сахар?");
    assert_eq!(response.intent, "unknown");
    assert!(
        response.answer.contains("символьного правила")
            || response.answer.contains("Links Notation"),
        "Russian unknown reply should be in Russian or reference Links Notation, got: {}",
        response.answer
    );
}

// ---------------------------------------------------------------------------
// Issue #29: "не понял" and other clarification prompts should be handled
// with a helpful clarification response, not the generic "unknown" fallback.
// ---------------------------------------------------------------------------

#[test]
fn russian_did_not_understand_returns_clarification_intent() {
    let response = answer("не понял");
    assert_eq!(
        response.intent, "clarification",
        "\"не понял\" should map to clarification intent, got: {}",
        response.intent
    );
}

#[test]
fn russian_clarification_reply_is_in_russian() {
    let response = answer("не понял");
    assert!(
        !response.answer.contains("symbolic rule")
            && !response.answer.contains("Links Notation fact"),
        "clarification reply must not be the generic unknown-intent fallback, got: {}",
        response.answer
    );
}

#[test]
fn english_did_not_understand_returns_clarification_intent() {
    let response = answer("I don't understand");
    assert_eq!(
        response.intent, "clarification",
        "\"I don't understand\" should map to clarification intent, got: {}",
        response.intent
    );
}

#[test]
fn english_dont_understand_variant_returns_clarification_intent() {
    let response = answer("I didn't understand");
    assert_eq!(
        response.intent, "clarification",
        "\"I didn't understand\" should map to clarification intent, got: {}",
        response.intent
    );
}

// ---------------------------------------------------------------------------
// Issue #30: "назови цвет" — "назови " prefix must route to concept_lookup,
// and "цвет" must resolve to the color concept record.
// The reporter's exact prompt was "назови цвет" which returned intent:unknown.
// ---------------------------------------------------------------------------

#[test]
fn russian_nazovi_prefix_routes_to_concept_lookup() {
    // "назови X" is a Russian imperative meaning "name X / tell me X".
    // It must be recognized as a concept_lookup prefix (issue #30).
    let response = answer("назови цвет");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "\"назови цвет\" should route to concept_lookup, got: {}",
        response.intent
    );
}

#[test]
fn russian_nazovi_tsvet_answer_references_color() {
    // The resolved answer must reference the color concept.
    let response = answer("назови цвет");
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("цвет") || lower.contains("color") || lower.contains("colour"),
        "\"назови цвет\" answer should describe a color, got: {}",
        response.answer
    );
}

// ---------------------------------------------------------------------------
// Issue #41: "Купи слона" — well-known Russian circular-joke idiom.
// The phrase should be recognized and answered with the traditional reply,
// not fall through to the "unknown" catch-all intent.
// ---------------------------------------------------------------------------

#[test]
fn kupi_slona_returns_dedicated_idiom_intent() {
    // Issue #41 reporter's exact prompt.
    let response = answer("Купи слона");
    assert_ne!(
        response.intent, "unknown",
        "\"Купи слона\" must not fall through to unknown intent; got: {}",
        response.intent
    );
    assert_eq!(
        response.intent, "kupi_slona",
        "\"Купи слона\" must map to the kupi_slona intent, got: {}",
        response.intent
    );
}

#[test]
fn kupi_slona_answer_includes_traditional_reply() {
    let response = answer("Купи слона");
    let lower = response.answer.to_lowercase();
    // The traditional comeback is "у всех есть слон, а у меня нет"
    // (everyone has an elephant, but I don't) and similar variants.
    assert!(
        lower.contains("слон") || lower.contains("всех"),
        "\"Купи слона\" answer should reference the elephant, got: {}",
        response.answer
    );
}

#[test]
fn kupi_slona_answer_is_in_russian() {
    let response = answer("Купи слона");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:ru"),
        "\"Купи слона\" should be tagged as Russian, got evidence links: {:?}",
        response.evidence_links
    );
}
