//! URL fetch, URL navigation, and browser-search handlers.

use crate::concepts::extract_concept_query;
use crate::engine::{normalize_prompt, select_rule_for, SelectedRule, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{projects_registry, ProjectRecord};
use crate::summarization::{describe_project, SummarizationConfig, SummarizationMode};
use crate::web_search_core::{
    WEB_SEARCH_PROVIDERS as CORE_WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K as CORE_WEB_SEARCH_RRF_K,
};

use super::finalize_simple;

/// Match prompts that explicitly ask the engine to perform an HTTP request
/// (e.g. `fetch google.com`, `Сделай запрос к google.com`). In the browser
/// web app the actual `fetch()` is attempted first, with an iframe fallback when
/// CORS blocks the request only after target frame-policy headers have been
/// checked. Non-fetch URL prompts (`Navigate to github.com`, `Visit github.com`,
/// ...) are handled by [`try_url_navigate`] instead.
pub fn try_http_fetch(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let url = extract_http_fetch_url(prompt, normalized)?;
    log.append("http_fetch:request", url.clone());
    let language = detect_language(prompt).slug();
    let project_summary = match_curated_github_url(&url).map(|project| {
        log.append("http_fetch:curated_project", project.repo_slug());
        log.append("summarization:mode", "standard".to_owned());
        log.append("summarization:language", language.to_owned());
        let config = SummarizationConfig::default()
            .with_mode(SummarizationMode::Standard)
            .with_language(language);
        describe_project(project, &config)
    });
    let body = project_summary.map_or_else(
        || {
            format!(
                "HTTP fetch requested for `{url}`.\n\n\
                 The browser web app attempts a direct `fetch()` first and shows the \
                 response body when the server allows CORS. If the request is blocked \
                 by CORS, the web app checks CORS-readable frame-policy metadata before \
                 deciding whether to show an embedded iframe or keep a direct external \
                 link.\n\n\
                 Source: [{url}]({url})"
            )
        },
        |summary| match language {
            "ru" => format!(
                "HTTP-запрос на `{url}`.\n\n\
                 Этот URL соответствует курируемому продвигаемому проекту. \
                 Резюме README (через formalize → summarize → \
                 deformalize): {summary}\n\n\
                 В браузерной демо-версии сначала выполняется прямой `fetch()`; \
                 если CORS блокирует ответ, проверяется frame-policy и при \
                 необходимости открывается iframe или показывается прямая \
                 ссылка.\n\n\
                 Source: [{url}]({url})"
            ),
            _ => format!(
                "HTTP fetch requested for `{url}`.\n\n\
                 This URL matches a curated promoted project. README summary \
                 (through the formalize → summarize → \
                 deformalize pipeline): {summary}\n\n\
                 The browser web app attempts a direct `fetch()` first and shows \
                 the response body when the server allows CORS. If the request \
                 is blocked by CORS, the web app checks CORS-readable \
                 frame-policy metadata before deciding whether to show an \
                 embedded iframe or keep a direct external link.\n\n\
                 Source: [{url}]({url})"
            ),
        },
    );
    Some(finalize_simple(
        prompt,
        log,
        "http_fetch",
        "response:http_fetch",
        &body,
        0.95,
    ))
}

/// Match an absolute URL against the curated project registry. Returns the
/// project record when the URL points to a `github.com/<org>/<name>`
/// repository whose `<org>/<name>` matches a curated entry. Sub-paths
/// (`/blob/`, `/tree/`, etc.) are also matched so a README fetch URL like
/// `https://github.com/link-assistant/hive-mind/blob/main/README.md` still
/// resolves to the curated record.
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
    let registry = registry_static();
    registry.projects.iter().find(|project| {
        project.org.eq_ignore_ascii_case(org) && project.name.eq_ignore_ascii_case(name)
    })
}

