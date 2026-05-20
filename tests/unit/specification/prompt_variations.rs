//! Issue #103 prompt-variation matrix.
//!
//! For every conversational surface, exercise 5–10 most probable input
//! variations across the four currently supported languages (English,
//! Russian, Hindi, Chinese). Helpers are generalised so a new category can
//! be added in one block instead of one test per language.
//!
//! References:
//! - `docs/case-studies/issue-103/README.md`
//! - `docs/case-studies/issue-103/raw-data/competitor-test-research.md`
//!
//! All issue-103 prompt categories in this file are active regression tests.

use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Generalised assertion helpers (R132).
// Each helper takes a slice of prompts and the expected single property to
// check on every prompt's answer. Helpers preserve the prompt text inside
// the failure message so the failing variation is obvious in CI logs.
// ---------------------------------------------------------------------------

fn assert_intent_for_each(prompts: &[&str], expected_intent: &str) {
    for prompt in prompts {
        let response = answer(prompt);
        assert_eq!(
            response.intent, expected_intent,
            "prompt {prompt:?} should yield intent {expected_intent:?}, got intent={} answer={}",
            response.intent, response.answer,
        );
    }
}

fn assert_intent_prefix_for_each(prompts: &[&str], expected_prefix: &str) {
    for prompt in prompts {
        let response = answer(prompt);
        assert!(
            response.intent.starts_with(expected_prefix),
            "prompt {prompt:?} should yield intent starting with {expected_prefix:?}, got intent={} answer={}",
            response.intent,
            response.answer,
        );
    }
}

fn assert_language_for_each(prompts: &[&str], expected_language_tag: &str) {
    for prompt in prompts {
        let response = answer(prompt);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == expected_language_tag),
            "prompt {prompt:?} should record evidence {expected_language_tag:?}, got links={:?}",
            response.evidence_links,
        );
    }
}

fn assert_answer_contains_for_each(prompts: &[&str], expected_fragments: &[&str]) {
    for prompt in prompts {
        let response = answer(prompt);
        let lower = response.answer.to_lowercase();
        assert!(
            expected_fragments
                .iter()
                .any(|fragment| lower.contains(&fragment.to_lowercase())),
            "prompt {prompt:?} answer should contain one of {expected_fragments:?}, got answer={}",
            response.answer,
        );
    }
}

fn assert_intent_not(prompts: &[&str], forbidden_intent: &str) {
    for prompt in prompts {
        let response = answer(prompt);
        assert_ne!(
            response.intent, forbidden_intent,
            "prompt {prompt:?} should not yield intent {forbidden_intent:?}, got answer={}",
            response.answer,
        );
    }
}

// ---------------------------------------------------------------------------
// Greeting matrix: 5+ variations × 4 languages.
//
// English, Russian, Hindi, Chinese natural-language greetings (Belebele/
// XCOPA-style: each language has authentic phrasings, not literal
// translations).
// ---------------------------------------------------------------------------

const ENGLISH_GREETINGS: &[&str] = &["Hi", "Hello", "Hey", "Hello!", "Hi.", "hELLO", "Hi!"];

const RUSSIAN_GREETINGS: &[&str] = &["Привет", "Здравствуйте", "привет!", "Шалом", "Шабат шалом"];

const HINDI_GREETINGS: &[&str] = &["नमस्ते", "नमस्कार", "राम राम", "सलाम", "हाय", "नमस्ते!"];

const CHINESE_GREETINGS: &[&str] = &["你好", "您好", "早上好", "早安", "嗨", "哈喽"];

#[test]
fn greeting_matrix_is_classified_as_greeting_across_languages() {
    for prompts in [
        ENGLISH_GREETINGS,
        RUSSIAN_GREETINGS,
        HINDI_GREETINGS,
        CHINESE_GREETINGS,
    ] {
        assert_intent_for_each(prompts, "greeting");
    }
}

#[test]
fn greeting_matrix_records_per_language_evidence() {
    for (prompts, tag) in [
        (ENGLISH_GREETINGS, "language:en"),
        (RUSSIAN_GREETINGS, "language:ru"),
        (HINDI_GREETINGS, "language:hi"),
        (CHINESE_GREETINGS, "language:zh"),
    ] {
        assert_language_for_each(prompts, tag);
    }
}

