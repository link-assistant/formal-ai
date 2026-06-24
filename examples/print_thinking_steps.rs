//! Print the concrete, human-readable thinking steps the universal solver emits
//! for a range of task classes (greeting, calculation in English and Russian,
//! code generation, translation, fact lookup, procedural how-to, arithmetic).
//!
//! This demonstrates the "deep thinking" projection from issue #488: every task
//! produces an ordered, naturalized reasoning trace where each step surfaces the
//! actual content (the prompt, the computed result, the looked-up entity, the
//! composed answer) instead of a generic category label, and the calculator's
//! reduction trace folds into one composite `compute` step with detailed
//! children (shown with a `↳` marker).
//!
//! Run with: `cargo run --example print_thinking_steps`

use formal_ai::solver::UniversalSolver;

fn main() {
    let prompts = [
        "Hi",
        "What is 8% of $50?",
        "Посчитай 1000 рублей в долларах",
        "Write me hello world program in Rust",
        "Translate 'hello' to Russian",
        "What is the capital of France?",
        "How do I reverse a list in Python?",
        "What is 2 + 2 * 3?",
    ];

    for prompt in prompts {
        let answer = UniversalSolver::default().solve(prompt);
        println!("\n=== PROMPT: {prompt:?} (intent={}) ===", answer.intent);
        for step in &answer.thinking_steps {
            let parent = step.parent_id.as_deref().map_or("   ", |_| "  ↳");
            println!(
                "{parent}[{:>2}] kind={:<26} step={:<22} lvl={:<8} summary={:?}",
                step.order, step.source_event, step.step, step.level, step.summary
            );
            println!("        detail={:?}", step.detail);
        }
    }
}
