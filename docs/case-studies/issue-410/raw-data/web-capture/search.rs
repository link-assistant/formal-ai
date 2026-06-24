//! Structured search-provider capture (issue #130).
//!
//! Turns a query + provider into a normalized, machine-readable result set so
//! that browser, CLI, and server callers all consume one consistent contract
//! instead of each reimplementing provider-specific scraping. Server-side and
//! CLI callers fetch provider pages directly (no CORS restriction), so this
//! module defaults to the `fetch` capture mode. Providers that expose a native
//! CORS/JSON API (Wikipedia) are preferred; HTML search engines are parsed
//! best-effort and report CAPTCHA/blocking through `diagnostics`.
//!
//! Normalized result shape (camelCase JSON):
//! ```json
//! {
//!   "query": "...", "provider": "...", "captureMode": "fetch",
//!   "capturedAt": "2026-05-18T20:30:00Z",
//!   "results": [{ "rank": 1, "title": "...", "url": "...", "snippet": "..." }],
//!   "diagnostics": { "status": 200, "blockedByCors": false,
//!                    "blockedByCaptcha": false, "sourceUrl": "..." }
//! }
//! ```

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::form_urlencoded::byte_serialize;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Providers understood by the search contract.
pub const SEARCH_PROVIDERS: [&str; 5] = ["wikipedia", "duckduckgo", "google", "bing", "brave"];

/// Default provider when none is supplied.
pub const DEFAULT_PROVIDER: &str = "wikipedia";

/// Default number of results requested/returned.
pub const DEFAULT_LIMIT: usize = 10;

/// A single normalized search result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResultItem {
    pub rank: usize,
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Structured diagnostics describing how the capture went.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchDiagnostics {
    pub status: u16,
    pub blocked_by_cors: bool,
    pub blocked_by_captcha: bool,
    pub source_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The full normalized search capture result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub query: String,
    pub provider: String,
    pub capture_mode: String,
    pub captured_at: String,
    pub results: Vec<SearchResultItem>,
    pub diagnostics: SearchDiagnostics,
}

/// Returns true if `provider` is one of the supported providers.
#[must_use]
pub fn is_supported_provider(provider: &str) -> bool {
    SEARCH_PROVIDERS.contains(&provider)
}

