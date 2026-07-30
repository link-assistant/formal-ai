//! Probe: what does the identifier rung (issue #844) actually render?
//!
//! Not a cargo target on its own — copy it to `examples/` and run
//! `cargo run --example issue-844-identifier-probe` to reproduce. Its recorded
//! output lives next to it in `issue-844-identifier-probe.txt`; it is what the
//! `ROLE_IDENTIFIER_RESERVED_WORD` doc example in `src/seed/roles/program.rs`
//! was checked against ("the type" -> `type_`, not the reserved `type`).

use formal_ai::summarization::identifier::{to_identifier, IdentifierBudget, NamingConvention};

fn main() {
    for phrase in [
        "the type of a match",
        "the type",
        "the type of the statement",
        "A deterministic summarizer merges statements from many sources",
        "Do not use the reserved word type",
        "if",
    ] {
        for convention in [
            NamingConvention::SnakeCase,
            NamingConvention::CamelCase,
            NamingConvention::PascalCase,
            NamingConvention::CommitSubject,
            NamingConvention::ScreamingSnakeCase,
        ] {
            println!(
                "{phrase:?} {convention:?} -> {:?}",
                to_identifier(phrase, convention, &IdentifierBudget::default())
            );
        }
        println!(
            "{phrase:?} commit -> {:?}",
            to_identifier(phrase, NamingConvention::SnakeCase, &IdentifierBudget::commit_subject())
        );
    }
}
