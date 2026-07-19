//! Forced-language seam coverage (issues #556, #658).
//!
//! `language::detect` carries a single forced-language seam so a
//! response-language follow-up can replay every localizable handler in the
//! requested language. Issue #658 split that seam into two cfg-gated
//! `forced_language` backends — a `thread_local!` cell on native builds and a
//! single-threaded `static` cell on the `wasm32` `no_std` worker — while
//! keeping identical behaviour. These tests pin that behaviour for every
//! supported language (`en`, `ru`, `hi`, `zh`) so the split can never silently
//! regress one language.

use formal_ai::language::{detect, set_forced_language, Language};

/// A prompt whose natural detection differs from the language we force onto it,
/// paired with the slug the forced seam must emit.
struct ForcedCase {
    /// Prompt written in one script...
    prompt: &'static str,
    /// ...whose natural detection we assert first.
    natural: Language,
    /// The language forced onto the seam for the replay.
    forced: Language,
    /// The `language:<slug>` slug the forced language must render.
    slug: &'static str,
}

const CASES: &[ForcedCase] = &[
    // english: a Latin prompt forced to stay English.
    ForcedCase {
        prompt: "Hello there",
        natural: Language::English,
        forced: Language::English,
        slug: "en",
    },
    // russian: an English prompt replayed with Russian forced onto it.
    ForcedCase {
        prompt: "Hello there",
        natural: Language::English,
        forced: Language::Russian,
        slug: "ru",
    },
    // hindi: a Cyrillic prompt replayed with Hindi forced onto it.
    ForcedCase {
        prompt: "Привет",
        natural: Language::Russian,
        forced: Language::Hindi,
        slug: "hi",
    },
    // chinese: a Devanagari prompt replayed with Chinese forced onto it.
    ForcedCase {
        prompt: "नमस्ते",
        natural: Language::Hindi,
        forced: Language::Chinese,
        slug: "zh",
    },
];

#[test]
fn forced_language_overrides_detection_for_every_supported_language() {
    for case in CASES {
        // The seam is inert before it is set: detection is natural.
        assert_eq!(
            detect(case.prompt),
            case.natural,
            "prompt {:?} should detect as {:?} with no forced language",
            case.prompt,
            case.natural,
        );

        {
            let _guard = set_forced_language(Some(case.forced));
            assert_eq!(
                detect(case.prompt),
                case.forced,
                "forcing {:?} must override detection of {:?}",
                case.forced,
                case.prompt,
            );
            assert_eq!(
                detect(case.prompt).slug(),
                case.slug,
                "forced {:?} must render slug {}",
                case.forced,
                case.slug,
            );
        }

        // Dropping the guard restores natural detection — the seam is a scoped
        // override, not a permanent mode switch.
        assert_eq!(
            detect(case.prompt),
            case.natural,
            "dropping the guard must restore natural detection of {:?}",
            case.prompt,
        );
    }
}

#[test]
fn nested_forced_language_guards_restore_the_previous_language() {
    // english outermost, then russian, then hindi, then chinese — each drop
    // peels back exactly one layer.
    let _english = set_forced_language(Some(Language::English));
    assert_eq!(detect("anything").slug(), "en");
    {
        let _russian = set_forced_language(Some(Language::Russian));
        assert_eq!(detect("anything").slug(), "ru");
        {
            let _hindi = set_forced_language(Some(Language::Hindi));
            assert_eq!(detect("anything").slug(), "hi");
            {
                let _chinese = set_forced_language(Some(Language::Chinese));
                assert_eq!(detect("anything").slug(), "zh");
            }
            // chinese dropped -> back to hindi.
            assert_eq!(detect("anything").slug(), "hi");
        }
        // hindi dropped -> back to russian.
        assert_eq!(detect("anything").slug(), "ru");
    }
    // russian dropped -> back to english.
    assert_eq!(detect("anything").slug(), "en");
}