/// Match prompts that ask the assistant to navigate to or display a URL
/// without performing an HTTP request (e.g. `Navigate to github.com`,
/// `Go to github.com`, `Перейди на github.com`). The browser web app renders
/// an iframe preview only when CORS-readable frame-policy metadata does not
/// report blocking X-Frame-Options or CSP frame-ancestors headers.
pub fn try_url_navigate(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let url = extract_url_navigate_url(prompt, normalized)?;
    log.append("url_navigate:request", url.clone());
    log.append("url_preview:frame_policy_check", url.clone());
    log.append("url_preview:external_link", url.clone());
    let body = format!(
        "I suggest opening this in a new tab: [{url}]({url}).\n\n\
         In the browser web app, this URL is checked with browser-readable \
         frame-policy metadata before any embedded preview is attempted. If \
         X-Frame-Options or CSP frame-ancestors blocks embedding, the web app \
         keeps the direct external link instead."
    );
    Some(finalize_simple(
        prompt,
        log,
        "url_navigate",
        "response:url_navigate",
        &body,
        0.95,
    ))
}

/// Reciprocal Rank Fusion constant used to combine the top-10 results returned
/// by each search provider. Re-exported from `crate::web_search_core` so the
/// CLI, server, browser worker, and the Rust→WASM port all share one value.
///
/// Source: <https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf>
pub const WEB_SEARCH_RRF_K: u32 = CORE_WEB_SEARCH_RRF_K;

/// Provider order used by the browser worker and by the offline Rust solver
/// when describing the multi-engine plan for `web_search`. Sourced from
/// `crate::web_search_core::WEB_SEARCH_PROVIDERS` so the WASM worker and the
/// JS planner cannot drift apart (issue #133).
pub const WEB_SEARCH_PROVIDERS: &[&str] = CORE_WEB_SEARCH_PROVIDERS;

pub fn try_web_search(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let query = extract_web_search_query(prompt, normalized)?;
    log.append("web_search:request", query.clone());
    for provider in WEB_SEARCH_PROVIDERS {
        log.append("web_search:provider", (*provider).to_owned());
    }
    log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));
    let provider_summary = WEB_SEARCH_PROVIDERS.join(", ");
    let language = detect_language(prompt).slug();
    let body = match language {
        "ru" => format!(
            "Поиск в интернете запрошен для `{query}`.\n\n\
             В браузерной демо-версии formal-ai по умолчанию использует DuckDuckGo \
             Instant Answer (CORS-совместимый, без ключа) и параллельно опрашивает \
             Internet Archive, Wikipedia REST, Wikidata и Wiktionary в указанном \
             порядке приоритета. Топ-10 ссылок от каждого провайдера объединяются \
             через reciprocal rank fusion (`score(d) = Σ 1 / ({WEB_SEARCH_RRF_K} + \
             rank_i(d))`), поэтому URL, которые встречаются у нескольких провайдеров, \
             всплывают вверх. Дубликаты одной и той же сущности (например, \
             Викидата + Википедия) сворачиваются в один пункт с пометкой \
             «Другие источники». Для произвольной страницы используйте \
             `fetch example.com`; если прямой `fetch()` заблокирован CORS, \
             браузер проверит frame-policy перед встроенным iframe.\n\n\
             Provider: duckduckgo (default)\n\
             Providers considered: {provider_summary}\n\
             Combined ranking: reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})"
        ),
        _ => format!(
            "Web search requested for `{query}`.\n\n\
             In the browser demo formal-ai defaults to the DuckDuckGo Instant \
             Answer endpoint (CORS-readable, keyless) and queries Internet Archive, \
             Wikipedia REST, Wikidata, and Wiktionary in that priority order. The \
             top-10 links from each provider are merged with reciprocal rank fusion \
             (`score(d) = Σ 1 / ({WEB_SEARCH_RRF_K} + rank_i(d))`), so URLs that \
             appear in more than one provider bubble up. Duplicate entries for the \
             same entity (e.g. Wikidata + Wikipedia) are collapsed into a single \
             bullet with an \"other sources\" footnote. For an arbitrary page, use \
             `fetch example.com`; if direct `fetch()` is blocked by CORS, the \
             browser checks frame policy before an embedded iframe.\n\n\
             Provider: duckduckgo (default)\n\
             Providers considered: {provider_summary}\n\
             Combined ranking: reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})"
        ),
    };
    Some(finalize_simple(
        prompt,
        log,
        "web_search",
        "response:web_search",
        &body,
        0.8,
    ))
}

