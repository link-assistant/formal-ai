//! File-based cache for raw HTTP responses from Wikipedia / Wikidata /
//! Wiktionary, keyed by the **semantic identity** of the resource.
//!
//! Issue #221 reshaped the cache so that:
//!
//! - **Wikidata** responses land under `wikidata-cache/` keyed by the
//!   Q-id, P-id or Lexeme-id mentioned in the URL.
//! - **Wiktionary** wikitext lands under `wiktionary-cache/<lang>/`
//!   keyed by the page title.
//! - **SPARQL** queries land under `wikidata-cache/sparql/` keyed by a
//!   short hash, because there is no natural semantic name for a SPARQL
//!   string.
//! - **Everything else** still uses a URL-hash filename, but in a
//!   dedicated `http-cache/misc/` bucket. Unrecognised hosts should be
//!   rare in practice — the cache is intentionally narrow.
//!
//! There is no pre-extracted translation table in this repo. Issue #398
//! stopped committing the legacy raw HTTP response bundle under
//! [`SEED_CACHE_DIR`]; the code below can still replay such a bundle when
//! one is generated locally or exists in an older checkout, but a clean
//! checkout normally has an empty legacy registry. Current reviewed source
//! snapshots for seed ids live as lossless Links Notation JSON under
//! `data/cache/wikidata/`.
//!
//! At runtime [`CachedHttpClient::get`] tries three layers in order:
//!
//! 1. A legacy `.lino` seed bundle, when present.
//! 2. The gitignored on-disk accelerator under
//!    `<DEFAULT_CACHE_DIR>/{wikidata,wiktionary,http}-cache/...` —
//!    populated by `FORMAL_AI_LIVE_API=1` runs on a developer machine.
//! 3. The live transport (only when `online == true`).
//!
//! This three-layer design keeps the repository light, lets the same
//! Wikidata or Wiktionary fetch feed translation, fact lookup and any
//! other formalization path without duplicating bytes, and keeps unit
//! tests offline by default.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::http::{HttpClient, HttpError};

/// Default cache root, relative to the crate root.
///
/// Per-source subfolders (`wikidata-cache/`, `wiktionary-cache/<lang>/`, …)
/// live as siblings of this directory so callers from other formalization
/// paths can share the same bytes.
pub const DEFAULT_CACHE_DIR: &str = "data";

/// Legacy directory under the repository root for raw HTTP response seeds.
///
/// Issue #398 removed the committed files from this path. The reader remains
/// for locally generated bundles and older checkouts; current source-id JSON
/// snapshots live under `data/cache/wikidata/`.
pub const SEED_CACHE_DIR: &str = "data/seed/api-cache";

/// Hard cap on the number of lines per seeded `.lino` cache file. Larger
/// files become unreadable; the refresh tool splits responses into
/// `<bucket>-partN.lino` files when needed.
pub const MAX_SEED_LINES_PER_FILE: usize = 1500;

/// Hard cap on the number of distinct entities (or properties) per bucket.
///
/// We never ship more than this many records in the seeded cache. This
/// keeps the repository small enough to review and stays honest about the
/// "lightweight by default" constraint from issue #221's reviewer
/// feedback.
pub const MAX_SEED_RECORDS_PER_BUCKET: usize = 128;

/// HTTP client that consults a file cache before delegating to a real
/// transport. Cache hits always short-circuit the underlying transport;
/// cache misses populate the cache after a successful fetch.
pub struct CachedHttpClient<T: HttpClient> {
    cache_dir: PathBuf,
    transport: T,
    /// When `true`, cache misses fall through to the transport. When
    /// `false`, cache misses return [`HttpError::Transport`] without any
    /// network access. Defaults to `false` so unit tests are offline-only.
    online: bool,
}

