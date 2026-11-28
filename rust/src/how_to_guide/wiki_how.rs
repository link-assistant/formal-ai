//! Recover guides through wikiHow's public search and canonical articles.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;
use url::Url;

use super::{
    CaptureExtractor, GuideStep, MIN_ACCEPTED_STEPS, SourceRecord, SourceTransport, StepExtractor,
    Walk, WalkSourceOutcome, matches_wiki_title, parse_url, percent_encode, push_steps,
};

const ARTICLE_DEPTH: usize = 2;

pub(super) fn recover<T: SourceTransport>(
    record: &SourceRecord,
    entry: &str,
    extractor: &StepExtractor<'_>,
    walk: &Walk<'_, T>,
    outcome: &mut WalkSourceOutcome,
) -> Option<Vec<GuideStep>> {
    let remaining = walk
        .bounds
        .max_pages_per_service
        .saturating_sub(outcome.pages);
    if walk.bounds.max_depth < ARTICLE_DEPTH || remaining < 2 {
        return None;
    }
    let endpoint = Url::parse(entry).ok()?;
    let query = search_query(extractor.task, extractor.language);
    let search = endpoint
        .join(&format!(
            "/Special:LSearch?fulltext=Search&search={}",
            percent_encode(&query)
        ))
        .ok()?;
    let capture = match walk.client.fetch(search.as_str()) {
        Ok(capture) => capture,
        Err(error) => {
            note_failure(
                outcome,
                "publisher_search_failed",
                search.as_str(),
                &error.to_string(),
            );
            return None;
        }
    };
    outcome.pages += 1;
    let mut attempts = remaining - 1;
    let mut seen = BTreeSet::new();
    let articles = result_articles(std::str::from_utf8(capture.bytes()).ok()?, &endpoint)
        .filter(|(title, _)| matches_wiki_title(extractor.task, title, extractor.language))
        .map(|(title, article)| (parse_url(entry, &title), article));
    for (api, article) in articles {
        if attempts == 0 {
            break;
        }
        if !seen.insert(article.clone()) {
            continue;
        }
        let mut items = if api == entry {
            Vec::new()
        } else {
            attempts -= 1;
            match walk.client.fetch(&api) {
                Ok(capture) => {
                    outcome.pages += 1;
                    let read = extractor.read(record, &capture, ARTICLE_DEPTH, 0, walk.bounds);
                    if read.items.len() < MIN_ACCEPTED_STEPS {
                        let reason = read.detail.as_deref().unwrap_or("insufficient_steps");
                        note_failure(outcome, "publisher_parse_failed", &api, reason);
                    }
                    read.items
                }
                Err(error) => {
                    note_failure(outcome, "publisher_parse_failed", &api, &error.to_string());
                    Vec::new()
                }
            }
        };
        let mut source = api.as_str();
        if items.len() < MIN_ACCEPTED_STEPS {
            if attempts == 0 {
                break;
            }
            attempts -= 1;
            source = article.as_str();
            let capture = match walk.client.fetch(source) {
                Ok(capture) => capture,
                Err(error) => {
                    note_failure(
                        outcome,
                        "publisher_article_failed",
                        source,
                        &error.to_string(),
                    );
                    continue;
                }
            };
            outcome.pages += 1;
            let steps = std::str::from_utf8(capture.bytes()).map_or_else(
                |_| Vec::new(),
                |body| article_steps(body, &article, extractor, walk.bounds.max_items),
            );
            if steps.len() < MIN_ACCEPTED_STEPS {
                note_failure(
                    outcome,
                    "publisher_article_failed",
                    source,
                    "no_matching_steps",
                );
                continue;
            }
            items.clear();
            push_steps(record, &capture, ARTICLE_DEPTH, &steps, &mut items);
        }
        let detail = crate::trace_record::line(
            "publisher_search",
            &[("search", search.to_string()), ("url", source.to_owned())],
        );
        outcome.detail = crate::trace_record::join(&outcome.detail, &detail);
        return Some(items);
    }
    None
}

