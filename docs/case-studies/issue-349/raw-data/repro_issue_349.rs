//! Live reproduction for issue #349.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example repro_issue_349
//! ```

use formal_ai::{ConversationTurn, SymbolicAnswer, UniversalSolver};

const FIRST_PROMPT: &str =
    "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
const PATH_ARGUMENT_PROMPT: &str = "Сделай так, чтобы программа принимала путь как аргумент";
const REVERSE_SORT_PROMPT: &str = "Сделай сортировку результатов в обратном порядке";

fn show(label: &str, prompt: &str, answer: &SymbolicAnswer) {
    println!("================================================================");
    println!("{label}");
    println!("PROMPT : {prompt}");
    println!("INTENT : {}", answer.intent);
    println!("CONF   : {:.2}", answer.confidence);
    println!("ANSWER : {}", answer.answer.replace('\n', "\n         "));
    println!();
}

fn main() {
    let solver = UniversalSolver::default();

    let first = solver.solve(FIRST_PROMPT);
    show("TURN 1 (initial list-files program)", FIRST_PROMPT, &first);

    let first_history = [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer.clone()),
    ];
    let path_argument = solver.solve_with_history(PATH_ARGUMENT_PROMPT, &first_history);
    show(
        "TURN 3 (path-argument modifier - currently handled)",
        PATH_ARGUMENT_PROMPT,
        &path_argument,
    );

    let full_history = [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer),
        ConversationTurn::user(PATH_ARGUMENT_PROMPT),
        ConversationTurn::assistant(path_argument.answer),
    ];
    let reverse_sort = solver.solve_with_history(REVERSE_SORT_PROMPT, &full_history);
    show(
        "TURN 5 (reverse-sort modifier - issue #349)",
        REVERSE_SORT_PROMPT,
        &reverse_sort,
    );
}
