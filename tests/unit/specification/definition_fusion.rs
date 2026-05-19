//! Cross-language definition fusion tests.
//!
//! Issue #63 asks formal-ai to go beyond a single Wikipedia summary by
//! combining definitions for the same concept across language editions. The
//! implemented contract is intentionally deterministic: merge only concept
//! records that share the same seed/Wikidata anchor, preserve source-language
//! labels, deduplicate exact repeated facts, and expose the provenance in the
//! evidence links.

use formal_ai::{FormalAiEngine, SolverConfig, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn answer_with_config(prompt: &str, config: SolverConfig) -> SymbolicAnswer {
    UniversalSolver::new(config).solve(prompt)
}

#[derive(Debug)]
struct DefinitionMergeExample<'a> {
    prompt: &'a str,
    merged_title: &'a str,
    source_languages: &'a [&'a str],
    wikidata: Option<&'a str>,
    expected_facts: &'a [(&'a str, &'a str)],
    expected_sources: &'a [(&'a str, &'a str)],
}

#[test]
fn definition_merge_examples_show_exact_behavior_across_terms_and_concepts() {
    let examples = [
        DefinitionMergeExample {
            prompt: "Merge Wikipedia definitions of IIR",
            merged_title: "infinite impulse response (IIR)",
            source_languages: &["en", "ru", "hi", "zh"],
            wikidata: Some("Q740073"),
            expected_facts: &[
                ("en", "recursive digital filter"),
                ("ru", "Фильтр с бесконечной импульсной характеристикой"),
                ("hi", "पुनरावर्ती डिजिटल फ़िल्टर"),
                ("zh", "递归型数字滤波器"),
            ],
            expected_sources: &[
                ("en", "https://en.wikipedia.org/wiki/Infinite_impulse_response"),
                (
                    "ru",
                    "https://ru.wikipedia.org/wiki/Фильтр_с_бесконечной_импульсной_характеристикой",
                ),
                ("hi", "https://hi.wikipedia.org/wiki/अनंत_आवेग_प्रतिक्रिया"),
                (
                    "zh",
                    "https://zh.wikipedia.org/wiki/%E6%97%A0%E9%99%90%E8%84%89%E5%86%B2%E5%93%8D%E5%BA%94",
                ),
            ],
        },
        DefinitionMergeExample {
            prompt: "Combine translated definitions for infinite impulse response",
            merged_title: "infinite impulse response (IIR)",
            source_languages: &["en", "ru", "hi", "zh"],
            wikidata: Some("Q740073"),
            expected_facts: &[("en", "non-zero over an infinite length of time")],
            expected_sources: &[(
                "en",
                "https://en.wikipedia.org/wiki/Infinite_impulse_response",
            )],
        },
        DefinitionMergeExample {
            prompt: "Fuse Wikipedia definitions of IIR filter across languages",
            merged_title: "infinite impulse response (IIR)",
            source_languages: &["en", "ru", "hi", "zh"],
            wikidata: Some("Q740073"),
            expected_facts: &[("en", "finite impulse response (FIR) filter")],
            expected_sources: &[(
                "en",
                "https://en.wikipedia.org/wiki/Infinite_impulse_response",
            )],
        },
        DefinitionMergeExample {
            prompt: "Merge translations for БИХ-фильтр using Wikipedia",
            merged_title: "infinite impulse response (IIR)",
            source_languages: &["en", "ru", "hi", "zh"],
            wikidata: Some("Q740073"),
            expected_facts: &[("ru", "образующий обратную связь")],
            expected_sources: &[(
                "ru",
                "https://ru.wikipedia.org/wiki/Фильтр_с_бесконечной_импульсной_характеристикой",
            )],
        },
        DefinitionMergeExample {
            prompt: "Merge Wikipedia definitions of color",
            merged_title: "color",
            source_languages: &["en", "ru", "hi", "zh"],
            wikidata: Some("Q1075"),
            expected_facts: &[
                ("en", "visual perceptual property"),
                ("ru", "электромагнитного излучения"),
                ("hi", "प्रकाश के विभिन्न तरंगदैर्घ्य"),
                ("zh", "不同波长光波"),
            ],
            expected_sources: &[
                ("en", "https://en.wikipedia.org/wiki/Color"),
                ("ru", "https://ru.wikipedia.org/wiki/Цвет"),
                ("hi", "https://hi.wikipedia.org/wiki/रंग"),
                ("zh", "https://zh.wikipedia.org/wiki/颜色"),
            ],
        },
        DefinitionMergeExample {
            prompt: "Combine translated definitions for colour",
            merged_title: "color",
            source_languages: &["en", "ru", "hi", "zh"],
            wikidata: Some("Q1075"),
            expected_facts: &[("en", "red, green, blue")],
            expected_sources: &[("en", "https://en.wikipedia.org/wiki/Color")],
        },
        DefinitionMergeExample {
            prompt: "Fuse translated definitions of रंग across languages",
            merged_title: "color",
            source_languages: &["en", "ru", "hi", "zh"],
            wikidata: Some("Q1075"),
            expected_facts: &[("hi", "सामान्य रंगों के उदाहरण")],
            expected_sources: &[("hi", "https://hi.wikipedia.org/wiki/रंग")],
        },
        DefinitionMergeExample {
            prompt: "Merge Wikipedia definitions of 颜色",
            merged_title: "color",
            source_languages: &["en", "ru", "hi", "zh"],
            wikidata: Some("Q1075"),
            expected_facts: &[("zh", "红色、橙色、黄色")],
            expected_sources: &[("zh", "https://zh.wikipedia.org/wiki/颜色")],
        },
        DefinitionMergeExample {
            prompt: "Merge Wikipedia definitions of KISS principle",
            merged_title: "KISS principle",
            source_languages: &["en", "ru"],
            wikidata: Some("Q649540"),
            expected_facts: &[
                ("en", "simplicity as a primary goal"),
                ("ru", "Не усложняй без необходимости"),
            ],
            expected_sources: &[
                ("en", "https://en.wikipedia.org/wiki/KISS_principle"),
                (
                    "ru",
                    "https://ru.wikipedia.org/wiki/KISS_(%D0%BF%D1%80%D0%B8%D0%BD%D1%86%D0%B8%D0%BF)",
                ),
            ],
        },
        DefinitionMergeExample {
            prompt: "Combine translated definitions for keep it simple, stupid",
            merged_title: "KISS principle",
            source_languages: &["en", "ru"],
            wikidata: Some("Q649540"),
            expected_facts: &[("en", "cornerstone of software engineering")],
            expected_sources: &[("en", "https://en.wikipedia.org/wiki/KISS_principle")],
        },
        DefinitionMergeExample {
            prompt: "Merge definitions of Links theory",
            merged_title: "Links meta-theory",
            source_languages: &["en", "ru"],
            wikidata: None,
            expected_facts: &[
                ("en", "compact set-theory projection"),
                ("ru", "представление информации через связи"),
            ],
            expected_sources: &[
                ("en", "https://github.com/link-foundation/meta-theory"),
                ("ru", "https://github.com/link-foundation/meta-theory"),
            ],
        },
        DefinitionMergeExample {
            prompt: "Combine translated definitions for теория связей",
            merged_title: "Links meta-theory",
            source_languages: &["en", "ru"],
            wikidata: None,
            expected_facts: &[("ru", "архивированные статьи о Links Theory")],
            expected_sources: &[("ru", "https://github.com/link-foundation/meta-theory")],
        },
        DefinitionMergeExample {
            prompt: "Merge definitions of Telegram Ads",
            merged_title: "Telegram Ads",
            source_languages: &["en", "ru"],
            wikidata: None,
            expected_facts: &[
                ("en", "native advertising platform"),
                ("ru", "официальная рекламная платформа Telegram"),
            ],
            expected_sources: &[
                ("en", "https://promote.telegram.org"),
                ("ru", "https://promote.telegram.org"),
            ],
        },
        DefinitionMergeExample {
            prompt: "Combine translated definitions for telegram advertising",
            merged_title: "Telegram Ads",
            source_languages: &["en", "ru"],
            wikidata: None,
            expected_facts: &[("en", "public channels with over 1,000 subscribers")],
            expected_sources: &[("en", "https://promote.telegram.org")],
        },
        DefinitionMergeExample {
            prompt: "Merge definitions of реклама в Telegram",
            merged_title: "Telegram Ads",
            source_languages: &["en", "ru"],
            wikidata: None,
            expected_facts: &[("ru", "минимальный бюджет")],
            expected_sources: &[("ru", "https://promote.telegram.org")],
        },
    ];

    assert!(
        (10..=20).contains(&examples.len()),
        "review requested 10-20 concrete examples; got {}",
        examples.len()
    );

    for example in examples {
        assert_definition_merge_example(&example);
    }
}

