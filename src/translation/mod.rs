//! Translation pipeline.
//!
//! Translates an arbitrary natural-language fragment from one language to
//! another by running the full
//! `source → formalize → semantic meta language → deformalize → target`
//! flow on top of Wikipedia, Wikidata and Wiktionary. There is no
//! pre-extracted translation table built into the binary: every answer is
//! the result of an actual API round-trip (live or replayed from the
//! seeded raw-response cache).
//!
//! 1. **Formalize** — fetch the source-edition Wiktionary page and the
//!    Wikidata Lexeme / Q-item that backs the surface so the surface
//!    collapses to a language-neutral [`MeaningId`].
//!
//! 2. **Deformalize** — render that [`MeaningId`] back into the target
//!    language by joining on Wikidata `P5137` ("item for this sense") and
//!    by parsing translation tables (`{{trans-top}}`, `{{перев-блок}}`,
//!    `=== Translations ===` / `=== Перевод ===`) on either the source-
//!    or target-edition Wiktionary page.
//!
//! Every successful HTTP response is preserved verbatim under
//! [`cache::DEFAULT_CACHE_DIR`] keyed by **semantic identity** of the
//! resource (Wikidata Q-id, Wiktionary `(lang, page)`, SPARQL query hash,
//! …) so a single fetch can feed translation, fact lookup, attribute
//! formalization or any other formalization path. The first ~128 most
//! frequent Wikidata entities and ~128 most frequent properties — plus
//! the Wiktionary pages they point at — are committed to the repository
//! under [`cache::SEED_CACHE_DIR`] so unit tests, the browser worker and
//! a clean CI checkout can all run the full pipeline offline without
//! hitting the network. Live fetches are gated on
//! `FORMAL_AI_LIVE_API=1`.
//!
//! ## Module layout
//!
//! - [`http`] — `curl`-backed HTTP client; mirrors [`crate::telegram_runtime`]
//!   so we don't pull a TLS crate into the core.
//! - [`cache`] — semantic-identity file cache for raw API responses, with
//!   support for replaying responses from a committed `.lino` seed bundle.
//! - [`meaning`] — [`MeaningId`], the semantic meta-language identity.
//! - [`wiktionary`] — Wiktionary client + wikitext parser.
//! - [`wikidata`] — Wikidata SPARQL + Lexeme / entity / property client.
//! - [`formatting`] — typography mirror (case + terminal punctuation).
//! - [`pipeline`] — orchestration (`TranslationPipeline::translate`).
//!
//! ## Default wiring
//!
//! Most callers want a process-wide translator that consults the seeded
//! raw-response cache first and falls through to live HTTP only when
//! `FORMAL_AI_LIVE_API=1`. Use [`translate_via_default_pipeline`] for that.

use std::sync::OnceLock;

pub mod cache;
pub mod formalization;
pub mod formatting;
pub mod http;
mod language_markers;
pub mod meaning;
pub mod pipeline;
pub mod prompt;
pub mod selection;
pub mod wikidata;
pub mod wiktionary;

pub use cache::CachedHttpClient;
pub use formalization::{
    formalize_prompt, formalize_prompt_candidates, FormalizationAnchor, FormalizationAnchorKind,
    FormalizationCandidate, FormalizationRole, FormalizationSlot,
};
pub use formatting::match_source_formatting;
pub use http::{CurlClient, HttpError};
pub(crate) use language_markers::{detect_source_language, detect_target_language};
pub use pipeline::{Translation, TranslationPipeline};
pub use prompt::extract_unquoted_translation_surface;
pub use selection::{
    formalization_probability_target, select_formalization_candidate,
    select_formalization_candidate_with_probability_store, softmax_formalization_scores,
    FormalizationDecision, FormalizationSelection, FormalizationSelectionConfig,
    FormalizationSelectionReason,
};

/// Process-wide cached HTTP client used by the default pipeline.
///
/// The client reads from the committed seed cache and the gitignored
/// local accelerator under `data/` first and falls through to the live
/// network only when `FORMAL_AI_LIVE_API=1` is set. This keeps unit
/// tests offline by default; integration runs that refresh the cache
/// opt in explicitly.
fn default_cached_client() -> &'static CachedHttpClient<CurlClient> {
    static CLIENT: OnceLock<CachedHttpClient<CurlClient>> = OnceLock::new();
    CLIENT.get_or_init(|| CachedHttpClient::new(cache::DEFAULT_CACHE_DIR, CurlClient::default()))
}

