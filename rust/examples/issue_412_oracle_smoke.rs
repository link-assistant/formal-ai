//! Issue #412: smoke-check the coding oracle fallback for languages the
//! verified catalog does not template. Run with:
//!   `cargo run --example issue_412_oracle_smoke`
use formal_ai::solve;

fn main() {
    for prompt in [
        "Write a hello world program in Kotlin",
        "write hello world in swift",
        "write me a hello world program in php",
        "write a hello world program in Rust", // catalog language: should NOT use oracle
    ] {
        let answer = solve(prompt);
        println!("=== {prompt}");
        println!("intent: {}", answer.intent);
        println!("{}\n", answer.answer);
    }
}
