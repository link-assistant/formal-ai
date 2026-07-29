//! Executed web search backed by exact source captures.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};
use crate::source_research::{execute_source_research, SourceResearchExecution};

use super::{WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K};
use crate::solver_handlers::finalize_simple;
use crate::solver_handlers::web_search_intent::extract_web_search_request;

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
    log.append("web_search:request", request.query.clone());
    log.append("web_search:query_kind", request.kind.as_str());
    match execute_source_research(client, &request.query, 0) {
        Ok(execution) => {
            execution.record(log);
            Some(answer_executed_web_search(
                prompt,
                &request.query,
                &execution,
                log,
            ))
        }
        Err(error) => {
            if matches!(error, crate::source_fetch::FetchError::OfflineCacheMiss(_)) {
                log.append("policy:offline", request.query.clone());
            }
            log.append("error:fetch", error.to_string());
            for provider in WEB_SEARCH_PROVIDERS {
                log.append("web_search:provider_planned", (*provider).to_owned());
            }
            log.append(
                "web_search:fusion_planned",
                format!("rrf:k={WEB_SEARCH_RRF_K}"),
            );
            let language = detect_language(prompt).slug();
            let body = seed::response_for("web_search_unavailable", language)
                .or_else(|| seed::response_for("web_search_unavailable", "en"))
                .unwrap_or_else(|| String::from("web_search_unavailable"))
                .replace("{query}", &request.query);
            Some(finalize_simple(
                prompt,
                log,
                "web_search",
                "response:web_search_unavailable",
                &body,
                0.0,
            ))
        }
    }
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
    query: &str,
    execution: &SourceResearchExecution,
    log: &mut EventLog,
) -> SymbolicAnswer {
    let results = execution
        .search
        .fused
        .iter()
        .enumerate()
        .map(|(index, result)| {
            let title = if result.title.trim().is_empty() {
                result.url.as_str()
            } else {
                result.title.as_str()
            };
            format!("{}. [{title}]({})", index + 1, result.url)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let language = detect_language(prompt).slug();
    let body = seed::response_for("web_search_live_results", language)
        .or_else(|| seed::response_for("web_search_live_results", "en"))
        .unwrap_or_else(|| String::from("{query}\n\n{results}"))
        .replace(concat!("{", "query", "}"), query)
        .replace(
            concat!("{", "count", "}"),
            &execution.search.fused.len().to_string(),
        )
        .replace(concat!("{", "results", "}"), &results);
    finalize_simple(prompt, log, "web_search", "response:web_search", &body, 0.8)
}
