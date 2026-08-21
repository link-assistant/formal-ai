//! Recognising retrieved payloads and pulling ordered steps out of them.
//!
//! Both production runtimes must read the same bytes the same way, so the
//! thresholds here (minimum step length, maximum step length, the `<li><b>`
//! navigation-item skip) are the exact ones the browser worker applies in
//! `howToExtractSteps`/`howToCompactStepText` (`formal_ai_worker_how_to_guide.js`); the
//! offline QA replay in `tests/unit/issue_991_*` and
//! `tests/web/issue-991-how-to-synthesis.test.mjs` compares both against the
//! same captures.

use serde_json::Value;

/// A step shorter than this is a caption or a navigation label, not a step.
pub const MIN_STEP_CHARS: usize = 40;
/// Steps are compacted to at most this many characters at a sentence boundary.
pub const MAX_STEP_CHARS: usize = 180;

/// The shape of a captured payload, decided from the bytes alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// A `MediaWiki` `action=parse` response: a rendered page.
    Parse {
        /// `parse.displaytitle`, compacted.
        title: String,
        /// `parse.text["*"]`, the rendered HTML.
        html: String,
    },
    /// A `MediaWiki` `action=opensearch` response: candidate page titles/urls.
    OpenSearch {
        /// Suggested page titles, in server order.
        titles: Vec<String>,
        /// Canonical page URLs parallel to `titles` (may be shorter).
        urls: Vec<String>,
    },
    /// A `MediaWiki` `action=query&list=search` full-text search response.
    Search {
        /// Matching page titles, best first.
        titles: Vec<String>,
    },
    /// A Stack Exchange `search/advanced` response with `filter=withbody`.
    Items {
        /// One entry per returned question, in server order.
        entries: Vec<ItemEntry>,
    },
    /// The bytes are gzip/deflate-compressed and cannot be read as text.
    Compressed,
    /// Readable bytes whose shape is not one this module can extract from.
    Unrecognized {
        /// Stable slug explaining why, used in the evidence trace.
        reason: String,
    },
}

/// One Stack Exchange question with its rendered body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemEntry {
    /// Question title.
    pub title: String,
    /// Canonical link to the question.
    pub link: String,
    /// Rendered HTML body.
    pub body: String,
    /// Stack Exchange question id, when the entry is a question rather than an
    /// answer; the procedure usually lives one hop deeper, in its answers.
    pub question_id: Option<u64>,
}

/// Decide the payload shape from the exact captured bytes.
#[must_use]
pub fn classify(bytes: &[u8]) -> Payload {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        return Payload::Compressed;
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Payload::Unrecognized {
            reason: String::from("not_utf8"),
        };
    };
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return Payload::Unrecognized {
            reason: String::from("not_json"),
        };
    };
    classify_value(&value)
}