#[test]
fn definition_merge_keeps_shared_anchor_and_source_evidence() {
    let response = answer("Combine translated definitions for IIR");

    assert_eq!(response.intent, "definition_merge");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q740073"),
        "merged definition should preserve the shared Wikidata anchor: {:?}",
        response.evidence_links
    );
    assert!(
        response.evidence_links.iter().any(|link| link
            .starts_with("source:http:https://en.wikipedia.org/wiki/Infinite_impulse_response")),
        "merged definition should cite the English Wikipedia source: {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("definition_merge:language:")),
        "merged definition should record language-level fusion events: {:?}",
        response.evidence_links
    );
}

#[test]
fn definition_merge_unknown_term_falls_back_to_unknown_intent() {
    let response = answer("Merge Wikipedia definitions of not-a-seeded-concept");

    assert_eq!(
        response.intent, "unknown",
        "unknown concepts should not fabricate merged definition output: {}",
        response.answer
    );
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("definition_merge:hit:")),
        "unknown concepts should not record a definition-merge hit: {:?}",
        response.evidence_links
    );
}

#[test]
fn definition_fusion_by_default_merges_plain_definition_prompts_when_enabled() {
    let config = SolverConfig {
        definition_fusion_by_default: true,
        ..Default::default()
    };

    let response = answer_with_config("What is IIR?", config);

    assert_eq!(response.intent, "definition_merge");
    assert!(
        response
            .answer
            .contains("Merged definition of infinite impulse response (IIR)"),
        "plain definition prompts should use merged output when the setting is enabled: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("definition_merge:mode:auto:")),
        "auto-fused prompt should leave mode evidence: {:?}",
        response.evidence_links
    );
}