impl<T: HttpClient> CachedHttpClient<T> {
    /// Construct a cached client. `online` defaults to whether the
    /// `FORMAL_AI_LIVE_API` environment variable is set to a truthy value.
    pub fn new(cache_dir: impl Into<PathBuf>, transport: T) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            transport,
            online: live_api_enabled(),
        }
    }

    /// Override the online flag explicitly. Useful for tests that want to
    /// force offline mode regardless of the environment.
    #[must_use]
    #[allow(dead_code)]
    pub const fn with_online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    /// Return the directory where cache entries are stored.
    #[must_use]
    #[allow(dead_code)]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Return whether the cache will fall through to the transport on
    /// misses.
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_online(&self) -> bool {
        self.online
    }

    fn cache_paths(&self, url: &str) -> (PathBuf, PathBuf) {
        let location = cache_location(url);
        let mut body = self.cache_dir.clone();
        body.push(&location.directory);
        body.push(format!("{}.body", location.stem));
        let mut meta = self.cache_dir.clone();
        meta.push(&location.directory);
        meta.push(format!("{}.url", location.stem));
        (body, meta)
    }
}

/// Where a URL should land inside the cache root.
///
/// `directory` is a relative subpath (e.g. `wiktionary-cache/en`) and
/// `stem` is the filename without extension (e.g. `apple`). The cache
/// writes `<root>/<directory>/<stem>.body` and `<root>/<directory>/<stem>.url`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLocation {
    pub directory: PathBuf,
    pub stem: String,
}

/// Decide the cache location for a URL.
///
/// The mapping is best-effort: well-known Wikidata / Wiktionary URLs land
/// under human-readable semantic folders, and anything else falls back to
/// a hashed bucket so we never refuse to cache a response.
#[must_use]
pub fn cache_location(url: &str) -> CacheLocation {
    if let Some(location) = classify_wiktionary(url) {
        return location;
    }
    if let Some(location) = classify_wikidata(url) {
        return location;
    }
    CacheLocation {
        directory: PathBuf::from("http-cache").join("misc"),
        stem: cache_key(url),
    }
}

fn classify_wiktionary(url: &str) -> Option<CacheLocation> {
    let host_start = url.find("://")? + 3;
    let after_scheme = &url[host_start..];
    let dot = after_scheme.find('.')?;
    let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    let host_rest = &after_scheme[dot..host_end];
    if !host_rest.starts_with(".wiktionary.org") {
        return None;
    }
    let lang = sanitize_segment(&after_scheme[..dot]);
    let page = wiktionary_page_from_url(url).unwrap_or_else(|| cache_key(url));
    Some(CacheLocation {
        directory: PathBuf::from("wiktionary-cache").join(lang),
        stem: page,
    })
}

fn wiktionary_page_from_url(url: &str) -> Option<String> {
    // Pages are fetched via the `parse` API:
    //   /w/api.php?action=parse&page=<title>&...
    let query_start = url.find('?')?;
    let query = &url[query_start + 1..];
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("page=") {
            let decoded = percent_decode(value);
            if !decoded.is_empty() {
                return Some(sanitize_segment(&decoded));
            }
        }
    }
    None
}

