//! Probe the deterministic engine with one prompt per stdin line.
//!
//! Issue #933 backfills a conversational variation corpus that must hold at
//! least five wording variations per language for every test case. A candidate
//! wording is only worth committing if the engine actually routes it, so this
//! probe answers each line and prints the routed intent plus the evidence links
//! it cited, tab separated:
//!
//!     printf 'hi\nпривет\n' | cargo run --example issue_933_variation_probe
//!     hi\tgreeting\tresponse:greeting\tHello — what would you like to explore?
//!
//! Kept in `experiments/` (CONTRIBUTING: keep the scripts that produced the
//! evidence) and wired as a cargo example through `examples/`.

use std::io::{self, BufRead};

use formal_ai::FormalAiEngine;

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let prompt = line.expect("stdin line");
        let prompt = prompt.trim_end_matches('\r');
        if prompt.is_empty() {
            continue;
        }
        let response = FormalAiEngine.answer(prompt);
        let answer = response.answer.replace('\n', " ⏎ ");
        let responses: Vec<&str> = response
            .evidence_links
            .iter()
            .filter(|link| link.starts_with("response:"))
            .map(String::as_str)
            .collect();
        println!(
            "{prompt}\t{}\t{}\t{answer}",
            response.intent,
            responses.join(","),
        );
    }
}