// ---------------------------------------------------------------------------
// Farewell matrix.
//
// Farewell prompts are part of the seeded intent-routing rule book but were
// previously only exercised by the seed test. Pin them here too so a
// regression in the matcher is caught alongside greetings.
// ---------------------------------------------------------------------------

const ENGLISH_FAREWELLS: &[&str] = &["bye", "goodbye", "ciao"];

const RUSSIAN_FAREWELLS: &[&str] = &["пока", "до свидания", "досвидания"];

const HINDI_FAREWELLS: &[&str] = &["अलविदा", "फिर मिलेंगे", "विदा", "बाय", "टाटा"];

const CHINESE_FAREWELLS: &[&str] = &["再见", "拜拜", "回见", "改天见", "后会有期"];

#[test]
fn farewell_matrix_is_classified_as_farewell_across_languages() {
    for prompts in [
        ENGLISH_FAREWELLS,
        RUSSIAN_FAREWELLS,
        HINDI_FAREWELLS,
        CHINESE_FAREWELLS,
    ] {
        assert_intent_for_each(prompts, "farewell");
    }
}

// ---------------------------------------------------------------------------
// Identity matrix.
//
// "Who are you?" and its variants are the most frequent identity prompt
// across MT-Bench, AlpacaEval, and WildBench. We exercise the same set in
// every supported language.
// ---------------------------------------------------------------------------

const ENGLISH_IDENTITY: &[&str] = &[
    "Who are you?",
    "what are you",
    "Tell me about yourself",
    "Introduce yourself",
    "What is formal-ai?",
    "What is formalai?",
];

const RUSSIAN_IDENTITY: &[&str] = &["Кто ты?", "что ты", "Привет. ты кто?"];

const HINDI_IDENTITY: &[&str] = &[
    "तुम कौन हो?",
    "आप कौन हैं?",
    "तू कौन है?",
    "अपना परिचय दो",
    "अपने बारे में बताओ",
];

const CHINESE_IDENTITY: &[&str] = &[
    "你是谁?",
    "您是谁?",
    "你是什么?",
    "介绍一下你自己",
    "告诉我你自己",
];

#[test]
fn identity_matrix_is_classified_as_identity_across_languages() {
    for prompts in [
        ENGLISH_IDENTITY,
        RUSSIAN_IDENTITY,
        HINDI_IDENTITY,
        CHINESE_IDENTITY,
    ] {
        assert_intent_for_each(prompts, "identity");
    }
}

#[test]
fn identity_matrix_mentions_formal_ai_in_every_language() {
    for prompts in [
        ENGLISH_IDENTITY,
        RUSSIAN_IDENTITY,
        HINDI_IDENTITY,
        CHINESE_IDENTITY,
    ] {
        assert_answer_contains_for_each(prompts, &["formal-ai", "formal ai", "formalai"]);
    }
}

// ---------------------------------------------------------------------------
// Clarification matrix.
//
// "I didn't understand" and its variants. Issue #29 added this category.
// ---------------------------------------------------------------------------

const ENGLISH_CLARIFICATION: &[&str] = &[
    "I don't understand",
    "I didn't understand",
    "I dont understand",
    "I am confused",
    "I'm confused",
    "What do you mean",
];

const RUSSIAN_CLARIFICATION: &[&str] = &[
    "не понял",
    "не понимаю",
    "не поняла",
    "не понятно",
    "непонятно",
];

const HINDI_CLARIFICATION: &[&str] = &["समझ नहीं आया", "समझ नहीं आई"];

const CHINESE_CLARIFICATION: &[&str] = &["我不明白", "我不懂", "听不懂"];

#[test]
fn clarification_matrix_is_classified_as_clarification_across_languages() {
    for prompts in [
        ENGLISH_CLARIFICATION,
        RUSSIAN_CLARIFICATION,
        HINDI_CLARIFICATION,
        CHINESE_CLARIFICATION,
    ] {
        assert_intent_for_each(prompts, "clarification");
    }
}

