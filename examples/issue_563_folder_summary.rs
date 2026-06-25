//! Issue #563 — recursive repository-resource summarization.
//!
//! Demonstrates that summarization is general over the repository tree: the same
//! entry point summarizes a single file *and* a directory (folder) of arbitrary
//! depth. A directory is summarized by the meta-algorithm loop applied
//! recursively — decompose into children, summarize each child, compose the
//! child summaries — with recursion depth bounded by the summarization mode
//! ladder (a `Full` folder describes its children in `Standard` detail, theirs
//! in `Short`, and everything deeper as a `Topic` label).
//!
//! Run with: `cargo run --example issue_563_folder_summary`

use formal_ai::summarization::{
    formalize_repository_resource, summarize_repository_resource, RepositoryEntry,
    SummarizationConfig, SummarizationMode,
};

fn main() {
    let tree = RepositoryEntry::directory(
        "src/summarization",
        vec![
            RepositoryEntry::file(
                "src/summarization/mod.rs",
                "//! Summarization pipeline.\npub fn summarize() {}\n",
            ),
            RepositoryEntry::file(
                "src/summarization/file.rs",
                "//! File summary.\npub struct RepositoryFileFormalization;\n\
                 pub fn formalize_repository_file() {}\n",
            ),
            RepositoryEntry::directory(
                "src/summarization/nested",
                vec![RepositoryEntry::file(
                    "src/summarization/nested/readme.md",
                    "# Nested\n\nThis explains nested things.\n\n```rust\npub fn x() {}\n```\n",
                )],
            ),
        ],
    );

    for mode in [
        SummarizationMode::Topic,
        SummarizationMode::Short,
        SummarizationMode::Standard,
        SummarizationMode::Full,
    ] {
        let config = SummarizationConfig::default().with_mode(mode);
        println!(
            "=== {mode:?} ===\n{}\n",
            summarize_repository_resource(&tree, &config)
        );
    }

    println!(
        "=== links_notation ===\n{}",
        formalize_repository_resource(&tree).links_notation()
    );
}
