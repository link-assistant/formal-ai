//! Print what Formal AI's symbolic engine concludes for a prompt (issue #1066).
//!
//! The agentic routes are clients of the engine, so a delivery that looks wrong
//! is either the route's fault or the engine's. This isolates the second half:
//! it shows the verdict alone, with no route involved.

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let trace = args.first().is_some_and(|flag| flag == "--trace");
    if trace {
        args.remove(0);
    }
    let prompt = args.join(" ");
    let answer = formal_ai::engine::FormalAiEngine.answer(&prompt);
    println!("intent: {}", answer.intent);
    println!("confidence: {}", answer.confidence);
    println!("inconclusive: {}", answer.is_inconclusive());
    println!(
        "defers_to_the_open_web: {}",
        answer.defers_to_the_open_web()
    );
    println!("--- answer ---\n{}", answer.answer);
    if trace {
        println!("--- links notation ---\n{}", answer.links_notation);
    }
}
