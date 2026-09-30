//! Executed web search backed by exact source captures.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::relative_meta_logic::SourceTier;
use crate::search_fusion::{
    SearchFusionExecution, SearchSourceClassification, execute_search_fusion,
};
use crate::seed;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};

use super::{WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K};
use crate::solver_handlers::finalize_simple;
use crate::solver_handlers::web_search_intent::{WebSearchQueryKind, extract_web_search_request};

/// Whether the runtime opted into live provider fetches through
/// `FORMAL_AI_LIVE_FETCH` (`1`, `true`, `yes`, or `on`).
///
/// Issue #1173 (E138): every executed-search entry point applies the same
/// combined offline rule `solver_dispatch.rs` applies at its `web_search`
/// call site — a runtime offline flag **or** a missing live-fetch opt-in
/// keeps the transport off — so the unknown-reasoning fallback cannot go
/// online behind the dispatcher's back.
pub fn live_fetch_enabled() -> bool {
    std::env::var("FORMAL_AI_LIVE_FETCH").is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

/// Execute the web search for an already-extracted `(prompt, query, kind)`
/// triple through a caller-supplied capture client and answer from it.
///
/// Issue #1173 (E138) R1/R2: the shared core behind the explicit `web_search`
/// route and the unknown-reasoning fallback. The search either executes — the
/// answer names the query and cites the captured sources — or the run
/// degrades to the localized `web_search_unavailable` response; the old
/// descriptive paragraph about the browser demo's search machinery is no
/// longer an answer anywhere.
pub fn execute_web_search_answer<T: SourceTransport>(
    prompt: &str,
    query: &str,
    query_kind: WebSearchQueryKind,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> SymbolicAnswer {
    log.append("web_search:request", query.to_owned());
    log.append("web_search:query_kind", query_kind.as_str());
    let language = detect_language(prompt).slug();
    match execute_search_fusion(client, query, language, 3, |_| {
        SearchSourceClassification::auto(SourceTier::IndependentCorroboration)
    }) {
        Ok(execution) => {
            log.append(
                "web_search:executed",
                format_args!(
                    "query={query} statements={} sources={}",
                    execution.answer.statements.len(),
                    execution.answer.sources.len()
                )
                .to_string(),
            );
            execution.record(log);
            answer_executed_web_search(prompt, &execution, log)
        }
        Err(error) => {
            if matches!(error, crate::source_fetch::FetchError::OfflineCacheMiss(_)) {
                log.append("policy:offline", query.to_owned());
            }
            log.append("error:fetch", error.to_string());
            for provider in WEB_SEARCH_PROVIDERS {
                log.append("web_search:provider_planned", (*provider).to_owned());
            }
            log.append(
                "web_search:fusion_planned",
                format!("rrf:k={WEB_SEARCH_RRF_K}"),
            );
            let body = seed::localized_response("web_search_unavailable", language)
                .unwrap_or_else(|| String::from("web_search_unavailable"))
                .replace("{query}", query);
            finalize_simple(
                prompt,
                log,
                "web_search",
                "response:web_search_unavailable",
                &body,
                0.0,
            )
        }
    }
}

/// Execute the recognized web search through a caller-supplied capture client.
///
/// This seam keeps tests deterministic and gives every runtime the same
/// provenance rule: provider and fusion events exist only after the provider
/// response has been captured and parsed successfully.
pub fn try_web_search_with_client<T: SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let request = extract_web_search_request(prompt, normalized)?;
    Some(execute_web_search_answer(
        prompt,
        &request.query,
        request.kind,
        log,
        client,
    ))
}

/// Use the process source cache, enabling transport only after the runtime has
/// explicitly opted into live fetches. Offline mode still reads valid captures.
pub fn try_web_search_with_offline(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    let cache_dir =
        std::env::var("FORMAL_AI_SOURCE_CACHE_DIR").unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(cache_dir, CurlSourceTransport).with_online(!offline);
    try_web_search_with_client(prompt, normalized, log, &client)
}

fn answer_executed_web_search(
    prompt: &str,
    execution: &SearchFusionExecution,
    log: &mut EventLog,
) -> SymbolicAnswer {
    let body = execution.render_markdown();
    finalize_simple(prompt, log, "web_search", "response:web_search", &body, 0.8)
}
