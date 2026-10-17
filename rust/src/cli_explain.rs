//! `formal-ai explain <answer-id>` — print the derivation record of a
//! previously returned answer (issue #1184, E148).
//!
//! The subcommand is deliberately distinct from the in-chat self-explanation
//! recipe (`agentic_coding::explain.rs`, keywords like "explain how Formal AI
//! works"): this surface takes a content-addressed answer id and reads a
//! durable record, it never inspects the running codebase (R7).

use clap::ValueEnum;
use formal_ai::runtime_cache::{load_derivation, runtime_cache_paths};
use std::error::Error;

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
    let derivation = load_derivation(answer_id)?.ok_or_else(|| {
        let directory = runtime_cache_paths().map_or_else(
            |error| error.to_string(),
            |paths| paths.derivation_directory.display().to_string(),
        );
        formal_ai::seed::report_text(
            "runtime_derivation_record_missing",
            &[("answer_id", answer_id), ("directory", &directory)],
        )
    })?;
    let rendered = match format {
        ExplainFormat::Text => derivation.explain_text(),
        ExplainFormat::Links => derivation.to_lino(),
    };
    print!("{rendered}");
    Ok(())
}