/// Translate `surface` from `source` to `target` using the default pipeline.
///
/// Uses the process-wide cached translator and returns the primary
/// candidate surface form along with the meaning id, so callers can
/// both render the answer and embed the meaning id in their trace.
///
/// Errors propagate as [`HttpError`]; zero-candidate translations are
/// returned as explicit gaps so callers can surface a traceable miss
/// without manufacturing a target-language placeholder.
pub fn translate_via_default_pipeline(
    surface: &str,
    source: &str,
    target: &str,
) -> Result<Translation, HttpError> {
    let pipeline = TranslationPipeline::new(default_cached_client());
    pipeline.translate(surface, source, target)
}

#[cfg(test)]
mod parity {
    //! Frozen behavioural baseline for the lexicon-driven translation cluster
    //! (issue #386).
    //!
    //! Before the conversion, source/target detection and unquoted-surface
    //! extraction were three hand-written disjunctions over hardcoded
    //! natural-language strings. They are now projected from the meaning lexicon
    //! (`data/seed/meanings-translation.lino`) by semantic *role*, *slot*, and
    //! *script*. This 93-row battery pins the observable input → `(source,
    //! target, surface)` mapping so the data-driven path can never silently
    //! diverge from the behaviour it replaced.
    //!
    //! Every row reproduces the pre-conversion output verbatim **except** the
    //! two marked `GAP-FILL` rows: the original code had no Russian "from Hindi"
    //! / "from Chinese" source markers, so `apple с хинди` and `apple с
    //! китайского` detected no source language at all. Filling those gaps was
    //! forced by the all-four-languages seed invariant and is an honest
    //! improvement, not a regression — the rows are called out explicitly here
    //! so the change is visible in review.
    use super::{
        detect_source_language, detect_target_language, extract_unquoted_translation_surface,
    };

