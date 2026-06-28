//! Reasoning-path tests: project-method documentation and how-to procedures.
//!
//! Extracted from `reasoning_paths.rs` to keep each file under the repository's
//! 1000-line limit. These tests exercise the same universal-solver loop and
//! event-log projection as the rest of the reasoning-path suite, covering the
//! docs-method-explanation handler (issue #223) and the source-backed how-to
//! procedure handler.

use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

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
fn telegraphic_how_order_prompt_routes_to_procedural_plan() {
    let response = answer("how order 3d print in nan chang vietnam?");
    assert_eq!(
        response.intent, "procedural_how_to",
        "telegraphic \"how order ...\" prompt must use the procedural handler; answer={}",
        response.answer,
    );

    let answer = response.answer.to_lowercase();
    for expected in ["order 3d print", "web search", "wikihow"] {
        assert!(
            answer.contains(expected),
            "procedural answer should mention {expected:?}; answer={}",
            response.answer,
        );
    }

    for expected in [
        "procedural_how_to:request:order 3d print in nan chang vietnam",
        "procedural_how_to:action:order",
        "procedural_how_to:object:3d print in nan chang vietnam",
        "web_search:request:how to order 3d print in nan chang vietnam",
    ] {
        assert!(
            has_evidence(&response, expected),
            "missing evidence prefix {expected:?}: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn telegraphic_how_requires_a_known_procedural_action() {
    let response = answer("how glorp widgets?");
    assert_ne!(
        response.intent, "procedural_how_to",
        "unknown verbs after bare \"how\" must not be promoted to procedures; answer={}",
        response.answer,
    );
}

#[test]
fn procedural_request_surfaces_stay_multilingual_after_elided_gate() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    for case in [
        Case {
            language: "en",
            prompt: "how to do 3d print ordering",
        },
        Case {
            language: "ru",
            prompt: "как сделать 3d print ordering",
        },
        Case {
            language: "hi",
            prompt: "कैसे करें 3d print ordering",
        },
        Case {
            language: "zh",
            prompt: "如何做 3d print ordering",
        },
    ] {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "procedural_how_to",
            "{} procedural surface should still route after the elided gate; answer={}",
            case.language, response.answer,
        );

        for expected in [
            "procedural_how_to:request:3d print ordering",
            "procedural_how_to:action:do",
            "procedural_how_to:object:3d print ordering",
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

// ---------------------------------------------------------------------------
// Issue #444: after a "how to X" turn, a follow-up that asks for the concrete
// instructions ("Can you give me specific instructions?") must rebind to the
// active procedure instead of falling through to the unknown opener. The user
// reported exactly this: "how to publish to npm" answered, then "Can you give
// me specific instructions?" returned "Unknown prompt".
// ---------------------------------------------------------------------------

#[test]
fn procedural_elaboration_followup_rebinds_to_prior_how_to() {
    let solver = UniversalSolver::default();
    let how_to_prompt = "how to publish to npm";
    let plan = solver.solve(how_to_prompt);
    assert_eq!(
        plan.intent, "procedural_how_to",
        "setup: the how-to turn must route to the procedural handler; answer={}",
        plan.answer,
    );

    let history = [
        ConversationTurn::user(how_to_prompt),
        ConversationTurn::assistant(plan.answer),
    ];
    let follow_up = solver.solve_with_history("Can you give me specific instructions?", &history);

    assert_eq!(
        follow_up.intent, "procedural_how_to",
        "elaboration follow-up must rebind to the procedure, not fall to unknown; answer={}",
        follow_up.answer,
    );
    let lowered = follow_up.answer.to_lowercase();
    assert!(
        lowered.contains("publish to npm"),
        "follow-up must restate the recovered task: {}",
        follow_up.answer,
    );
    for expected in [
        "procedural_how_to:followup",
        "procedural_how_to:request:publish to npm",
        "web_search:request:how to publish to npm",
    ] {
        assert!(
            has_evidence(&follow_up, expected),
            "missing evidence prefix {expected:?}: {:?}",
            follow_up.evidence_links,
        );
    }
}

// The elaboration follow-up must only fire while a how-to procedure is on the
// table; a bare "give me specific instructions" with no prior procedure stays a
// normal prompt and never spoofs a procedural answer.
#[test]
fn procedural_elaboration_requires_a_prior_how_to() {
    let solver = UniversalSolver::default();
    let answer = solver.solve("Can you give me specific instructions?");
    assert_ne!(
        answer.intent, "procedural_how_to",
        "no prior procedure means no procedural rebind; answer={}",
        answer.answer,
    );
}

// Parity across languages: a user who asks the how-to in one language and then
// requests the concrete steps (in the same or another supported language) must
// still rebind to the procedure rather than fall to unknown.
#[test]
fn procedural_elaboration_followup_covers_supported_languages() {
    struct Case {
        language: &'static str,
        how_to: &'static str,
        elaboration: &'static str,
        task_fragment: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            how_to: "how to publish to npm",
            elaboration: "give me the exact steps",
            task_fragment: "publish to npm",
        },
        Case {
            language: "ru",
            how_to: "как сделать чай",
            elaboration: "дай конкретные инструкции",
            task_fragment: "чай",
        },
        Case {
            language: "hi",
            how_to: "कैसे करें SPEC development",
            elaboration: "विस्तृत निर्देश दो",
            task_fragment: "spec development",
        },
        Case {
            language: "zh",
            how_to: "如何做 SPEC development",
            elaboration: "给我具体步骤",
            task_fragment: "spec development",
        },
    ];

    for case in cases {
        let solver = UniversalSolver::default();
        let plan = solver.solve(case.how_to);
        assert_eq!(
            plan.intent, "procedural_how_to",
            "{} setup how-to must route procedurally; answer={}",
            case.language, plan.answer,
        );
        let history = [
            ConversationTurn::user(case.how_to),
            ConversationTurn::assistant(plan.answer),
        ];
        let follow_up = solver.solve_with_history(case.elaboration, &history);
        assert_eq!(
            follow_up.intent, "procedural_how_to",
            "{} elaboration {:?} must rebind to the procedure; answer={}",
            case.language, case.elaboration, follow_up.answer,
        );
        assert!(
            follow_up.answer.to_lowercase().contains(case.task_fragment),
            "{} follow-up should restate {:?}; answer={}",
            case.language,
            case.task_fragment,
            follow_up.answer,
        );
        assert!(
            has_evidence(&follow_up, "procedural_how_to:followup"),
            "{} missing followup evidence: {:?}",
            case.language,
            follow_up.evidence_links,
        );
    }
}

// Breadth across topics in the same scope: the maintainer asked us to cover
// "x10 more test cases, with different topics in the same scope". The rebind is
// fully generic — it recovers the prior task from the conversation regardless of
// subject — so a single data-driven table exercises a dozen unrelated how-to
// domains (cooking, vehicle maintenance, networking, gardening, finance, …) and
// a variety of elaboration phrasings. Each must rebind to its own procedure and
// emit the followup evidence, with no cross-talk between topics.
#[test]
fn procedural_elaboration_followup_covers_many_topics() {
    struct Case {
        how_to: &'static str,
        elaboration: &'static str,
        task_fragment: &'static str,
    }

    let cases = [
        Case {
            how_to: "how to publish to npm",
            elaboration: "Can you give me specific instructions?",
            task_fragment: "publish to npm",
        },
        Case {
            how_to: "how to bake sourdough bread",
            elaboration: "give me the exact steps",
            task_fragment: "bake sourdough bread",
        },
        Case {
            how_to: "how to change a car tire",
            elaboration: "the steps please",
            task_fragment: "change a car tire",
        },
        Case {
            how_to: "how to set up a home wifi network",
            elaboration: "more details please",
            task_fragment: "set up a home wifi network",
        },
        Case {
            how_to: "how to brew espresso",
            elaboration: "explain it step by step",
            task_fragment: "brew espresso",
        },
        Case {
            how_to: "how to write a resume",
            elaboration: "give me detailed instructions",
            task_fragment: "write a resume",
        },
        Case {
            how_to: "how to train a puppy",
            elaboration: "be more specific",
            task_fragment: "train a puppy",
        },
        Case {
            how_to: "how to file a tax return",
            elaboration: "give me the specific steps",
            task_fragment: "file a tax return",
        },
        Case {
            how_to: "how to plant a tree",
            elaboration: "give me the exact instructions",
            task_fragment: "plant a tree",
        },
        Case {
            how_to: "how to tie a tie",
            elaboration: "step-by-step please",
            task_fragment: "tie a tie",
        },
        Case {
            how_to: "how to start a podcast",
            elaboration: "give me specific steps",
            task_fragment: "start a podcast",
        },
        Case {
            how_to: "how to meditate",
            elaboration: "explain in detail",
            task_fragment: "meditate",
        },
    ];

    assert!(
        cases.len() >= 10,
        "issue #444 asks for at least ten additional topic cases; have {}",
        cases.len(),
    );

    for case in cases {
        let solver = UniversalSolver::default();
        let plan = solver.solve(case.how_to);
        assert_eq!(
            plan.intent, "procedural_how_to",
            "setup how-to {:?} must route procedurally; answer={}",
            case.how_to, plan.answer,
        );
        let history = [
            ConversationTurn::user(case.how_to),
            ConversationTurn::assistant(plan.answer),
        ];
        let follow_up = solver.solve_with_history(case.elaboration, &history);
        assert_eq!(
            follow_up.intent, "procedural_how_to",
            "elaboration {:?} after {:?} must rebind to the procedure; answer={}",
            case.elaboration, case.how_to, follow_up.answer,
        );
        assert!(
            follow_up.answer.to_lowercase().contains(case.task_fragment),
            "follow-up after {:?} should restate {:?}; answer={}",
            case.how_to,
            case.task_fragment,
            follow_up.answer,
        );
        assert!(
            has_evidence(&follow_up, "procedural_how_to:followup"),
            "missing followup evidence for {:?}: {:?}",
            case.how_to,
            follow_up.evidence_links,
        );
        assert!(
            has_evidence(
                &follow_up,
                &format!("procedural_how_to:request:{}", case.task_fragment),
            ),
            "follow-up after {:?} should recover the exact task; evidence={:?}",
            case.how_to,
            follow_up.evidence_links,
        );
    }
}
