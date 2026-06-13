use super::*;

#[test]
fn probability_evidence_reranks_formalization_across_supported_languages() {
    // Issue #449 extends `ProbabilityRankingConfig` (evidence count, counted
    // utility, transition thresholds) ported from arXiv:2605.00940. That config
    // now flows through the formalization selector in `src/translation/selection.rs`
    // for every supported language, not just English, so this pins that
    // language-facing path across en, ru, hi, and zh.
    struct LanguageCase {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        LanguageCase {
            language: "en",
            prompt: "apple is a fruit",
        },
        LanguageCase {
            language: "ru",
            prompt: "яблоко это фрукт",
        },
        LanguageCase {
            language: "hi",
            prompt: "सेब एक फल है",
        },
        LanguageCase {
            language: "zh",
            prompt: "苹果是水果",
        },
    ];

    let config = FormalizationSelectionConfig {
        temperature: 0.0,
        guess_probability: 1.0,
        questioning_rigor: 0.0,
    };

    for case in cases {
        let candidates = formalize_prompt_candidates(case.prompt, case.language);
        assert!(
            !candidates.is_empty(),
            "language={} should formalize at least one candidate",
            case.language
        );

        let baseline = select_formalization_candidate(&candidates, config, case.prompt);
        let baseline_index = baseline
            .selected_index()
            .expect("baseline should select a candidate");
        let baseline_target = formalization_probability_target(&candidates[baseline_index]);

        // Reinforce a different candidate when the prompt is ambiguous, otherwise
        // the only candidate. Either way the evidence must reach the selector
        // through the extended ranking config and drive the decision.
        let reinforced_index = if candidates.len() > 1 {
            (baseline_index + 1) % candidates.len()
        } else {
            baseline_index
        };
        let reinforced_target = formalization_probability_target(&candidates[reinforced_index]);

        // Two confirmations so the evidence count `C` is non-trivial, at a weight
        // high enough to dominate the structural prior under argmax.
        let mut store = ProbabilityStore::new();
        for recorded_at in ["2026-05-26T00:00:00Z", "2026-05-26T00:01:00Z"] {
            store.record(ProbabilityEvidence::symbolic(
                &reinforced_target,
                "reinforced_by_prior_dialog",
                5.0,
                "source:dialog:test",
                recorded_at,
            ));
        }
        assert_eq!(
            store.target_evidence_count(&reinforced_target, false, None),
            2,
            "language={} should accumulate the evidence count",
            case.language
        );

        let reranked = select_formalization_candidate_with_probability_store(
            &candidates,
            config,
            case.prompt,
            &store,
            false,
        );
        let reranked_target = formalization_probability_target(
            reranked
                .selected_candidate()
                .expect("evidence-backed selection should select a candidate"),
        );

        assert_eq!(
            reranked_target, reinforced_target,
            "language={} evidence should drive selection to the reinforced target",
            case.language
        );
        if reinforced_index != baseline_index {
            assert_ne!(
                reranked_target, baseline_target,
                "language={} strong evidence should flip away from the baseline",
                case.language
            );
        }
    }
}
