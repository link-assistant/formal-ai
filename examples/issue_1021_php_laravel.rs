//! Issue #1021 / #723: what Formal AI answers today for a PHP Laravel request.
use formal_ai::FormalAiEngine;

fn main() {
    for prompt in [
        "write me a Rust program",
        "write me a Python script",
        "write me a PHP program",
        "write me PHP Laravel code",
        "write me Ruby on Rails code",
        "напиши мне код на Python Django",
        "write a rust program that reverses a linked list",
    ] {
        let response = FormalAiEngine.answer(prompt);
        println!(
            "=== {prompt}\n-- intent: {}\n{}\n",
            response.intent, response.answer
        );
    }
}
