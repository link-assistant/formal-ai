//! Issue #706 detection probe.
//!
//! Copy into `examples/` and run with
//! `cargo run --example issue_706_detect_probe` to print the detected slug for
//! a few prompts that mix scripts (a Latin proper name inside Russian/Chinese
//! text must stay Russian/Chinese, not become Spanish).
fn main() {
    for prompt in [
        "Расскажи о julián andrés quiñones?",
        "介绍一下 julián andrés quiñones?",
        "¿Cómo estás?",
        "hello there",
    ] {
        println!("{prompt} => {}", formal_ai::language::detect(prompt).slug());
    }
}
