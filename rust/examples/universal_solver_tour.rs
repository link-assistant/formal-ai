//! Walk through every specialized handler in the universal solver. Run with:
//! `cargo run --example universal_solver_tour`.
//!
//! Each prompt is routed through the same `FormalAiEngine::answer` entry
//! point used by the library, CLI, HTTP server, Telegram bot and web demo,
//! so the printed intent + answer reflect exactly what those interfaces
//! return.

use formal_ai::{ConversationTurn, FormalAiEngine, UniversalSolver};

fn main() {
    let single_turn_prompts = [
        ("Greeting", "Hi"),
        ("Identity", "What is formal-ai?"),
        ("Arithmetic — symbols", "What is 7 * (3 + 4)?"),
        ("Arithmetic — words", "What is 10 plus 20 times 3?"),
        ("Arithmetic — divide", "Compute 100 - 25 % 7"),
        ("Arithmetic — error", "What is 5 / 0?"),
        ("Concept — Wikipedia", "What is Wikipedia?"),
        ("Concept — Links Notation", "Tell me about Links Notation"),
        ("Concept — Wikidata", "What does Wikidata mean?"),
        ("Concept — WebAssembly", "What is WebAssembly?"),
        (
            "Hello-world program",
            "Write me hello world program in Rust",
        ),
        (
            "JavaScript — explicit",
            "Please run this javascript:\n```js\nconsole.log(1 + 2);\n```",
        ),
        (
            "Source refresh",
            "Refresh source 'links-notation' from cache.",
        ),
        ("Translation", "Translate 'Hello, world!' to Russian."),
    ];

    println!("# Universal solver tour\n");
    for (label, prompt) in single_turn_prompts {
        let response = FormalAiEngine.answer(prompt);
        println!("---");
        println!("## {label}");
        println!("prompt    : {prompt}");
        println!("intent    : {}", response.intent);
        println!("answer    : {}", response.answer);
        println!("confidence: {:.2}", response.confidence);
    }

    println!("\n# Multi-turn conversation (solve_with_history)\n");
    let history = [
        ConversationTurn::user("Hi, my name is Ada."),
        ConversationTurn::assistant("Hi, how may I help you?"),
        ConversationTurn::user("What is 2 + 2?"),
        ConversationTurn::assistant("2 + 2 = 4"),
    ];
    let follow_ups = [
        "What is my name?",
        "What was my previous question?",
        "Summarize the conversation so far.",
    ];
    for prompt in follow_ups {
        let response = UniversalSolver::default().solve_with_history(prompt, &history);
        println!("---");
        println!("prompt    : {prompt}");
        println!("intent    : {}", response.intent);
        println!("answer    : {}", response.answer);
        println!("confidence: {:.2}", response.confidence);
    }
}