// ---------------------------------------------------------------------------
// Concept lookup matrix.
//
// "What is X?" is one of the highest-frequency conversational prompts in
// MT-Bench, AlpacaEval, and WildBench, and it is the canonical
// formalization target: each X resolves to a Wikidata Q-id. We exercise
// the multilingual phrasings of the question for terms that the seed
// concept table covers (Wikipedia, Rust, doublet).
// ---------------------------------------------------------------------------

const ENGLISH_CONCEPT_LOOKUPS: &[&str] = &[
    "What is Wikipedia?",
    "Tell me about Wikipedia.",
    "Define Wikipedia.",
    "What is Rust?",
    "What is a doublet?",
    "What is Links Notation?",
];

const RUSSIAN_CONCEPT_LOOKUPS: &[&str] = &["Что такое Википедия?", "Что такое Rust?"];

const HINDI_CONCEPT_LOOKUPS: &[&str] = &[
    "विकिपीडिया क्या है?",
    "रस्ट क्या है?",
    "रंग क्या है?",
    "विकिडेटा क्या है?",
    "आईआईआर क्या है?",
];

const CHINESE_CONCEPT_LOOKUPS: &[&str] = &[
    "维基百科是什么?",
    "颜色是什么?",
    "维基数据是什么?",
    "无限脉冲响应是什么?",
    "rust语言是什么?",
];

#[test]
fn concept_lookup_matrix_is_classified_as_concept_lookup_across_languages() {
    for prompts in [
        ENGLISH_CONCEPT_LOOKUPS,
        RUSSIAN_CONCEPT_LOOKUPS,
        HINDI_CONCEPT_LOOKUPS,
        CHINESE_CONCEPT_LOOKUPS,
    ] {
        assert_intent_prefix_for_each(prompts, "concept_lookup");
    }
}

// ---------------------------------------------------------------------------
// Capabilities matrix.
//
// Issue #49: "что ты умеешь?" / "what can you do?" must not fall through
// to unknown. Capability questions show up across every chatbot benchmark
// (Chatbot Arena's "Open conversation" category, WildBench "About the
// model").
// ---------------------------------------------------------------------------

const ENGLISH_CAPABILITIES: &[&str] = &["what can you do?", "what can you do", "what you can do?"];

const RUSSIAN_CAPABILITIES: &[&str] = &[
    "что ты умеешь?",
    "что ты умеешь",
    // Issue #49 covered the slang follow-up: "что за дичь?" maps to
    // capabilities too, as a frustrated form of the same question.
    "что за дичь?",
];

#[test]
fn capabilities_matrix_is_classified_as_capabilities_across_languages() {
    for prompts in [ENGLISH_CAPABILITIES, RUSSIAN_CAPABILITIES] {
        assert_intent_for_each(prompts, "capabilities");
    }
}

#[test]
fn capabilities_matrix_never_falls_through_to_unknown() {
    for prompts in [ENGLISH_CAPABILITIES, RUSSIAN_CAPABILITIES] {
        assert_intent_not(prompts, "unknown");
    }
}

// ---------------------------------------------------------------------------
// Hello-world code-generation matrix.
//
// The hello-world seeds cover Rust, Python, JavaScript, Go, C, and
// TypeScript. We re-pin them here as a single block so adding new
// languages or aliases only requires extending the tuples.
// ---------------------------------------------------------------------------

const HELLO_WORLD_VARIATIONS: &[(&str, &str)] = &[
    // English natural language.
    ("Write me hello world program in Rust", "hello_world_rust"),
    ("Write hello world in Python", "hello_world_python"),
    (
        "Show me hello world in JavaScript",
        "hello_world_javascript",
    ),
    ("hello world in Go", "hello_world_go"),
    ("hello world in C", "hello_world_c"),
    ("hello world in TypeScript", "hello_world_typescript"),
    // English-language aliases.
    ("hello world in rs", "hello_world_rust"),
    ("hello world in node", "hello_world_javascript"),
    ("hello world in py", "hello_world_python"),
    ("hello world in golang", "hello_world_go"),
    // Russian transliteration (issue #53).
    ("Напиши хелло ворлд на питоне", "hello_world_python"),
    ("хелло ворлд на джаваскрипт", "hello_world_javascript"),
    ("хелло ворлд на расте", "hello_world_rust"),
];

