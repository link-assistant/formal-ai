#!/usr/bin/env rust-script
//! Locate the first issue #938 seed line rejected by canonical Links Notation.
//!
//! ```cargo
//! [dependencies]
//! links-notation = "0.13.0"
//! ```

use std::{env, fs};

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "data/seed/coding-idioms.lino".to_owned());
    let text = fs::read_to_string(path).expect("read Lino document");
    let lines = text.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| line.starts_with("  coding_meta_algorithm "))
        .expect("shared meta-algorithm block");

    for end in start..lines.len() {
        let candidate = lines[..=end].join("\n");
        if let Err(error) = links_notation::parse_lino(candidate.trim()) {
            println!("first rejected line {}: {}", end + 1, lines[end]);
            println!("{error}");
            return;
        }
    }
    println!("all {} lines parse", lines.len());
}