    /// `(prompt, expected source, expected target, expected surface)`.
    /// `"-"` denotes `None` — the same sentinel the capture harness used, so
    /// each row reads identically to the frozen golden TSV.
    const BATTERY: &[(&str, &str, &str, &str)] = &[
        // --- source markers: English ---------------------------------
        ("translate apple from english", "en", "-", "-"),
        ("переведи apple с английского", "en", "-", "-"),
        ("apple अंग्रेजी से", "en", "-", "-"),
        ("apple अंग्रेज़ी से", "en", "-", "-"),
        ("从英语翻译 apple", "en", "-", "-"),
        ("从英文翻译 apple", "en", "-", "-"),
        // --- source markers: Russian ---------------------------------
        ("translate apple from russian", "ru", "-", "-"),
        ("apple с русского", "ru", "-", "-"),
        ("apple रूसी से", "ru", "-", "-"),
        ("从俄语翻译 apple", "ru", "-", "-"),
        // --- source markers: Hindi -----------------------------------
        ("translate apple from hindi", "hi", "-", "-"),
        ("apple हिंदी से", "hi", "-", "-"),
        ("apple हिन्दी से", "hi", "-", "-"),
        ("从印地语翻译 apple", "hi", "-", "-"),
        ("从印地文翻译 apple", "hi", "-", "-"),
        // --- source markers: Chinese ---------------------------------
        ("translate apple from chinese", "zh", "-", "-"),
        ("apple चीनी से", "zh", "-", "-"),
        ("从中文翻译 apple", "zh", "-", "-"),
        ("从汉语翻译 apple", "zh", "-", "-"),
        ("从漢語翻译 apple", "zh", "-", "-"),
        // --- target markers: English ---------------------------------
        ("translate apple to english", "-", "en", "apple"),
        ("переведи apple на английский", "-", "en", "apple"),
        ("apple на английском", "-", "en", "-"),
        ("apple अंग्रेजी में", "-", "en", "-"),
        ("apple अंग्रेज़ी में", "-", "en", "-"),
        ("apple 成英文", "-", "en", "-"),
        ("apple 成英语", "-", "en", "-"),
        ("apple 为英文", "-", "en", "-"),
        ("apple 为英语", "-", "en", "-"),
        ("apple 為英文", "-", "en", "-"),
        ("apple 為英语", "-", "en", "-"),
        ("apple 到英文", "-", "en", "-"),
        ("apple 到英语", "-", "en", "-"),
        // --- target markers: Russian ---------------------------------
        ("translate apple to russian", "-", "ru", "apple"),
        ("apple на русский", "-", "ru", "-"),
        ("apple 成俄语", "-", "ru", "-"),
        ("apple 成俄語", "-", "ru", "-"),
        ("apple 为俄语", "-", "ru", "-"),
        ("apple 为俄語", "-", "ru", "-"),
        ("apple 為俄语", "-", "ru", "-"),
        ("apple 為俄語", "-", "ru", "-"),
        ("apple 到俄语", "-", "ru", "-"),
        ("apple 到俄語", "-", "ru", "-"),
        // --- target markers: Hindi -----------------------------------
        ("translate apple to hindi", "-", "hi", "apple"),
        ("apple на хинди", "-", "hi", "-"),
        ("apple हिंदी में", "-", "hi", "-"),
        ("apple हिन्दी में", "-", "hi", "-"),
        ("apple 成印地语", "-", "hi", "-"),
        ("apple 成印地文", "-", "hi", "-"),
        ("apple 为印地语", "-", "hi", "-"),
        ("apple 为印地文", "-", "hi", "-"),
        ("apple 為印地语", "-", "hi", "-"),
        ("apple 為印地文", "-", "hi", "-"),
        ("apple 到印地语", "-", "hi", "-"),
        ("apple 到印地文", "-", "hi", "-"),
        // --- target markers: Chinese ---------------------------------
        ("translate apple to chinese", "-", "zh", "apple"),
        ("apple на китайский", "-", "zh", "-"),
        ("apple चीनी में", "-", "zh", "-"),
        ("apple 成中文", "-", "zh", "-"),
        ("apple 成汉语", "-", "zh", "-"),
        ("apple 成漢語", "-", "zh", "-"),
        ("apple 为中文", "-", "zh", "-"),
        ("apple 为汉语", "-", "zh", "-"),
        ("apple 为漢語", "-", "zh", "-"),
        ("apple 為中文", "-", "zh", "-"),
        ("apple 為汉语", "-", "zh", "-"),
        ("apple 為漢語", "-", "zh", "-"),
        ("apple 到中文", "-", "zh", "-"),
        ("apple 到汉语", "-", "zh", "-"),
        ("apple 到漢語", "-", "zh", "-"),
        // --- combined source+target ----------------------------------
        (
            "translate apple from english to russian",
            "en",
            "ru",
            "apple from english",
        ),
        (
            "переведи яблоко с английского на русский",
            "en",
            "ru",
            "яблоко с английского",
        ),
        ("把 apple 从中文 翻译成英文", "zh", "en", "apple 从中文"),
        // --- extraction: English circumfix ---------------------------
        ("translate apple to russian", "-", "ru", "apple"),
        ("Translate Apple to Russian", "-", "ru", "Apple"),
        ("translate apple to russian.", "-", "ru", "apple"),
        ("translate \"apple\" to russian", "-", "ru", "-"),
        ("translate apple", "-", "-", "-"),
        ("what is apple", "-", "-", "-"),
        (
            "translate the red apple to russian",
            "-",
            "ru",
            "the red apple",
        ),
        // --- extraction: Russian circumfix ---------------------------
        ("переведи яблоко на английский", "-", "en", "яблоко"),
        (
            "переведи красное яблоко на английский",
            "-",
            "en",
            "красное яблоко",
        ),
        // --- extraction: Hindi ---------------------------------------
        ("apple का हिंदी में अनुवाद करो", "-", "hi", "apple"),
        ("सेब को अंग्रेजी में अनुवाद करो", "-", "en", "सेब"),
        // `हिंदी मे` (मे, not में) is not a target marker, so target is None,
        // but the object particle `का` still bounds the surface — asymmetry
        // present in the original behaviour and preserved verbatim.
        ("apple का हिंदी मे अनुवाद करो", "-", "-", "apple"),
        // --- extraction: Chinese -------------------------------------
        ("把 apple 翻译成中文", "-", "zh", "apple"),
        ("将苹果翻译成英文", "-", "en", "苹果"),
        ("翻译 apple 成中文", "-", "zh", "apple"),
        ("把 apple 翻译为英文", "-", "en", "apple"),
        ("把 apple 翻译到英文", "-", "en", "apple"),
        // --- GAP-FILL: original code had no Russian "from Hindi" /
        //     "from Chinese" source markers, so these were `-`/`-`/`-`.
        //     The seed now supplies them; source detection improves to hi/zh.
        ("apple с хинди", "hi", "-", "-"),
        ("apple с китайского", "zh", "-", "-"),
        // `रूसी में` already carried a Russian *target* marker pre-conversion.
        ("apple रूसी में", "-", "ru", "-"),
    ];

    fn opt(sentinel: &str) -> Option<&str> {
        (sentinel != "-").then_some(sentinel)
    }

    #[test]
    fn translation_cluster_matches_frozen_behaviour() {
        for &(prompt, source, target, surface) in BATTERY {
            let normalized = prompt.to_lowercase();
            assert_eq!(
                detect_source_language(&normalized),
                opt(source),
                "source language mismatch for {prompt:?}"
            );
            assert_eq!(
                detect_target_language(&normalized),
                opt(target),
                "target language mismatch for {prompt:?}"
            );
            assert_eq!(
                extract_unquoted_translation_surface(prompt).as_deref(),
                opt(surface),
                "extracted surface mismatch for {prompt:?}"
            );
        }
    }
}
