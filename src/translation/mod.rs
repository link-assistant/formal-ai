//! Translation pipeline.
//!
//! Translates an arbitrary natural-language fragment from one language to
//! another by combining a tiny offline dictionary with live HTTP fallbacks:
//!
//! 1. **Offline dictionary** (`data/seed/translations.lino`, ≤128 entries) —
//!    the [`dictionary`] module embeds a single Links Notation file that
//!    lists the most common nouns we want to translate without hitting the
//!    network. It is the only built-in translation data; it ships in both
//!    the native binary (via `include_str!`) and the browser worker (which
//!    fetches the same file).
//!
//! 2. **Wiktionary** — the source-edition entry hosts a translation table
//!    that lists candidate surface forms for every target language. When
//!    the source edition has no entry, we try the target-language edition
//!    in reverse (Russian Wiktionary, for instance, lists English
//!    translations under `=== Перевод ===` / `{{перев-блок}}`).
//!
//! 3. **Wikidata Lexemes** — when a Lexeme exists for the source surface,
//!    we run a SPARQL `?lemma` query that joins on `P5137` ("item for this
//!    sense") so any target-language Lexeme sharing that sense becomes a
//!    candidate. This also gives the [`MeaningId`] a language-neutral
//!    Wikidata anchor.
//!
//! Stages 2 + 3 only run when the dictionary misses, and even then they
//! are gated on `FORMAL_AI_LIVE_API=1` so unit tests stay offline.
//! Successful responses are cached on disk by semantic identity (Wikidata
//! Q-id, Wiktionary `(lang, lemma)`) under `data/wikidata-cache/` and
//! `data/wiktionary-cache/` so the same data can be reused by other
//! formalization paths — not just translation.
//!
//! ## Module layout
//!
//! - [`dictionary`] — 128-noun offline lookup, shared with the browser worker.
//! - [`http`] — `curl`-backed HTTP client; mirrors [`crate::telegram_runtime`]
//!   so we don't pull a TLS crate into the core.
//! - [`cache`] — semantic-identity file cache for raw API responses.
//! - [`meaning`] — [`MeaningId`], the semantic meta-language identity.
//! - [`wiktionary`] — Wiktionary client + wikitext parser.
//! - [`wikidata`] — Wikidata SPARQL + Lexeme search client.
//! - [`formatting`] — typography mirror (case + terminal punctuation).
//! - [`pipeline`] — orchestration (`TranslationPipeline::translate`).
//!
//! ## Default wiring
//!
//! Most callers want a process-wide translator that consults the offline
//! dictionary first and falls through to the cached HTTP pipeline. Use
//! [`translate_via_default_pipeline`] for that.

use std::sync::OnceLock;

pub mod cache;
pub mod dictionary;
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
/// The client reads from the semantic-identity cache directories under
/// `data/` first and falls through to the live network only when
/// `FORMAL_AI_LIVE_API=1` is set. This keeps unit tests offline by default;
/// integration runs that refresh the cache opt in explicitly.
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
