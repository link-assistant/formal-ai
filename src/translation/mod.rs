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
pub mod wikidata;
pub mod wiktionary;

pub use cache::CachedHttpClient;
pub use formalization::{
    formalize_prompt, FormalizationAnchor, FormalizationAnchorKind, FormalizationCandidate,
    FormalizationRole, FormalizationSlot,
};
pub use formatting::match_source_formatting;
pub use http::{CurlClient, HttpError};
pub(crate) use language_markers::{detect_source_language, detect_target_language};
pub use pipeline::{Translation, TranslationPipeline};
pub use prompt::extract_unquoted_translation_surface;

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
