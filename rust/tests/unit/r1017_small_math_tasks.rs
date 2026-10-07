//! Small computed tasks put to Formal AI itself while building it (R1017).
//!
//! Each answer is computed, never looked up or answered with an algorithm:
//! the weekday of a stated date (`calendar/date_weekday.rs`, counting days
//! from 1970-01-01), the primality of one stated number
//! (`statistics/primality.rs`, trial division with a factor for a composite),
//! a spelled function application ("the square root of 144" read as
//! `sqrt(144)`, `calculation/function_phrase.rs`) and a list transformation
//! asked without code (`numeric_list/stated_result.rs`). The cue words live in
//! the seed and the prose in `data/seed/multilingual-responses-quantities.lino`.
//! Twin: `rust/tests/web/r1017-small-math-tasks.test.mjs`.

use formal_ai::FormalAiEngine;

#[test]
fn the_weekday_of_a_stated_date_is_computed() {
    for (prompt, expected) in [
        (
            "What day of the week was 2024-02-29?",
            "2024-02-29 is a Thursday. Counting from 1970-01-01, a Thursday, it is day 19782: 19782 = 7 × 2826 + 0, and Thursday + 0 days = Thursday.",
        ),
        (
            "What weekday is 2025-12-25?",
            "2025-12-25 is a Thursday. Counting from 1970-01-01, a Thursday, it is day 20447: 20447 = 7 × 2921 + 0, and Thursday + 0 days = Thursday.",
        ),
        (
            "What day of the week was 1 January 2000?",
            "2000-01-01 is a Saturday. Counting from 1970-01-01, a Thursday, it is day 10957: 10957 = 7 × 1565 + 2, and Thursday + 2 days = Saturday.",
        ),
        (
            "Which day of the week is July 4, 1776?",
            "1776-07-04 is a Thursday. Counting from 1970-01-01, a Thursday, it is day -70672: -70672 = 7 × -10096 + 0, and Thursday + 0 days = Thursday.",
        ),
        (
            "What day of the week was 1969-12-31?",
            "1969-12-31 is a Wednesday. Counting from 1970-01-01, a Thursday, it is day -1: -1 = 7 × -1 + 6, and Thursday + 6 days = Wednesday.",
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "calendar_date_weekday",
            "{prompt}: {}",
            response.answer
        );
        assert_eq!(response.answer.as_str(), expected, "{prompt}");
    }
}

#[test]
fn the_date_weekday_shows_its_day_count_from_the_epoch() {
    let response = FormalAiEngine.answer("What day of the week was 2024-02-29?");
    assert_eq!(
        response.answer,
        "2024-02-29 is a Thursday. Counting from 1970-01-01, a Thursday, it is day 19782: 19782 = 7 × 2826 + 0, and Thursday + 0 days = Thursday."
    );
    let russian = FormalAiEngine.answer("Какой день недели был 2024-02-29?");
    assert!(
        russian.answer.starts_with("2024-02-29 — четверг."),
        "{}",
        russian.answer
    );
}

#[test]
fn an_impossible_date_or_an_offset_is_not_a_date_weekday() {
    for prompt in [
        "What day of the week is 2023-02-29?",
        "What day of the week is 3 days after 2024-02-29?",
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_ne!(
            response.intent, "calendar_date_weekday",
            "{prompt}: {}",
            response.answer
        );
    }
}

#[test]
fn primality_of_one_stated_number_is_decided_by_trial_division() {
    let prime = FormalAiEngine.answer("Is 97 a prime number?");
    assert_eq!(prime.intent, "number_primality", "{}", prime.answer);
    assert_eq!(
        prime.answer,
        "Yes, 97 is a prime number: no integer from 2 to ⌊√97⌋ = 9 divides it, so its only divisors are 1 and 97."
    );
    let composite = FormalAiEngine.answer("Is 91 prime?");
    assert_eq!(composite.intent, "number_primality", "{}", composite.answer);
    assert!(
        composite
            .answer
            .starts_with("No, 91 is not a prime number: 91 = 7 × 13"),
        "{}",
        composite.answer
    );
    let one = FormalAiEngine.answer("Is 1 a prime number?");
    assert!(
        one.answer.starts_with("No, 1 is not a prime number"),
        "{}",
        one.answer
    );
    let two = FormalAiEngine.answer("Is 2 prime?");
    assert!(
        two.answer.starts_with("Yes, 2 is a prime number"),
        "{}",
        two.answer
    );
    let russian = FormalAiEngine.answer("Является ли 97 простым числом?");
    assert!(
        russian.answer.starts_with("Да, 97 — простое число"),
        "{}",
        russian.answer
    );
}

#[test]
fn a_prime_question_about_a_range_or_an_ordinal_is_not_one_number() {
    for prompt in [
        "What is the largest prime below 100?",
        "What is the 10th prime?",
        "Pick a prime number between 14 and 18",
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_ne!(
            response.intent, "number_primality",
            "{prompt}: {}",
            response.answer
        );
    }
}

#[test]
fn a_spelled_function_application_is_computed() {
    for (prompt, expected) in [
        ("What is the square root of 144?", "sqrt(144) = 12"),
        ("Calculate the square root of 81", "sqrt(81) = 9"),
        ("Сколько будет квадратный корень из 144?", "sqrt(144) = 12"),
        ("What is the square root of 2?", "sqrt(2) = 1.414213562373095"),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "calculation",
            "{prompt}: {}",
            response.answer
        );
        assert_eq!(response.answer.as_str(), expected, "{prompt}");
    }
}

#[test]
fn a_statement_about_a_function_value_is_not_a_calculation() {
    let response = FormalAiEngine.answer("Prove that the square root of 2 is irrational");
    assert_ne!(response.intent, "calculation", "{}", response.answer);
}

#[test]
fn a_list_transformation_without_code_answers_with_the_result() {
    let ascending = FormalAiEngine.answer("Sort the numbers 5, 2, 9, 1");
    assert_eq!(
        ascending.intent, "numeric_list_result",
        "{}",
        ascending.answer
    );
    assert_eq!(
        ascending.answer,
        "5, 2, 9, 1 sorted in ascending order: 1, 2, 5, 9."
    );
    let descending = FormalAiEngine.answer("Sort the numbers 5, 2, 9, 1 in descending order");
    assert_eq!(
        descending.answer,
        "5, 2, 9, 1 sorted in descending order: 9, 5, 2, 1."
    );
    let russian = FormalAiEngine.answer("Отсортируй числа 5, 2, 9, 1");
    assert_eq!(russian.answer, "5, 2, 9, 1 по возрастанию: 1, 2, 5, 9.");
    let coded = FormalAiEngine.answer("Sort the numbers 5, 2, 9, 1 in JavaScript");
    assert_eq!(coded.intent, "write_program", "{}", coded.answer);
}