/// Normalize whitespace and decode basic HTML entities in extracted text.
fn clean_text(text: &str) -> String {
    // `scraper` already returns decoded text nodes, but snippets assembled from
    // multiple nodes can carry stray whitespace; collapse it to a single line.
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Build the provider-native source URL for a query.
///
/// # Errors
///
/// Returns an error string when `provider` is not supported.
pub fn build_search_url(provider: &str, query: &str, limit: usize) -> Result<String, String> {
    let q: String = byte_serialize(query.as_bytes()).collect();
    match provider {
        "wikipedia" => Ok(format!(
            "https://en.wikipedia.org/w/rest.php/v1/search/page?q={q}&limit={limit}"
        )),
        "duckduckgo" => Ok(format!("https://html.duckduckgo.com/html/?q={q}")),
        "google" => Ok(format!("https://www.google.com/search?q={q}&num={limit}")),
        "bing" => Ok(format!("https://www.bing.com/search?q={q}&count={limit}")),
        "brave" => Ok(format!("https://search.brave.com/search?q={q}")),
        other => Err(format!(
            "Unknown search provider \"{other}\". Supported: {}",
            SEARCH_PROVIDERS.join(", ")
        )),
    }
}

/// Detect provider CAPTCHA / bot-block interstitials in an HTML body.
#[must_use]
pub fn looks_like_captcha(html: &str) -> bool {
    let lower = html.to_lowercase();
    lower.contains("captcha")
        || lower.contains("unusual traffic")
        || lower.contains("are you a robot")
        || lower.contains("/sorry/index")
        || lower.contains("automated queries")
}

/// Decode a `DuckDuckGo` redirect href (`//duckduckgo.com/l/?uddg=...`).
fn resolve_duckduckgo_href(href: &str) -> String {
    if href.is_empty() {
        return String::new();
    }
    let normalized = href
        .strip_prefix("//")
        .map_or_else(|| href.to_string(), |stripped| format!("https:{stripped}"));
    if let Ok(parsed) = url::Url::parse(&normalized) {
        if let Some((_, value)) = parsed.query_pairs().find(|(k, _)| k == "uddg") {
            return value.into_owned();
        }
        return parsed.to_string();
    }
    href.to_string()
}

/// Wikipedia REST search page entry.
#[derive(Debug, Deserialize)]
struct WikiPage {
    key: Option<String>,
    title: Option<String>,
    excerpt: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WikiResponse {
    pages: Option<Vec<WikiPage>>,
}

fn strip_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

fn parse_wikipedia(body: &str, limit: usize) -> Vec<SearchResultItem> {
    let parsed: WikiResponse = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let pages = parsed.pages.unwrap_or_default();
    pages
        .into_iter()
        .take(limit)
        .enumerate()
        .map(|(i, page)| {
            let key = page
                .key
                .clone()
                .or_else(|| page.title.clone())
                .unwrap_or_default();
            let title = clean_text(&page.title.or(page.key).unwrap_or_default());
            let snippet_raw = page.excerpt.or(page.description).unwrap_or_default();
            let snippet = clean_text(&strip_tags(&snippet_raw));
            let encoded: String = byte_serialize(key.as_bytes()).collect();
            SearchResultItem {
                rank: i + 1,
                title,
                url: format!("https://en.wikipedia.org/wiki/{encoded}"),
                snippet,
            }
        })
        .collect()
}

/// Extract trimmed text content of the first element matching `selector`.
fn first_text(element: &scraper::ElementRef, selector: &Selector) -> String {
    element
        .select(selector)
        .next()
        .map(|el| clean_text(&el.text().collect::<String>()))
        .unwrap_or_default()
}

fn parse_duckduckgo(doc: &Html, limit: usize) -> Vec<SearchResultItem> {
    let body_sel = Selector::parse(".result__body").unwrap();
    let web_sel = Selector::parse(".web-result").unwrap();
    let anchor_sel = Selector::parse("a.result__a").unwrap();
    let snippet_sel = Selector::parse(".result__snippet").unwrap();

    let mut containers: Vec<_> = doc.select(&body_sel).collect();
    if containers.is_empty() {
        containers = doc.select(&web_sel).collect();
    }

    let mut results = Vec::new();
    for el in containers {
        if results.len() >= limit {
            break;
        }
        if let Some(anchor) = el.select(&anchor_sel).next() {
            let title = clean_text(&anchor.text().collect::<String>());
            let url = resolve_duckduckgo_href(anchor.value().attr("href").unwrap_or_default());
            let snippet = first_text(&el, &snippet_sel);
            if !title.is_empty() && !url.is_empty() {
                results.push(SearchResultItem {
                    rank: results.len() + 1,
                    title,
                    url,
                    snippet,
                });
            }
        }
    }
    results
}

fn parse_google(doc: &Html, limit: usize) -> Vec<SearchResultItem> {
    let block_sel = Selector::parse("div.g, div.tF2Cxc, div.MjjYud").unwrap();
    let anchor_sel = Selector::parse("a[href^=\"http\"]").unwrap();
    let title_sel = Selector::parse("h3").unwrap();
    let snippet_sel = Selector::parse("div[data-sncf], .VwiC3b, .IsZvec").unwrap();

    let mut results = Vec::new();
    for el in doc.select(&block_sel) {
        if results.len() >= limit {
            break;
        }
        let url = el
            .select(&anchor_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or_default()
            .to_string();
        let title = first_text(&el, &title_sel);
        let snippet = first_text(&el, &snippet_sel);
        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResultItem {
                rank: results.len() + 1,
                title,
                url,
                snippet,
            });
        }
    }
    results
}

fn parse_bing(doc: &Html, limit: usize) -> Vec<SearchResultItem> {
    let block_sel = Selector::parse("li.b_algo").unwrap();
    let anchor_sel = Selector::parse("h2 a").unwrap();
    let snippet_sel = Selector::parse(".b_caption p, p").unwrap();

    let mut results = Vec::new();
    for el in doc.select(&block_sel) {
        if results.len() >= limit {
            break;
        }
        if let Some(anchor) = el.select(&anchor_sel).next() {
            let title = clean_text(&anchor.text().collect::<String>());
            let url = anchor.value().attr("href").unwrap_or_default().to_string();
            let snippet = first_text(&el, &snippet_sel);
            if !title.is_empty() && !url.is_empty() {
                results.push(SearchResultItem {
                    rank: results.len() + 1,
                    title,
                    url,
                    snippet,
                });
            }
        }
    }
    results
}

fn parse_brave(doc: &Html, limit: usize) -> Vec<SearchResultItem> {
    let block_sel = Selector::parse("div.snippet").unwrap();
    let anchor_sel = Selector::parse("a[href^=\"http\"]").unwrap();
    let title_sel = Selector::parse(".snippet-title, .title").unwrap();
    let snippet_sel = Selector::parse(".snippet-description, .snippet-content").unwrap();

    let mut results = Vec::new();
    for el in doc.select(&block_sel) {
        if results.len() >= limit {
            break;
        }
        let anchor = el.select(&anchor_sel).next();
        let url = anchor
            .and_then(|a| a.value().attr("href"))
            .unwrap_or_default()
            .to_string();
        let mut title = first_text(&el, &title_sel);
        if title.is_empty() {
            if let Some(a) = anchor {
                title = clean_text(&a.text().collect::<String>());
            }
        }
        let snippet = first_text(&el, &snippet_sel);
        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResultItem {
                rank: results.len() + 1,
                title,
                url,
                snippet,
            });
        }
    }
    results
}

