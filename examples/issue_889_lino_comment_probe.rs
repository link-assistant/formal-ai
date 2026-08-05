//! Issue #889: check that the generated thinking seed files are canonical Links
//! Notation.
//!
//! Two encoding rules bit this seed during development and are easy to trip
//! again when the generator's header text is edited: `parse_lino` reads a colon
//! inside a `#` comment as structure, and it reads a blank line between the
//! header and the root as the end of the document. Run this after
//! `python3 experiments/issue-889/generate_thinking_seed.py` for a fast answer
//! before the slower `cargo test --test unit -- data_files::` gate.
use links_notation::parse_lino as parse_canonical_lino;

const SEEDS: &[&str] = &[
    "data/seed/multilingual-responses-thinking.lino",
    "data/seed/multilingual-responses-thinking-narrative.lino",
];

fn main() {
    let mut failed = false;
    for path in SEEDS {
        let content = std::fs::read_to_string(path).expect("read the seed file");
        match parse_canonical_lino(content.trim()) {
            Ok(_) => println!("{path}: parses"),
            Err(error) => {
                failed = true;
                println!("{path}: INVALID -- {error}");
                for (number, line) in content.lines().enumerate() {
                    if line.starts_with('#') && line.contains(':') {
                        println!("  line {}: comment contains a colon", number + 1);
                    }
                    if line.is_empty() {
                        println!("  line {}: blank line ends the document", number + 1);
                    }
                }
            }
        }
    }
    assert!(
        !failed,
        "the issue #889 seed files must be canonical Links Notation"
    );
}