#[test]
fn hello_world_matrix_routes_to_per_language_intent() {
    for (prompt, expected_intent) in HELLO_WORLD_VARIATIONS {
        let response = answer(prompt);
        assert_eq!(
            response.intent, *expected_intent,
            "prompt {prompt:?} should yield intent {expected_intent:?}, got intent={} answer={}",
            response.intent, response.answer,
        );
    }
}

#[test]
fn hello_world_matrix_emits_a_code_block_per_language() {
    let fence_for_intent = |intent: &str| -> &'static str {
        match intent {
            "hello_world_rust" => "```rust",
            "hello_world_python" => "```python",
            "hello_world_javascript" => "```javascript",
            "hello_world_go" => "```go",
            "hello_world_c" => "```c",
            "hello_world_typescript" => "```typescript",
            other => panic!("unexpected hello-world intent {other:?}"),
        }
    };
    for (prompt, expected_intent) in HELLO_WORLD_VARIATIONS {
        let response = answer(prompt);
        let fence = fence_for_intent(expected_intent);
        assert!(
            response.answer.contains(fence),
            "prompt {prompt:?} (intent {expected_intent}) should include {fence} fence, got: {}",
            response.answer,
        );
    }
}

// ---------------------------------------------------------------------------
// Math / calculator matrix (R124 augmentation).
//
// `calculator_delegation.rs` already exercises 5-10 prompts per language
// for calculator-backed math. This matrix adds the highest-frequency
// "simple arithmetic in natural language" prompts that show up in GSM8K /
// MT-Bench Math / AlpacaEval-style benchmarks.
// ---------------------------------------------------------------------------

const ENGLISH_BASIC_MATH: &[&str] = &[
    "What is 2 + 2?",
    "What is 5 * 6?",
    "What is 100 / 4?",
    "What is 10 - 3?",
];

const RUSSIAN_BASIC_MATH: &[&str] = &["Сколько будет 2 + 2?", "Сколько будет два плюс два?"];

#[test]
fn basic_math_matrix_is_classified_as_calculation() {
    for prompts in [ENGLISH_BASIC_MATH, RUSSIAN_BASIC_MATH] {
        assert_intent_for_each(prompts, "calculation");
    }
}

// ---------------------------------------------------------------------------
// Refusal / safety matrix (MT-Bench / Chatbot Arena overlap).
//
// Issue #39 added the policy_inappropriate_content intent. We add a few
// variations so a regression that re-enables a vulgar prompt is caught
// here too.
// ---------------------------------------------------------------------------

const ENGLISH_REFUSAL: &[&str] = &["suck my dick"];

#[test]
fn refusal_matrix_routes_to_policy_inappropriate_content() {
    assert_intent_for_each(ENGLISH_REFUSAL, "policy_inappropriate_content");
}

// ---------------------------------------------------------------------------
// Idiom matrix.
//
// "Купи слона" (issue #41) and its variations. Idioms in non-English
// languages are a known weakness of competitor models (Aya, Belebele).
// ---------------------------------------------------------------------------

const RUSSIAN_IDIOMS: &[&str] = &["Купи слона", "купи слона"];

#[test]
fn idiom_matrix_routes_to_dedicated_idiom_intent() {
    assert_intent_for_each(RUSSIAN_IDIOMS, "kupi_slona");
}

// ---------------------------------------------------------------------------
// Determinism guarantee for the matrix (R82 corollary).
//
// Every matrix prompt must be deterministic. Pin a couple to make sure a
// regression that introduces non-determinism is caught by the variations
// suite, not only by the dedicated `chat_surface` tests.
// ---------------------------------------------------------------------------

#[test]
fn matrix_answers_are_deterministic_for_identical_prompts() {
    let sample_prompts = ["Hi", "Привет", "你好", "What is Rust?", "Кто ты?"];
    for prompt in sample_prompts {
        let first = answer(prompt);
        let second = answer(prompt);
        assert_eq!(
            first, second,
            "matrix prompt {prompt:?} should be deterministic"
        );
    }
}

