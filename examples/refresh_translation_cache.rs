//! Refresh the translation cache from the live Wikipedia / Wikidata /
//! Wiktionary APIs.
//!
//! Run with:
//!
//! ```bash
//! FORMAL_AI_LIVE_API=1 cargo run --example refresh_translation_cache
//! ```
//!
//! The example exercises every phrase used by
//! `tests/unit/specification/translation_via_links.rs` so the resulting
//! `data/translation-cache/` files cover the offline test suite end-to-end.
//!
//! The example is **idempotent**: it re-runs the pipeline against each
//! pair, so already-cached entries are re-used and missing entries are
//! fetched once. It exits non-zero if any pair returns an empty candidate
//! list — that surfaces gaps in the live source data before commit.

use formal_ai::translation::{CachedHttpClient, CurlClient, TranslationPipeline};

fn main() {
    let pairs: &[(&str, &str, &str)] = &[
        // Russian → English
        ("как у тебя дела", "ru", "en"),
        ("как дела", "ru", "en"),
        ("спасибо", "ru", "en"),
        ("привет", "ru", "en"),
        ("да", "ru", "en"),
        ("нет", "ru", "en"),
        // Issue #217: single Russian noun
        ("яблоко", "ru", "en"),
        // English → Russian
        ("hello", "en", "ru"),
        ("thank you", "en", "ru"),
        ("yes", "en", "ru"),
        ("no", "en", "ru"),
        // Issue #216: single English noun
        ("apple", "en", "ru"),
        // English → Hindi
        ("hello", "en", "hi"),
        ("apple", "en", "hi"),
        // English → Chinese
        ("hello", "en", "zh"),
        ("apple", "en", "zh"),
    ];

    let cache_dir = std::env::var("FORMAL_AI_TRANSLATION_CACHE_DIR")
        .unwrap_or_else(|_| "data/translation-cache".to_owned());
    println!("cache_dir = {cache_dir}");
    println!(
        "live = {} (set FORMAL_AI_LIVE_API=1 to hit the network)",
        std::env::var("FORMAL_AI_LIVE_API").unwrap_or_default(),
    );

    let http = CachedHttpClient::new(&cache_dir, CurlClient::default());
    let pipeline = TranslationPipeline::new(&http);

    let mut gaps: Vec<String> = Vec::new();
    for (surface, source, target) in pairs {
        match pipeline.translate(surface, source, target) {
            Ok(translation) => {
                let candidate = translation
                    .primary_surface()
                    .unwrap_or("<empty>")
                    .to_owned();
                println!(
                    "{source}→{target} \"{surface}\" -> \"{candidate}\" \
                     (meaning={}, provenance={:?})",
                    translation.meaning, translation.provenance,
                );
                if translation.candidates.is_empty() {
                    gaps.push(format!("{source}→{target} \"{surface}\""));
                }
            }
            Err(error) => {
                eprintln!("{source}→{target} \"{surface}\" -> ERROR: {error}");
                gaps.push(format!("{source}→{target} \"{surface}\""));
            }
        }
    }

    if !gaps.is_empty() {
        eprintln!("\n{} translation gap(s):", gaps.len());
        for gap in &gaps {
            eprintln!("  - {gap}");
        }
        std::process::exit(1);
    }
    println!("\nAll {} pairs cached successfully.", pairs.len());
}
