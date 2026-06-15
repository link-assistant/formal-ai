//! Issue #411: the short Russian behavior-rule request `Покажи правила`
//! answered `intent: unknown`.
//!
//! Run: `cargo run --example repro_issue_411`

use formal_ai::UniversalSolver;

fn main() {
    let response = UniversalSolver::default().solve("Покажи правила");

    println!("intent = {}", response.intent);
    println!("---- answer ----\n{}", response.answer);
    println!("---- trace ----\n{}", response.links_notation);
}
