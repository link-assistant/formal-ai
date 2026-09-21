//! Structural URL parsing and seed-driven fetch/navigation intent recognition.

use crate::engine::normalize_prompt;
use crate::seed;

pub(super) fn extract_http_fetch_url(prompt: &str, normalized: &str) -> Option<String> {
    let (raw_candidate, url) = first_url_candidate(prompt)?;
    is_http_fetch_prompt(prompt, normalized, &raw_candidate).then_some(url)
}

pub(super) fn extract_url_navigate_url(prompt: &str, normalized: &str) -> Option<String> {
    let (raw_candidate, url) = first_url_candidate(prompt)?;
    is_url_navigate_prompt(prompt, normalized, &raw_candidate).then_some(url)
}

pub(super) fn first_url_candidate(prompt: &str) -> Option<(String, String)> {
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

pub(super) const fn is_url_wrapper_punctuation(character: char) -> bool {
    matches!(
        character,
        '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`' | '«' | '»'
    )
}

pub(super) const fn is_url_trailing_punctuation(character: char) -> bool {
    matches!(character, '.' | ',' | '!' | '?' | ';' | ':' | '…')
}

pub(in crate::solver_handlers) fn normalize_url_candidate(candidate: &str) -> Option<String> {
    let candidate = candidate.trim();
    if candidate.is_empty() || candidate.contains(char::is_whitespace) || candidate.contains('@') {
        return None;
    }
    let lower = candidate.to_lowercase();
    let url = if lower.starts_with("http://") || lower.starts_with("https://") {
        candidate.to_owned()
    } else {
        let host_candidate = candidate.split(['/', '?', '#']).next().unwrap_or_default();
        if super::super::web_search_intent::probable_local_file_name(host_candidate) {
            return None;
        }
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
    looks_like_hostname(host).then_some(url)
}

pub(super) fn looks_like_hostname(value: &str) -> bool {
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

fn is_http_fetch_prompt(prompt: &str, normalized: &str, _raw_candidate: &str) -> bool {
    let normalized_words = normalize_prompt(prompt);
    let raw = prompt.trim_start().to_lowercase();
    role_evidences_web_intent(
        seed::ROLE_HTTP_FETCH,
        &[normalized_words.as_str(), normalized, raw.as_str()],
    )
}

fn is_url_navigate_prompt(prompt: &str, normalized: &str, raw_candidate: &str) -> bool {
    let normalized_words = normalize_prompt(prompt);
    let prompt_trimmed = prompt.trim_start();
    if prompt_trimmed.starts_with(raw_candidate) {
        return true;
    }
    let raw = prompt_trimmed.to_lowercase();
    role_evidences_web_intent(
        seed::ROLE_URL_NAVIGATE,
        &[normalized_words.as_str(), normalized, raw.as_str()],
    )
}
