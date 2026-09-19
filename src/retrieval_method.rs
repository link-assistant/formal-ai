//! The M2 retrieval family interpreter (issue #1138 B9, plan 09 leaf 17).
//!
//! One retrieval procedure replaces the specialized retrieval handlers:
//! resolve the prompt's unknown subject, consult the sources the registry
//! declares for the need, read the capture cache or fetch, and render the
//! answer with provenance in the prompt's language. The module embeds no
//! domain vocabulary and no canned answer — the subjects come from the
//! lexicon's own unknown-surface rule, the sources come from
//! `data/seed/sources-registry.lino`, and both the with-capture template and
//! the no-capture response are seed records. What remains here is exactly the
//! part that cannot be expressed as links yet: the bounded walk, the cache
//! discipline, and the attribution.

use crate::SourceTransport;
use crate::concept_lookup::{ConceptSense, unknown_surfaces};
use crate::event_log::EventLog;
use crate::how_to_guide::ServicePreferences;
use crate::language::detect as detect_language;
use crate::service_accessibility::ServiceAccessibilityCache;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};
use crate::source_walk::LookupBounds;

/// Consult the registry for every surface, logging each outcome.
///
/// This is the one walk the M2 family runs. A hit is logged with the digest
/// and licence of the exact bytes it came from; a miss names every source
/// consulted, so an absence is attributable rather than bare; a source the
/// operator opted out of stays visible beside whatever the others answered.
pub fn walk_senses<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    preferences: &ServicePreferences,
    availability: &mut ServiceAccessibilityCache,
    bounds: LookupBounds,
    now: u64,
    surfaces: &[String],
    language: &str,
    log: &mut EventLog,
) -> Vec<ConceptSense> {
    if surfaces.is_empty() {
        return Vec::new();
    }
    log.append("retrieval_method:surfaces", surfaces.join("|"));
    let mut senses = Vec::new();
    for surface in surfaces.iter().take(bounds.max_services) {
        let outcome = crate::concept_lookup::lookup_surface(
            surface,
            language,
            client,
            preferences,
            &bounds,
            availability,
            now,
        );
        if outcome.items.is_empty() {
            log.append(
                "retrieval_method:miss",
                crate::trace_record::payload(&[
                    ("surface", surface.clone()),
                    (
                        "consulted",
                        outcome
                            .outcomes
                            .iter()
                            .map(|row| format!("{} {}", row.source_id, row.status))
                            .collect::<Vec<_>>()
                            .join(" "),
                    ),
                ]),
            );
        }
        for row in outcome
            .outcomes
            .iter()
            .filter(|row| row.status == "disabled")
        {
            log.append(
                "retrieval_method:disabled",
                crate::trace_record::payload(&[
                    ("surface", surface.clone()),
                    ("source", row.source_id.clone()),
                    ("settings_key", row.detail.clone()),
                ]),
            );
        }
        for sense in &outcome.items {
            log.append(
                "retrieval_method:hit",
                crate::trace_record::payload(&[
                    ("surface", surface.clone()),
                    ("source", sense.source_id.clone()),
                    ("sha256", sense.sha256.clone()),
                    ("license", sense.license_name.clone()),
                ]),
            );
        }
        senses.extend(outcome.items);
    }
    senses
}

/// Run the production retrieval procedure for one prompt.
///
/// The walk is bounded before it starts, reads the capture cache first, and
/// reaches the network only when the configuration and the environment both
/// allow it — the same discipline every other source consumer runs under.
pub fn retrieve(
    config: crate::solver::SolverConfig,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Vec<ConceptSense> {
    let language = detect_language(prompt).slug();
    let surfaces = unknown_surfaces(normalized, language);
    if surfaces.is_empty() {
        return Vec::new();
    }
    let bounds = LookupBounds::default();
    let cache_root = crate::coding::synthesis_runtime::source_cache_root();
    let online = !config.offline && crate::coding::synthesis_runtime::live_fetch_enabled();
    let client = CachedSourceClient::new(&cache_root, CurlSourceTransport).with_online(online);
    let preferences = crate::solver_handler_how_synthesis::service_preferences_from_env(log);
    let mut availability = ServiceAccessibilityCache::load(&cache_root);
    let now = crate::service_accessibility::unix_now();
    walk_senses(
        &client,
        &preferences,
        &mut availability,
        bounds,
        now,
        &surfaces,
        language,
        log,
    )
}

/// Render the retrieval answer: every capture beside its provenance, or the
/// seeded no-capture response when the walk found nothing.
///
/// The template is a seed record with `{lemma}`, `{gloss}`, `{source_name}`,
/// `{source_url}` and `{license_name}` placeholders; it carries all connective
/// prose so no answer wording is authored in Rust. A walk with captures and no
/// template degrades to the senses as links — data, never invented prose.
pub fn render_answer(senses: &[ConceptSense], template: Option<&str>, fallback: &str) -> String {
    if senses.is_empty() {
        return fallback.to_owned();
    }
    let Some(template) = template.map(str::trim).filter(|value| !value.is_empty()) else {
        return senses
            .iter()
            .map(ConceptSense::to_links_notation)
            .collect::<Vec<_>>()
            .join("\n");
    };
    senses
        .iter()
        .map(|sense| render_one(sense, template))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_one(sense: &ConceptSense, template: &str) -> String {
    let source_name = crate::seed::source_record(&sense.source_id)
        .map(|record| record.name)
        .unwrap_or_else(|| sense.source_id.clone());
    template
        .replace("{lemma}", &sense.lemma)
        .replace("{gloss}", &sense.gloss)
        .replace("{source_name}", &source_name)
        .replace("{source_url}", &sense.source_url)
        .replace("{license_name}", &sense.license_name)
}