const PROMOTED_PROJECT_ORGS: &[&str] = &["link-assistant", "link-foundation", "linksplatform"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepositoryPlatform {
    GitHub,
    GitLab,
    Bitbucket,
}

impl RepositoryPlatform {
    const fn label(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::GitLab => "GitLab",
            Self::Bitbucket => "Bitbucket",
        }
    }

    const fn host(self) -> &'static str {
        match self {
            Self::GitHub => "github.com",
            Self::GitLab => "gitlab.com",
            Self::Bitbucket => "bitbucket.org",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepositoryReference {
    platform: RepositoryPlatform,
    owner: String,
    name: String,
    url: String,
}

impl RepositoryReference {
    fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

/// Lookup repository/project prompts. Runs *after* `concept_lookup` so
/// seed-backed concept terms (`Links Notation`, `Wikipedia`, `Rust`, …) keep
/// their existing intent.
///
/// With promotion enabled (the default), known projects from Link Assistant,
/// Link Foundation, and `LinksPlatform` are listed first. With promotion
/// disabled, the same prompts stay on the generic repository lookup path.
pub fn try_project_lookup(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
    promote_associative_repositories: bool,
) -> Option<SymbolicAnswer> {
    if let Some(repo) = repository_from_prompt(prompt) {
        if promote_associative_repositories {
            if let Some(project) = promoted_project_by_repo(&repo.owner, &repo.name) {
                return Some(render_project_lookup(prompt, log, project));
            }
        }
        return Some(render_generic_repository_lookup(
            prompt,
            log,
            Some(&repo),
            promote_associative_repositories,
        ));
    }
    if matches!(select_rule_for(prompt), SelectedRule::Identity) {
        return None;
    }

    let project = matched_project(prompt)?;
    if promote_associative_repositories && is_promoted_project(project) {
        return Some(render_project_lookup(prompt, log, project));
    }

    Some(render_generic_repository_lookup(
        prompt,
        log,
        None,
        promote_associative_repositories,
    ))
}

fn render_project_lookup(
    prompt: &str,
    log: &mut EventLog,
    project: &ProjectRecord,
) -> SymbolicAnswer {
    let language = detect_language(prompt).slug();
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Short)
        .with_language(language);
    let description = describe_project(project, &config);
    let display_name = project.display_name_for(language);
    let repo_slug = project.repo_slug();
    let project_url = project.url.clone();

    log.append("project:promoted", repo_slug.clone());
    log.append("source", project_url.clone());
    log.append("summarization:mode", "short".to_owned());
    log.append("summarization:language", language.to_owned());
    log.append("web_search:request", display_name.to_owned());
    for provider in WEB_SEARCH_PROVIDERS {
        log.append("web_search:provider", (*provider).to_owned());
    }
    log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));

    let provider_summary = WEB_SEARCH_PROVIDERS.join(", ");
    let promoted_orgs = PROMOTED_PROJECT_ORGS.join(", ");
    let body = match language {
        "ru" => format!(
            "В контексте репозиториев {promoted_orgs} под `{display_name}` я прежде всего \
             имею в виду [{repo_slug}]({project_url}) — {description}\n\n\
             Другие найденные в интернете репозитории и сущности должна показывать \
             браузерная демо-версия через поиск по запросу `{display_name}`. \
             Провайдеры: {provider_summary}. Ранжирование: reciprocal rank fusion \
             (k = {WEB_SEARCH_RRF_K}). Продвижение ассоциативных репозиториев \
             можно отключить, тогда ответ пойдет по обычному поиску GitHub, \
             GitLab и Bitbucket."
        ),
        _ => format!(
            "In the {promoted_orgs} repository context, `{display_name}` should first mean \
             [{repo_slug}]({project_url}) — {description}\n\n\
             Other repositories and entities found online are shown by the browser \
             demo through a web search for `{display_name}`. Providers: \
             {provider_summary}. Combined ranking: reciprocal rank fusion \
             (k = {WEB_SEARCH_RRF_K}). Associative repository promotion can be \
             switched off; then the answer follows the generic GitHub, GitLab, \
             and Bitbucket project lookup path."
        ),
    };
    finalize_simple(
        prompt,
        log,
        "project_lookup",
        "response:project_lookup",
        &body,
        0.9,
    )
}

