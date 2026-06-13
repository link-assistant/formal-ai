//! Reasoning-path tests: project-method documentation and how-to procedures.
//!
//! Extracted from `reasoning_paths.rs` to keep each file under the repository's
//! 1000-line limit. These tests exercise the same universal-solver loop and
//! event-log projection as the rest of the reasoning-path suite, covering the
//! docs-method-explanation handler (issue #223) and the source-backed how-to
//! procedure handler.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

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
// Issue #223: project-method documentation prompts should answer from the
// project's own docs, scoped to the named method.
// ---------------------------------------------------------------------------

#[test]
fn pandas_join_method_question_uses_official_docs_summary() {
    let response = answer("how the join method works in pandas");
    assert_eq!(
        response.intent, "docs_method_explanation",
        "pandas join method question must route to official-docs summary; answer={}",
        response.answer,
    );

    let answer = response.answer.to_lowercase();
    for expected in ["dataframe.join", "index", "other", "how"] {
        assert!(
            answer.contains(expected),
            "answer should mention {expected:?}; answer={}",
            response.answer,
        );
    }

    for expected in [
        "docs_method:project:pandas",
        "docs_method:method:pandas.DataFrame.join",
        "docs_method:source_kind:official-docs",
        "source:https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html",
    ] {
        assert!(
            has_evidence(&response, expected),
            "missing evidence prefix {expected:?}: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn pandas_join_method_docs_prompt_covers_supported_languages() {
    struct DocsMethodCase {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        DocsMethodCase {
            language: "en",
            prompt: "how the join method works in pandas",
        },
        DocsMethodCase {
            language: "ru",
            prompt: "объясни как работает метод join в pandas",
        },
        DocsMethodCase {
            language: "hi",
            prompt: "समझाओ pandas में join विधि कैसे काम करती है",
        },
        DocsMethodCase {
            language: "zh",
            prompt: "请解释 pandas 中的 join 方法如何工作 以及它如何使用索引",
        },
    ];

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "docs_method_explanation",
            "{} pandas join docs prompt must resolve; answer={}",
            case.language, response.answer,
        );
        assert!(
            response.answer.contains("DataFrame.join"),
            "answer should remain scoped to pandas.DataFrame.join for {}: {}",
            case.language,
            response.answer,
        );
        assert!(
            has_evidence(&response, &format!("language:{}", case.language)),
            "missing language evidence for {}: {:?}",
            case.language,
            response.evidence_links,
        );
        assert!(
            has_evidence(&response, "docs_method:method:pandas.DataFrame.join"),
            "missing docs-method evidence for {}: {:?}",
            case.language,
            response.evidence_links,
        );
    }
}

// ---------------------------------------------------------------------------
// Issue #172: procedural "how to X Y" prompts should discover source-backed
// procedure steps instead of returning the unknown fallback.
// ---------------------------------------------------------------------------

#[test]
fn how_to_make_tea_uses_source_backed_procedure_plan() {
    let response = answer("How to make tea?");
    assert_eq!(
        response.intent, "procedural_how_to",
        "\"How to make tea?\" must use the procedural handler; answer={}",
        response.answer,
    );

    let answer = response.answer.to_lowercase();
    for expected in [
        "make tea",
        "wikipedia",
        "wikidata",
        "wikihow",
        "web search",
        "recursive",
    ] {
        assert!(
            answer.contains(expected),
            "procedural answer should mention {expected:?}; answer={}",
            response.answer,
        );
    }

    for expected in [
        "procedural_how_to:request:make tea",
        "procedural_how_to:action:make",
        "procedural_how_to:object:tea",
        "procedural_how_to:stage:wikipedia",
        "procedural_how_to:stage:wikidata",
        "procedural_how_to:stage:wikihow_api",
        "http_fetch:request:https://www.wikihow.com/api.php",
        "web_search:request:how to make tea",
        "web_search:provider:wikipedia",
        "web_search:provider:wikidata",
        "procedural_how_to:stage:recursive_fetch_check",
    ] {
        assert!(
            has_evidence(&response, expected),
            "missing evidence prefix {expected:?}: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn how_to_prepare_fried_potatoes_falls_back_to_web_search() {
    let response = answer("How to prepare fried potatoes?");
    assert_eq!(
        response.intent, "procedural_how_to",
        "\"How to prepare fried potatoes?\" must use the procedural handler; answer={}",
        response.answer,
    );

    let answer = response.answer.to_lowercase();
    for expected in [
        "prepare fried potatoes",
        "fried potatoes",
        "fallback",
        "fetch",
    ] {
        assert!(
            answer.contains(expected),
            "procedural answer should mention {expected:?}; answer={}",
            response.answer,
        );
    }

    for expected in [
        "procedural_how_to:request:prepare fried potatoes",
        "procedural_how_to:action:prepare",
        "procedural_how_to:object:fried potatoes",
        "procedural_how_to:wikihow_candidate:Prepare-Fried-Potatoes",
        "web_search:request:how to prepare fried potatoes",
        "procedural_how_to:stage:recursive_fetch_check",
    ] {
        assert!(
            has_evidence(&response, expected),
            "missing evidence prefix {expected:?}: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn how_to_procedure_is_general_not_memoized_to_examples() {
    let response = answer("How can I calibrate a torque wrench?");
    assert_eq!(
        response.intent, "procedural_how_to",
        "arbitrary procedural prompts must not fall back to unknown; answer={}",
        response.answer,
    );

    let answer = response.answer.to_lowercase();
    assert!(answer.contains("calibrate a torque wrench"));
    assert!(
        !answer.contains("make tea") && !answer.contains("fried potatoes"),
        "answer must be generated from the requested task, not memoized examples: {}",
        response.answer,
    );

    for expected in [
        "procedural_how_to:request:calibrate a torque wrench",
        "procedural_how_to:action:calibrate",
        "procedural_how_to:object:a torque wrench",
        "web_search:request:how to calibrate a torque wrench",
    ] {
        assert!(
            has_evidence(&response, expected),
            "missing evidence prefix {expected:?}: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn spec_driven_typo_how_to_prompts_cover_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    for case in [
        Case {
            language: "en",
            prompt: "How to do SPEC dirven development step by step?",
        },
        Case {
            language: "ru",
            prompt: "как сделать SPEC dirven development? напиши по шагам",
        },
        Case {
            language: "hi",
            prompt: "कैसे करें SPEC dirven development? चरणों में बताओ",
        },
        Case {
            language: "zh",
            prompt: "如何做 SPEC dirven development？按步骤写",
        },
    ] {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "procedural_how_to",
            "{} how-to prompt must not fall back to unknown; answer={}",
            case.language, response.answer,
        );

        let answer = response.answer.to_lowercase();
        for expected in ["spec driven development", "wikihow", "web search"] {
            assert!(
                answer.contains(expected),
                "{} procedural answer should mention {expected:?}; answer={}",
                case.language,
                response.answer,
            );
        }

        for expected in [
            "procedural_how_to:request:spec driven development",
            "procedural_how_to:action:do",
            "procedural_how_to:object:spec driven development",
            "spelling_correction:dirven->driven",
            "web_search:request:how to spec driven development",
        ] {
            assert!(
                has_evidence(&response, expected),
                "{} missing evidence prefix {expected:?}: {:?}",
                case.language,
                response.evidence_links,
            );
        }
    }
}
