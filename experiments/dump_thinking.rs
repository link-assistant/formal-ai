//! Experiment: print the naturalized thinking trace for greeting vs wellbeing
//! so we can see how "robotic" the current per-intent thinking display is (R8).

use formal_ai::{render_thinking_steps, FormalAiEngine};

fn main() {
    for prompt in ["Hello", "How are you?", "как дела", "2 + 2", "What is the capital of France?"] {
        let response = FormalAiEngine.answer(prompt);
        println!("==================================================");
        println!("PROMPT: {prompt:?}  ->  intent={}", response.intent);
        println!("--- naturalized (human) ---");
        println!("{}", render_thinking_steps(&response.thinking_steps));
        println!("--- raw steps (step | detail | level | parent) ---");
        for s in &response.thinking_steps {
            println!(
                "  [{}] {} | {} | {} | parent={:?}",
                s.order, s.step, s.detail, s.level, s.parent_id
            );
        }
    }
}