fn classify_wikidata(url: &str) -> Option<CacheLocation> {
    if !url.contains("wikidata.org") {
        return None;
    }
    let query_start = url.find('?')?;
    let query = &url[query_start + 1..];
    let mut action: Option<String> = None;
    let mut srsearch: Option<String> = None;
    let mut ids: Option<String> = None;
    let mut sparql: Option<String> = None;
    let mut titles: Option<String> = None;
    let mut search_term: Option<String> = None;
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("action=") {
            action = Some(percent_decode(value));
        } else if let Some(value) = pair.strip_prefix("srsearch=") {
            srsearch = Some(percent_decode(value));
        } else if let Some(value) = pair.strip_prefix("search=") {
            search_term = Some(percent_decode(value));
        } else if let Some(value) = pair.strip_prefix("ids=") {
            ids = Some(percent_decode(value));
        } else if let Some(value) = pair.strip_prefix("query=") {
            sparql = Some(percent_decode(value));
        } else if let Some(value) = pair.strip_prefix("titles=") {
            titles = Some(percent_decode(value));
        }
    }
    if sparql.is_some() || url.contains("/sparql") || url.contains("query.wikidata.org") {
        return Some(CacheLocation {
            directory: PathBuf::from("wikidata-cache").join("sparql"),
            stem: cache_key(url),
        });
    }
    let stem = match action.as_deref() {
        Some("wbsearchentities") => srsearch
            .as_deref()
            .or(search_term.as_deref())
            .map_or_else(|| cache_key(url), sanitize_segment),
        Some("wbgetentities" | "query") => ids
            .as_deref()
            .or(titles.as_deref())
            .map_or_else(|| cache_key(url), sanitize_segment),
        _ => cache_key(url),
    };
    let sub = match action.as_deref() {
        Some("wbsearchentities") => "search",
        Some("wbgetentities") => "entities",
        Some("query") => "query",
        _ => "misc",
    };
    Some(CacheLocation {
        directory: PathBuf::from("wikidata-cache").join(sub),
        stem,
    })
}

fn sanitize_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else if ch == ' ' || ch == '+' {
            out.push('_');
        } else {
            out.push('-');
        }
    }
    if out.len() > 96 {
        // Avoid blowing past common filesystem name limits when a SPARQL
        // search returns a freakishly long title.
        let mut boundary = 96;
        while !out.is_char_boundary(boundary) {
            boundary -= 1;
        }
        out.truncate(boundary);
        out.push('~');
        out.push_str(&cache_key(value)[..8]);
    }
    if out.is_empty() {
        cache_key(value)
    } else {
        out
    }
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == b'%' && i + 2 < bytes.len() {
            let hi = hex_nibble(bytes[i + 1]);
            let lo = hex_nibble(bytes[i + 2]);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        if byte == b'+' {
            out.push(b' ');
        } else {
            out.push(byte);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Compute a deterministic, filesystem-safe key for the cache.
///
/// Uses FNV-1a 64-bit which is stable across platforms and Rust versions.
/// We do not need cryptographic strength — collisions are tolerable because
/// the sibling `.url` file records the exact URL for audit.
#[must_use]
pub fn cache_key(url: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in url.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    format!("{hash:016x}")
}

/// Read the cache truthiness flag from the environment. Accepts `1`,
/// `true`, `yes`, `on` (case-insensitive). Anything else is treated as
/// disabled.
fn live_api_enabled() -> bool {
    std::env::var("FORMAL_AI_LIVE_API").is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

impl<T: HttpClient> HttpClient for CachedHttpClient<T> {
    fn get(&self, url: &str) -> Result<String, HttpError> {
        // Layer 1: committed seed bundle (deterministic, ships in git).
        if let Some(body) = seed_response(url) {
            return Ok(body);
        }

        // Layer 2: gitignored on-disk accelerator under `cache_dir`.
        let (body_path, meta_path) = self.cache_paths(url);
        if let Ok(body) = fs::read_to_string(&body_path) {
            return Ok(body);
        }
        if !self.online {
            return Err(HttpError::Transport(format!(
                "translation cache miss for {url} and offline mode is active; \
                 set FORMAL_AI_LIVE_API=1 to fetch and populate the cache",
            )));
        }

        // Layer 3: live transport.
        let body = self.transport.get(url)?;
        let parent = body_path.parent().unwrap_or(&self.cache_dir);
        if let Err(error) = fs::create_dir_all(parent) {
            return Err(HttpError::Transport(format!(
                "failed to create cache directory {}: {error}",
                parent.display(),
            )));
        }
        if let Err(error) = fs::write(&body_path, &body) {
            return Err(HttpError::Transport(format!(
                "failed to write cache body {}: {error}",
                body_path.display(),
            )));
        }
        if let Err(error) = fs::write(&meta_path, url) {
            return Err(HttpError::Transport(format!(
                "failed to write cache url marker {}: {error}",
                meta_path.display(),
            )));
        }
        Ok(body)
    }
}

// ---------------------------------------------------------------------------
// Seeded raw API response bundle
// ---------------------------------------------------------------------------

// The list of legacy seed `.lino` files is generated by `build.rs`, which
// walks `data/seed/api-cache/` at build time. When the directory is absent
// the generated registry is empty.
include!(concat!(env!("OUT_DIR"), "/seed_bundle_files.rs"));

/// All legacy seed `.lino` files in deterministic (sorted) order.
#[must_use]
pub fn seed_files() -> Vec<(&'static str, &'static str)> {
    SEED_BUNDLE_FILES.to_vec()
}

/// Look up a seeded response body by URL. Returns `None` when the URL
/// is not part of the committed bundle.
#[must_use]
pub fn seed_response(url: &str) -> Option<String> {
    seed_index().get(url).cloned()
}

/// Process-wide lazily-parsed URL → body map. Loaded once on first
/// access from the embedded `.lino` seed files. Records sharing a URL
/// (a body whose chunks span multiple `.lino` parts) have their chunks
/// concatenated in the file/record order returned by [`seed_files`], so
/// an oversize response can be split across as many `<bucket>-partN.lino`
/// files as needed without breaking lookup.
fn seed_index() -> &'static HashMap<String, String> {
    static INDEX: OnceLock<HashMap<String, String>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut chunks: HashMap<String, String> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        for (_name, contents) in seed_files() {
            for (url, body) in parse_seed_chunks(contents) {
                let entry = chunks.entry(url.clone()).or_insert_with(|| {
                    order.push(url.clone());
                    String::new()
                });
                entry.push_str(&body);
            }
        }
        let mut index = HashMap::new();
        for url in order {
            if let Some(body) = chunks.remove(&url) {
                index.insert(url, body);
            }
        }
        index
    })
}

