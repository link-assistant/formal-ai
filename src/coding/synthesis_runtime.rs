//! Runtime assembly for research-backed coding synthesis.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::coding::composition;
use crate::coding::concept_discovery::{CatalogFunction, DiscoveryCatalog};
use crate::coding::discovered_procedures::DiscoveredProcedureLedger;
use crate::coding::function_catalog::python_docs::{StdlibIndex, fetch_index};
use crate::coding::function_catalog::wikifunctions::{
    fetch_function, fetch_implementations, search_functions,
};
use crate::coding::task_spec::CodingTaskSpec;
use crate::event_log::EventLog;
use crate::language::Language;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};

pub fn discovery_catalog(
    spec: &CodingTaskSpec,
    log: &mut EventLog,
    live: bool,
) -> DiscoveryCatalog {
    let cache_dir = std::env::var("FORMAL_AI_SOURCE_CACHE_DIR")
        .or_else(|_| std::env::var("FORMAL_AI_CACHE_DIR"))
        .unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(cache_dir, CurlSourceTransport).with_online(live);
    let stdlib = if live {
        fetch_index(&client).unwrap_or_default()
    } else {
        static OFFLINE_STDLIB: OnceLock<StdlibIndex> = OnceLock::new();
        OFFLINE_STDLIB
            .get_or_init(|| fetch_index(&client).unwrap_or_default())
            .clone()
    };

    let mut phrases = spec.requirement_sentences.clone();
    phrases.push(spec.name.replace('_', " "));
    let mut matches = Vec::new();
    let mut seen_matches = BTreeSet::new();
    for phrase in phrases {
        for language in [spec.prose_language.as_str(), "en"] {
            match search_functions(&client, &phrase, language) {
                Ok(found) => {
                    for matched in found {
                        if seen_matches.insert(matched.page_title.clone()) {
                            log.append(
                                "source:http",
                                format!(
                                    "url={};fetched_at={};sha256={};catalog_match={}",
                                    matched.source_url,
                                    matched.fetched_at,
                                    matched.sha256,
                                    matched.page_title
                                ),
                            );
                            matches.push(matched);
                        }
                    }
                }
                Err(error) => {
                    log.append("synthesis:source_miss", error.to_string());
                }
            }
            if language == "en" || spec.prose_language == "en" {
                break;
            }
        }
    }

    let mut functions = Vec::new();
    for matched in matches
        .iter()
        .filter(|item| item.match_rate >= 0.65)
        .take(4)
    {
        let Ok(definition) = fetch_function(&client, &matched.page_title) else {
            continue;
        };
        let implementations =
            fetch_implementations(&client, &definition.implementation_zids).unwrap_or_default();
        functions.push(CatalogFunction {
            definition,
            implementations,
        });
    }
    DiscoveryCatalog::new(stdlib, matches, functions)
}

pub fn live_fetch_enabled() -> bool {
    std::env::var("FORMAL_AI_LIVE_FETCH").is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

pub fn procedure_ledger() -> Option<DiscoveredProcedureLedger> {
    std::env::var("FORMAL_AI_CACHE_DIR")
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(DiscoveredProcedureLedger::new)
}

pub fn response_language(spec: &CodingTaskSpec) -> Language {
    crate::language::from_slug(&spec.prose_language).unwrap_or(Language::English)
}

fn localized_template(intent: &str, language: Language) -> String {
    crate::seed::response_for(intent, language.slug())
        .or_else(|| crate::seed::response_for(intent, Language::English.slug()))
        .unwrap_or_default()
}

pub fn research_trail_line(language: Language, trail: &str) -> String {
    let values = [("trail", trail)];
    crate::seed::render_response("coding_synthesis_research_trail", language.slug(), &values)
        .or_else(|| {
            crate::seed::render_response(
                "coding_synthesis_research_trail",
                Language::English.slug(),
                &values,
            )
        })
        .unwrap_or_default()
}

pub fn render_answer(selected: &composition::VerifiedDraft, language: Language) -> String {
    let count = selected.assertion_count.to_string();
    let values = [
        ("source", selected.source.as_str()),
        ("count", count.as_str()),
    ];
    let mut body =
        crate::seed::render_response("coding_synthesis_verified_answer", language.slug(), &values)
            .or_else(|| {
                crate::seed::render_response(
                    "coding_synthesis_verified_answer",
                    Language::English.slug(),
                    &values,
                )
            })
            .unwrap_or_default();
    if !selected.source_urls.is_empty() {
        let _ = write!(
            body,
            "\n{}",
            localized_template("coding_synthesis_sources_heading", language)
        );
        for url in &selected.source_urls {
            let license = match url.split('/').nth(2) {
                Some("wikifunctions.org" | "www.wikifunctions.org") => "Apache-2.0",
                Some("docs.python.org") => "PSF-2.0",
                _ => "source-declared",
            };
            let _ = write!(body, "\n- {url} ({license})");
        }
    }
    body
}
