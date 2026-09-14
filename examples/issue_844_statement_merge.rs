//! Issue #844 — merging many sources into one context.
//!
//! The issue's target behaviour, end to end and deterministically: point the
//! pipeline at a Stack Overflow question, recursively gather the question, its
//! answers and the material they link to, merge the text, deduplicate *facts*
//! rather than sentences, rank what remains by how often and how authoritatively
//! it is asserted, recheck the survivors, and present the important ones shorter
//! — down to a single identifier.
//!
//! Fetching goes through [`SourceProvider`] because the real network path is
//! issue #843's job; the documents below are the ones a fetcher would return.
//! Everything after the fetch is the shipped code.
//!
//! Run with: `cargo run --example issue_844_statement_merge`

use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::summarization::{
    FetchedSource, GatheringPlan, SourceCache, SourceProvider, SummarizationConfig,
    SummarizationMode, gather, merge_into_context,
};

/// The pages a fetcher would return for the question, keyed by URL.
struct StackOverflowThread {
    pages: Vec<FetchedSource>,
}

impl SourceProvider for StackOverflowThread {
    fn fetch(&mut self, url: &str) -> Option<FetchedSource> {
        println!("  fetch {url}");
        self.pages.iter().find(|page| page.url == url).cloned()
    }
}

fn thread() -> StackOverflowThread {
    StackOverflowThread {
        pages: vec![
            FetchedSource::new(
                "https://stackoverflow.com/q/1",
                SourceTier::OriginalJournalism,
                "How do I install formal-ai?",
            )
            .supplying("question")
            .linking("https://stackoverflow.com/a/1")
            .linking("https://stackoverflow.com/a/2")
            .linking("https://stackoverflow.com/a/3"),
            FetchedSource::new(
                "https://stackoverflow.com/a/1",
                SourceTier::IndependentCorroboration,
                "Install it with cargo install formal-ai. \
                 The crate is published on crates.io.",
            )
            .supplying("install")
            .linking("https://docs.rs/formal-ai"),
            // The same fact in different words: same terms, so one fact.
            FetchedSource::new(
                "https://stackoverflow.com/a/2",
                SourceTier::IndependentCorroboration,
                "The install is cargo install formal-ai.",
            )
            .supplying("install"),
            // A contradiction, kept as an edge rather than averaged away.
            FetchedSource::new(
                "https://stackoverflow.com/a/3",
                SourceTier::Unoriginal,
                "The crate is not published on crates.io.",
            ),
            FetchedSource::new(
                "https://docs.rs/formal-ai",
                SourceTier::OriginalFirstParty,
                "Install it with cargo install formal-ai.",
            )
            .supplying("install")
            // Back to the question: a citation cycle, ended by the fixpoint.
            .linking("https://stackoverflow.com/q/1"),
        ],
    }
}

fn main() {
    let plan = GatheringPlan::new("formal-ai installation", 3)
        .requiring("question")
        .requiring("install")
        .requiring("license")
        .seeded_with("https://stackoverflow.com/q/1");

    println!("=== recursive gathering (cold cache) ===");
    let mut cache = SourceCache::new();
    let gathered = gather(&plan, &mut thread(), &mut cache);
    println!("{}\n", gathered.trace());
    println!(
        "cached: {} urls, {} distinct bodies\n",
        cache.url_count(),
        cache.body_count()
    );

    println!("=== replay from the warm cache ===");
    let replayed = gather(&plan, &mut thread(), &mut cache);
    println!(
        "no fetch lines above means the provider was never called; \
         byte-identical trace: {}\n",
        replayed.trace() == gathered.trace()
    );

    let merged = merge_into_context("issue-844-stack-overflow", &gathered.observations);

    println!("=== merged facts, ranked by evidence ===");
    for item in &merged.ranked {
        println!(
            "{:>3}  p={:.3}  {}  [{}]",
            item.score.weight,
            item.probability.get(),
            item.statement.representative.text,
            item.evidence_summary(merged.total_sources()),
        );
    }
    println!();

    println!("=== merges, with the justification of each ===");
    for link in &merged.report.links {
        println!(
            "  \"{}\" from {} folded in on <{}>",
            link.absorbed, link.source, link.justification
        );
    }
    println!();

    println!("=== disagreements, reported rather than resolved ===");
    for line in merged.disagreements() {
        println!("  {line}");
    }
    println!();

    println!("=== recheck before presenting ===");
    println!("{}\n", merged.recheck().trace());

    println!("=== the ladder, all the way down ===");
    for mode in [
        SummarizationMode::Full,
        SummarizationMode::Standard,
        SummarizationMode::Short,
        SummarizationMode::Topic,
        SummarizationMode::Identifier,
    ] {
        // The thread asks how to install: the install command is the answer, not
        // the boilerplate a project summary would drop.
        let config = SummarizationConfig::default()
            .keeping_boilerplate()
            .with_mode(mode);
        println!("{mode:?}: {}", merged.checked_summary(&config));
    }
    println!();

    println!("=== the context as links ===\n{}", merged.links_notation());
}
