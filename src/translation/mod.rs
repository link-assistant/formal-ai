//! Translation pipeline.
//!
//! Translates an arbitrary natural-language fragment from one language to
//! another by going through three real, online resources:
//!
//! 1. **Wiktionary** — the source-edition entry hosts a translation table
//!    that lists candidate surface forms for every target language. When
//!    the source edition has no entry, we try the target-language edition
//!    in reverse (Russian Wiktionary, for instance, lists English
//!    translations under `=== Перевод ===` / `{{перев-блок}}`).
//!
//! 2. **Wikidata Lexemes** — when a Lexeme exists for the source surface,
//!    we run a SPARQL `?lemma` query that joins on `P5137` ("item for this
//!    sense") so any target-language Lexeme sharing that sense becomes a
//!    candidate. This also gives the [`MeaningId`] a language-neutral
//!    Wikidata anchor.
//!
//! 3. **No offline registry**. There are no hand-curated translation pairs
//!    in this module. Tests run against committed responses from the real
//!    APIs, captured under `data/translation-cache/`. Run with
//!    `FORMAL_AI_LIVE_API=1` to refresh the cache against the live network.
//!
//! ## Module layout
//!
//! - [`http`] — `curl`-backed HTTP client; mirrors [`crate::telegram_runtime`]
//!   so we don't pull a TLS crate into the core.
//! - [`cache`] — file cache for raw API responses keyed by URL.
//! - [`meaning`] — [`MeaningId`], the semantic meta-language identity.
//! - [`wiktionary`] — Wiktionary client + wikitext parser.
//! - [`wikidata`] — Wikidata SPARQL + Lexeme search client.
//! - [`formatting`] — typography mirror (case + terminal punctuation).
//! - [`pipeline`] — orchestration (`TranslationPipeline::translate`).
//!
//! ## Default wiring
//!
//! Most callers want a process-wide cached translator that reads from the
//! committed cache by default. Use [`default_translator`] for that.

use std::sync::OnceLock;

pub mod cache;
pub mod formatting;
pub mod http;
mod language_markers;
pub mod meaning;
pub mod pipeline;
pub mod prompt;
pub mod wikidata;
pub mod wiktionary;

pub use cache::CachedHttpClient;
pub use formatting::match_source_formatting;
pub use http::{CurlClient, HttpError};
pub(crate) use language_markers::{detect_source_language, detect_target_language};
pub use pipeline::{Translation, TranslationPipeline};
pub use prompt::extract_unquoted_translation_surface;

/// Process-wide cached HTTP client used by the default pipeline.
///
/// The client reads from `data/translation-cache/` first and falls through
/// to the live network only when `FORMAL_AI_LIVE_API=1` is set. This means
/// production callers can rely on the committed cache; integration runs
/// that refresh the cache opt in explicitly.
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
/// Errors propagate as [`HttpError`]; the caller decides whether to render
/// a placeholder or surface the error to the user.
pub fn translate_via_default_pipeline(
    surface: &str,
    source: &str,
    target: &str,
) -> Result<Translation, HttpError> {
    let pipeline = TranslationPipeline::new(default_cached_client());
    pipeline.translate(surface, source, target)
}
