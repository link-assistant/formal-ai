//! URL fetch, URL navigation, and browser-search handlers.

use crate::concepts::extract_concept_query;
use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, projects_registry, ProjectRecord};
use crate::summarization::{describe_project, SummarizationConfig, SummarizationMode};
use crate::web_search_core::{
    WEB_SEARCH_PROVIDERS as CORE_WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K as CORE_WEB_SEARCH_RRF_K,
};

use super::finalize_simple;
use super::web_search_intent::{extract_web_search_request, WebSearchQueryKind};

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
    let request = extract_web_search_request(prompt, normalized)?;
    Some(answer_web_search_query(
        prompt,
        &request.query,
        request.kind,
        log,
    ))
}

pub fn answer_web_search_query(
    prompt: &str,
    query: &str,
    query_kind: WebSearchQueryKind,
    log: &mut EventLog,
) -> SymbolicAnswer {
    log.append("web_search:request", query.to_owned());
    log.append("web_search:query_kind", query_kind.as_str());
    for provider in WEB_SEARCH_PROVIDERS {
        log.append("web_search:provider", (*provider).to_owned());
    }
    log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));
    let provider_summary = WEB_SEARCH_PROVIDERS.join(", ");
    let language = detect_language(prompt).slug();
    let is_latest_news_request = matches!(query_kind, WebSearchQueryKind::LatestNews);
    let is_research_request = matches!(
        query_kind,
        WebSearchQueryKind::ImplicitResearchQuestion
            | WebSearchQueryKind::EnumerationResearchRequest
    );
    let body = match language {
        "ru" if is_latest_news_request => format!(
            "Запрошены последние новости для `{query}`.\n\n\
             В браузерной демо-версии formal-ai такой запрос идет через веб-поиск: \
             DuckDuckGo Instant Answer по умолчанию, затем Internet Archive, \
             Wikipedia REST, Wikidata, Wiktionary и Wikinews (Викиновости, \
             https://www.wikinews.org/) в указанном порядке приоритета. Топ-10 \
             ссылок от каждого провайдера объединяются через reciprocal rank \
             fusion (`score(d) = Σ 1 / ({WEB_SEARCH_RRF_K} + rank_i(d))`), а \
             диагностика записывает провайдеры, ранги, объединение и итоговые \
             ссылки, чтобы рассуждение можно было проверить.\n\n\
             Provider: duckduckgo (default)\n\
             Providers considered: {provider_summary}\n\
             Combined ranking: reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})"
        ),
        "ru" if is_research_request => format!(
            "Распознан исследовательский вопрос для `{query}`.\n\n\
             Чтобы ответить на такой вопрос без локального правила, браузерная \
             демо-версия formal-ai ищет проверяемые источники: по умолчанию \
             DuckDuckGo Instant Answer (CORS-совместимый, без ключа), затем \
             Internet Archive, Wikipedia REST, Wikidata, Wiktionary и Wikinews \
             в указанном \
             порядке приоритета. Топ-10 ссылок от каждого провайдера объединяются \
             через reciprocal rank fusion (`score(d) = Σ 1 / ({WEB_SEARCH_RRF_K} + \
             rank_i(d))`), а диагностика записывает провайдеры, ранги, объединение \
             и итоговые ссылки, чтобы рассуждение можно было проверить.\n\n\
             Provider: duckduckgo (default)\n\
             Providers considered: {provider_summary}\n\
             Combined ranking: reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})"
        ),
        "ru" => format!(
            "Поиск в интернете запрошен для `{query}`.\n\n\
             В браузерной демо-версии formal-ai по умолчанию использует DuckDuckGo \
             Instant Answer (CORS-совместимый, без ключа) и параллельно опрашивает \
             Internet Archive, Wikipedia REST, Wikidata, Wiktionary и Wikinews \
             в указанном \
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
        _ if is_latest_news_request => format!(
            "Latest-news search requested for `{query}`.\n\n\
             In the browser demo formal-ai uses web search for freshness-sensitive \
             news prompts: DuckDuckGo Instant Answer by default, then Internet \
             Archive, Wikipedia REST, Wikidata, Wiktionary, and Wikinews \
             (https://www.wikinews.org/) in priority order. The top-10 links \
             from each provider are merged with reciprocal rank fusion \
             (`score(d) = Σ 1 / ({WEB_SEARCH_RRF_K} + rank_i(d))`), and \
             diagnostics record each provider, rank, fusion step, and final \
             source link so the reasoning path can be inspected.\n\n\
             Provider: duckduckgo (default)\n\
             Providers considered: {provider_summary}\n\
             Combined ranking: reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})"
        ),
        _ if is_research_request => format!(
            "Open research question detected for `{query}`.\n\n\
             To answer this without a local rule, the browser demo searches \
             verifiable sources: DuckDuckGo Instant Answer by default, then \
             Internet Archive, Wikipedia REST, Wikidata, Wiktionary, and \
             Wikinews in priority order. The top-10 links from each provider are merged \
             with reciprocal rank fusion (`score(d) = Σ 1 / ({WEB_SEARCH_RRF_K} + \
             rank_i(d))`), and diagnostics record each provider, rank, fusion \
             step, and final source link so the reasoning path can be inspected.\n\n\
             Provider: duckduckgo (default)\n\
             Providers considered: {provider_summary}\n\
             Combined ranking: reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})"
        ),
        _ => format!(
            "Web search requested for `{query}`.\n\n\
             In the browser demo formal-ai defaults to the DuckDuckGo Instant \
             Answer endpoint (CORS-readable, keyless) and queries Internet Archive, \
             Wikipedia REST, Wikidata, Wiktionary, and Wikinews in that priority order. The \
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
    finalize_simple(prompt, log, "web_search", "response:web_search", &body, 0.8)
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
    suppress_identity_route: bool,
) -> Option<SymbolicAnswer> {
    try_project_lookup_internal(
        prompt,
        prompt,
        log,
        promote_associative_repositories,
        suppress_identity_route,
        None,
    )
}

pub fn try_explicit_repository_lookup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    promote_associative_repositories: bool,
    suppress_identity_route: bool,
    response_language: Option<&str>,
) -> Option<SymbolicAnswer> {
    if is_text_url_extraction_prompt(normalized) {
        return None;
    }
    if extract_http_fetch_url(prompt, normalized).is_some()
        || extract_url_navigate_url(prompt, normalized).is_some()
    {
        return None;
    }
    repository_from_prompt(prompt)?;
    try_project_lookup_internal(
        prompt,
        prompt,
        log,
        promote_associative_repositories,
        suppress_identity_route,
        response_language,
    )
}

pub fn try_project_lookup_with_response_language(
    prompt: &str,
    lookup_prompt: &str,
    log: &mut EventLog,
    promote_associative_repositories: bool,
    suppress_identity_route: bool,
    response_language: &str,
) -> Option<SymbolicAnswer> {
    try_project_lookup_internal(
        prompt,
        lookup_prompt,
        log,
        promote_associative_repositories,
        suppress_identity_route,
        Some(response_language),
    )
}

fn try_project_lookup_internal(
    prompt: &str,
    lookup_prompt: &str,
    log: &mut EventLog,
    promote_associative_repositories: bool,
    suppress_identity_route: bool,
    response_language: Option<&str>,
) -> Option<SymbolicAnswer> {
    if is_text_url_extraction_prompt(lookup_prompt) {
        return None;
    }
    if let Some(repo) = repository_from_prompt(lookup_prompt) {
        if promote_associative_repositories {
            if let Some(project) = promoted_project_by_repo(&repo.owner, &repo.name) {
                return Some(render_project_lookup(
                    prompt,
                    log,
                    project,
                    response_language,
                ));
            }
        }
        return Some(render_generic_repository_lookup(
            prompt,
            log,
            Some(&repo),
            promote_associative_repositories,
            response_language,
        ));
    }
    if suppress_identity_route {
        return None;
    }

    let project = matched_project(lookup_prompt)?;
    if promote_associative_repositories && is_promoted_project(project) {
        return Some(render_project_lookup(
            prompt,
            log,
            project,
            response_language,
        ));
    }

    Some(render_generic_repository_lookup(
        prompt,
        log,
        None,
        promote_associative_repositories,
        response_language,
    ))
}

fn render_project_lookup(
    prompt: &str,
    log: &mut EventLog,
    project: &ProjectRecord,
    response_language: Option<&str>,
) -> SymbolicAnswer {
    let language = response_language.unwrap_or_else(|| detect_language(prompt).slug());
    // Issue #556: record the forced target language so a response-language
    // replay's trace and evidence carry `language_to:<code>` even though the
    // localized surface itself is produced by the summarizer below.
    if response_language.is_some() {
        log.append("language_to", language.to_owned());
    }
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
        "hi" => format!(
            "{promoted_orgs} repository context में `{display_name}` से मेरा पहला \
             मतलब [{repo_slug}]({project_url}) है — {description}\n\n\
             दूसरे matching repositories और entities browser demo में \
             `{display_name}` web search से दिखाए जाते हैं. Providers: \
             {provider_summary}. Combined ranking: reciprocal rank fusion \
             (k = {WEB_SEARCH_RRF_K}). Associative repository promotion बंद \
             करने पर जवाब generic GitHub, GitLab, और Bitbucket project lookup \
             path से जाता है."
        ),
        "zh" => format!(
            "在 {promoted_orgs} 仓库上下文中，`{display_name}` 首先指 \
             [{repo_slug}]({project_url}) — {description}\n\n\
             浏览器 demo 会通过 `{display_name}` 的网页搜索显示其他匹配的仓库和实体。\
             Providers: {provider_summary}. Combined ranking: reciprocal rank fusion \
             (k = {WEB_SEARCH_RRF_K}). 关闭 associative repository promotion 后，\
             回答会走通用 GitHub、GitLab 和 Bitbucket project lookup 路径。"
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
    response_language: Option<&str>,
) -> SymbolicAnswer {
    let language = response_language.unwrap_or_else(|| detect_language(prompt).slug());
    // Issue #556: mirror the promoted-project renderer — a forced response
    // language is recorded on the trace so the whole answerable class, not just
    // promoted projects, carries `language_to:<code>` when replayed.
    if response_language.is_some() {
        log.append("language_to", language.to_owned());
    }
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
            "hi" => format!(
                "यह {label} पर रिपॉजिटरी [{slug}]({url}) के लिए lookup है.\n\n\
                 Generic project_lookup path GitHub, GitLab, और Bitbucket पर \
                 README या project descriptions summarize कर सकता है, किसी \
                 single name के लिए special case के बिना. अगर repository \
                 promoted organizations में है और promotion enabled है, तो वह \
                 पहले दिखाई जाएगी.",
                url = repo.url
            ),
            "zh" => format!(
                "这是 {label} 上的仓库查询：[{slug}]({url})。\n\n\
                 通用 project_lookup 路径可以汇总 GitHub、GitLab 和 Bitbucket 上的 \
                 README 或项目描述，不需要为单个名称写特殊规则。如果该仓库属于 promoted \
                 organizations 且 promotion 已开启，它会优先显示。",
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
        "hi" => format!(
            "यह project या repository के लिए generic project_lookup request है.\n\n\
             मैं किसी specific repository को प्राथमिकता नहीं दे रहा हूँ क्योंकि \
             associative repository promotion disabled है. अगला step {provider_summary} \
             और similar hosts पर matching projects खोजना और summarize करना है."
        ),
        "zh" => format!(
            "这是一个关于项目或仓库的通用 project_lookup 请求。\n\n\
             我不会优先选择某个特定仓库，因为 associative repository promotion 已关闭。\
             下一步是在 {provider_summary} 和类似托管平台中搜索并汇总匹配项目。"
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
    if owner.chars().all(|ch| ch.is_ascii_digit()) && name.chars().all(|ch| ch.is_ascii_digit()) {
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

fn is_text_url_extraction_prompt(normalized: &str) -> bool {
    let normalized = normalized.trim_start().to_ascii_lowercase();
    (normalized.starts_with("extract url ") || normalized.starts_with("extract urls "))
        && normalized.contains(" from ")
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

/// Capability-intent probe (issue #680): the absolute URL a fetch-intent prompt
/// names — for *any* phrasing — or [`None`] when the prompt carries no HTTP-fetch
/// intent. This is the same `http_fetch` meaning the prose handler
/// ([`try_http_fetch`]) recognises, exposed so the deterministic agentic planner
/// (`crate::agentic_coding::planner`) can route a web-fetch request to the
/// advertised fetch tool instead of answering in prose. The planner and the prose
/// path therefore reason about fetch intent through one lexicon-driven detector
/// and never drift. `normalized` mirrors what the specialized-handler dispatch
/// passes every handler — the lowercased prompt (see `meta_method_dispatch`).
#[must_use]
pub fn http_fetch_url_for(prompt: &str) -> Option<String> {
    extract_http_fetch_url(prompt, &prompt.to_lowercase())
}

/// URL that an advertised agent fetch tool can satisfy.
///
/// Agent CLIs expose one fetch capability for both explicit HTTP requests and
/// requests to open or visit a URL. The prose solver keeps those intents
/// distinct, while this probe maps either intent onto that available tool.
#[must_use]
pub fn agentic_fetch_url_for(prompt: &str) -> Option<String> {
    let normalized = prompt.to_lowercase();
    extract_http_fetch_url(prompt, &normalized)
        .or_else(|| extract_url_navigate_url(prompt, &normalized))
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

pub(super) fn normalize_url_candidate(candidate: &str) -> Option<String> {
    let candidate = candidate.trim();
    if candidate.is_empty() || candidate.contains(char::is_whitespace) || candidate.contains('@') {
        return None;
    }
    let lower = candidate.to_lowercase();
    let url = if lower.starts_with("http://") || lower.starts_with("https://") {
        candidate.to_owned()
    } else {
        let host_candidate = candidate.split(['/', '?', '#']).next().unwrap_or_default();
        if lower.starts_with("www.") || looks_like_hostname(host_candidate) {
            format!("https://{candidate}")
        } else {
            return None;
        }
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

/// Does a meaning carrying `role` evidence one of the prompt's lowercased forms?
///
/// The surface words for the web intents are no longer a hardcoded per-language
/// list — they live once in `data/seed/meanings-web-navigation.lino` as the
/// `http_fetch` and `url_navigate` meanings, and this code names only the
/// language-independent role plus the `…` (U+2026) slot shape (issue #386). Each
/// surface form is bucketed by [`seed::Slot`] exactly as the how-cluster handler
/// does ([`crate::solver_handler_how`]):
/// * [`seed::Slot::Prefix`] — the literal before `…` must *begin* a form (so
///   "fetch …" matches "fetch google.com"); [`WordForm::before_slot`] keeps the
///   trailing space, reproducing the prior `"fetch "` prefix exactly.
/// * [`seed::Slot::Bare`] — the whole text must appear *anywhere* in a form (so
///   "запрос к" matches "сделать запрос к google.com").
///
/// `forms` are the lowercased views a web prompt is matched against (the
/// word-normalized prompt, the engine-normalized prompt, and the raw lowercased
/// prompt — see [`is_http_fetch_prompt`]). The result is a pure OR over every
/// (form, surface) pair, so the bucket order only affects short-circuiting, not
/// the outcome — behaviour is identical to the prior inline arrays.
fn role_evidences_web_intent(role: &str, forms: &[&str]) -> bool {
    let word_forms = seed::lexicon().role_word_forms(role);
    if word_forms
        .iter()
        .filter(|form| form.slot() == seed::Slot::Prefix)
        .any(|form| {
            let prefix = form.before_slot();
            forms.iter().any(|form_text| form_text.starts_with(prefix))
        })
    {
        return true;
    }
    word_forms
        .iter()
        .filter(|form| form.slot() == seed::Slot::Bare)
        .any(|form| {
            let marker = form.text.as_str();
            forms.iter().any(|form_text| form_text.contains(marker))
        })
}

/// Does the prompt ask the engine to perform an HTTP request?
///
/// Recognised by the `http_fetch` meaning's surface forms (see
/// [`role_evidences_web_intent`]): the browser worker will attempt a real
/// `fetch()` for these prompts before falling back to iframe. The match runs
/// over three lowercased views — the word-normalized prompt, the
/// engine-normalized prompt, and the raw lowercased prompt — so both
/// punctuation-stripped and verbatim phrasings are covered.
fn is_http_fetch_prompt(prompt: &str, normalized: &str, _raw_candidate: &str) -> bool {
    let normalized_words = normalize_prompt(prompt);
    let raw = prompt.trim_start().to_lowercase();
    role_evidences_web_intent(
        seed::ROLE_HTTP_FETCH,
        &[normalized_words.as_str(), normalized, raw.as_str()],
    )
}

/// Does the prompt ask the engine to open / show a page (not fetch its bytes)?
///
/// A bare URL is navigation by itself; otherwise recognised by the
/// `url_navigate` meaning's surface forms (see [`role_evidences_web_intent`]).
/// For these prompts the browser worker must NOT attempt `fetch()` — it returns
/// a direct external link the user can open in a new tab.
fn is_url_navigate_prompt(prompt: &str, normalized: &str, raw_candidate: &str) -> bool {
    let normalized_words = normalize_prompt(prompt);
    let prompt_trimmed = prompt.trim_start();
    if prompt_trimmed.starts_with(raw_candidate) {
        // Bare URL — treat as navigation, not a request to fetch.
        return true;
    }
    let raw = prompt_trimmed.to_lowercase();
    role_evidences_web_intent(
        seed::ROLE_URL_NAVIGATE,
        &[normalized_words.as_str(), normalized, raw.as_str()],
    )
}
