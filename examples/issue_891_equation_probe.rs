//! Issue #891 — probe the production solver with equation prompts.
//!
//! The equation corpus in `data/benchmarks/equation-type-corpus.lino` records an
//! expected answer for every equation type. This example is the tool that
//! *verifies* those expectations against the live engine: it reads prompts (one
//! per line, `#` comments and blank lines skipped) from the file named on the
//! command line — or from stdin when no file is given — and prints one
//! tab-separated row per prompt:
//!
//! ```text
//! <prompt>\t<intent>\t<engine>\t<answer>
//! ```
//!
//! Run it with:
//!
//! ```bash
//! cargo run --example issue_891_equation_probe -- experiments/issue-891-equation-prompts.txt
//! ```

use std::env;
use std::fs;
use std::io::{self, Read};

use formal_ai::FormalAiEngine;

fn read_stdin() -> String {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("cannot read prompts from stdin");
    buffer
}

fn main() {
    let input = env::args().nth(1).map_or_else(read_stdin, |path| {
        fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read prompt list {path}: {err}"))
    });

    for line in input.lines() {
        let prompt = line.trim();
        if prompt.is_empty() || prompt.starts_with('#') {
            continue;
        }
        let response = FormalAiEngine.answer(prompt);
        let engine = response
            .evidence_links
            .iter()
            .find_map(|link| link.strip_prefix("calculation:engine:"))
            .unwrap_or("-");
        println!(
            "{prompt}\t{}\t{engine}\t{}",
            response.intent,
            response.answer.replace('\n', "\\n"),
        );
    }
}
