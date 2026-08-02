//! Offline HTTP-fetch answers backed by the bundled project registry.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{localized_response, ProjectRecord};
use crate::summarization::{describe_project, SummarizationConfig, SummarizationMode};

use super::{finalize_simple, web_requests::registry_static};

pub(super) fn try_curated_http_fetch(
    prompt: &str,
    url: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let project = match_curated_github_url(url)?;
    let language = detect_language(prompt).slug();
    log.append("http_fetch:curated_project", project.repo_slug());
    log.append("summarization:mode", "standard".to_owned());
    log.append("summarization:language", language.to_owned());
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Standard)
        .with_language(language);
    let summary = describe_project(project, &config);
    let body = localized_response("http_fetch_curated_project", language)?
        .replace(&["{", "url", "}"].concat(), url)
        .replace(&["{", "summary", "}"].concat(), &summary);
    Some(finalize_simple(
        prompt,
        log,
        "http_fetch",
        "response:http_fetch",
        &body,
        0.95,
    ))
}

/// Match a GitHub URL against local seed data without claiming a retrieval.
fn match_curated_github_url(url: &str) -> Option<&'static ProjectRecord> {
    let lower = url.to_lowercase();
    let after_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))?;
    let after_host = after_scheme.strip_prefix("github.com/")?;
    let mut segments = after_host.split('/');
    let org = segments.next()?.trim_matches('/');
    let name = segments.next()?.trim_matches('/');
    if org.is_empty() || name.is_empty() {
        return None;
    }
    registry_static().projects.iter().find(|project| {
        project.org.eq_ignore_ascii_case(org) && project.name.eq_ignore_ascii_case(name)
    })
}
