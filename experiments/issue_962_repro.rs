//! Issue #962 reproduction: word-operator arithmetic parity across en | ru | hi | zh.
//!
//! Copy into `examples/` and run with
//! `cargo run -p formal-ai --example issue_962_repro`.
//!
//! Before the seed fix in PR #976 the three Hindi/Chinese rows below printed
//! `intent=unknown`; the English and Russian rows already printed
//! `intent=calculation`. The three separated root causes were:
//!   1. `जमा` / `加` were not lexicalised as `addition` operator surfaces;
//!   2. `कितना होता है` was not a `calculation_result_query` cue, so even the
//!      already-seeded `जोड़` failed on the exact phrasing the issue reported;
//!   3. the same infix-vs-compound gap existed for minus/times/divide.
use formal_ai::{FormalAiEngine, UniversalSolver as _};

fn main() {
    for prompt in [
        // Already working before the fix — kept as the parity baseline.
        "What is 2 plus 2?",
        "Сколько будет 2 плюс 2?",
        // The three prompts reported in the issue.
        "2 जोड़ 2 कितना होता है?",
        "2 जमा 2 कितना होता है?",
        "2 加 2 等于多少?",
        // The holistic pass: the other operators had the same gap.
        "4 घटा 2 कितना है?",
        "4 减 2 等于多少?",
        "3 गुणा 2 कितना है?",
        "3 乘 2 等于多少?",
        "6 बटा 2 कितना है?",
        "6 除 2 等于多少?",
    ] {
        let response = FormalAiEngine.answer(prompt);
        let head: String = response.answer.chars().take(48).collect();
        println!(
            "{prompt:?} -> intent={} answer={}",
            response.intent,
            head.replace('\n', " | ")
        );
    }
}
