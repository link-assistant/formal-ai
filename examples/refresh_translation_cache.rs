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
    // Common-noun seed list. Each entry produces both directions
    // (ru↔en) plus an en→hi and en→zh fan-out so the demo covers every
    // supported language for the same word. The set targets the
    // vocabulary the umbrella issue #221 demands: any common noun,
    // not just the seeded phrase set from #216 / #217.
    let common_english_nouns = [
        "apple",
        "tomato",
        "cucumber",
        "potato",
        "carrot",
        "onion",
        "garlic",
        "bread",
        "milk",
        "water",
        "tea",
        "coffee",
        "sugar",
        "salt",
        "cheese",
        "butter",
        "egg",
        "rice",
        "pasta",
        "fish",
        "meat",
        "chicken",
        "beef",
        "pork",
        "lemon",
        "orange",
        "banana",
        "grape",
        "strawberry",
        "peach",
        "pear",
        "watermelon",
        "cabbage",
        "pepper",
        "mushroom",
        "house",
        "car",
        "book",
        "dog",
        "cat",
        "horse",
        "bird",
        "tree",
        "flower",
        "river",
        "mountain",
        "sun",
        "moon",
        "star",
        "sea",
        "city",
        "table",
        "chair",
        "door",
        "window",
        "school",
        "hospital",
        "doctor",
        "teacher",
        "student",
        "child",
        "friend",
        "family",
        "mother",
        "father",
        "brother",
        "sister",
        "computer",
        "phone",
        "music",
        "language",
        "country",
    ];
    let common_russian_nouns = [
        "яблоко",
        "помидор",
        "огурец",
        "картофель",
        "морковь",
        "лук",
        "чеснок",
        "хлеб",
        "молоко",
        "вода",
        "чай",
        "кофе",
        "сахар",
        "соль",
        "сыр",
        "масло",
        "яйцо",
        "рис",
        "макароны",
        "рыба",
        "мясо",
        "курица",
        "говядина",
        "свинина",
        "лимон",
        "апельсин",
        "банан",
        "виноград",
        "клубника",
        "персик",
        "груша",
        "арбуз",
        "капуста",
        "перец",
        "гриб",
        "дом",
        "машина",
        "книга",
        "собака",
        "кошка",
        "лошадь",
        "птица",
        "дерево",
        "цветок",
        "река",
        "гора",
        "солнце",
        "луна",
        "звезда",
        "море",
        "город",
        "стол",
        "стул",
        "дверь",
        "окно",
        "школа",
        "больница",
        "врач",
        "учитель",
        "студент",
        "ребёнок",
        "друг",
        "семья",
        "мать",
        "отец",
        "брат",
        "сестра",
        "компьютер",
        "телефон",
        "музыка",
        "язык",
        "страна",
    ];

    let mut pairs: Vec<(String, &str, &str)> = Vec::new();
    // Russian → English phrases used by greeting/regression tests.
    for surface in [
        "как у тебя дела",
        "как дела",
        "спасибо",
        "привет",
        "да",
        "нет",
    ] {
        pairs.push((surface.to_owned(), "ru", "en"));
    }
    // English → Russian phrases used by greeting/regression tests.
    for surface in ["hello", "thank you", "yes", "no"] {
        pairs.push((surface.to_owned(), "en", "ru"));
        pairs.push((surface.to_owned(), "en", "hi"));
        pairs.push((surface.to_owned(), "en", "zh"));
    }
    // Issue #221 (umbrella): common-noun coverage in every direction so
    // `Переведи "помидор" на английский.` and friends work offline.
    for english in &common_english_nouns {
        pairs.push(((*english).to_owned(), "en", "ru"));
        pairs.push(((*english).to_owned(), "en", "hi"));
        pairs.push(((*english).to_owned(), "en", "zh"));
    }
    for russian in &common_russian_nouns {
        pairs.push(((*russian).to_owned(), "ru", "en"));
    }

    let cache_dir = std::env::var("FORMAL_AI_TRANSLATION_CACHE_DIR")
        .unwrap_or_else(|_| formal_ai::translation::cache::DEFAULT_CACHE_DIR.to_owned());
    println!("cache_dir = {cache_dir}");
    println!(
        "live = {} (set FORMAL_AI_LIVE_API=1 to hit the network)",
        std::env::var("FORMAL_AI_LIVE_API").unwrap_or_default(),
    );

    let http = CachedHttpClient::new(&cache_dir, CurlClient::default());
    let pipeline = TranslationPipeline::new(&http);

    let total = pairs.len();
    let mut gaps: Vec<String> = Vec::new();
    for (surface, source, target) in &pairs {
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
    println!("\nAll {total} pairs cached successfully.");
}
