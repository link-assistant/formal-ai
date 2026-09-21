//! Issue #702 — ask a running dialogue what is left to reach the goal.
//!
//! ```bash
//! cargo run --example world_state_dialogue
//! ```
//!
//! The `chat` CLI subcommand is single-shot, so this example is the runnable
//! reproduction of the issue's scripted multi-turn dialogue: it replays a few
//! turns through [`UniversalSolver::solve_with_history`] and prints the answer,
//! its evidence links, and the Links Notation of the model behind it — first
//! with the world model off (the default: the handler declines, nothing is
//! traced), then with it opted in.

use formal_ai::solver::{ConversationTurn, SolverConfig, UniversalSolver};
use formal_ai::world_model_dialog::{DialogueWorldModel, WorldModelMode};

/// One dialogue per supported language: a fact, a wish, and the state question.
const DIALOGUES: [[&str; 3]; 4] = [
    [
        "the door is closed",
        "I want the door to be open",
        "what is left to do?",
    ],
    [
        "дверь закрыта",
        "я хочу чтобы дверь была открыта",
        "что осталось сделать?",
    ],
    ["दरवाज़ा बंद है", "मुझे चाहिए दरवाज़ा खुला", "क्या बाकी है?"],
    ["门是关着的", "我想要门是开着的", "还剩什么?"],
];

fn history(turns: &[&str]) -> Vec<ConversationTurn> {
    turns
        .iter()
        .flat_map(|turn| {
            [
                ConversationTurn::user(*turn),
                ConversationTurn::assistant("noted"),
            ]
        })
        .collect()
}

fn solver(mode: WorldModelMode) -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        world_model_mode: mode,
        ..SolverConfig::default()
    })
}

fn main() {
    for mode in [WorldModelMode::Off, WorldModelMode::Track] {
        println!("=== world_model_mode = {} ===", mode.slug());
        let solver = solver(mode);
        for [fact, wish, question] in DIALOGUES {
            let turns = history(&[fact, wish]);
            let answer = solver.solve_with_history(question, &turns);
            println!("  user: {fact}");
            println!("  user: {wish}");
            println!("  user: {question}");
            println!("  intent: {}", answer.intent);
            println!("  answer: {}", answer.answer.replace('\n', "\n          "));
            let traced: Vec<&String> = answer
                .evidence_links
                .iter()
                .filter(|link| link.starts_with("world_state:"))
                .collect();
            println!("  world-state evidence: {traced:?}\n");
        }
    }

    // The same model, inspected directly: the difference is a links network, and
    // a forecast says what an action would do before anything runs.
    let mut model = DialogueWorldModel::new();
    model.observe_user("the door is closed");
    model.observe_user("I want the door to be open");
    println!("=== model behind the answer ===");
    println!("{}", model.links_notation());
}
