//! Debug `extract_translations` against cached Wiktionary pages.

use formal_ai::translation::{
    wiktionary::{extract_translation_blocks, Wiktionary},
    CachedHttpClient, CurlClient,
};

fn main() {
    let cache_dir = std::env::var("FORMAL_AI_TRANSLATION_CACHE_DIR")
        .unwrap_or_else(|_| "data/translation-cache".to_owned());
    let http = CachedHttpClient::new(cache_dir, CurlClient::default());

    let pages = [
        ("en", "hello", "zh"),
        ("zh", "你好", "en"),
        ("zh", "喂", "en"),
        ("ru", "как у тебя дела", "en"),
        ("ru", "как дела", "en"),
    ];
    for (edition, page, target) in pages {
        let wiktionary = Wiktionary::new(edition, &http);
        match wiktionary.wikitext(page) {
            Ok(wikitext) => {
                let blocks = extract_translation_blocks(&wikitext, target);
                println!("{edition}:{page:?} -> {target} ({} blocks)", blocks.len());
                for (idx, block) in blocks.iter().enumerate() {
                    let surfaces: Vec<String> = block.iter().map(|c| c.surface.clone()).collect();
                    println!("  block {idx}: {surfaces:?}");
                }
            }
            Err(error) => println!("{edition}:{page:?} -> ERROR {error}"),
        }
    }
}
