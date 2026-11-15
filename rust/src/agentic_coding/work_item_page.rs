//! A fetched work-item page is a document around the work item: keep its
//! title and body, drop the site chrome, and reject a page that has no body so
//! the API reads run instead (issue #1560). The markers are data in
//! `data/meta/work-item-steps.lino`.
//! JavaScript twin: js/agentic/work_item_page.mjs.

use super::work_item_steps::fill;

const NAMED_ENTITIES: [(&str, char); 7] = [
    ("amp", '&'),
    ("lt", '<'),
    ("gt", '>'),
    ("quot", '"'),
    ("apos", '\''),
    ("nbsp", '\u{a0}'),
    ("copy", '\u{a9}'),
];

fn alternatives(key: &str) -> Vec<String> {
    fill(key, &[])
        .split('|')
        .filter(|marker| !marker.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The character an entity-shaped name stands for, if any.
fn entity(name: &str) -> Option<char> {
    let Some(number) = name.strip_prefix('#') else {
        return NAMED_ENTITIES
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, character)| *character);
    };
    let code = match number.strip_prefix(['x', 'X']) {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => number.parse::<u32>(),
    };
    code.ok().filter(|code| *code > 0).and_then(char::from_u32)
}

fn entity_shaped(name: &str) -> bool {
    match name.strip_prefix('#') {
        Some(number) => match number.strip_prefix(['x', 'X']) {
            Some(hex) => (1..=6).contains(&hex.len()) && hex.bytes().all(|b| b.is_ascii_hexdigit()),
            None => (1..=7).contains(&number.len()) && number.bytes().all(|b| b.is_ascii_digit()),
        },
        None => !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_lowercase()),
    }
}

/// One pass, so `&amp;lt;` stays `&lt;`. An entity-shaped reference that
/// names no character is kept as written.
pub(super) fn decode_entities(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        decoded.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        match after
            .split_once(';')
            .filter(|(name, _)| entity_shaped(name))
        {
            Some((name, tail)) => {
                match entity(name) {
                    Some(character) => decoded.push(character),
                    None => decoded.push_str(&rest[at..at + name.len() + 2]),
                }
                rest = tail;
            }
            None => {
                decoded.push('&');
                rest = after;
            }
        }
    }
    decoded.push_str(rest);
    decoded
}

/// The title suffix or a chrome line marks a site page.
pub(super) fn is_work_item_page(raw: &str) -> bool {
    let chrome = alternatives("page_chrome_lines");
    let first = raw
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");
    first.ends_with(&fill("page_title_suffix", &[]))
        || raw
            .lines()
            .map(str::trim)
            .any(|line| chrome.iter().any(|marker| marker == line))
}

fn page_title(first: &str) -> String {
    let suffix = fill("page_title_suffix", &[]);
    let separator = fill("page_title_separator", &[]);
    let named = first.strip_suffix(suffix.as_str()).unwrap_or(first);
    let parts = named.split(separator.as_str()).collect::<Vec<_>>();
    // "<title> · Issue #N · owner/repo": the last two parts are the page's address.
    if parts.len() >= 3 {
        parts[..parts.len() - 2].join(&separator).trim().to_owned()
    } else {
        named.trim().to_owned()
    }
}

fn earliest(text: &str, markers: &[String], from: usize) -> Option<(usize, usize)> {
    markers
        .iter()
        .filter_map(|marker| {
            text[from..]
                .find(marker.as_str())
                .map(|at| (from + at, marker.len()))
        })
        .min_by_key(|(at, _)| *at)
}

fn body_lines(text: &str) -> String {
    let end_lines = alternatives("page_body_end_lines");
    let mut kept: Vec<&str> = Vec::new();
    for line in text.lines().map(str::trim) {
        if end_lines.iter().any(|marker| marker == line) {
            break;
        }
        if line.is_empty() && kept.last().is_none_or(|last| last.is_empty()) {
            continue;
        }
        kept.push(line);
    }
    while kept.last().is_some_and(|last| last.is_empty()) {
        kept.pop();
    }
    kept.join("\n")
}

/// Text that is not a site page (gh, REST or a connector's extracted record)
/// is returned unchanged. A page yields its title and body; a page without a
/// body yields `None`.
#[must_use]
pub fn work_item_page_text(raw: &str) -> Option<String> {
    if !is_work_item_page(raw) {
        return Some(raw.to_owned());
    }
    let text = decode_entities(raw);
    let (start, length) = earliest(&text, &alternatives("page_body_start"), 0)?;
    let from = start + length;
    let end =
        earliest(&text, &alternatives("page_body_end"), from).map_or(text.len(), |(at, _)| at);
    let body = body_lines(&text[from..end]);
    if body.is_empty() {
        return None;
    }
    let first = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");
    let title = page_title(first);
    Some(if title.is_empty() {
        body
    } else {
        format!("{title}\n\n{body}")
    })
}
