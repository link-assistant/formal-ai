//! Issue #1021 / #862 / #863: what Formal AI answers today for a named
//! programming exercise, after the `cp` misrouting is gone.
//!
//! Before this branch both prompts lowered to a shell `cp`. They no longer do.
//! This probe records what they reach *instead*, so the case study can report
//! the remaining gap from measurement rather than from assumption.
use formal_ai::FormalAiEngine;

fn main() {
    for prompt in [
        "Give me example of how to do copy stdin to stdout in Rust",
        "Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust",
        "write a program that copies stdin to stdout",
        "write a Rust program that copies stdin to stdout",
    ] {
        let response = FormalAiEngine.answer(prompt);
        println!(
            "=== {prompt}\n-- intent: {}\n{}\n",
            response.intent, response.answer
        );
    }
}