// ---------------------------------------------------------------------------
// Competitor-derived prompt categories.
//
// These prompt shapes are common in MT-Bench, AlpacaEval, WildBench, Aya,
// and Belebele. They are active tests because issue #103 asks us to adopt
// the frequent categories as executable behavior.
// ---------------------------------------------------------------------------

const SUMMARIZATION_PROMPTS: &[&str] = &[
    "Summarize the Wikipedia article on Rust in one paragraph.",
    "Give me a one-paragraph summary of formal-ai.",
    "Can you summarize Rust?",
    "Summarise the Wikipedia entry for Rust.",
    "Summary of Wikipedia please.",
    "Please summarize formal-ai in one paragraph.",
];

#[test]
fn summarization_intent_routes_to_summarization_handler() {
    for prompt in SUMMARIZATION_PROMPTS {
        let response = answer(prompt);
        assert!(
            response.intent.starts_with("summarize"),
            "prompt {prompt:?} should route to a summarize* intent, got: {}",
            response.intent,
        );
    }
}

const BRAINSTORMING_PROMPTS: &[&str] = &[
    "Give me five ideas for an open-source side project.",
    "Brainstorm ten names for a code review tool.",
    "Suggest five open-source utilities for developers.",
    "Brainstorm 5 small tools for link notation.",
    "Give me 5 ideas for a local-first AI helper.",
    "Brainstorm ten names for a symbolic assistant.",
];

#[test]
fn brainstorming_intent_routes_to_brainstorm_handler() {
    for prompt in BRAINSTORMING_PROMPTS {
        let response = answer(prompt);
        assert!(
            response.intent.starts_with("brainstorm"),
            "prompt {prompt:?} should route to a brainstorm* intent, got: {}",
            response.intent,
        );
        let expected_last_number = if prompt.contains("ten") { "10." } else { "5." };
        assert!(
            response.answer.contains(expected_last_number),
            "prompt {prompt:?} should return the requested number of ideas, got: {}",
            response.answer,
        );
    }
}

#[test]
fn web_search_online_variant_routes_to_web_search_handler() {
    let response = answer("Search online for Genshin Impact");
    assert_eq!(
        response.intent, "web_search",
        "reported search-online phrasing should route to web_search, got {} with answer {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.to_lowercase().contains("genshin impact"),
        "web search response should preserve the query, got {}",
        response.answer,
    );
}

// Curated Link Assistant / Link Foundation project tests live in
// `project_lookups.rs` so this file stays under the 1000-line cap.

// Fact-lookup matrix: 5-10 input variations per fact across every supported
// language. Each prompt must route to `fact_lookup`, surface a Wikidata
// Q-id evidence link, and return the localized summary from the seed.
const ENGLISH_FACTUAL_PROMPTS: &[&str] = &[
    "Who wrote The Lord of the Rings?",
    "Who is the author of The Lord of the Rings?",
    "When was the Eiffel Tower built?",
    "What year did construction of the Eiffel Tower start?",
    "What is the capital of Japan?",
    "Which city is Japan's capital?",
    "What is the capital of Russia?",
    "Which city is Russia's capital?",
    "Who painted the Mona Lisa?",
    "Who is the painter of the Mona Lisa?",
    "What is the speed of light?",
    "How fast is the speed of light?",
];

const RUSSIAN_FACTUAL_PROMPTS: &[&str] = &[
    "Кто написал Властелин колец?",
    "Кто автор Властелина колец?",
    "Когда построили Эйфелеву башню?",
    "Какова столица Японии?",
    "Какова столица России?",
    "столица россии",
    "Кто написал Мону Лизу?",
    "Чему равна скорость света?",
];

const HINDI_FACTUAL_PROMPTS: &[&str] = &[
    "द लॉर्ड ऑफ द रिंग्स किसने लिखी?",
    "एफिल टॉवर कब बनी?",
    "जापान की राजधानी क्या है?",
    "रूस की राजधानी क्या है?",
    "मोना लिसा किसने बनाई?",
    "प्रकाश की गति कितनी है?",
];

