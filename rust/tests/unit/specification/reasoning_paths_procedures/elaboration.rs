use formal_ai::{ConversationTurn, UniversalSolver};

use super::has_evidence;

// ---------------------------------------------------------------------------
// Issue #444: after a "how to X" turn, a follow-up that asks for the concrete
// instructions ("Can you give me specific instructions?") must rebind to the
// active procedure instead of falling through to the unknown opener. The user
// reported exactly this: "how to publish to npm" answered, then "Can you give
// me specific instructions?" returned "Unknown prompt".
// ---------------------------------------------------------------------------

pub(super) fn procedural_elaboration_followup_rebinds_to_prior_how_to() {
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
pub(super) fn procedural_elaboration_requires_a_prior_how_to() {
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
pub(super) fn procedural_elaboration_followup_covers_supported_languages() {
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
pub(super) fn procedural_elaboration_followup_covers_many_topics() {
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
