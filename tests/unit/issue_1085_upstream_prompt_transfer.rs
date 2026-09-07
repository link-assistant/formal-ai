//! Issue #1085 (D5.3): the curated 13/13 must transfer to the upstream prompt.
//!
//! `data/benchmarks/external-results.lino` scored HumanEval 0/20 and MBPP 0/20
//! on every scheduled run while the curated slice passed, and both seeded tasks
//! were in those twenty. The scheduled log named the causes: the HumanEval
//! answer was the unknown opener because the candidate copied `List[float]`
//! from the upstream signature without `from typing import List`, so its own
//! verification raised `NameError`; the MBPP candidate copied a *call* out of an
//! `assert` as if it were a signature and did not parse. These tests drive the
//! benchmark solver with the exact prompt shapes `src/external_benchmarks/cases.rs`
//! builds and grade the answers with the upstream criterion.

use std::path::PathBuf;

use formal_ai::external_benchmarks::manifest::Grading;
use formal_ai::external_benchmarks::{BenchmarkCase, Expectation, benchmark_solver, grade};

const HUMANEVAL_0_PROMPT: &str = "from typing import List\n\n\ndef has_close_elements(numbers: List[float], threshold: float) -> bool:\n    \"\"\" Check if in given list of numbers, are any two numbers closer to each other than\n    given threshold.\n    >>> has_close_elements([1.0, 2.0, 3.0], 0.5)\n    False\n    >>> has_close_elements([1.0, 2.8, 3.0, 4.0, 5.0, 2.0], 0.3)\n    True\n    \"\"\"\n";

const HUMANEVAL_0_TEST: &str = "METADATA = {\n    'author': 'jt',\n    'dataset': 'test'\n}\n\n\ndef check(candidate):\n    assert candidate([1.0, 2.0, 3.9, 4.0, 5.0, 2.2], 0.3) == True\n    assert candidate([1.0, 2.0, 3.9, 4.0, 5.0, 2.2], 0.05) == False\n    assert candidate([1.0, 2.0, 5.9, 4.0, 5.0], 0.95) == True\n    assert candidate([1.0, 2.0, 5.9, 4.0, 5.0], 0.8) == False\n    assert candidate([1.0, 2.0, 3.0, 4.0, 5.0, 2.0], 0.1) == True\n    assert candidate([1.1, 2.2, 3.1, 4.1, 5.1], 1.0) == True\n    assert candidate([1.1, 2.2, 3.1, 4.1, 5.1], 0.5) == False\n\n";

fn workspace(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-issue-1085-{name}-{}-{nonce}",
        std::process::id()
    ))
}

fn python_available() -> bool {
    std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

#[test]
fn the_upstream_humaneval_prompt_shape_is_answered_and_graded_by_the_upstream_test() {
    if !python_available() {
        eprintln!("python3 is not available; the upstream grader cannot run here");
        return;
    }
    let case = BenchmarkCase {
        id: "HumanEval/0".to_owned(),
        prompt: format!(
            "Complete this Python function. Reply with the full implementation in a ```python code block.\n\n{HUMANEVAL_0_PROMPT}"
        ),
        expectation: Expectation::PythonUnitTest {
            test_code: HUMANEVAL_0_TEST.to_owned(),
            entry_point: "has_close_elements".to_owned(),
        },
    };
    let response = benchmark_solver().solve(&case.prompt);
    assert!(
        response.answer.contains("```python"),
        "the upstream prompt shape must reach the synthesis handler: {}",
        response.answer
    );
    assert!(
        response.answer.contains("from typing import List"),
        "the prompt's import travels with the candidate: {}",
        response.answer
    );
    let dir = workspace("humaneval");
    let outcome = grade::grade_case(&case, Grading::PythonUnitTest, &response.answer, &dir);
    std::fs::remove_dir_all(&dir).ok();
    assert!(outcome.passed, "{}", outcome.detail);
}

#[test]
fn the_upstream_mbpp_prompt_shape_does_not_mistake_an_assertion_for_a_signature() {
    if !python_available() {
        eprintln!("python3 is not available; the upstream grader cannot run here");
        return;
    }
    let asserts = vec![
        "assert similar_elements((3, 4, 5, 6),(5, 7, 4, 10)) == (4, 5)".to_owned(),
        "assert similar_elements((1, 2, 3, 4),(5, 4, 3, 7)) == (3, 4)".to_owned(),
        "assert similar_elements((11, 12, 14, 13),(17, 15, 14, 13)) == (13, 14)".to_owned(),
    ];
    let case = BenchmarkCase {
        id: "MBPP/2".to_owned(),
        prompt: format!(
            "Write a function to find the similar elements from the given two tuple lists.\nReply with the Python code in a ```python code block. It must pass these tests:\n{}",
            asserts.join("\n")
        ),
        expectation: Expectation::PythonAsserts {
            setup: String::new(),
            asserts,
        },
    };
    let response = benchmark_solver().solve(&case.prompt);
    assert!(
        response
            .answer
            .contains("def similar_elements(test_tup1, test_tup2):"),
        "a call inside an assertion is not a signature: {}",
        response.answer
    );
    let dir = workspace("mbpp");
    let outcome = grade::grade_case(&case, Grading::PythonAsserts, &response.answer, &dir);
    std::fs::remove_dir_all(&dir).ok();
    assert!(outcome.passed, "{}", outcome.detail);
}
