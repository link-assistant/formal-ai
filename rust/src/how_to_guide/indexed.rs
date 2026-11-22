//! Resolve web index candidates to a publisher's actual ordered procedure.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use super::{
    GuideSourceOutcome, HowToGuide, ServicePreferences, matches_wiki_title, push_steps, topic_words,
};
use crate::event_log::EventLog;
use crate::source_fetch::{CachedSourceClient, SourceTransport, search_page_url};
use crate::web_search_core::execute_duckduckgo_search;

fn title_matches(task: &str, label: &str, language: &str) -> bool {
    static GERUNDS: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)\b([a-z]{3,})ing\b").unwrap());
    if matches_wiki_title(task, label, language) {
        return true;
    }
    if language != "en" {
        return false;
    }
    matches_wiki_title(
        task,
        &GERUNDS.replace_all(label, "${0} ${1} ${1}e"),
        language,
    )
}

fn publisher_enabled(host: &str, preferences: &ServicePreferences) -> bool {
    !crate::seed::external_trusted_sources()
        .iter()
        .any(|record| {
            if record.api.is_empty() || preferences.allows(record) {
                return false;
            }
            let Ok(url) = url::Url::parse(&record.api.replace("{language}", "en")) else {
                return false;
            };
            let Some(publisher) = url.host_str() else {
                return false;
            };
            let domain = publisher
                .split_once('.')
                .filter(|(first, _)| {
                    *first == "www" || record.api_language.iter().any(|lang| lang == *first)
                })
                .map_or(publisher, |(_, base)| base);
            host == domain
                || host
                    .strip_suffix(domain)
                    .is_some_and(|prefix| prefix.ends_with('.'))
        })
}

pub(crate) fn discover_indexed_procedure<T: SourceTransport, P: SourceTransport>(
    guide: &mut HowToGuide,
    language: &str,
    queries: &[String],
    search: &CachedSourceClient<T>,
    pages: &CachedSourceClient<P>,
    preferences: &ServicePreferences,
    log: &mut EventLog,
) {
    let Some(mut record) =
        crate::seed::source_record("web_page").filter(|record| preferences.allows(record))
    else {
        return;
    };
    let task_words = topic_words(&guide.task);
    let mut visited = BTreeSet::new();
    for query in queries {
        log.append("web_search:request", query.clone());
        let search = match execute_duckduckgo_search(search, query) {
            Ok(search) => search,
            Err(error) => {
                log.append("procedural_how_to:index_failure", error.to_string());
                continue;
            }
        };
        for capture in search.captures {
            capture.record(log);
        }
        for diagnostic in search.component_diagnostics {
            log.append("procedural_how_to:index_diagnostic", diagnostic);
        }
        for candidate in search.fused {
            let Some(url) = search_page_url(&candidate.url) else {
                continue;
            };
            let host = url.host_str().unwrap_or_default();
            if !publisher_enabled(host, preferences) || !visited.insert(url.to_string()) {
                continue;
            }
            let mut outcome = GuideSourceOutcome::new(&record.id, "no_steps", url.as_str());
            outcome.pages = 1;
            match pages.fetch(url.as_str()) {
                Ok(capture) => {
                    let text = String::from_utf8_lossy(capture.bytes());
                    let steps = super::extract::indexed_procedure_steps(
                        &text,
                        |title| {
                            title_matches(&guide.task, title, language)
                                || task_words.first().is_some_and(|action| {
                                    title_matches(action, title, language)
                                        && title_matches(
                                            &guide.task,
                                            &format!("{title} {host}"),
                                            language,
                                        )
                                })
                        },
                        guide.bounds.max_items,
                    );
                    if !steps.is_empty() {
                        record.name = host.to_owned();
                        guide.steps.clear();
                        push_steps(&record, &capture, 1, &steps, &mut guide.steps);
                        outcome.status = "contributed".to_owned();
                        outcome.steps = steps.len();
                    }
                }
                Err(error) => {
                    outcome.status = "capture_failed".to_owned();
                    outcome.detail = format!("{url} {error}");
                }
            }
            guide.outcomes.push(outcome);
            if guide.is_sufficient() || visited.len() >= guide.bounds.max_pages_per_service {
                return;
            }
        }
    }
}
