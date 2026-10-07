//! Issue #1177: the gaps the requirement ledger recorded as open, closed in
//! both runtimes (browser twins pinned by
//! `rust/tests/web/issue-1177-requirement-closure.test.mjs`).
//!
//! - R1177-2: the composed regular expression is matched against positive
//!   and negative examples derived from the same constraints, by the subset
//!   matcher in `rust/src/solver_handlers/regex_witness.rs` (the browser
//!   compiles it with `RegExp`); no input of the user's is run.
//! - R1177-5: every line of the explained code is listed — a matched line
//!   with its construct's grounding reference, an unmatched one named as
//!   unexplained — and keyword-headed, closing and augmented-assignment
//!   constructs are seed rows (`head`, `only`, `infix`), not code.
//! - R1177-7: a function outside every problem shape is tested from the
//!   examples the request states ("square(3) returns 9", joined by a seeded
//!   `test_example_relation` word); with no shape and no example the request
//!   is refused by name instead of answered with a smoke skeleton.

use formal_ai::event_log::EventLog;
use formal_ai::web_engine_core::normalize_prompt;

/// The answer a handler gives to `prompt`, normalized as the engine does.
fn answer_of(
    handler: fn(&str, &str, &mut EventLog) -> Option<formal_ai::engine::SymbolicAnswer>,
    prompt: &str,
) -> String {
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    handler(prompt, &normalized, &mut log)
        .unwrap_or_else(|| panic!("the handler should answer: {prompt}"))
        .answer
}

#[test]
fn r1177_2_regex_is_matched_against_derived_examples() {
    let answer = answer_of(
        formal_ai::handle_regex_synthesis,
        "Write a regular expression for three or more digits",
    );
    assert_eq!(
        answer,
        "Composed regular expression:\n\n```regex\n^\\d{3,}$\n```\n\nHow the constraints composed:\n  - '3 digits' -> `\\d{3,}` (the main run)\n  - anchored with ^ and $ so the whole string must match, not just a substring\n\nVerified structurally: groups are balanced, every repetition is well-formed, and every character class is closed.\nVerified against examples derived from the constraints: the pattern text was matched by the subset matcher in rust/src/solver_handlers/regex_witness.rs (anchors, classes, groups and repetitions read from the pattern itself) and accepted all 2 positive examples (\"777\", \"77777\") and rejected all 2 negative examples (\"77\", \"a77\"). No input of yours was run; test the pattern on your real inputs before relying on it."
    );
}

#[test]
fn r1177_5_every_line_is_listed_with_its_grounding_or_named_unexplained() {
    let answer = answer_of(
        formal_ai::handle_code_explanation,
        "Explain this code:\n```python\nimport math\n# mean of the values\ndef average(xs):\n    total = 0\n    for x in xs:\n        total += x\n    while total > 100:\n        total -= 1\n    print(total)\n    yield total\n```",
    );
    assert_eq!(
        answer,
        "A structural explanation, line by line. No code was executed.\n\n  - `import math` \u{2014} loads math so the names it defines can be used below (https://docs.python.org/3.12/reference/simple_stmts.html#the-import-statement)\n  - `# mean of the values` \u{2014} is a comment, ignored when the code runs; it tells the reader: mean of the values (https://docs.python.org/3.12/reference/lexical_analysis.html#comments)\n  - `def average(xs):` \u{2014} defines a function named average which runs its indented body whenever it is called (https://docs.python.org/3.12/reference/compound_stmts.html#function-definitions)\n  - `total = 0` \u{2014} binds the name total to the value on the right (https://docs.python.org/3.12/reference/simple_stmts.html#assignment-statements)\n  - `for x in xs:` \u{2014} binds x to each element of xs in turn and runs the indented body once per element (https://docs.python.org/3.12/reference/compound_stmts.html#the-for-statement)\n  - `total += x` \u{2014} updates total in place by applying the operator += with x (https://docs.python.org/3.12/reference/simple_stmts.html#augmented-assignment-statements)\n  - `while total > 100:` \u{2014} repeats its body for as long as total > 100 stays true, checking before each pass (https://docs.python.org/3.12/reference/compound_stmts.html#the-while-statement)\n  - `total -= 1` \u{2014} updates total in place by applying the operator -= with 1 (https://docs.python.org/3.12/reference/simple_stmts.html#augmented-assignment-statements)\n  - `print(total)` \u{2014} calls the function named print with the arguments total (https://docs.python.org/3.12/reference/expressions.html#calls)\n  - `yield total` \u{2014} no construct in the table matches this line, so it is left unexplained rather than guessed\nCoverage: 9 of 10 lines matched a construct; every line is listed above, each matched one with the reference that defines its construct.\nOverall: the function name `average` promises the arithmetic mean: the sum of the elements divided by their count; the canonical expression of that promise is `sum(xs) / len(xs)` (https://en.wikipedia.org/wiki/Arithmetic_mean).\n\nCost: a single pass over the input, so the work grows linearly (O(n)).\nMethod, stated honestly: each line was matched against the construct table in data/seed/meanings-code-structure-explanations.lino; nothing was run and nothing outside the table was guessed."
    );
}

#[test]
fn r1177_7_stated_examples_become_cases_for_an_unknown_shape() {
    let english = answer_of(
        formal_ai::handle_test_generation,
        "Write tests for `square(n)` where square(3) returns 9, square(-2) should return 4 and square(0) == 0",
    );
    assert_eq!(
        english,
        "Generated pytest suite (Python). NOT executed \u{2014} executing generated tests is out of scope here (that is issue #1185's scope); review the cases and run them yourself with `pytest -q`.\n\n```python\ndef test_square_example_1():\n    assert square(3) == 9\ndef test_square_example_2():\n    assert square(-2) == 4\ndef test_square_example_3():\n    assert square(0) == 0\n```\n\nDerivation:   - 'square(3) returns 9' ->     assert square(3) == 9\n  - 'square(-2) should return 4' ->     assert square(-2) == 4\n  - 'square(0) == 0' ->     assert square(0) == 0\n"
    );
    let russian = answer_of(
        formal_ai::handle_test_generation,
        "Напиши тесты для `is_prime(n)`: is_prime(7) возвращает True, is_prime(8) возвращает False",
    );
    assert_eq!(
        russian,
        "Generated pytest suite (Python). NOT executed \u{2014} executing generated tests is out of scope here (that is issue #1185's scope); review the cases and run them yourself with `pytest -q`.\n\n```python\ndef test_is_prime_example_1():\n    assert is_prime(7) is True\ndef test_is_prime_example_2():\n    assert is_prime(8) is False\n```\n\nDerivation:   - 'is_prime(7) возвращает True' ->     assert is_prime(7) is True\n  - 'is_prime(8) возвращает False' ->     assert is_prime(8) is False\n"
    );
}

#[test]
fn r1177_7_an_unknown_shape_without_examples_is_refused_by_name() {
    let answer = answer_of(
        formal_ai::handle_test_generation,
        "Write tests for `is_even(n)`",
    );
    assert_eq!(
        answer,
        "Recognized a request for tests of `is_even`, but its name matches no problem shape in the test-case table (palindrome, average, max) and the request states no example with its expected value, so I will not guess what it must return. State examples as a call followed by its expected value, such as `is_even(<input>) returns <expected>` with a number, a quoted string, a list, True, False or None, and each one becomes a test case."
    );
}