fn render_generic_repository_lookup(
    prompt: &str,
    log: &mut EventLog,
    repository: Option<&RepositoryReference>,
    promotion_enabled: bool,
) -> SymbolicAnswer {
    let language = detect_language(prompt).slug();
    let provider_summary = ["GitHub", "GitLab", "Bitbucket"].join(", ");
    if !promotion_enabled {
        log.append("project_lookup:promotion", "disabled".to_owned());
    }
    if let Some(repo) = repository {
        let slug = repo.slug();
        append_repository_evidence(log, repo.platform, slug.clone());
        log.append("source", repo.url.clone());
        let label = repo.platform.label();
        let body = match language {
            "ru" => format!(
                "Это запрос о репозитории [{slug}]({url}) на {label}.\n\n\
                 Обычный путь project_lookup ищет и резюмирует README или описание \
                 проекта на GitHub, GitLab и Bitbucket без особого правила для \
                 отдельного названия. Если репозиторий находится в продвигаемых \
                 организациях и продвижение включено, он будет показан первым.",
                url = repo.url
            ),
            _ => format!(
                "This is a repository lookup for [{slug}]({url}) on {label}.\n\n\
                 The generic project_lookup path can summarize README or project \
                 descriptions from GitHub, GitLab, and Bitbucket without a special \
                 case for any single name. If the repository belongs to a promoted \
                 organization and promotion is enabled, that repository is listed \
                 first.",
                url = repo.url
            ),
        };
        return finalize_simple(
            prompt,
            log,
            "project_lookup",
            "response:project_lookup",
            &body,
            0.82,
        );
    }

    log.append("project_lookup:repository_hosts", provider_summary.clone());
    let body = match language {
        "ru" => format!(
            "Это обычный запрос project_lookup о проекте или репозитории.\n\n\
             Я не выделяю специальный репозиторий, потому что продвижение \
             ассоциативных репозиториев отключено. Дальше следует искать и \
             резюмировать подходящие проекты на {provider_summary} и похожих \
             хостингах."
        ),
        _ => format!(
            "This is a generic project_lookup request for a project or repository.\n\n\
             I am not privileging a specific repository because associative \
             repository promotion is disabled. The next step is to search and \
             summarize matching projects across {provider_summary} and similar \
             hosts."
        ),
    };
    finalize_simple(
        prompt,
        log,
        "project_lookup",
        "response:project_lookup",
        &body,
        0.72,
    )
}

fn append_repository_evidence(log: &mut EventLog, platform: RepositoryPlatform, slug: String) {
    let kind = match platform {
        RepositoryPlatform::GitHub => "project_lookup:repository:github",
        RepositoryPlatform::GitLab => "project_lookup:repository:gitlab",
        RepositoryPlatform::Bitbucket => "project_lookup:repository:bitbucket",
    };
    log.append(kind, slug);
}

/// Match the concept term against the curated project registry. Returns the
/// matching [`ProjectRecord`] or `None` when no curated project matches.
fn matched_project(prompt: &str) -> Option<&'static ProjectRecord> {
    let query = extract_concept_query(prompt)?;
    let term = normalize_concept_term(&query.term);
    if term.is_empty() {
        return None;
    }
    let registry = registry_static();
    if let Some(project) = registry.by_alias(&term) {
        return Some(project);
    }
    if term == "hivemind" {
        return registry.by_alias("hive mind");
    }
    None
}

fn promoted_project_by_repo(org: &str, name: &str) -> Option<&'static ProjectRecord> {
    let registry = registry_static();
    registry.projects.iter().find(|project| {
        is_promoted_project(project)
            && project.org.eq_ignore_ascii_case(org)
            && project.name.eq_ignore_ascii_case(name)
    })
}

fn is_promoted_project(project: &ProjectRecord) -> bool {
    PROMOTED_PROJECT_ORGS
        .iter()
        .any(|org| project.org.eq_ignore_ascii_case(org))
}

