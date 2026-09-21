//! The executed half of the procedural "how to X" handler — issue #991.
//!
//! [`crate::solver_handler_how::try_how_to_procedure`] records the *plan*: the
//! services it would consult and the gates it would apply. That plan is still
//! the right answer when nothing can be retrieved, but issue #991 requires the
//! native path to actually run the same bounded source-selection and synthesis
//! contract the browser worker runs, and to answer from the captured bytes when
//! it succeeds.
//!
//! This module is that seam. It reads the user's service opt-outs, walks the
//! enabled registry services through [`crate::how_to_guide`], persists the
//! per-service accessibility record, and answers with the synthesised guide —
//! falling back to the discovery plan, unchanged, whenever the evidence is
//! insufficient. Nothing here reaches the network by itself: every byte arrives
//! through [`CachedSourceClient`], so an offline run replays committed captures
//! and a live run refreshes them along the identical code path.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::how_to_guide::{GuideBounds, HowToGuide, ServicePreferences, synthesize_how_to_guide};
use crate::seed::external_service_settings_keys;
use crate::service_accessibility::{ServiceAccessibilityCache, unix_now};
use crate::solver_handler_how::{procedural_how_to_task, try_how_to_procedure};
use crate::solver_handlers::finalize_simple;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};

/// Comma-separated `settings_key`s the environment opts out of, mirroring the
/// browser settings toggles for runtimes that have no settings UI (the CLI, the
/// HTTP server, and the test suites).
pub const DISABLED_SERVICES_ENV: &str = "FORMAL_AI_DISABLED_SERVICES";

/// Environment override for where the capture cache and the accessibility
/// record live; the same variable the executed web search reads.
const CACHE_DIR_ENV: &str = "FORMAL_AI_SOURCE_CACHE_DIR";

/// Service opt-outs declared by the environment.
///
/// Only keys the registry actually declares are honoured, so a typo disables
/// nothing silently: it is reported as `settings:unknown_service_key` instead.
#[must_use]
pub fn service_preferences_from_env(log: &mut EventLog) -> ServicePreferences {
    let Ok(raw) = std::env::var(DISABLED_SERVICES_ENV) else {
        return ServicePreferences::default();
    };
    let declared = external_service_settings_keys();
    let mut preferences = ServicePreferences::default();
    for key in raw.split(',').map(str::trim).filter(|key| !key.is_empty()) {
        if declared.iter().any(|declared| declared == key) {
            log.append("settings:service_disabled", key.to_owned());
            preferences = preferences.with(key, false);
        } else {
            log.append("settings:unknown_service_key", key.to_owned());
        }
    }
    preferences
}

/// Execute the recognised procedural request through a caller-supplied capture
/// client, the caller's settings, and the caller's accessibility record.
///
/// This seam keeps tests deterministic: the same call replays a committed
/// capture tree offline and refreshes it online, and both runs must produce the
/// identical guide, trace, and evidence.
pub fn try_how_to_procedure_with_client<T: SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
    preferences: &ServicePreferences,
    availability: &mut ServiceAccessibilityCache,
) -> Option<SymbolicAnswer> {
    let task = procedural_how_to_task(normalized)?;
    let bounds = GuideBounds::default();
    let guide = synthesize_how_to_guide(
        &task,
        client,
        preferences,
        &bounds,
        availability,
        unix_now(),
    );
    if let Err(error) = availability.save() {
        log.append("service_accessibility:save_failed", error.to_string());
    }
    guide.record(log);
    if !guide.is_sufficient() {
        // Not enough corroborated procedure to assert steps. The discovery plan
        // is still a truthful answer, and it is exactly what the runtime
        // answered before this handler existed, so the fallback is a strict
        // superset of the prior behaviour rather than a replacement.
        log.append("procedural_how_to:fallback", "discovery_plan".to_owned());
        return try_how_to_procedure(prompt, normalized, log);
    }
    Some(answer_synthesized_guide(prompt, &guide, log))
}

/// Use the process source cache and the environment's settings.
///
/// Transport is enabled only after the runtime has explicitly opted into live
/// fetches. Offline mode still replays valid captures, so a committed QA
/// capture tree produces the full guide with no network at all.
pub fn try_how_to_procedure_with_offline(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    let cache_dir = std::env::var(CACHE_DIR_ENV).unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(&cache_dir, CurlSourceTransport).with_online(!offline);
    let preferences = service_preferences_from_env(log);
    let mut availability = ServiceAccessibilityCache::load(&cache_dir);
    try_how_to_procedure_with_client(
        prompt,
        normalized,
        log,
        &client,
        &preferences,
        &mut availability,
    )
}

fn answer_synthesized_guide(
    prompt: &str,
    guide: &HowToGuide,
    log: &mut EventLog,
) -> SymbolicAnswer {
    finalize_simple(
        prompt,
        log,
        "procedural_how_to",
        "response:procedural_how_to_guide",
        &guide.markdown(),
        0.82,
    )
}