const CHINESE_FACTUAL_PROMPTS: &[&str] = &[
    "魔戒是谁写的?",
    "埃菲尔铁塔建于何时?",
    "日本的首都是什么?",
    "俄罗斯的首都是什么?",
    "蒙娜丽莎是谁画的?",
    "光速是多少?",
];

#[test]
fn factual_qna_matrix_records_wikidata_anchor() {
    for prompts in [
        ENGLISH_FACTUAL_PROMPTS,
        RUSSIAN_FACTUAL_PROMPTS,
        HINDI_FACTUAL_PROMPTS,
        CHINESE_FACTUAL_PROMPTS,
    ] {
        for prompt in prompts {
            let response = answer(prompt);
            assert_eq!(
                response.intent, "fact_lookup",
                "prompt {prompt:?} should route to fact_lookup, got {}",
                response.intent,
            );
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link.contains("wikidata") || link.contains('Q')),
                "prompt {prompt:?} should record a Wikidata anchor, got links: {:?}",
                response.evidence_links,
            );
        }
    }
}

#[test]
fn factual_qna_matrix_records_per_language_evidence() {
    for (prompts, tag) in [
        (ENGLISH_FACTUAL_PROMPTS, "language:en"),
        (RUSSIAN_FACTUAL_PROMPTS, "language:ru"),
        (HINDI_FACTUAL_PROMPTS, "language:hi"),
        (CHINESE_FACTUAL_PROMPTS, "language:zh"),
    ] {
        assert_language_for_each(prompts, tag);
    }
}

#[test]
fn russian_capital_russia_prompt_returns_moscow() {
    let response = answer("столица россии");
    assert_eq!(
        response.intent, "fact_lookup",
        "reported prompt should route to fact_lookup, got {} with answer {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("Москва"),
        "reported prompt should answer in Russian with Moscow, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q159"),
        "reported prompt should record the Russia Wikidata anchor, got links: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q649"),
        "reported prompt should record the Moscow Wikidata anchor, got links: {:?}",
        response.evidence_links,
    );
}

// Issue #127 follow-up: the structured fact-query pipeline pre-warms the
// cache from `data/seed/facts.lino` records that carry a `relation` field.
// Every country in the matrix below has a `relation "capital"` seed entry,
// so every prompt — across English/Russian/Hindi/Chinese — must route to
// `fact_lookup` and surface the subject Q-ID, the value Q-ID, and the
// structured `fact_query:*` trace events.
//
// (country_label, expected_subject_qid, expected_value_qid, expected_answer_fragment, prompts)
type CapitalCase = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
);

const CAPITAL_CASES: &[CapitalCase] = &[
    (
        "Russia",
        "Q159",
        "Q649",
        "Moscow",
        &[
            "What is the capital of Russia?",
            "Which city is Russia's capital?",
            "capital of the Russian Federation",
        ],
    ),
    (
        "Japan",
        "Q17",
        "Q1490",
        "Tokyo",
        &[
            "What is the capital of Japan?",
            "Which city is Japan's capital?",
        ],
    ),
    (
        "France",
        "Q142",
        "Q90",
        "Paris",
        &[
            "What is the capital of France?",
            "What is the capital of the French Republic?",
        ],
    ),
    (
        "Germany",
        "Q183",
        "Q64",
        "Berlin",
        &[
            "What is the capital of Germany?",
            "What is Germany's capital?",
        ],
    ),
    (
        "China",
        "Q148",
        "Q956",
        "Beijing",
        &[
            "What is the capital of China?",
            "Which city is the capital of the People's Republic of China?",
        ],
    ),
    (
        "India",
        "Q668",
        "Q987",
        "New Delhi",
        &[
            "What is the capital of India?",
            "Which city is India's capital?",
        ],
    ),
    (
        "Brazil",
        "Q155",
        "Q2844",
        "Brasília",
        &[
            "What is the capital of Brazil?",
            "Which city is Brazil's capital?",
        ],
    ),
    (
        "United States",
        "Q30",
        "Q61",
        "Washington",
        &[
            "What is the capital of the United States?",
            "What is the capital of the USA?",
        ],
    ),
    (
        "United Kingdom",
        "Q145",
        "Q84",
        "London",
        &[
            "What is the capital of the United Kingdom?",
            "What is the capital of the UK?",
        ],
    ),
];