fn classify_value(value: &Value) -> Payload {
    if let Some(error) = value.get("error") {
        let reason = error
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("api_error");
        return Payload::Unrecognized {
            reason: format!("api_error:{reason}"),
        };
    }
    if let Some(parse) = value.get("parse") {
        let html = parse
            .get("text")
            .and_then(|text| text.get("*"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let title = parse
            .get("displaytitle")
            .or_else(|| parse.get("title"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        return Payload::Parse {
            title: compact_step_text(title),
            html: html.to_owned(),
        };
    }
    if let Some(results) = value
        .get("query")
        .and_then(|query| query.get("search"))
        .and_then(Value::as_array)
    {
        let titles = results
            .iter()
            .filter_map(|result| result.get("title"))
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        return Payload::Search { titles };
    }
    if let Some(items) = value.get("items").and_then(Value::as_array) {
        let entries = items
            .iter()
            .map(|item| ItemEntry {
                title: compact_step_text(
                    item.get("title")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                ),
                link: item
                    .get("link")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                body: item
                    .get("body")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                question_id: item.get("question_id").and_then(Value::as_u64),
            })
            .collect();
        return Payload::Items { entries };
    }
    if let Some(array) = value.as_array()
        && array.len() >= 2
    {
        let titles = string_array(array.get(1));
        let urls = string_array(array.get(3));
        return Payload::OpenSearch { titles, urls };
    }
    Payload::Unrecognized {
        reason: String::from("unknown_shape"),
    }
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Remove markup, keeping the text nodes separated by single spaces.
#[must_use]
pub fn strip_html(value: &str) -> String {
    let mut text = String::with_capacity(value.len());
    let mut inside_tag = false;
    for character in value.chars() {
        match character {
            '<' => {
                inside_tag = true;
                text.push(' ');
            }
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }
    text
}

/// Decode the handful of entities `MediaWiki` and Stack Exchange emit.
#[must_use]
pub fn decode_entities(value: &str) -> String {
    let mut decoded = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('&') {
        decoded.push_str(&rest[..start]);
        let tail = &rest[start..];
        let Some(end) = tail.find(';').filter(|end| *end <= 10) else {
            decoded.push('&');
            rest = &tail[1..];
            continue;
        };
        let entity = &tail[1..end];
        match entity {
            "nbsp" | "#160" => decoded.push(' '),
            "amp" => decoded.push('&'),
            "quot" => decoded.push('"'),
            "apos" | "#039" => decoded.push('\''),
            "lt" => decoded.push('<'),
            "gt" => decoded.push('>'),
            _ => {
                let numeric = entity
                    .strip_prefix('#')
                    .and_then(|digits| digits.parse::<u32>().ok())
                    .and_then(char::from_u32);
                if let Some(character) = numeric {
                    decoded.push(character);
                } else {
                    decoded.push('&');
                    decoded.push_str(entity);
                    decoded.push(';');
                }
            }
        }
        rest = &tail[end + 1..];
    }
    decoded.push_str(rest);
    decoded
}

/// Strip markup and reference markers, collapse whitespace, and cut the result
/// at a sentence boundary so a step stays one readable instruction.
#[must_use]
pub fn compact_step_text(value: &str) -> String {
    let text = collapse_whitespace(&drop_reference_markers(&decode_entities(&strip_html(
        value,
    ))));
    if text.chars().count() <= MAX_STEP_CHARS {
        return text;
    }
    if let Some(sentence) = first_sentence(&text) {
        return sentence;
    }
    let truncated: String = text.chars().take(MAX_STEP_CHARS - 3).collect();
    format!("{}...", truncated.trim_end())
}

fn drop_reference_markers(value: &str) -> String {
    let mut cleaned = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('[') {
        let tail = &rest[start..];
        let marker = tail
            .find(']')
            .filter(|end| *end > 1 && tail[1..*end].chars().all(|item| item.is_ascii_digit()));
        cleaned.push_str(&rest[..start]);
        if let Some(end) = marker {
            rest = &tail[end + 1..];
        } else {
            cleaned.push('[');
            rest = &tail[1..];
        }
    }
    cleaned.push_str(rest);
    cleaned
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn first_sentence(text: &str) -> Option<String> {
    let mut length = 0;
    for (index, character) in text.char_indices() {
        length += 1;
        if !matches!(character, '.' | '!' | '?') {
            continue;
        }
        if !(MIN_STEP_CHARS..=MAX_STEP_CHARS).contains(&length) {
            continue;
        }
        let boundary = index + character.len_utf8();
        if text[boundary..].starts_with(char::is_whitespace) {
            return Some(text[..boundary].trim().to_owned());
        }
    }
    None
}

/// Ordered steps found in rendered HTML.
///
/// List items are the only step carrier: prose paragraphs describe, lists
/// instruct. Items whose first child is bold are section labels in the
/// `MediaWiki` skin, so they are skipped exactly as the browser worker skips
/// `<li><b>`.
#[must_use]
pub fn extract_steps(html: &str, limit: usize) -> Vec<String> {
    let mut steps: Vec<String> = Vec::new();
    for item in list_items(html) {
        if item.trim_start().starts_with("<b>") {
            continue;
        }
        let text = compact_step_text(&item);
        if text.chars().count() < MIN_STEP_CHARS || steps.contains(&text) {
            continue;
        }
        steps.push(text);
        if steps.len() >= limit {
            break;
        }
    }
    steps
}

/// The inner HTML of every `<li>` element, in document order.
fn list_items(html: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<li") {
        let tail = &rest[start..];
        let Some(open_end) = tail.find('>') else {
            break;
        };
        let inner_start = open_end + 1;
        let end = tail[inner_start..]
            .find("</li>")
            .map_or(tail.len(), |offset| inner_start + offset);
        items.push(tail[inner_start..end].to_owned());
        rest = &tail[end.min(tail.len())..];
        if rest.is_empty() {
            break;
        }
        rest = rest.get(1..).unwrap_or_default();
    }
    items
}

/// Same-wiki article titles linked from rendered HTML, deduplicated.
///
/// Used to recurse one level deeper when a captured page is a redirect or a
/// disambiguation stub that carries no steps of its own.
#[must_use]
pub fn wiki_link_titles(html: &str, limit: usize) -> Vec<String> {
    let mut titles: Vec<String> = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("href=\"/wiki/") {
        let tail = &rest[start + "href=\"/wiki/".len()..];
        let end = tail.find('"').unwrap_or(tail.len());
        let target = &tail[..end];
        rest = &tail[end.min(tail.len())..];
        if target.is_empty() || target.contains(':') || target.contains('#') {
            continue;
        }
        let title = decode_entities(target).replace('_', " ");
        if !titles.contains(&title) {
            titles.push(title);
            if titles.len() >= limit {
                break;
            }
        }
    }
    titles
}
