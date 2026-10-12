//! `formal-ai explain <answer-id>` — print the derivation record of a
//! previously returned answer (issue #1184, E148).
//!
//! The subcommand is deliberately distinct from the in-chat self-explanation
//! recipe (`agentic_coding::explain.rs`, keywords like "explain how Formal AI
//! works"): this surface takes a content-addressed answer id and reads a
//! durable record, it never inspects the running codebase (R7).

use crate::cli_paths::resolve_root;
use clap::ValueEnum;
use formal_ai::derivation::{Derivation, miss_message};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ExplainFormat {
    /// Readable per-stage explanation; a stage the route never populated is
    /// printed as "not recorded" (R5).
    Text,
    /// The canonical Links Notation record, as persisted under
    /// `data/cache/derivations/<answer_id>.lino`.
    Links,
}

pub fn run_explain(answer_id: &str, format: ExplainFormat) -> Result<(), Box<dyn Error>> {
    let root = resolve_root(None, Path::new("."));
    let derivation =
        Derivation::load(&root, answer_id).ok_or_else(|| miss_message(&root, answer_id))?;
    let rendered = match format {
        ExplainFormat::Text => derivation.explain_text(),
        ExplainFormat::Links => derivation.to_lino(),
    };
    print!("{rendered}");
    Ok(())
}