#[test]
fn capital_matrix_resolves_every_seeded_country() {
    for (country, subject_qid, value_qid, expected_substr, prompts) in CAPITAL_CASES {
        for prompt in *prompts {
            let response = answer(prompt);
            assert_eq!(
                response.intent, "fact_lookup",
                "{country} prompt {prompt:?} should route to fact_lookup, got {}",
                response.intent,
            );
            assert!(
                response.answer.contains(expected_substr),
                "{country} prompt {prompt:?} should mention {expected_substr}, got: {}",
                response.answer,
            );
            let subject_link = format!("wikidata:{subject_qid}");
            let value_link = format!("wikidata:{value_qid}");
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link == &subject_link),
                "{country} prompt {prompt:?} should record {subject_link}, got: {:?}",
                response.evidence_links,
            );
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link == &value_link),
                "{country} prompt {prompt:?} should record {value_link}, got: {:?}",
                response.evidence_links,
            );
        }
    }
}

#[test]
fn capital_matrix_records_structured_fact_query_trace() {
    // The Russian "столица России" trace should include the structured
    // `fact_query:relation:capital` event and the subject term, so the
    // browser memory and the Rust solver agree on the reasoning shape.
    let response = answer("столица россии");
    let has_relation = response
        .evidence_links
        .iter()
        .any(|link| link == "fact_query:relation:capital");
    assert!(
        has_relation,
        "structured trace should record `fact_query:relation:capital`, got: {:?}",
        response.evidence_links,
    );
    let has_subject = response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("fact_query:subject:"));
    assert!(
        has_subject,
        "structured trace should record `fact_query:subject:*`, got: {:?}",
        response.evidence_links,
    );
    let has_cache_hit = response
        .evidence_links
        .iter()
        .any(|link| link == "fact_query:cache:hit:seed");
    assert!(
        has_cache_hit,
        "structured trace should record a seed cache hit, got: {:?}",
        response.evidence_links,
    );
}

const MULTI_TURN_COREFERENCE_PROMPTS: &[&str] = &[
    // After a previous "I love Rust." turn, this prompt should resolve "it".
    "What features make it different from C?",
    "How is it different from C?",
    "Why is it safer than C?",
    "Compare it with C.",
    "What makes it safer than C?",
];

#[test]
fn multi_turn_coreference_resolves_pronoun_against_history() {
    let solver = UniversalSolver::default();
    let history = [ConversationTurn::user("I love Rust.")];
    for prompt in MULTI_TURN_COREFERENCE_PROMPTS {
        let response = solver.solve_with_history(prompt, &history);
        assert!(
            response.intent.starts_with("coreference"),
            "prompt {prompt:?} should route to coreference*, got: {}",
            response.intent,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("prior_turn:")),
            "prompt {prompt:?} should reference a prior_turn evidence link, got: {:?}",
            response.evidence_links,
        );
    }
}

const ROLEPLAY_PROMPTS: &[&str] = &[
    "Pretend you are Albert Einstein and explain relativity to a teenager.",
    "Act as Albert Einstein and explain relativity simply.",
    "Roleplay as a teacher explaining relativity.",
    "Explain like you are Ada Lovelace teaching algorithms.",
    "Pretend you are a patient teacher and explain time dilation.",
];

#[test]
fn roleplay_intent_routes_to_roleplay_handler() {
    for prompt in ROLEPLAY_PROMPTS {
        let response = answer(prompt);
        assert!(
            response.intent.starts_with("roleplay"),
            "prompt {prompt:?} should route to a roleplay* intent, got: {}",
            response.intent,
        );
        if prompt.contains("Ada Lovelace") {
            assert!(
                response.answer.to_lowercase().contains("algorithm"),
                "prompt {prompt:?} should keep the Ada Lovelace topic grounded in algorithms, got: {}",
                response.answer,
            );
        }
    }
}
