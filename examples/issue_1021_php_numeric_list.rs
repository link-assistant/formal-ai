//! Issue #1021: run the PHP code the numeric-list composer generates.
//!
//! Cataloguing PHP (issue #723) means every route that iterates the catalog now
//! has to reach PHP too, including the universal list-coding composer of issue
//! #395. Tree-sitter proves the generated source parses; only `php` proves it
//! runs, so this example prints each generated program and its prompt so the
//! verification log records what was executed.
use formal_ai::FormalAiEngine;

fn main() {
    for prompt in [
        "Sort the numbers 3, 1, 2 in PHP, give me the code",
        "Sort the numbers 3, 1, 2 in PHP in reverse order, give me the code",
        "Reverse the numbers 1, 2, 3 in PHP, give me the code",
        "Sort the strings \"pear\", \"apple\", \"banana\" in PHP, give me the code",
        "Sum the numbers 3, 5, 6, 7, 8 in PHP, give me the code",
        "Multiply the numbers 2, 3, 4 in PHP, show me the code",
        "Find the minimum of 5, 3, 8, 1, 9 in PHP code",
        "Find the maximum of 5, 3, 8, 1, 9 in PHP code",
    ] {
        let response = FormalAiEngine.answer(prompt);
        println!(
            "=== {prompt}\n-- intent: {}\n{}\n",
            response.intent, response.answer
        );
    }
}
