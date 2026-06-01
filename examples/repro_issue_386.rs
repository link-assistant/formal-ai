//! Live reproduction for issue #386.
//!
//! Builds the same reverse-sorted file-listing program as issue #349, then
//! sends the subtractive follow-up **"Отмени сортировку"** ("Cancel the
//! sorting"). At report time this returned `intent: unknown`; the fix routes it
//! back to `write_program` with the sort removed.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example repro_issue_386
//! ```

use formal_ai::{ConversationTurn, SymbolicAnswer, UniversalSolver};

const FIRST_PROMPT: &str =
    "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
const PATH_ARGUMENT_PROMPT: &str = "Сделай так, чтобы программа принимала путь как аргумент";
const REVERSE_SORT_PROMPT: &str = "Сделай сортировку результатов в обратном порядке";
const CANCEL_SORT_PROMPT: &str = "Отмени сортировку";

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

    let history_after_first = [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer.clone()),
    ];
    let path_argument = solver.solve_with_history(PATH_ARGUMENT_PROMPT, &history_after_first);
    show("TURN 3 (path-argument modifier)", PATH_ARGUMENT_PROMPT, &path_argument);

    let history_after_path = [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer.clone()),
        ConversationTurn::user(PATH_ARGUMENT_PROMPT),
        ConversationTurn::assistant(path_argument.answer.clone()),
    ];
    let reverse_sort = solver.solve_with_history(REVERSE_SORT_PROMPT, &history_after_path);
    show("TURN 5 (reverse-sort modifier - issue #349)", REVERSE_SORT_PROMPT, &reverse_sort);

    let history_after_reverse = [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer),
        ConversationTurn::user(PATH_ARGUMENT_PROMPT),
        ConversationTurn::assistant(path_argument.answer),
        ConversationTurn::user(REVERSE_SORT_PROMPT),
        ConversationTurn::assistant(reverse_sort.answer),
    ];
    let cancel_sort = solver.solve_with_history(CANCEL_SORT_PROMPT, &history_after_reverse);
    show("TURN 7 (cancel-sort modifier - issue #386)", CANCEL_SORT_PROMPT, &cancel_sort);
}
