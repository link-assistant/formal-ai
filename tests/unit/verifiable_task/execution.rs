//! Issue #1138 B8 (plan 08, L9–L11): the program runs, and its stdout is the answer.
//!
//! What separates this route from a pattern table is that something is actually
//! executed: the composed program is run and what it printed is the answer. The
//! self-checks are observations, not adjectives — an answer that only *ran* is
//! reported as computed-but-uncorroborated, "verified" is reserved for agreement
//! or a round trip, and two disjoint derivations that disagree produce a gap
//! rather than a guess.

use formal_ai::FormalAiEngine;

use super::{LANGUAGES, case};

fn trailing_number(answer: &str) -> Option<String> {
    let digits: String = answer
        .chars()
        .rev()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    if digits.is_empty() {
        None
    } else {
        Some(digits.chars().rev().collect())
    }
}

/// The trace shows the run, and the value the program printed is the value the
/// answer carries.
#[test]
fn the_composed_program_is_executed_and_its_output_is_the_answer() {
    for language in LANGUAGES {
        let paraphrase = case("arithmetic_narrative", language);
        let response = FormalAiEngine.answer(&paraphrase.prompt);

        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("verifiable_task:executed")),
            "{language}: the trace must show the composed program being run, got {:?}",
            response.evidence_links
        );
        let value = trailing_number(&response.answer).unwrap_or_else(|| {
            panic!(
                "{language}: a numeric expectation answers with a number: {}",
                response.answer
            )
        });
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.ends_with(&value) || link.contains(&value)),
            "{language}: the answer's value must be the observed stdout, not a separate claim"
        );
    }
}

/// When the route cannot corroborate a value it reports the gap. A guess is
/// never returned, and the research trail says what was tried.
#[test]
fn two_independent_derivations_that_disagree_yield_a_gap() {
    for language in LANGUAGES {
        let paraphrase = case("honest_gap", language);
        let response = FormalAiEngine.answer(&paraphrase.prompt);

        assert!(
            trailing_number(&response.answer).is_none(),
            "{language}: an uncorroborated derivation may not present a number: {}",
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("verifiable_task:gap")),
            "{language}: the gap is stated explicitly, got {:?}",
            response.evidence_links
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("verifiable_task:searched")),
            "{language}: the research trail names what was searched"
        );
    }
}

/// "Ran" is the weakest check there is. An answer that passed only that one is
/// labelled computed-but-uncorroborated and never called verified.
#[test]
fn an_answer_with_only_the_ran_check_is_not_called_verified() {
    for language in LANGUAGES {
        let paraphrase = case("unit_conversion", language);
        let response = FormalAiEngine.answer(&paraphrase.prompt);

        let checks: Vec<&String> = response
            .evidence_links
            .iter()
            .filter(|link| link.starts_with("verifiable_task:check:"))
            .collect();
        assert!(
            !checks.is_empty(),
            "{language}: every returned answer names the checks it passed, got {:?}",
            response.evidence_links
        );

        let corroborated = checks.iter().any(|link| {
            link.ends_with("agreement") || link.ends_with("round_trip")
        });
        if !corroborated {
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link.starts_with("verifiable_task:uncorroborated")),
                "{language}: an answer that only ran must be labelled uncorroborated, got {checks:?}"
            );
        }
    }
}

/// For a named unknown the round trip is available: substituting the value back
/// into the stated equation must satisfy both sides.
#[test]
fn an_unknown_value_round_trips_through_its_equation() {
    for language in LANGUAGES {
        let paraphrase = case("named_unknown", language);
        let response = FormalAiEngine.answer(&paraphrase.prompt);

        assert!(
            response.answer.contains("12"),
            "{language}: 7 * y = 84 has one solution and the answer must carry it: {}",
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("verifiable_task:check:round_trip")),
            "{language}: the round trip is the check that makes this answer verified, got {:?}",
            response.evidence_links
        );
    }
}