/// Parse a provider response body into normalized result rows.
///
/// Pure function (no network) so it can be unit-tested against fixtures.
/// Returns the parsed rows and whether the body looked like a CAPTCHA wall.
#[must_use]
pub fn parse_search_results(
    provider: &str,
    body: &str,
    limit: usize,
) -> (Vec<SearchResultItem>, bool) {
    if provider == "wikipedia" {
        return (parse_wikipedia(body, limit), false);
    }
    let blocked = looks_like_captcha(body);
    let doc = Html::parse_document(body);
    let results = match provider {
        "duckduckgo" => parse_duckduckgo(&doc, limit),
        "google" => parse_google(&doc, limit),
        "bing" => parse_bing(&doc, limit),
        "brave" => parse_brave(&doc, limit),
        _ => Vec::new(),
    };
    (results, blocked)
}

/// Render a normalized search result as Markdown.
#[must_use]
pub fn format_search_as_markdown(result: &SearchResult) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# Search results for \"{}\"", result.query));
    lines.push(String::new());
    lines.push(format!("- Provider: `{}`", result.provider));
    lines.push(format!("- Capture mode: `{}`", result.capture_mode));
    lines.push(format!("- Captured at: {}", result.captured_at));
    lines.push(format!("- Source: {}", result.diagnostics.source_url));
    if result.diagnostics.blocked_by_captcha {
        lines.push("- ⚠️ Provider returned a CAPTCHA / bot-block page.".to_string());
    }
    lines.push(String::new());
    if result.results.is_empty() {
        lines.push("_No results._".to_string());
        return lines.join("\n");
    }
    for item in &result.results {
        lines.push(format!("{}. [{}]({})", item.rank, item.title, item.url));
        if !item.snippet.is_empty() {
            lines.push(format!("   {}", item.snippet));
        }
    }
    lines.join("\n")
}

