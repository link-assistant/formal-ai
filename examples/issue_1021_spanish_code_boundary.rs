//! Where a Spanish coding request lands, and why the `c` in `código` mattered.
//!
//! `contains_token` decided word boundaries with `is_ascii_alphanumeric`, so the
//! `ó` after the `c` of `código` looked like a boundary and the one-letter alias
//! of the language C matched inside the Spanish word for *code*. Every Spanish
//! request mentioning code was therefore a request for a C program (issue
//! #1021). This probe prints where each prompt lands; restoring the ASCII test
//! in `src/coding/catalog/mod.rs` reproduces the defect on the first three
//! lines.
//!
//! Run with `cargo run --example issue_1021_spanish_code_boundary`.

use formal_ai::UniversalSolver;

fn main() {
    let solver = UniversalSolver::default();
    for prompt in [
        // The defect: code named in Spanish, no language named at all.
        "escribe código",
        "necesito código",
        "dame código",
        // A task named in Spanish, still no language.
        "copiar stdin a stdout en código",
        // The boundary is not switched off: these still resolve.
        "escribe código en Python",
        "copiar stdin a stdout en C",
        "write me a C program that counts to three",
    ] {
        println!("{prompt:45} -> {}", solver.solve(prompt).intent);
    }
}
