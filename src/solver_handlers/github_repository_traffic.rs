//! GitHub repository traffic visibility answers.
//!
//! Issue #497 reported a Russian prompt asking whether someone can know if
//! anybody visited "your repo" on GitHub. The handler composes platform,
//! repository, traffic, and question roles from the seed lexicon, then answers
//! from official GitHub traffic documentation links.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, response_for};
use crate::solver_handlers::finalize_simple;

const TRAFFIC_UI_DOC: &str = "https://docs.github.com/en/repositories/viewing-activity-and-data-for-your-repository/viewing-traffic-to-a-repository";
const TRAFFIC_API_DOC: &str = "https://docs.github.com/en/rest/metrics/traffic";
const DEFAULT_REPOSITORY: &str = "link-assistant/formal-ai";
const REPOSITORY_PLACEHOLDER: &str = concat!("{", "repository", "}");
const TRAFFIC_UI_DOCS_PLACEHOLDER: &str = concat!("{", "traffic_ui_docs", "}");
const TRAFFIC_API_DOCS_PLACEHOLDER: &str = concat!("{", "traffic_api_docs", "}");

pub fn try_github_repository_traffic(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let language = detect_language(prompt);
    if !is_github_repository_traffic_question(normalized, language.slug()) {
        return None;
    }

    let repository = seed::agent_info()
        .get("repository")
        .cloned()
        .unwrap_or_else(|| DEFAULT_REPOSITORY.to_owned());

    log.append("github_repository_traffic:platform", "github".to_owned());
    log.append("github_repository_traffic:repository", repository.clone());
    log.append(
        "github_repository_traffic:access",
        "push_or_write_access_required".to_owned(),
    );
    log.append(
        "github_repository_traffic:window",
        "last_14_days".to_owned(),
    );
    log.append(
        "github_repository_traffic:aggregate",
        "views_uniques_clones_referrers_paths".to_owned(),
    );
    log.append(
        "github_repository_traffic:privacy",
        "no_individual_identity".to_owned(),
    );
    log.append("source", TRAFFIC_UI_DOC.to_owned());
    log.append("source", TRAFFIC_API_DOC.to_owned());

    let body = github_repository_traffic_body(language.slug(), &repository);
    Some(finalize_simple(
        prompt,
        log,
        "github_repository_traffic",
        "response:github_repository_traffic",
        &body,
        0.92,
    ))
}

fn is_github_repository_traffic_question(normalized: &str, language: &str) -> bool {
    let lexicon = seed::lexicon();
    let mut languages = vec![language];
    if language != "en" {
        languages.push("en");
    }

    let has_platform = lexicon.mentions_role_in_languages_raw(
        seed::ROLE_GITHUB_REPOSITORY_PLATFORM,
        normalized,
        &languages,
    );
    let has_repository = lexicon.mentions_role_in_languages_raw(
        seed::ROLE_REPOSITORY_REFERENCE,
        normalized,
        &languages,
    );
    let has_traffic = lexicon.mentions_role_in_languages_raw(
        seed::ROLE_GITHUB_REPOSITORY_TRAFFIC_SIGNAL,
        normalized,
        &languages,
    );
    let has_question = lexicon.mentions_role_in_languages_raw(
        seed::ROLE_GITHUB_REPOSITORY_TRAFFIC_QUESTION,
        normalized,
        &languages,
    );

    has_platform && has_repository && has_traffic && has_question
}

fn github_repository_traffic_body(language: &str, repository: &str) -> String {
    let fallback = format!(
        "Partly. For a GitHub repository such as {repository}, GitHub can show aggregate traffic to people with push or write access: views, unique visitors, clones, referring sites, and popular content for the recent traffic window. It does not show the identity of an individual visitor. Check GitHub Insights > Traffic or the REST traffic endpoints: {traffic_ui_doc}; {traffic_api_doc}.",
        repository = repository,
        traffic_ui_doc = TRAFFIC_UI_DOC,
        traffic_api_doc = TRAFFIC_API_DOC
    );
    response_for("github_repository_traffic", language)
        .or_else(|| response_for("github_repository_traffic", "en"))
        .unwrap_or(fallback)
        .replace(REPOSITORY_PLACEHOLDER, repository)
        .replace(TRAFFIC_UI_DOCS_PLACEHOLDER, TRAFFIC_UI_DOC)
        .replace(TRAFFIC_API_DOCS_PLACEHOLDER, TRAFFIC_API_DOC)
}