/// Parse a `.lino` seed bundle into `(url, body)` pairs.
///
/// Each `response_<short_id>` block produces one pair. Records sharing a
/// URL are returned separately in the order they appear — call
/// [`seed_index`] (which concatenates them) if you want the assembled body.
///
/// The grammar is intentionally narrow and stays human-readable so a
/// reviewer can inspect the raw API JSON without decoding tooling:
///
/// ```text
/// response_<short_id>
///   url "<full URL>"
///   body "<chunk 1>"
///   body "<chunk 2>"
///   ...
/// ```
///
/// `body` chunks store the raw response text using Links Notation's
/// doubled-quote escape: one literal `"` is written as two consecutive
/// `"` characters; no other characters are escaped. Concatenating the
/// unescaped chunks reproduces the original API response byte-for-byte.
#[must_use]
pub fn parse_seed_bundle(text: &str) -> Vec<(String, String)> {
    parse_seed_chunks(text)
}

/// Parse a `.lino` seed bundle into `(url, body_chunk)` pairs.
///
/// Used by [`seed_index`] so split-body records (multiple records with
/// the same URL across `<bucket>-partN.lino` files) can be concatenated
/// into a single body.
#[must_use]
pub fn parse_seed_chunks(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current_url: Option<String> = None;
    let mut current_body: String = String::new();

    let flush = |url: &mut Option<String>, body: &mut String, out: &mut Vec<(String, String)>| {
        if let Some(url_value) = url.take() {
            if body.is_empty() {
                body.clear();
            } else {
                out.push((url_value, std::mem::take(body)));
            }
        }
    };

    for raw_line in text.lines() {
        let trimmed = raw_line.trim_end_matches(['\r', '\n']);
        if trimmed.trim().is_empty() {
            continue;
        }
        let indent = trimmed.bytes().take_while(|b| *b == b' ').count();
        let content = &trimmed[indent..];
        if indent == 0 {
            flush(&mut current_url, &mut current_body, &mut out);
            if content.starts_with("response_") {
                current_url = Some(String::new());
            }
            continue;
        }
        if current_url.is_none() {
            continue;
        }
        if let Some(value) = strip_kv(content, "url") {
            current_url = Some(unescape_lino_string(value));
        } else if let Some(value) = strip_kv(content, "body") {
            current_body.push_str(&unescape_lino_string(value));
        }
    }
    flush(&mut current_url, &mut current_body, &mut out);
    out
}