fn note_failure(outcome: &mut WalkSourceOutcome, event: &str, url: &str, reason: &str) {
    let detail = crate::trace_record::line(
        event,
        &[("url", url.to_owned()), ("reason", reason.to_owned())],
    );
    outcome.detail = crate::trace_record::join(&outcome.detail, &detail);
}

/// Include one alternate aspect where the language's title policy declares one.
fn search_query(task: &str, language: &str) -> String {
    let policy = |key: &str| crate::rule_interpreter::handler_policy("procedural_how_to", key);
    let Some(prefix) = policy(&format!("aspect-prefix-{language}")) else {
        return task.to_owned();
    };
    let endings = policy(&format!("infinitive-endings-{language}")).unwrap_or_default();
    let is_infinitive = |word: &str| {
        endings.split_whitespace().any(|ending| {
            word.strip_suffix(ending)
                .is_some_and(|stem| stem.chars().count() >= 3)
        })
    };
    let Some(word) = task
        .split(|character| !crate::source_walk::is_topic_character(character))
        .find(|word| is_infinitive(word))
    else {
        return task.to_owned();
    };
    let alternate = match word.strip_prefix(&prefix) {
        Some(base) if is_infinitive(base) => base.to_owned(),
        Some(_) => return task.to_owned(),
        None => format!("{prefix}{word}"),
    };
    format!("{task} {alternate}")
}

fn result_articles<'a>(
    html: &'a str,
    endpoint: &'a Url,
) -> impl Iterator<Item = (String, Url)> + 'a {
    static RESULT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"<a\s+class="result_link"\s+href="([^"]+)""#)
            .expect("valid result-link pattern")
    });
    RESULT.captures_iter(html).filter_map(|anchor| {
        let url = endpoint.join(&anchor[1]).ok()?;
        if url.origin() != endpoint.origin()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return None;
        }
        let slug = article_slug(&url)?;
        Some((slug.replace(['-', '_'], " "), url))
    })
}

fn article_slug(url: &Url) -> Option<String> {
    let slug = url.path().strip_prefix('/')?;
    // Decode a single path segment without treating '+' as a form space.
    let escaped = slug
        .replace('+', "%2B")
        .replace('&', "%26")
        .replace('=', "%3D");
    let (title, _) = url::form_urlencoded::parse(escaped.as_bytes()).next()?;
    (!title.is_empty() && !title.contains(['/', ':', '\u{fffd}'])).then(|| title.into_owned())
}

fn article_steps(
    html: &str,
    expected: &Url,
    extractor: &StepExtractor<'_>,
    limit: usize,
) -> Vec<String> {
    static CANONICAL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)<link\b[^>]*\brel=["']canonical["'][^>]*\bhref=["']([^"']+)["']"#)
            .expect("valid canonical-link pattern")
    });
    let canonical = CANONICAL
        .captures(html)
        .and_then(|link| Url::parse(&link[1]).ok());
    if !canonical.is_some_and(|url| {
        url.origin() == expected.origin()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && article_slug(&url) == article_slug(expected)
    }) {
        return Vec::new();
    }
    step_lists(html, extractor.task, extractor.language, limit, false)
}

/// Select a complete method for the requested scope and keep its top-level order.
pub(super) fn steps(html: &str, task: &str, language: &str, limit: usize) -> Vec<String> {
    step_lists(html, task, language, limit, true)
}

