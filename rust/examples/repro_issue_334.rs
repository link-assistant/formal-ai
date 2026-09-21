// Issue #334: the GitHub Pages WASM-worker demo ran a 2-step agent plan that
// both steps failed on. Step 1 ("Write a Python function that calculates the
// Fibonacci sequence recursively.") returned "I didn't understand you" because
// `fibonacci` was not a catalog task, and step 2 ("calculate the 10th Fibonacci
// number and multiply it by 8% of 500. Show me the code and the final result.")
// returned "unparseable" because the natural-language word problem never reduced
// to a calculator expression.
//
// This example shows both steps resolving correctly: step 1 generates a verified
// recursive Python program that prints F(10) = 55, and step 2 reduces to
// `55 * 8% of 500` and evaluates to 2200.
//
// Run with: cargo run --example repro_issue_334
use formal_ai::FormalAiEngine;

fn main() {
    let cases = [
        (
            "Step 1: recursive Fibonacci program (Python)",
            "Write a Python function that calculates the Fibonacci sequence recursively.",
        ),
        (
            "Step 2: Fibonacci word problem -> arithmetic",
            "calculate the 10th Fibonacci number and multiply it by 8% of 500. \
             Show me the code and the final result.",
        ),
        (
            "Coding prompt no longer misread as a unit conversion",
            "Write a program that computes the 10th Fibonacci number",
        ),
        (
            "Cardinal-word Fibonacci reference (F(5) = 5, 5 * 10 = 50)",
            "the fifth Fibonacci number multiplied by 10",
        ),
    ];
    for (label, prompt) in &cases {
        let response = FormalAiEngine.answer(prompt);
        println!("=== {label} ===");
        println!("PROMPT: {prompt}");
        println!("INTENT: {}", response.intent);
        println!("ANSWER:\n{}", response.answer);
        println!();
    }
}