/// Extract a `"value"` after `key ` from a line. Returns `None` when the
/// line does not match `key "value"`.
fn strip_kv<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let rest = content.strip_prefix(key)?;
    let rest = rest.strip_prefix(' ')?;
    let rest = rest.strip_prefix('"')?;
    rest.strip_suffix('"')
}

/// Maximum number of characters per `body "..."` chunk when writing
/// seed records. Wide enough to keep large API responses compact, narrow
/// enough that each line stays inside a standard editor viewport.
pub const SEED_BODY_CHUNK_CHARS: usize = 200;

/// Escape a string for embedding as a double-quoted Links Notation value.
///
/// Links Notation uses **doubled-quote escapes**: every literal `"` in
/// the value is doubled to `""`. Backslashes and all other bytes — including
/// CJK, Cyrillic, Devanagari, etc. — pass through verbatim so reviewers see
/// the raw API JSON, not a re-encoded form.
#[must_use]
pub fn escape_lino_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        if ch == '"' {
            out.push('"');
            out.push('"');
        } else {
            out.push(ch);
        }
    }
    out
}

/// Reverse of [`escape_lino_string`].
///
/// Collapses every `""` pair into a single `"`. A lone trailing `"` (which
/// should never appear in well-formed seed records) is preserved verbatim.
#[must_use]
pub fn unescape_lino_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' && chars.peek() == Some(&'"') {
            out.push('"');
            chars.next();
        } else {
            out.push(ch);
        }
    }
    out
}

/// Split a string into chunks of at most `chars` characters at character
/// boundaries. Used to keep individual `body "..."` lines bounded so
/// `.lino` files stay reviewable.
///
/// A chunk never **starts** with a `"` character, because Links Notation
/// detects the quote-delimiter count by counting consecutive leading `"`
/// chars: if a chunk began with `"`, wrapping it as `"<chunk>"` would
/// produce `"""...` and the parser would treat it as a 3-quote delimited
/// string, mis-parsing the value. We therefore extend a chunk past any
/// trailing run of `"` chars so the next chunk starts on a non-`"` byte.
#[must_use]
pub fn split_body_into_chunks(body: &str, chars: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if body.is_empty() {
        return out;
    }
    let chars_vec: Vec<char> = body.chars().collect();
    let total = chars_vec.len();
    let mut start = 0usize;
    while start < total {
        let mut end = (start + chars).min(total);
        while end < total && chars_vec[end] == '"' {
            end += 1;
        }
        out.push(chars_vec[start..end].iter().collect());
        start = end;
    }
    out
}

/// Emit a single `response_<short_id>` record into a `.lino` buffer.
///
/// Public so the refresh-cache example (and tests) can produce seed
/// files using the exact format [`parse_seed_bundle`] consumes. Bodies
/// are split into [`SEED_BODY_CHUNK_CHARS`]-char chunks and escaped with
/// [`escape_lino_string`] so the raw API JSON remains human-readable.
pub fn write_seed_record(out: &mut String, short_id: &str, url: &str, body: &str) {
    out.push_str("response_");
    out.push_str(short_id);
    out.push('\n');
    out.push_str("  url \"");
    out.push_str(&escape_lino_string(url));
    out.push_str("\"\n");
    for chunk in split_body_into_chunks(body, SEED_BODY_CHUNK_CHARS) {
        out.push_str("  body \"");
        out.push_str(&escape_lino_string(&chunk));
        out.push_str("\"\n");
    }
}

#[path = "../source_tests/translation/cache/tests.rs"]
mod tests;
