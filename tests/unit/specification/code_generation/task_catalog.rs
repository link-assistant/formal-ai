//! Coding-task catalog tests: the deterministic exercises (`FizzBuzz`,
//! factorial, string reversal, sums, recursive Fibonacci) and their
//! per-language coverage across the popular languages (issue #386 split).

use super::{answer, assert_write_program_parameters, POPULAR_LANGUAGES};

// ---------------------------------------------------------------------------
// Issue #330: the catalog must support a wider range of coding tasks than just
// hello-world. The classic deterministic exercises below (FizzBuzz, factorial,
// string reversal, sum 1..=10) each resolve through the parameterized
// write_program intent, carry a verified output, and are reachable in every
// supported prompt language (English, Russian, Hindi, Chinese). The JavaScript
// worker (`src/web/formal_ai_worker.js`) mirrors the same catalog, so the
// per-language coverage here keeps the Rust and JS engines in lockstep.
// ---------------------------------------------------------------------------

#[test]
fn english_fizzbuzz_in_rust_returns_program() {
    let response = answer("Write me a FizzBuzz program in Rust");
    assert_write_program_parameters(&response, "rust", "fizzbuzz");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("% 15 == 0"));
    // The verified deterministic output is surfaced for the novice to compare.
    assert!(response.answer.contains("FizzBuzz"));
    assert!(
        response.answer.contains("How it works:"),
        "FizzBuzz answer should explain how it works, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Hello, world!"),
        "FizzBuzz must not be routed through the legacy hello-world shortcut: {}",
        response.answer
    );
}

#[test]
fn russian_factorial_of_five_in_python_returns_program() {
    // Russian: "factorial of 5 in Python" (питоне = Python).
    let response = answer("Напиши факториал 5 на питоне");
    assert_write_program_parameters(&response, "python", "factorial");
    assert!(response.answer.contains("```python"));
    assert!(
        response.answer.contains("range(1, 6)"),
        "factorial template should multiply 1..=5, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Как это работает:"),
        "Russian factorial answer should explain how it works, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("факториал 5"),
        "Russian explanation should describe the factorial, got: {}",
        response.answer
    );
}

#[test]
fn hindi_reverse_string_in_rust_returns_program() {
    // Hindi: "reverse the string, write in Rust".
    let response = answer("Rust में स्ट्रिंग को उलटें");
    assert_write_program_parameters(&response, "rust", "reverse_string");
    assert!(response.answer.contains("```rust"));
    assert!(
        response.answer.contains(".rev()"),
        "Rust reverse template should reverse the characters, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("यह कैसे काम करता है:"),
        "Hindi reverse answer should explain how it works, got: {}",
        response.answer
    );
}

#[test]
fn chinese_sum_to_ten_in_go_returns_program() {
    // Chinese: "write a program in Go that sums 1 to 10".
    let response = answer("用 Go 写 1到10的和 的程序");
    assert_write_program_parameters(&response, "go", "sum_to_ten");
    assert!(response.answer.contains("```go"));
    assert!(
        response.answer.contains("total"),
        "Go sum template should accumulate a total, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("工作原理："),
        "Chinese sum answer should explain how it works, got: {}",
        response.answer
    );
    // The verified deterministic output (55) is shown for comparison.
    assert!(response.answer.contains("55"));
}

#[test]
fn english_recursive_fibonacci_in_python_returns_program() {
    // Issue #334 step 1: the website demo asked to "Write a Python function
    // that calculates the Fibonacci sequence recursively." Before the catalog
    // gained a Fibonacci task this answered "I didn't understand you". The
    // recursive template defines `fibonacci`, calls itself, and prints the 10th
    // term (F(1)=F(2)=1 -> F(10)=55).
    let response =
        answer("Write a Python function that calculates the Fibonacci sequence recursively.");
    assert_write_program_parameters(&response, "python", "fibonacci");
    assert!(response.answer.contains("```python"));
    assert!(
        response.answer.contains("def fibonacci"),
        "Python Fibonacci template should define a fibonacci function, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("fibonacci(n - 1)")
            && response.answer.contains("fibonacci(n - 2)"),
        "Fibonacci template should recurse on n-1 and n-2, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("How it works:"),
        "Fibonacci answer should explain how it works, got: {}",
        response.answer
    );
    // The verified deterministic output for the 10th term is surfaced.
    assert!(response.answer.contains("55"));
}

#[test]
fn fizzbuzz_supported_for_every_popular_language() {
    for (language, slug, fence) in POPULAR_LANGUAGES {
        let response = answer(&format!("Write me a FizzBuzz program in {language}"));
        assert_write_program_parameters(&response, slug, "fizzbuzz");
        assert!(
            response.answer.contains(fence),
            "missing FizzBuzz template for {language}: {}",
            response.answer
        );
    }
}

#[test]
fn new_coding_tasks_are_each_supported_in_a_popular_language() {
    // factorial, reverse_string and sum_to_ten round out the wider-range catalog
    // alongside FizzBuzz; each resolves with its verified deterministic output.
    let cases: &[(&str, &str, &str)] = &[
        (
            "Write me a Python program for the factorial of 5",
            "factorial",
            "120",
        ),
        (
            "Write me a Python program to reverse the string hello",
            "reverse_string",
            "olleh",
        ),
        (
            "Write me a Python program for the sum from 1 to 10",
            "sum_to_ten",
            "55",
        ),
    ];
    for (prompt, task, expected_output) in cases {
        let response = answer(prompt);
        assert_write_program_parameters(&response, "python", task);
        assert!(
            response.answer.contains("```python"),
            "{task} answer should include a Python code block, got: {}",
            response.answer
        );
        assert!(
            response.answer.contains(expected_output),
            "{task} answer should surface the verified output {expected_output}, got: {}",
            response.answer
        );
    }
}