#[test]
fn definition_fusion_by_default_preserves_contextual_concept_lookup() {
    let config = SolverConfig {
        definition_fusion_by_default: true,
        ..Default::default()
    };

    let response = answer_with_config("what is IIR in ML?", config);

    assert_eq!(response.intent, "concept_lookup_in_context");
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("definition_merge:mode:auto")),
        "contextual questions should not be auto-fused: {:?}",
        response.evidence_links
    );
}

fn assert_definition_merge_example(example: &DefinitionMergeExample<'_>) {
    let response = answer(example.prompt);

    assert_eq!(
        response.intent, "definition_merge",
        "prompt should route to definition_merge: {}",
        example.prompt
    );
    assert!(
        response
            .answer
            .contains(&format!("Merged definition of {}", example.merged_title)),
        "merged answer should name the resolved concept for prompt {:?}: {}",
        example.prompt,
        response.answer
    );
    assert_eq!(
        answer_source_languages(&response.answer),
        example.source_languages,
        "merged answer should disclose the exact source-language set for prompt {:?}: {}",
        example.prompt,
        response.answer
    );
    for (language, fact) in example.expected_facts {
        assert!(
            answer_has_fact(&response.answer, language, fact),
            "merged answer should include a {language} fact containing {:?} for prompt {:?}: {}",
            fact,
            example.prompt,
            response.answer
        );
    }
    for (language, source) in example.expected_sources {
        assert!(
            answer_has_source(&response.answer, language, source),
            "merged answer should include a {language} source {:?} for prompt {:?}: {}",
            source,
            example.prompt,
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("source:http:{source}")),
            "merged answer should expose source evidence {:?} for prompt {:?}: {:?}",
            source,
            example.prompt,
            response.evidence_links
        );
    }
    for language in example.source_languages {
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("definition_merge:language:{language}")),
            "merged answer should expose language evidence {language:?} for prompt {:?}: {:?}",
            example.prompt,
            response.evidence_links
        );
    }
    if let Some(qid) = example.wikidata {
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("wikidata:{qid}")),
            "merged answer should preserve Wikidata anchor {qid} for prompt {:?}: {:?}",
            example.prompt,
            response.evidence_links
        );
    }
}

fn answer_source_languages(answer: &str) -> Vec<&str> {
    answer
        .lines()
        .find_map(|line| line.strip_prefix("Source languages: "))
        .unwrap_or("")
        .split(", ")
        .filter(|language| !language.is_empty())
        .collect()
}

fn answer_has_fact(answer: &str, language: &str, text: &str) -> bool {
    answer.lines().any(|line| {
        line.strip_prefix(&format!("- [{language}] "))
            .is_some_and(|fact| fact.contains(text))
    })
}

fn answer_has_source(answer: &str, language: &str, source: &str) -> bool {
    answer.lines().any(|line| {
        line.strip_prefix(&format!("- [{language}] "))
            .is_some_and(|line_source| line_source.contains(source))
    })
}
