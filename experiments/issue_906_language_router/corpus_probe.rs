//! Issue #906 probe: print the routing outcome of every corpus prompt.
//!
//! Run with `cargo run --example issue_906_corpus_probe` after copying into
//! `examples/`, or read the recorded output in `corpus.txt`. It exists so the
//! table-driven regression corpus in
//! `tests/unit/issue_906_language_router.rs` encodes *observed* behaviour
//! rather than guessed behaviour.

use formal_ai::FormalAiEngine;

const PROMPTS: &[&str] = &[
    "Write me hello world program in Rust",
    "hello world in python",
    "hello world in js",
    "write a hello world program in python",
    "напиши программу hello world на python",
    "hello world in elvish",
    "hello world in the elvish language",
    "Write a program that prints hello world.",
    "write a program",
    "Create a file named hello.txt in the current directory whose entire content is the single line: Hello World.",
    "Create a file named hello.txt containing Hello World, in JavaScript.",
    "Fix the failing CI job in Rust.",
    "Fix the failing CI job in the current directory.",
    "count to three in rust",
    "reverse a string in python",
    "hello world in 3 steps",
    "Meet me in Paris",
    "what is rust",
];

fn main() {
    for prompt in PROMPTS {
        let response = FormalAiEngine.answer(prompt);
        let language = trace_value(&response.links_notation, "program_parameter:language");
        let task = trace_value(&response.links_notation, "program_parameter:task");
        println!(
            "prompt={prompt:?} intent={} language={language:?} task={task:?}",
            response.intent
        );
    }
}

fn trace_value(links_notation: &str, key: &str) -> Option<String> {
    links_notation.lines().find_map(|line| {
        let rest = line.trim().strip_prefix(key)?;
        Some(rest.trim().to_owned())
    })
}
