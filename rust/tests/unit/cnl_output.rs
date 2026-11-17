//! These tests cover the declared lexical corpus, not general CNL conformance.
use ::formal_ai::cnl_output::*;

#[test]
fn declared_multilingual_meanings_round_trip_without_certifying_unchecked_standards() {
    let policy = cnl_policy();
    for (language, simple, dense) in [
        ("en", "I know everything.", "I am omniscient."),
        ("ru", "Я знаю всё.", "Я всеведущ."),
        ("hi", "मैं सब कुछ जानता हूँ।", "मैं सर्वज्ञ हूँ।"),
        ("zh", "我知道一切。", "我无所不知。"),
        // Spanish participates in the same realization and profile matrix.
        ("es", "Yo sé todo.", "Soy omnisciente."),
    ] {
        let document = formalize_output(simple, language, policy);
        let graph = output_links_notation(&document);
        let configuration = output_configuration(None, Some("max-precision"), policy).unwrap();
        let rendered = render_output(&document, configuration, policy, &());
        assert_eq!(rendered.text, dense);
        assert!(!rendered.notice.is_empty());
        assert_eq!(output_source(&rendered.content, language, policy), dense);
        assert_eq!(rendered.links_notation, graph);
        let dense_document = formalize_output(dense, language, policy);
        for profile in &policy.profiles {
            for density in ["standard", "primes", "top-37", "max-precision"] {
                let configuration =
                    output_configuration(Some(&profile.id), Some(density), policy).unwrap();
                let projected = project_output(&dense_document, configuration, policy);
                assert_eq!(
                    output_links_notation(&formalize_output(&projected, language, policy)),
                    graph
                );
                {
                    assert!(
                        !validate_output(&projected, language, configuration, policy, &())
                            .compliant
                    );
                }
            }
        }
        let simple = project_output(&dense_document, OutputConfig::default(), policy);
        assert_eq!(
            output_links_notation(&formalize_output(&simple, language, policy)),
            graph
        );
    }
}

#[test]
fn lexical_density_preserves_known_named_profile_constraints() {
    let policy = cnl_policy();
    let document = formalize_output("I am omniscient.", "en", policy);
    for density in ["max-precision", "top-200"] {
        let configuration = output_configuration(Some("e-prime"), Some(density), policy).unwrap();
        let rendered = render_output(&document, configuration, policy, &());
        assert_eq!(rendered.text, "I know everything.");
        assert!(rendered.report.violations.is_empty());
        assert!(!rendered.report.compliant);
        assert_eq!(
            output_links_notation(&formalize_output(&rendered.text, "en", policy)),
            rendered.links_notation
        );
    }
}

#[test]
fn opaque_input_and_configuration_limits_are_lossless_on_every_target() {
    let policy = cnl_policy();
    for input in [
        "I do not know everything.",
        "I know everything except the password.",
        "`I am omniscient.`",
        "uhh?? nope lol 🤔",
        "Not every person knows everything.",
    ] {
        let document = formalize_output(input, "en", policy);
        for profile in &policy.profiles {
            let configuration =
                output_configuration(Some(&profile.id), Some("max-precision"), policy).unwrap();
            assert_eq!(project_output(&document, configuration, policy), input);
        }
    }
    assert_eq!(
        frequency_tier("top-9007199254740991"),
        Some(9_007_199_254_740_991)
    );
    for invalid in [
        "top-0",
        "top-01",
        "top-1.5",
        "top-1e3",
        "top-9007199254740992",
    ] {
        assert!(output_configuration(None, Some(invalid), policy).is_err());
    }
    assert_eq!(output_words("मैं सब कुछ जानता हूँ।", "hi").len(), 5);
    assert_eq!(output_words("我知道。", "zh").len(), 3);
}

#[test]
fn e_prime_contractions_and_missing_frequency_evidence_do_not_receive_certificates() {
    let policy = cnl_policy();
    let configuration = output_configuration(Some("e-prime"), None, policy).unwrap();
    for text in [
        "I’m here.",
        "John's tired.",
        "John's hat fell.",
        "I ain't ready.",
        "Y'all're here.",
        "’Tis ready.",
        "Thou wast here.",
        "Thou beest wise.",
    ] {
        assert!(!validate_output(text, "en", configuration, policy, &()).compliant);
    }
    let report = validate_output("I know this.", "en", configuration, policy, &());
    assert!(!report.compliant);
    assert!(report.unchecked.contains(&"e_prime_english".to_owned()));
    let configuration = output_configuration(Some("e-prime"), Some("top-1"), policy).unwrap();
    let report = validate_output("Alice sees Bob.", "en", configuration, policy, &());
    assert!(!report.compliant);
    assert!(
        report
            .unchecked
            .contains(&"frequency_vocabulary".to_owned())
    );
    for invalid in [
        r#"{"output_standard":"unknown"}"#,
        r#"{"lexical_density":"top-0"}"#,
    ] {
        assert!(
            serde_json::from_str::<::formal_ai::protocol::ChatCompletionRequest>(invalid).is_err()
        );
    }
}
