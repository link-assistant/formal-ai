//! Does the dead end a languageless coding request reaches lead anywhere?
//!
//! `мне нужен код` names neither a task nor a language, so the answer is a
//! question (issue #1021, R1021-31). This probe asks the question back: with
//! the follow-up supplying what the first prompt left out, does the catalog
//! answer, or does the conversation dead-end twice?
//!
//! Run with `cargo run --example issue_1021_languageless_followup`.

use formal_ai::{ConversationTurn, UniversalSolver};

fn main() {
    let solver = UniversalSolver::default();
    let mut history: Vec<ConversationTurn> = Vec::new();
    for prompt in [
        "мне нужен код",
        "посчитай до трёх на Python",
        "I need code",
        "count to three in Python",
    ] {
        let answer = solver.solve_with_history(prompt, &history);
        println!(
            "=== {prompt}\n-- intent: {}\n{}\n",
            answer.intent, answer.answer
        );
        history.push(ConversationTurn::user(prompt));
        history.push(ConversationTurn::assistant(&answer.answer));
    }
}
