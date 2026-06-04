//! Dump the full diagnostic trace for issue #386 turn 7 ("Отмени сортировку")
//! so the integration test asserts the real reasoning-chain strings.
//!
//! Run with: `cargo run --example issue_386_diagnostic_trace`

use formal_ai::{ConversationTurn, SolverConfig, UniversalSolver};

const FIRST_PROMPT: &str =
    "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
const PATH_ARGUMENT_PROMPT: &str = "Сделай так, чтобы программа принимала путь как аргумент";
const REVERSE_SORT_PROMPT: &str = "Сделай сортировку результатов в обратном порядке";
const CANCEL_SORT_PROMPT: &str = "Отмени сортировку";

fn main() {
    let solver = UniversalSolver::new(SolverConfig {
        diagnostic_mode: true,
        ..SolverConfig::default()
    });

    let first = solver.solve(FIRST_PROMPT);
    let path = solver.solve_with_history(
        PATH_ARGUMENT_PROMPT,
        &[
            ConversationTurn::user(FIRST_PROMPT),
            ConversationTurn::assistant(first.answer.clone()),
        ],
    );
    let reverse = solver.solve_with_history(
        REVERSE_SORT_PROMPT,
        &[
            ConversationTurn::user(FIRST_PROMPT),
            ConversationTurn::assistant(first.answer.clone()),
            ConversationTurn::user(PATH_ARGUMENT_PROMPT),
            ConversationTurn::assistant(path.answer.clone()),
        ],
    );
    let cancel = solver.solve_with_history(
        CANCEL_SORT_PROMPT,
        &[
            ConversationTurn::user(FIRST_PROMPT),
            ConversationTurn::assistant(first.answer),
            ConversationTurn::user(PATH_ARGUMENT_PROMPT),
            ConversationTurn::assistant(path.answer),
            ConversationTurn::user(REVERSE_SORT_PROMPT),
            ConversationTurn::assistant(reverse.answer),
        ],
    );

    println!("INTENT: {}", cancel.intent);
    println!("==== FULL DIAGNOSTIC ANSWER ====");
    println!("{}", cancel.answer);
}
