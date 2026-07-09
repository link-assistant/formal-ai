//! Issue #531: link-native pattern inference over 1D sequences and 2D grids.
//!
//! Run with `cargo run --example issue_531_pattern_inference_demo`. Each prompt
//! is solved through the public [`formal_ai::solve`] entry point, showing how a
//! concrete sequence or grid routes to the `pattern_inference` handler while a
//! bare definitional question routes to the concept lookup instead.

fn main() {
    let prompts = [
        "find the pattern in 1 2 1 2 1 2",
        "what comes next in 7 7 7 7",
        "is the sequence A B B A a palindrome?",
        "what is the pattern in this grid?\n1 2 1\n3 4 3",
        "what is a pattern?",
    ];
    for prompt in prompts {
        let answer = formal_ai::solve(prompt);
        println!("PROMPT: {}", prompt.replace('\n', " / "));
        println!("  intent: {}", answer.intent);
        println!("  answer: {}", answer.answer.replace('\n', "\n          "));
        println!();
    }
}
