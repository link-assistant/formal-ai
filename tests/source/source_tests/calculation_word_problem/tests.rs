use super::*;

fn normalize_word_problem(expression: &str) -> Option<String> {
    normalize_word_problem_detailed(expression).map(|normalization| normalization.expression)
}

#[test]
fn fibonacci_convention_matches_catalog() {
    assert_eq!(fibonacci_value(1), 1);
    assert_eq!(fibonacci_value(2), 1);
    assert_eq!(fibonacci_value(5), 5);
    assert_eq!(fibonacci_value(10), 55);
}

#[test]
fn resolves_fibonacci_and_rewrites_operator() {
    // The Fibonacci reference becomes 55, the spelled-out operator becomes
    // `*`, and the trailing instruction sentence is dropped. The leading
    // "calculate" verb is left for the calculator wrapper-stripping stage.
    assert_eq!(
        normalize_word_problem(
            "calculate the 10th Fibonacci number and multiply it by 8% of 500. \
                 Show me the code and the final result."
        )
        .as_deref(),
        Some("calculate 55 * 8% of 500"),
    );
    assert_eq!(
        normalize_word_problem("the fifth Fibonacci number multiplied by 10").as_deref(),
        Some("5 * 10"),
    );
}

#[test]
fn resolves_box_relations_and_total() {
    let normalized = normalize_word_problem_detailed(
        "I have 3 boxes. Box A has twice as many apples as Box B. \
             Box C has 5 more apples than Box A. If Box B has 10 apples, \
             how many apples are there in total? Show your reasoning step by step.",
    )
    .expect("box total problem should normalize");
    assert_eq!(normalized.expression, "20 + 10 + 25");
    assert_eq!(
        normalized.reasoning_steps,
        vec![
            "Box B = 10 apples.",
            "Box A = 2 * 10 = 20 apples.",
            "Box C = 20 + 5 = 25 apples.",
            "Total = 20 + 10 + 25 apples.",
        ],
    );
    assert_eq!(normalized.result_label.as_deref(), Some("apples"));
}

#[test]
fn resolves_train_meeting_problem_with_verification_steps() {
    let normalized = normalize_word_problem_detailed(
        "Solve this step-by-step, but with verification at each stage: \
             Problem: \"A train leaves Moscow at 60 km/h. Another leaves St. \
             Petersburg at 80 km/h. Distance: 700 km. When/where do they meet?\"",
    )
    .expect("train meeting problem should normalize");
    assert_eq!(normalized.expression, "700 / (60 + 80)");
    assert_eq!(normalized.result_label, None);
    assert!(
        normalized
            .reasoning_steps
            .iter()
            .any(|step| step.contains("[STEP 1]") && step.contains("[VERIFY]")),
        "verification-tagged steps should be preserved: {:?}",
        normalized.reasoning_steps,
    );
    assert!(
        normalized
            .reasoning_steps
            .iter()
            .any(|step| step.contains("300 km from Moscow")),
        "meeting point from Moscow should be explained: {:?}",
        normalized.reasoning_steps,
    );
    assert!(
        normalized
            .reasoning_steps
            .iter()
            .any(|step| step.contains("400 km from St. Petersburg")),
        "meeting point from St. Petersburg should be explained: {:?}",
        normalized.reasoning_steps,
    );
}

#[test]
fn decimals_are_never_split_on_their_dot() {
    // "3.14" must not become "3. 14" — the period is flanked by digits, so it
    // stays inside its sentence and the whole expression is unchanged.
    assert_eq!(normalize_word_problem("What is 3.14 * 2"), None);
}

#[test]
fn pure_instruction_text_is_left_alone() {
    assert_eq!(normalize_word_problem("Show me the code"), None);
    assert_eq!(normalize_word_problem(""), None);
}
