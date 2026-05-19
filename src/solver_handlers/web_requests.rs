//! URL fetch, URL navigation, and browser-search handlers.

use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::web_search_core::{
    WEB_SEARCH_PROVIDERS as CORE_WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K as CORE_WEB_SEARCH_RRF_K,
};

use super::finalize_simple;

/// Match prompts that explicitly ask the engine to perform an HTTP request
/// (e.g. `fetch google.com`, `Сделай запрос к google.com`). In the browser
/// web app the actual `fetch()` is attempted first, with an iframe fallback when
/// CORS blocks the request. Non-fetch URL prompts (`Navigate to github.com`,
/// `Visit github.com`, ...) are handled by [`try_url_navigate`] instead.
pub fn try_http_fetch(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let url = extract_http_fetch_url(prompt, normalized)?;
    log.append("http_fetch:request", url.clone());
    let body = format!(
        "HTTP fetch requested for `{url}`.\n\n\
         The browser web app attempts a direct `fetch()` first and shows the \
         response body when the server allows CORS. If the request is blocked \
         by CORS, the page falls back to an embedded iframe with open-in-new-tab \
         and full-screen controls.\n\n\
         Source: [{url}]({url})"
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

/// Match prompts that ask the assistant to navigate to or display a URL
/// without performing an HTTP request (e.g. `Navigate to github.com`,
/// `Go to github.com`, `Перейди на github.com`). The browser web app renders
/// a direct external link; no `fetch()` is attempted.
pub fn try_url_navigate(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let url = extract_url_navigate_url(prompt, normalized)?;
    log.append("url_navigate:request", url.clone());
    log.append("url_preview:external_link", url.clone());
    let body = format!(
        "I suggest opening this in a new tab: [{url}]({url}).\n\n\
         This web app cannot reliably confirm ahead of time whether an \
         arbitrary site allows embedding, so I am using a direct external link \
         instead of an embedded preview."
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
             Wikipedia REST и Wikidata. Топ-10 ссылок объединяются через \
             reciprocal rank fusion (`score(d) = Σ 1 / ({WEB_SEARCH_RRF_K} + rank_i(d))`), \
             поэтому URL, которые встречаются у нескольких провайдеров, всплывают \
             вверх. Для произвольной страницы используйте `fetch example.com`; если \
             прямой `fetch()` заблокирован CORS, страница откроется во встроенном \
             iframe.\n\n\
             Provider: duckduckgo (default)\n\
             Providers considered: {provider_summary}\n\
             Combined ranking: reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})"
        ),
        _ => format!(
            "Web search requested for `{query}`.\n\n\
             In the browser demo formal-ai defaults to the DuckDuckGo Instant \
             Answer endpoint (CORS-readable, keyless) and queries Wikipedia REST \
             and Wikidata in parallel. The top-10 links from each provider are \
             merged with reciprocal rank fusion (`score(d) = Σ 1 / ({WEB_SEARCH_RRF_K} + rank_i(d))`), \
             so URLs that appear in more than one provider bubble up. For an \
             arbitrary page, use `fetch example.com`; if direct `fetch()` is \
             blocked by CORS, the page opens in an embedded iframe.\n\n\
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
        "web search for ",
        "find on the internet ",
        "find online ",
        "look up online ",
        "найди в интернете ",
        "поищи в интернете ",
        "поиск в интернете ",
        "найди онлайн ",
        "поищи онлайн ",
        "найди в сети ",
        "поищи в сети ",
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