fn repository_from_prompt(prompt: &str) -> Option<RepositoryReference> {
    first_url_candidate(prompt)
        .and_then(|(_, url)| repository_from_url(&url))
        .or_else(|| repository_from_concept_term(prompt))
}

fn repository_from_concept_term(prompt: &str) -> Option<RepositoryReference> {
    let query = extract_concept_query(prompt)?;
    let term = query.term.trim();
    if term.contains("://") || looks_like_hostname(term) {
        return normalize_url_candidate(term).and_then(|url| repository_from_url(&url));
    }
    repository_from_slug(term)
}

fn repository_from_slug(term: &str) -> Option<RepositoryReference> {
    let trimmed = term
        .trim()
        .trim_matches(is_url_wrapper_punctuation)
        .trim_end_matches(is_url_trailing_punctuation);
    let mut segments = trimmed.split('/');
    let owner = clean_repository_segment(segments.next()?)?;
    let name = clean_repository_segment(segments.next()?)?;
    if segments.next().is_some() {
        return None;
    }
    Some(RepositoryReference {
        platform: RepositoryPlatform::GitHub,
        url: format!("https://github.com/{owner}/{name}"),
        owner,
        name,
    })
}

fn repository_from_url(url: &str) -> Option<RepositoryReference> {
    let after_scheme = url.split_once("://")?.1;
    let (host_port, path_with_suffix) = after_scheme.split_once('/')?;
    let host = host_port
        .split(':')
        .next()
        .unwrap_or_default()
        .trim_start_matches("www.")
        .to_ascii_lowercase();
    let platform = match host.as_str() {
        "github.com" => RepositoryPlatform::GitHub,
        "gitlab.com" => RepositoryPlatform::GitLab,
        "bitbucket.org" => RepositoryPlatform::Bitbucket,
        _ => return None,
    };
    let path = path_with_suffix
        .split(['?', '#'])
        .next()
        .unwrap_or_default();
    let mut segments = path.split('/').filter(|segment| !segment.is_empty());
    let owner = clean_repository_segment(segments.next()?)?;
    let name = clean_repository_segment(segments.next()?)?;
    Some(RepositoryReference {
        platform,
        url: format!("https://{}/{owner}/{name}", platform.host()),
        owner,
        name,
    })
}

fn clean_repository_segment(segment: &str) -> Option<String> {
    let trimmed = segment.trim().trim_end_matches(".git");
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return None;
    }
    Some(trimmed.to_owned())
}

