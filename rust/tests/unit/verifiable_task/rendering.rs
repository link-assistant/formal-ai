//! Issue #1138 B8 (plan 08, L3/L6): expectations bridge to evidence and
//! rendering obeys the task's declared answer shape.

use formal_ai::obligation_ledger::ObligationExpectation;
use formal_ai::verifiable_task::{AnswerShape, TaskExpectation, recognise_verifiable};

use super::{LANGUAGES, case};

#[test]
fn every_answer_expectation_becomes_one_symbolic_observation() {
    for expectation in [
        TaskExpectation::Numeric { unit: None },
        TaskExpectation::Count {
            subject: String::from("subject"),
        },
        TaskExpectation::EditedText {
            source: String::from("source"),
        },
        TaskExpectation::Unknown {
            name: String::from("x"),
        },
        TaskExpectation::Callable {
            examples: Vec::new(),
        },
        TaskExpectation::Stdout { expected: None },
        TaskExpectation::Boolean,
    ] {
        let check_id = format!("derivation:{}", expectation.slug());
        assert_eq!(
            expectation.to_obligation_expectation(&check_id),
            ObligationExpectation::SymbolicCheck { check_id },
            "an answer shape is settled only by the generated symbolic check"
        );
    }
}

#[test]
fn rendering_obeys_the_declared_shape_in_every_supported_language() {
    for language in LANGUAGES {
        let numeric = recognise_verifiable(&case("arithmetic_narrative", language).prompt)
            .expect("the numeric case is recognised");
        assert_eq!(numeric.shape, AnswerShape::TrailingNumber);
        let rendered = numeric.render("3", "derived");
        assert!(
            rendered.trim_end().ends_with('3'),
            "{language}: a trailing-number answer must end in its observed value: {rendered:?}"
        );

        let edited = recognise_verifiable(&case("instructed_edit", language).prompt)
            .expect("the edit case is recognised");
        assert_eq!(edited.shape, AnswerShape::WholeBody);
        assert_eq!(
            edited.render("replacement", "ignored"),
            "replacement",
            "{language}: an edited-text result is the whole body, without explanatory decoration"
        );

        let unknown = recognise_verifiable(&case("named_unknown", language).prompt)
            .expect("the unknown case is recognised");
        assert_eq!(unknown.shape, AnswerShape::DelimitedValue);
        let rendered = unknown.render("12", "derived");
        assert_eq!(
            rendered, "12",
            "{language}: a scalar unknown is emitted without explanatory numbers"
        );
    }
}
