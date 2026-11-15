// Issue #1550: an isolated type and lint check of rust/src/meta_executor.rs
// with stub dependencies (no crate build). Usage:
//   clippy-driver --edition 2024 --crate-type lib --emit=metadata -o /tmp/meta.rmeta \
//     -W clippy::all -W clippy::pedantic -W clippy::nursery experiments/issue-1550/rust-typecheck/lib.rs
#![allow(dead_code)]
pub mod coding {
    #[must_use]
    pub fn contains_cjk(_text: &str) -> bool {
        false
    }
}
pub mod formalization {
    #[must_use]
    pub fn structural_rule_values(_rule: &str, _kind: &str) -> Vec<String> {
        Vec::new()
    }
    #[must_use]
    pub fn grammar_vocabulary(_language: &str, _kind: &str) -> Vec<String> {
        Vec::new()
    }
}
#[path = "../../../rust/src/least_action.rs"]
pub mod least_action;
#[path = "../../../rust/src/meta_executor.rs"]
pub mod meta_executor;
