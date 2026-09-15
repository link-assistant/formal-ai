//! Rosetta Code task-page discovery through its `MediaWiki` API.

use crate::seed::percent_encode;
use crate::source_fetch::{CachedSourceClient, FetchError, SourceTransport};

const API: &str = "https://rosettacode.org/w/api.php";
const PAGE_BASE: &str = "https://rosettacode.org/wiki/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosettaExample {
    pub task: String,
    pub language: String,
    pub code: String,
    pub source_url: String,
    pub license: String,
    pub sha256: String,
    pub fetched_at: String,
    pub cached: bool,
}

pub fn fetch_example<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    page_title: &str,
    language_slug: &str,
) -> Result<RosettaExample, FetchError> {
    let language = language_heading(language_slug).ok_or_else(|| {
        FetchError::Cache(format!("rosetta_code_unknown_language:{language_slug}"))
    })?;
    let title = canonical_page_title(page_title);
    let url = format!(
        "{API}?action=parse&page={}&prop=wikitext%7Cdisplaytitle&format=json&origin=*",
        percent_encode(&title)
    );
    let capture = client.fetch(&url)?;
    let value: serde_json::Value = serde_json::from_slice(capture.bytes())
        .map_err(|error| FetchError::Cache(format!("rosetta_code_invalid_json:{error}")))?;
    let task = value
        .pointer("/parse/title")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| FetchError::Cache(String::from("rosetta_code_missing_title")))?;
    let wikitext = value
        .pointer("/parse/wikitext/*")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| FetchError::Cache(String::from("rosetta_code_missing_wikitext")))?;
    let code = code_in_language_section(wikitext, language).ok_or_else(|| {
        FetchError::Cache(format!("rosetta_code_missing_example:{title}:{language}"))
    })?;
    Ok(RosettaExample {
        task: task.to_owned(),
        language: language_slug.to_owned(),
        code,
        source_url: format!("{PAGE_BASE}{title}"),
        license: String::from("GFDL-1.2"),
        sha256: capture.sha256().to_owned(),
        fetched_at: capture.fetched_at().to_owned(),
        cached: capture.cached(),
    })
}

#[must_use]
pub fn page_title_from_url(prompt: &str) -> Option<String> {
    let lowercase = prompt.to_ascii_lowercase();
    let marker = PAGE_BASE.to_ascii_lowercase();
    let start = lowercase.find(&marker)? + marker.len();
    let tail = &prompt[start..];
    let encoded = tail
        .split_whitespace()
        .next()?
        .trim_end_matches(['.', ',', ';', ')', ']', '}']);
    if encoded.is_empty()
        || !encoded
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'%' | b'/'))
    {
        return None;
    }
    Some(percent_decode(encoded).replace('/', "_"))
}

#[must_use]
pub fn page_title_for_task(task_slug: &str) -> String {
    let mut title = task_slug.to_owned();
    if let Some(first) = title.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    title
}

fn canonical_page_title(title: &str) -> String {
    title.trim().replace(' ', "_")
}

fn language_heading(slug: &str) -> Option<&'static str> {
    Some(match slug {
        "c" => "C",
        "cpp" => "C++",
        "csharp" => "C#",
        "go" => "Go",
        "java" => "Java",
        "javascript" => "JavaScript",
        "kotlin" => "Kotlin",
        "php" => "PHP",
        "python" => "Python",
        "ruby" => "Ruby",
        "rust" => "Rust",
        "scala" => "Scala",
        "typescript" => "TypeScript",
        _ => return None,
    })
}

fn code_in_language_section(wikitext: &str, language: &str) -> Option<String> {
    let lowercase = wikitext.to_lowercase();
    let heading = format!("{{{{header|{}}}}}", language.to_lowercase());
    let heading_start = lowercase.find(&heading)?;
    let after_heading = heading_start + heading.len();
    let remainder = &wikitext[after_heading..];
    let remainder_lower = &lowercase[after_heading..];
    let section_end = remainder_lower
        .find("=={{header|")
        .unwrap_or(remainder.len());
    let section = &remainder[..section_end];
    extract_tag_body(section, "syntaxhighlight").or_else(|| extract_tag_body(section, "lang"))
}

fn extract_tag_body(section: &str, tag: &str) -> Option<String> {
    let lower = section.to_lowercase();
    let open = format!("<{tag}");
    let tag_start = lower.find(&open)?;
    let body_start = tag_start + section[tag_start..].find('>')? + 1;
    let close = format!("</{tag}>");
    let body_end = body_start + lower[body_start..].find(&close)?;
    let code = section[body_start..body_end]
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    Some(code.trim().to_owned())
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) = (hex(bytes[index + 1]), hex(bytes[index + 2]))
        {
            decoded.push(high * 16 + low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

const fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