fn normalize_concept_term(value: &str) -> String {
    normalize_prompt(value)
        .replace(['-', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lazy-init the curated projects registry once per process. The seed file is
/// embedded via `include_str!` and the parsed form is immutable, so a `OnceLock`
/// is enough and avoids re-parsing on every prompt.
fn registry_static() -> &'static crate::seed::ProjectsRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<crate::seed::ProjectsRegistry> = OnceLock::new();
    REGISTRY.get_or_init(projects_registry)
}

fn extract_http_fetch_url(prompt: &str, normalized: &str) -> Option<String> {
    let (raw_candidate, url) = first_url_candidate(prompt)?;
    if !is_http_fetch_prompt(prompt, normalized, &raw_candidate) {
        return None;
    }
    Some(url)
}

fn extract_url_navigate_url(prompt: &str, normalized: &str) -> Option<String> {
    let (raw_candidate, url) = first_url_candidate(prompt)?;
    if !is_url_navigate_prompt(prompt, normalized, &raw_candidate) {
        return None;
    }
    Some(url)
}

fn first_url_candidate(prompt: &str) -> Option<(String, String)> {
    for token in prompt.split_whitespace() {
        let trimmed = trim_url_token(token);
        if let Some(url) = normalize_url_candidate(trimmed) {
            return Some((trimmed.to_owned(), url));
        }
    }
    None
}

fn trim_url_token(token: &str) -> &str {
    token
        .trim_matches(is_url_wrapper_punctuation)
        .trim_end_matches(is_url_trailing_punctuation)
}

const fn is_url_wrapper_punctuation(character: char) -> bool {
    matches!(
        character,
        '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`' | '«' | '»'
    )
}

const fn is_url_trailing_punctuation(character: char) -> bool {
    matches!(character, '.' | ',' | '!' | '?' | ';' | ':' | '…')
}

fn normalize_url_candidate(candidate: &str) -> Option<String> {
    let candidate = candidate.trim();
    if candidate.is_empty() || candidate.contains(char::is_whitespace) || candidate.contains('@') {
        return None;
    }
    let lower = candidate.to_lowercase();
    let url = if lower.starts_with("http://") || lower.starts_with("https://") {
        candidate.to_owned()
    } else if lower.starts_with("www.") || looks_like_hostname(candidate) {
        format!("https://{candidate}")
    } else {
        return None;
    };
    let after_scheme = url.split_once("://")?.1;
    let host_port = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let host = host_port.split(':').next().unwrap_or_default();
    if !looks_like_hostname(host) {
        return None;
    }
    Some(url)
}

fn looks_like_hostname(value: &str) -> bool {
    let host = value.trim();
    if !host.contains('.') || host.starts_with('.') || host.ends_with('.') {
        return false;
    }
    let labels: Vec<&str> = host.split('.').collect();
    if labels.iter().any(|label| label.is_empty()) {
        return false;
    }
    let Some(tld) = labels.last() else {
        return false;
    };
    if tld.len() < 2 {
        return false;
    }
    labels.iter().all(|label| {
        label
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    })
}

/// Prefixes that mean "perform an HTTP request" — the browser worker will
/// attempt a real `fetch()` for these prompts before falling back to iframe.
const HTTP_FETCH_PREFIXES: &[&str] = &[
    "fetch ",
    "fetch url ",
    "http fetch ",
    "request ",
    "make request to ",
    "send request to ",
    "сделай запрос ",
    "сделай http запрос ",
    "выполни запрос ",
    "выполни http запрос ",
    "запроси ",
    "получи ",
    "http запрос к ",
    "http запрос на ",
    "сделать запрос к ",
    "выполнить запрос к ",
];

/// Markers that mean "perform an HTTP request" even when they appear after
/// other words in the prompt.
const HTTP_FETCH_MARKERS: &[&str] = &[
    "make a request to",
    "make an http request to",
    "send a request to",
    "send an http request to",
    "http request to",
    "http get to",
    "fetch the url",
    "fetch this url",
    "fetch the page",
    "сделай запрос к",
    "сделай запрос на",
    "сделай http запрос к",
    "сделай http запрос на",
    "выполни запрос к",
    "выполни запрос на",
    "выполни http запрос к",
    "выполни http запрос на",
    "запрос к",
    "запрос на",
    "http запрос к",
    "http запрос на",
];

fn is_http_fetch_prompt(prompt: &str, normalized: &str, _raw_candidate: &str) -> bool {
    let normalized_words = normalize_prompt(prompt);
    let raw = prompt.trim_start().to_lowercase();
    if HTTP_FETCH_PREFIXES.iter().any(|prefix| {
        normalized_words.starts_with(prefix)
            || normalized.starts_with(prefix)
            || raw.starts_with(prefix)
    }) {
        return true;
    }
    HTTP_FETCH_MARKERS.iter().any(|marker| {
        normalized_words.contains(marker) || normalized.contains(marker) || raw.contains(marker)
    })
}

/// Prefixes that mean "navigate to / show this page" — the browser worker
/// must NOT attempt `fetch()` for these prompts; it returns a direct external
/// link that the user can open in a new tab.
const URL_NAVIGATE_PREFIXES: &[&str] = &[
    "navigate to ",
    "navigate ",
    "go to ",
    "goto ",
    "visit ",
    "browse to ",
    "browse ",
    "show ",
    "show me ",
    "display ",
    "load ",
    "open ",
    "open url ",
    "open the url ",
    "open site ",
    "open website ",
    "open page ",
    "open the page ",
    "open the website ",
    "take me to ",
    "preview ",
    "view ",
    "see ",
    "перейди ",
    "перейди на ",
    "переходи на ",
    "переходи ",
    "перейдите на ",
    "открой ",
    "открой сайт ",
    "открой страницу ",
    "открой ссылку ",
    "открой урл ",
    "покажи ",
    "покажи сайт ",
    "покажи страницу ",
    "покажи мне ",
    "загрузи ",
    "загрузи страницу ",
    "посети ",
    "зайди на ",
    "зайди ",
    "просмотри ",
    "отобрази ",
];

/// Markers (anywhere in the prompt) that route to the URL navigation intent.
const URL_NAVIGATE_MARKERS: &[&str] = &[
    "navigate to",
    "go to",
    "goto",
    "browse to",
    "take me to",
    "open the page",
    "open the site",
    "open the website",
    "open the url",
    "open url",
    "перейди на",
    "переходи на",
    "перейдите на",
    "открой сайт",
    "открой страницу",
    "открой ссылку",
    "открой урл",
    "покажи сайт",
    "покажи страницу",
    "зайди на",
];

fn is_url_navigate_prompt(prompt: &str, normalized: &str, raw_candidate: &str) -> bool {
    let normalized_words = normalize_prompt(prompt);
    let prompt_trimmed = prompt.trim_start();
    if prompt_trimmed.starts_with(raw_candidate) {
        // Bare URL — treat as navigation, not a request to fetch.
        return true;
    }
    let raw = prompt_trimmed.to_lowercase();
    if URL_NAVIGATE_PREFIXES.iter().any(|prefix| {
        normalized_words.starts_with(prefix)
            || normalized.starts_with(prefix)
            || raw.starts_with(prefix)
    }) {
        return true;
    }
    URL_NAVIGATE_MARKERS.iter().any(|marker| {
        normalized_words.contains(marker) || normalized.contains(marker) || raw.contains(marker)
    })
}

fn extract_web_search_query(prompt: &str, normalized: &str) -> Option<String> {
    let normalized_words = normalize_prompt(prompt);
    if normalized_words.starts_with("search conversations ")
        || normalized_words.starts_with("search my conversations ")
        || normalized_words.starts_with("search my chats ")
    {
        return None;
    }
    let prefixes = [
        "search the web for ",
        "search web for ",
        "search the internet for ",
        "search internet for ",
        "search online for ",
        "search for information about ",
        "search for information on ",
        "web search for ",
        "find on the internet ",
        "find online ",
        "find information about ",
        "find information on ",
        "find info about ",
        "find info on ",
        "look up information about ",
        "look up information on ",
        "look up info about ",
        "look up info on ",
        "look up online ",
        "найди в интернете ",
        "поищи в интернете ",
        "поиск в интернете ",
        "найди онлайн ",
        "поищи онлайн ",
        "найди в сети ",
        "поищи в сети ",
        "найди информацию в интернете о ",
        "найди информацию в интернете об ",
        "поищи информацию в интернете о ",
        "поищи информацию в интернете об ",
        "найди информацию о ",
        "найди информацию об ",
        "найди информацию про ",
        "найди информацию по ",
        "найти информацию о ",
        "найти информацию об ",
        "поищи информацию о ",
        "поищи информацию об ",
        "поищи информацию про ",
        "поищи информацию по ",
        "найди инфу о ",
        "найди инфу об ",
        "поищи инфу о ",
        "поищи инфу об ",
        "найди сведения о ",
        "найди сведения об ",
        "поищи сведения о ",
        "поищи сведения об ",
        "найди материалы о ",
        "найди материалы об ",
        "поищи материалы о ",
        "поищи материалы об ",
    ];
    for prefix in prefixes {
        if let Some(query) = normalized_words.strip_prefix(prefix) {
            let query = clean_search_query(query);
            if !query.is_empty() && normalize_url_candidate(&query).is_none() {
                return Some(query);
            }
        }
        if let Some(query) = normalized.strip_prefix(prefix) {
            let query = clean_search_query(query);
            if !query.is_empty() && normalize_url_candidate(&query).is_none() {
                return Some(query);
            }
        }
    }
    None
}

fn clean_search_query(value: &str) -> String {
    value
        .trim()
        .trim_matches(is_url_wrapper_punctuation)
        .trim_end_matches(is_url_trailing_punctuation)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
