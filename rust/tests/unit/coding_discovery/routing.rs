use formal_ai::{IntentKind, formalize_intent, solve};

const HUMANEVAL_SUM_PRODUCT: &str = r#"Complete this Python function. Reply with the full implementation in a ```python code block.

from typing import List, Tuple

def sum_product(numbers: List[int]) -> Tuple[int, int]:
    \"\"\" Return a tuple consisting of a sum and a product of all the integers in a list.
    >>> sum_product([])
    (0, 1)
    >>> sum_product([1, 2, 3, 4])
    (10, 24)
    \"\"\""#;

const MBPP_SHAPE: &str = r"Write a function to find the shared elements from two tuples.
Reply with the full implementation in a ```python code block. It must pass these tests:
assert similar_elements((3, 4, 5, 6), (5, 7, 4, 10)) == (4, 5)
assert similar_elements((1, 2), (3, 4)) == ()";

#[test]
fn structural_coding_shapes_reach_synthesis_without_preempting_catalog_matches() {
    for (prompt, expected_route) in [
        (HUMANEVAL_SUM_PRODUCT, "program_synthesis"),
        (MBPP_SHAPE, "write_program"),
    ] {
        let intent = formalize_intent(prompt, "en", None);
        assert_eq!(intent.kind, IntentKind::Task, "{intent:?}");
        assert_eq!(intent.route.as_deref(), Some(expected_route), "{intent:?}");
        assert!(
            intent.has_relevant_handler("program_synthesis"),
            "{intent:?}"
        );
    }
}

#[test]
fn coding_task_is_not_claimed_by_concept_or_arithmetic_routes() {
    let response = solve(HUMANEVAL_SUM_PRODUCT);
    assert!(
        matches!(
            response.intent.as_str(),
            "write_program" | "write_program_skill_gap"
        ),
        "coding task was misrouted as {}",
        response.intent,
    );

    let mbpp = solve(MBPP_SHAPE);
    assert!(
        matches!(
            mbpp.intent.as_str(),
            "write_program" | "write_program_skill_gap"
        ),
        "MBPP task was misrouted as {}",
        mbpp.intent,
    );
}

#[test]
fn noncoding_function_question_still_reaches_arithmetic() {
    let prompt = "What is the value of the function f(x)=2x at 3?";
    assert!(formal_ai::coding_task_spec::recognise(prompt).is_none());
    let response = solve(prompt);
    assert_eq!(response.intent, "calculation_error");
    assert_eq!(
        response.answer,
        "I parsed 'the value of the function f(x)=2x at 3' as an arithmetic request but could not evaluate it: expression could not be parsed."
    );
}
