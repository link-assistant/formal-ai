//! Execute bounded procedural discovery using live or cached sources (#991).
//! The registry and client apply opt-outs and network policy, cache service
//! accessibility, and report when there are too few verified steps.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::how_to_guide::{GuideBounds, ServicePreferences, synthesize_how_to_guide_in};
use crate::language::detect;
use crate::seed::external_service_settings_keys;
use crate::service_accessibility::{ServiceAccessibilityCache, unix_now};
use crate::solver_handler_how::{
    internet_access_help_requested, procedural_how_to_task, procedural_index_queries,
    try_how_to_procedure, try_procedural_how_to_followup_with,
};
use crate::solver_handlers::finalize_simple;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};

/// Comma-separated `settings_key`s the environment opts out of, mirroring the
/// browser settings toggles for runtimes that have no settings UI (the CLI, the
/// HTTP server, and the test suites).
pub const DISABLED_SERVICES_ENV: &str = "FORMAL_AI_DISABLED_SERVICES";

/// Environment override for where the capture cache and the accessibility
/// record live; the same variable the executed web search reads.
const CACHE_DIR_ENV: &str = "FORMAL_AI_SOURCE_CACHE_DIR";

fn source_cache_directory() -> String {
    std::env::var(CACHE_DIR_ENV).unwrap_or_else(|_| String::from("data"))
}

/// Apply declared service opt-outs and report unknown keys.
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

/// Use the supplied capture client, settings and accessibility record for
/// both offline replays and live refreshes.
pub fn try_how_to_procedure_with_client<T: SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
    preferences: &ServicePreferences,
    availability: &mut ServiceAccessibilityCache,
) -> Option<SymbolicAnswer> {
    try_how_to_procedure_with_clients(
        prompt,
        normalized,
        log,
        client,
        client,
        preferences,
        availability,
    )
}

fn try_how_to_procedure_with_clients<T: SourceTransport, P: SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
    pages: &CachedSourceClient<P>,
    preferences: &ServicePreferences,
    availability: &mut ServiceAccessibilityCache,
) -> Option<SymbolicAnswer> {
    if crate::legality_warning::assess(prompt, normalized).is_some() {
        return None;
    }
    if let Some(answer) =
        crate::solver_handler_how::practical_help::with_client(prompt, normalized, log, client)
    {
        return Some(answer);
    }
    if internet_access_help_requested(prompt, normalized) {
        return None;
    }
    let task = procedural_how_to_task(normalized)?;
    let language = detect(prompt).slug();
    let mut guide = synthesize_how_to_guide_in(
        (&task, language),
        client,
        preferences,
        &GuideBounds::default(),
        availability,
        unix_now(),
    );
    if let Err(error) = availability.save() {
        log.append("service_accessibility:save_failed", error.to_string());
    }
    if !guide.is_sufficient() {
        crate::how_to_guide::indexed::discover_indexed_procedure(
            &mut guide,
            language,
            &procedural_index_queries(normalized),
            client,
            pages,
            preferences,
            log,
        );
    }
    guide.record(log);
    if !guide.is_sufficient() {
        log.append(
            "procedural_how_to:fallback",
            "insufficient_evidence".to_owned(),
        );
        return try_how_to_procedure(prompt, normalized, log);
    }
    Some(finalize_simple(
        prompt,
        log,
        "procedural_how_to",
        "response:procedural_how_to_guide",
        &guide.answer_markdown(language),
        0.82,
    ))
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
    let directory = source_cache_directory();
    let client = CachedSourceClient::new(&directory, CurlSourceTransport).with_online(!offline);
    let pages = CachedSourceClient::new(
        std::path::Path::new(&directory).join("public-search-pages"),
        crate::source_fetch::PublicPageTransport,
    )
    .with_online(!offline);
    let preferences = service_preferences_from_env(log);
    let mut availability = ServiceAccessibilityCache::load(&directory);
    try_how_to_procedure_with_clients(
        prompt,
        normalized,
        log,
        &client,
        &pages,
        &preferences,
        &mut availability,
    )
}

/// Resolve the prior procedural task with the same capture policy as a fresh turn.
/// The follow-up matcher reconstructs the original prompt and its normalised
/// form so the guide can use its language and query the original goal.
pub(crate) fn try_procedural_followup_with_offline(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    try_procedural_how_to_followup_with(prompt, normalized, log, |prior, canonical, log| {
        try_how_to_procedure_with_offline(prior, canonical, log, offline)
    })
}

#[cfg(test)]
#[path = "../tests/source/source_tests/solver_handler_how_synthesis/production_tests.rs"]
mod tests;
