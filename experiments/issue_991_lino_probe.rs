#!/usr/bin/env rust-script
//! Probe which line of a Links Notation file the canonical parser rejects.
//!
//! Issue #991: the new `data/meta/` registries must parse with the same
//! `links-notation` crate `tests/unit/data_files.rs` uses, and a whole-file
//! `Eof` error says nothing about where the trouble is. This bisects by
//! truncating the file line by line and reporting the first prefix that fails.
//!
//! Usage: rust-script experiments/issue_991_lino_probe.rs data/meta/seed-registry.lino
//!
//! ```cargo
//! [dependencies]
//! links-notation = "0.13.0"
//! ```

use links_notation::parse_lino;

fn main() {
    for path in std::env::args().skip(1) {
        let source = std::fs::read_to_string(&path).expect("readable");
        let lines: Vec<&str> = source.lines().collect();
        let mut first_bad: Option<usize> = None;
        for end in 1..=lines.len() {
            let prefix = lines[..end].join("\n");
            if parse_lino(prefix.trim()).is_err() {
                first_bad = Some(end);
                break;
            }
        }
        // A prefix can fail while the file is fine -- a block header truncated
        // before its children is incomplete -- so report the whole file too.
        let whole = parse_lino(source.trim());
        match (first_bad, whole) {
            (_, Err(error)) => println!("{path}: REJECTED -- {error}"),
            (None, Ok(_)) => println!("{path}: parses"),
            (Some(end), Ok(_)) => println!(
                "{path}: parses whole; prefix through line {end} is incomplete: {:?}",
                lines[end - 1]
            ),
        }
    }
}
