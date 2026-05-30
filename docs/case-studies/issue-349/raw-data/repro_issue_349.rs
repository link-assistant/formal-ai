//! Live reproduction for issue #349.
//!
//! Multi-turn Russian coding dialog:
//!   1. (user)      write a Rust program that lists files in the current dir
//!   2. (assistant) <program>
//!   3. (user)      make the program accept a path as an argument   <-- WORKS
//!   4. (assistant) <updated program>
//!   5. (user)      sort the results in reverse order               <-- BUG: "unknown"
//!
//! Run: `cargo run --example repro_issue_349`
use formal_ai::{solve_with_history, ConversationTurn};

fn show(label: &str, prompt: &str, history: &[ConversationTurn]) {
    let answer = solve_with_history(prompt, history);
    println!("================================================================");
    println!("{label}");
    println!("PROMPT : {prompt}");
    println!("INTENT : {}", answer.intent);
    println!("CONF   : {:.2}", answer.confidence);
    let body = answer.answer.replace('\n', "\n         ");
    println!("ANSWER : {body}");
    println!();
}

fn main() {
    let u1 = ConversationTurn::user(
        "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории",
    );
    let a1 = ConversationTurn::assistant(
        "Вот минимальная программа на языке Rust (list files in the current directory): \
         fn main() { /* read_dir, sort, print */ }",
    );
    let u2 = ConversationTurn::user("Сделай так, чтобы программа принимала путь как аргумент");
    let a2 = ConversationTurn::assistant(
        "Вот минимальная программа на языке Rust (list files in the directory given as a \
         path argument): fn main() { /* args, read_dir, sort, print */ }",
    );

    // Turn 3 — the path-argument modifier follow-up. This WORKS today because
    // `path_argument` is the one hard-coded entry in PROGRAM_MODIFIERS.
    show(
        "TURN 3 (path-argument modifier — handled by the hard-coded allowlist)",
        "Сделай так, чтобы программа принимала путь как аргумент",
        &[u1.clone(), a1.clone()],
    );

    // Turn 5 — the reverse-sort modifier follow-up. This is the BUG: it has no
    // program-noun, never routes to write_program, and matches no rule => unknown.
    show(
        "TURN 5 (reverse-sort modifier — the issue #349 bug)",
        "Сделай сортировку результатов в обратном порядке",
        &[u1, a1, u2, a2],
    );
}