/// Capture structured search results for a query from a provider.
///
/// `captured_at` is injected (RFC 3339 timestamp) so the result is
/// deterministic for callers and tests. A transport failure is recorded in
/// `diagnostics` rather than returned as an error, mirroring the JS contract.
///
/// # Errors
///
/// Returns an error string for an empty query or unsupported provider.
pub async fn search(
    query: &str,
    provider: &str,
    limit: usize,
    capture_mode: &str,
    captured_at: &str,
) -> Result<SearchResult, String> {
    if query.trim().is_empty() {
        return Err("Missing `query` parameter".to_string());
    }
    if !is_supported_provider(provider) {
        return Err(format!(
            "Unknown search provider \"{provider}\". Supported: {}",
            SEARCH_PROVIDERS.join(", ")
        ));
    }

    let source_url = build_search_url(provider, query, limit)?;
    let mut diagnostics = SearchDiagnostics {
        status: 0,
        blocked_by_cors: false,
        blocked_by_captcha: false,
        source_url: source_url.clone(),
        error: None,
    };
    let mut results = Vec::new();

    let accept = if provider == "wikipedia" {
        "application/json"
    } else {
        "text/html,application/xhtml+xml"
    };

    match reqwest::Client::builder().user_agent(USER_AGENT).build() {
        Ok(client) => {
            match client
                .get(&source_url)
                .header("Accept", accept)
                .header("Accept-Language", "en-US,en;q=0.9")
                .send()
                .await
            {
                Ok(response) => {
                    diagnostics.status = response.status().as_u16();
                    match response.text().await {
                        Ok(body) => {
                            let (parsed, blocked) = parse_search_results(provider, &body, limit);
                            results = parsed;
                            diagnostics.blocked_by_captcha = blocked;
                        }
                        Err(e) => diagnostics.error = Some(e.to_string()),
                    }
                }
                Err(e) => diagnostics.error = Some(e.to_string()),
            }
        }
        Err(e) => diagnostics.error = Some(e.to_string()),
    }

    Ok(SearchResult {
        query: query.to_string(),
        provider: provider.to_string(),
        capture_mode: capture_mode.to_string(),
        captured_at: captured_at.to_string(),
        results,
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIKI_JSON: &str = r#"{"pages":[
        {"id":1,"key":"Formal_methods","title":"Formal methods","excerpt":"the <span>study</span> of <b>formal</b>","description":"rigorous"},
        {"id":2,"key":"Formal_system","title":"Formal system","excerpt":"an abstract structure","description":""}
    ]}"#;

    const DDG_HTML: &str = r#"
        <div class="result__body">
          <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fa&rut=abc">First &amp; Best</a>
          <div class="result__snippet">Snippet about the <b>first</b> result</div>
        </div>
        <div class="result__body">
          <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.org%2Fb">Second result</a>
          <div class="result__snippet">Snippet two</div>
        </div>
    "#;

    const BING_HTML: &str = r#"
        <ol id="b_results">
          <li class="b_algo">
            <h2><a href="https://bing-result.example/1">Bing One</a></h2>
            <div class="b_caption"><p>Bing snippet one</p></div>
          </li>
        </ol>
    "#;

    #[test]
    fn builds_wikipedia_url() {
        assert_eq!(
            build_search_url("wikipedia", "formal", 5).unwrap(),
            "https://en.wikipedia.org/w/rest.php/v1/search/page?q=formal&limit=5"
        );
    }

    #[test]
    fn rejects_unknown_provider_url() {
        assert!(build_search_url("yahoo", "x", 5).is_err());
    }

    #[test]
    fn parses_wikipedia_json() {
        let (results, blocked) = parse_search_results("wikipedia", WIKI_JSON, 10);
        assert!(!blocked);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Formal methods");
        assert_eq!(
            results[0].url,
            "https://en.wikipedia.org/wiki/Formal_methods"
        );
        assert_eq!(results[0].snippet, "the study of formal");
        assert_eq!(
            results[1].url,
            "https://en.wikipedia.org/wiki/Formal_system"
        );
    }

    #[test]
    fn respects_limit() {
        let (results, _) = parse_search_results("wikipedia", WIKI_JSON, 1);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn parses_duckduckgo_and_decodes_redirects() {
        let (results, _) = parse_search_results("duckduckgo", DDG_HTML, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "First & Best");
        assert_eq!(results[0].url, "https://example.com/a");
        assert_eq!(results[0].snippet, "Snippet about the first result");
        assert_eq!(results[1].url, "https://example.org/b");
    }

    #[test]
    fn parses_bing() {
        let (results, _) = parse_search_results("bing", BING_HTML, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Bing One");
        assert_eq!(results[0].url, "https://bing-result.example/1");
        assert_eq!(results[0].snippet, "Bing snippet one");
    }

    #[test]
    fn empty_json_yields_no_results() {
        let (results, _) = parse_search_results("wikipedia", "not json", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn detects_captcha() {
        assert!(looks_like_captcha("Please solve the CAPTCHA"));
        assert!(looks_like_captcha(
            "Our systems have detected unusual traffic"
        ));
        assert!(!looks_like_captcha("normal results page"));
    }

    #[test]
    fn formats_markdown() {
        let result = SearchResult {
            query: "formal-ai".to_string(),
            provider: "wikipedia".to_string(),
            capture_mode: "fetch".to_string(),
            captured_at: "2026-05-30T00:00:00Z".to_string(),
            results: vec![SearchResultItem {
                rank: 1,
                title: "Formal methods".to_string(),
                url: "https://en.wikipedia.org/wiki/Formal_methods".to_string(),
                snippet: "study of formal".to_string(),
            }],
            diagnostics: SearchDiagnostics {
                status: 200,
                blocked_by_cors: false,
                blocked_by_captcha: false,
                source_url: "https://example.com".to_string(),
                error: None,
            },
        };
        let md = format_search_as_markdown(&result);
        assert!(md.contains("# Search results for \"formal-ai\""));
        assert!(md.contains("1. [Formal methods](https://en.wikipedia.org/wiki/Formal_methods)"));
        assert!(md.contains("study of formal"));
    }

    #[test]
    fn serializes_camel_case_contract() {
        let result = SearchResult {
            query: "q".to_string(),
            provider: "wikipedia".to_string(),
            capture_mode: "fetch".to_string(),
            captured_at: "t".to_string(),
            results: vec![],
            diagnostics: SearchDiagnostics {
                status: 200,
                blocked_by_cors: false,
                blocked_by_captcha: false,
                source_url: "u".to_string(),
                error: None,
            },
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"captureMode\""));
        assert!(json.contains("\"capturedAt\""));
        assert!(json.contains("\"blockedByCaptcha\""));
        assert!(json.contains("\"sourceUrl\""));
        assert!(!json.contains("\"error\""));
    }
}
