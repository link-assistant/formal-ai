//! URL preview and browser-search handlers.

use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;

use super::finalize_simple;

pub fn try_http_fetch(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let url = extract_fetch_url(prompt, normalized)?;
    log.append("http_fetch:request", url.clone());
    log.append("url_preview:iframe", url.clone());
    let body = format!(
        "URL requested for `{url}`.\n\n\
         Open this link: [{url}]({url}).\n\n\
         The browser demo also shows the page in an embedded iframe when the \
         site allows framing. Use the open-in-new-tab control if the site blocks \
         embedding, or the full-screen control to view it at viewport size."
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

pub fn try_web_search(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let query = extract_web_search_query(prompt, normalized)?;
    log.append("web_search:request", query.clone());
    log.append("web_search:provider", "wikipedia".to_owned());
    let language = detect_language(prompt).slug();
    let body = match language {
        "ru" => format!(
            "Поиск в интернете запрошен для `{query}`.\n\n\
             В браузерной демо-версии formal-ai сначала использует CORS-совместимый \
             поиск Wikipedia и возвращает ранжированные ссылки. Для произвольной \
             страницы используйте запрос вида `fetch example.com`; если прямой \
             `fetch()` заблокирован CORS, страница откроется во встроенном iframe.\n\n\
             Provider: wikipedia"
        ),
        _ => format!(
            "Web search requested for `{query}`.\n\n\
             In the browser demo formal-ai queries the CORS-enabled Wikipedia \
             search endpoint first and returns ranked links. For an arbitrary \
             page, use `fetch example.com`; if direct `fetch()` is blocked by \
             CORS, the page opens in an embedded iframe.\n\n\
             Provider: wikipedia"
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

fn extract_fetch_url(prompt: &str, normalized: &str) -> Option<String> {
    let (raw_candidate, url) = first_url_candidate(prompt)?;
    if !is_url_request_prompt(prompt, normalized, &raw_candidate) {
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

fn is_url_request_prompt(prompt: &str, normalized: &str, raw_candidate: &str) -> bool {
    let normalized_words = normalize_prompt(prompt);
    let prompt_trimmed = prompt.trim_start();
    if prompt_trimmed.starts_with(raw_candidate) {
        return true;
    }
    let prefixes = [
        "fetch ",
        "get ",
        "open ",
        "navigate to ",
        "go to ",
        "visit ",
        "browse to ",
        "show ",
        "show me ",
        "display ",
        "load ",
        "request ",
        "fetch url ",
        "open url ",
        "navigate url ",
        "go to url ",
        "сделай запрос ",
        "выполни запрос ",
        "запроси ",
        "получи ",
        "открой ",
        "открой сайт ",
        "покажи ",
        "загрузи ",
        "перейди ",
        "перейди на ",
    ];
    if prefixes
        .iter()
        .any(|prefix| normalized_words.starts_with(prefix) || normalized.starts_with(prefix))
    {
        return true;
    }
    let markers = [
        "make a request to",
        "send a request to",
        "http request to",
        "request to",
        "navigate to",
        "go to",
        "browse to",
        "сделай запрос к",
        "сделай запрос на",
        "выполни запрос к",
        "выполни запрос на",
        "запрос к",
        "запрос на",
    ];
    markers
        .iter()
        .any(|marker| normalized_words.contains(marker) || normalized.contains(marker))
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
