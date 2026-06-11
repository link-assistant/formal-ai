//! Issue #412: conversational numeric-list follow-up.
//!
//! The deployed assistant answered the bare Russian prompt
//! `Отсортируй 4, 3, 1, 17, 8, 9, 15` with `intent: unknown` because that turn
//! names no programming language and does not ask for code. This example
//! replays the reported conversation — a first turn established a JavaScript
//! code+result context — and shows that `solve_with_history` now recovers the
//! language (and the code request) from the conversation and continues the
//! coding context.
//!
//! Run: `cargo run --example repro_issue_412`

use formal_ai::{ConversationTurn, UniversalSolver};

fn main() {
    let solver = UniversalSolver::default();
    let history = vec![
        ConversationTurn::user(
            "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат",
        ),
        ConversationTurn::assistant(
            "Вот код на JavaScript, который сортирует числа 3, 5, 6, 7, 8 по возрастанию:\n\n```javascript\nconst numbers = [3, 5, 6, 7, 8];\nconst sorted = [...numbers].sort((a, b) => a - b);\nconsole.log(sorted.join(\", \"));\n```\n\nРезультат: 3, 5, 6, 7, 8",
        ),
    ];

    let response = solver.solve_with_history("Отсортируй 4, 3, 1, 17, 8, 9, 15", &history);

    println!("intent = {}", response.intent);
    println!("---- answer ----\n{}", response.answer);
    println!("---- trace ----\n{}", response.links_notation);
}