fn step_lists(
    html: &str,
    task: &str,
    language: &str,
    limit: usize,
    allow_legacy: bool,
) -> Vec<String> {
    use super::extract::{compact_step_text, extract_steps, step_text};
    static TAG: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?is)<h[1-6](?:\s[^>]*)?>(.*?)</h[1-6]>|<(/?)(ol|ul|li)(?:\s([^>]*))?>")
            .expect("valid method tag pattern")
    });
    static HEADLINE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?is)<span class="mw-headline"[^>]*>(.*?)</span>"#)
            .expect("valid MediaWiki headline pattern")
    });
    let scope = crate::rule_interpreter::handler_policy(
        "procedural_how_to",
        &format!("scoped-method-markers-{language}"),
    )
    .unwrap_or_default();
    let scope = crate::seed::parser::split_pipe_list(&scope);
    let markers: Vec<&str> = scope
        .iter()
        .map(String::as_str)
        .filter(|marker| !marker.is_empty())
        .collect();
    let requested = task.to_lowercase();
    let mut rank = 1;
    let mut best_rank = None;
    let mut best = Vec::new();
    let mut current: Option<Vec<String>> = None;
    let mut item_start = None;
    let mut depth = 0_usize;
    let mut saw_ordered_list = false;
    for tag in TAG.captures_iter(html) {
        if let Some(heading) = tag.get(1) {
            if depth == 0 {
                let headline = HEADLINE.captures(heading.as_str());
                let label =
                    compact_step_text(headline.as_ref().map_or(heading.as_str(), |name| &name[1]));
                rank = method_rank(&label, &requested, language, &markers);
            }
            continue;
        }
        let closing = &tag[2] == "/";
        let kind = tag[3].to_ascii_lowercase();
        let span = tag.get(0).expect("matched method tag");
        if kind == "li" {
            if depth == 1
                && let Some(items) = current.as_mut()
            {
                if !closing {
                    item_start = Some(span.end());
                } else if let Some(start) = item_start.take()
                    && items.len() < limit
                    && let Some(text) = if allow_legacy {
                        step_text(&html[start..span.start()])
                    } else {
                        article_step_text(&html[start..span.start()])
                    }
                    && !items.contains(&text)
                {
                    items.push(text);
                }
            }
        } else if closing {
            depth = depth.saturating_sub(1);
            if depth == 0
                && let Some(items) = current.take()
                && items.len() >= MIN_ACCEPTED_STEPS
                && best_rank.is_none_or(|best| rank > best)
            {
                best_rank = Some(rank);
                best = items;
            }
            if depth == 0 {
                item_start = None;
            }
        } else {
            if depth == 0 && kind == "ol" {
                saw_ordered_list = true;
                let attributes = tag.get(4).map_or("", |attributes| attributes.as_str());
                if attributes.contains("steps_list_2")
                    || (allow_legacy && attributes.trim().is_empty())
                {
                    current = Some(Vec::new());
                }
            }
            depth += 1;
        }
    }
    if allow_legacy && !saw_ordered_list {
        extract_steps(html, limit)
    } else {
        best
    }
}

fn article_step_text(item: &str) -> Option<String> {
    use super::extract::{MIN_STEP_CHARS, compact_step_text, step_text};
    static BODY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)<div\b[^>]*\bclass=["']step["'][^>]*>"#)
            .expect("valid public step pattern")
    });
    static REFERENCE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?is)<sup\b[^>]*\bclass=["']reference\b[^>]*>.*?</sup>"#)
            .expect("valid public reference pattern")
    });
    if let Some(body) = BODY.find(item) {
        let text = compact_step_text(&REFERENCE.replace_all(&item[body.end()..], ""));
        (text.chars().count() >= MIN_STEP_CHARS).then_some(text)
    } else {
        step_text(item)
    }
}

fn method_rank(label: &str, task: &str, language: &str, markers: &[&str]) -> u8 {
    let words = label.to_lowercase();
    let scopes = markers
        .iter()
        .filter(|marker| has_scope(&words, marker, language));
    let mut rank = 1;
    for marker in scopes {
        if has_scope(task, marker, language) {
            return 3;
        }
        rank = 0;
    }
    if rank > 0 && matches_wiki_title(task, label, language) {
        2
    } else {
        rank
    }
}

fn has_scope(text: &str, marker: &str, language: &str) -> bool {
    if language == "zh" {
        return text.contains(marker);
    }
    text.split(|character| !crate::source_walk::is_topic_character(character))
        .any(|word| word.starts_with(marker))
}
