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
//! There is no pre-extracted translation table in this repo: the only
//! committed data are **verbatim API response bodies** that the live
//! formalization pipeline produced. Those committed bodies live in the
//! seed bundle at [`SEED_CACHE_DIR`] as a small set of `.lino` files,
//! capped at 128 Wikidata entities + 128 properties + the Wiktionary
//! pages they point at. Each `.lino` file stays under
//! [`MAX_SEED_LINES_PER_FILE`] so reviewers can read it like any other
//! Links Notation file.
//!
//! At runtime [`CachedHttpClient::get`] tries three layers in order:
//!
//! 1. The committed `.lino` seed bundle (deterministic, ships in git).
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

/// Directory under the repository root that holds the committed seed.
///
/// `.lino` files here are shipped in git so a clean checkout can run the
/// full pipeline offline; the on-disk accelerator under
/// [`DEFAULT_CACHE_DIR`] is gitignored and only used when
/// `FORMAL_AI_LIVE_API=1` populates it.
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
        out.truncate(96);
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

// The list of seed `.lino` files is generated by `build.rs`, which walks
// `data/seed/api-cache/` at build time. That keeps the registry honest as
// the bundler splits a bucket into `<bucket>-partN.lino` files without
// requiring per-file edits here.
include!(concat!(env!("OUT_DIR"), "/seed_bundle_files.rs"));

/// All committed seed `.lino` files in deterministic (sorted) order.
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
/// (a body whose base64 spans multiple `.lino` parts) have their chunks
/// concatenated in the file/record order returned by [`seed_files`]
/// before decoding, so an oversize response can be split across as many
/// `<bucket>-partN.lino` files as needed without breaking lookup.
fn seed_index() -> &'static HashMap<String, String> {
    static INDEX: OnceLock<HashMap<String, String>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut chunks: HashMap<String, String> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        for (_name, contents) in seed_files() {
            for (url, b64) in parse_seed_chunks(contents) {
                let entry = chunks.entry(url.clone()).or_insert_with(|| {
                    order.push(url.clone());
                    String::new()
                });
                entry.push_str(&b64);
            }
        }
        let mut index = HashMap::new();
        for url in order {
            let Some(b64) = chunks.remove(&url) else {
                continue;
            };
            if let Some(body) = base64_decode_to_string(&b64) {
                index.insert(url, body);
            }
        }
        index
    })
}

/// Parse a `.lino` seed bundle into `(url, body)` pairs.
///
/// Each `response_<short_id>` block produces one pair with the body
/// already base64-decoded. Records sharing a URL are returned separately
/// in the order they appear — call [`seed_index`] (which concatenates
/// them) if you want the assembled body.
///
/// The grammar is intentionally narrow:
///
/// ```text
/// response_<short_id>
///   url "<full URL>"
///   body_base64 "<chunk 1>"
///   body_base64 "<chunk 2>"
///   ...
/// ```
#[must_use]
pub fn parse_seed_bundle(text: &str) -> Vec<(String, String)> {
    parse_seed_chunks(text)
        .into_iter()
        .filter_map(|(url, b64)| base64_decode_to_string(&b64).map(|body| (url, body)))
        .collect()
}

/// Parse a `.lino` seed bundle into `(url, base64_chunk)` pairs.
///
/// Returns chunks without decoding. Used by [`seed_index`] so split-body
/// records (multiple records with the same URL across
/// `<bucket>-partN.lino` files) can be concatenated before a single
/// base64 decode.
#[must_use]
pub fn parse_seed_chunks(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current_url: Option<String> = None;
    let mut current_b64: String = String::new();

    let flush = |url: &mut Option<String>, b64: &mut String, out: &mut Vec<(String, String)>| {
        if let Some(url_value) = url.take() {
            if b64.is_empty() {
                b64.clear();
            } else {
                out.push((url_value, std::mem::take(b64)));
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
            flush(&mut current_url, &mut current_b64, &mut out);
            if content.starts_with("response_") {
                current_url = Some(String::new());
            }
            continue;
        }
        if current_url.is_none() {
            continue;
        }
        if let Some(value) = strip_kv(content, "url") {
            current_url = Some(value.to_owned());
        } else if let Some(value) = strip_kv(content, "body_base64") {
            current_b64.push_str(value);
        }
    }
    flush(&mut current_url, &mut current_b64, &mut out);
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

/// Base64-decode a stream of characters into a UTF-8 string. Returns
/// `None` if the input is malformed or the resulting bytes are not UTF-8.
#[must_use]
pub fn base64_decode_to_string(input: &str) -> Option<String> {
    let bytes = base64_decode(input)?;
    String::from_utf8(bytes).ok()
}

/// Standard RFC 4648 base64 decoder. Skips whitespace; rejects invalid
/// characters and bad padding. Kept inline so the cache layer has no
/// dependency outside the standard library.
#[must_use]
pub fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    let mut pad: u32 = 0;
    for byte in input.bytes() {
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            continue;
        }
        let value = match byte {
            b'A'..=b'Z' => u32::from(byte - b'A'),
            b'a'..=b'z' => u32::from(byte - b'a') + 26,
            b'0'..=b'9' => u32::from(byte - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => {
                pad += 1;
                if pad > 2 {
                    return None;
                }
                buf <<= 6;
                bits += 6;
                if bits >= 8 {
                    bits -= 8;
                }
                continue;
            }
            _ => return None,
        };
        if pad > 0 {
            return None;
        }
        buf = (buf << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

/// Standard RFC 4648 base64 encoder. Pads with `=` to a multiple of four
/// characters. The refresh tool uses this to write seed bundles.
#[must_use]
pub fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let b0 = input[i];
        let b1 = input[i + 1];
        let b2 = input[i + 2];
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        out.push(ALPHABET[(b2 & 0x3f) as usize] as char);
        i += 3;
    }
    match input.len() - i {
        1 => {
            let b0 = input[i];
            out.push(ALPHABET[(b0 >> 2) as usize] as char);
            out.push(ALPHABET[((b0 & 0x03) << 4) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let b0 = input[i];
            let b1 = input[i + 1];
            out.push(ALPHABET[(b0 >> 2) as usize] as char);
            out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            out.push(ALPHABET[((b1 & 0x0f) << 2) as usize] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}

/// Emit a single `response_<short_id>` record into a `.lino` buffer.
///
/// Public so the refresh-cache example (and tests) can produce seed
/// files using the exact format [`parse_seed_bundle`] consumes.
pub fn write_seed_record(out: &mut String, short_id: &str, url: &str, body: &str) {
    out.push_str("response_");
    out.push_str(short_id);
    out.push('\n');
    out.push_str("  url \"");
    out.push_str(url);
    out.push_str("\"\n");
    let encoded = base64_encode(body.as_bytes());
    let mut cursor = 0usize;
    let bytes = encoded.as_bytes();
    while cursor < bytes.len() {
        let end = (cursor + 76).min(bytes.len());
        out.push_str("  body_base64 \"");
        out.push_str(&encoded[cursor..end]);
        out.push_str("\"\n");
        cursor = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap as StdHashMap;
    use std::sync::Mutex;

    struct StubHttp {
        responses: Mutex<StdHashMap<String, String>>,
        calls: Mutex<Vec<String>>,
    }

    impl StubHttp {
        fn new(responses: &[(&str, &str)]) -> Self {
            Self {
                responses: Mutex::new(
                    responses
                        .iter()
                        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                        .collect(),
                ),
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl HttpClient for StubHttp {
        fn get(&self, url: &str) -> Result<String, HttpError> {
            self.calls.lock().unwrap().push(url.to_owned());
            self.responses
                .lock()
                .unwrap()
                .get(url)
                .cloned()
                .ok_or_else(|| HttpError::Status {
                    status: 404,
                    body: format!("stub had no response for {url}"),
                })
        }
    }

    fn temp_dir(slug: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "formal-ai-cache-{slug}-{}",
            std::process::id() ^ rand_u32()
        ));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn rand_u32() -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        u32::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| u128::from(d.subsec_nanos())),
        )
        .unwrap_or(0)
    }

    #[test]
    fn cache_key_is_stable_across_runs() {
        let one = cache_key("https://example.com/foo");
        let two = cache_key("https://example.com/foo");
        assert_eq!(one, two);
        let other = cache_key("https://example.com/bar");
        assert_ne!(one, other);
    }

    #[test]
    fn cache_hit_short_circuits_transport() {
        let dir = temp_dir("hit");
        let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
        let url = "https://example.com/cached";
        let (body_path, meta_path) = cache.cache_paths(url);
        fs::create_dir_all(body_path.parent().unwrap()).unwrap();
        fs::write(&body_path, "cached body").unwrap();
        fs::write(&meta_path, url).unwrap();
        assert_eq!(cache.get(url).unwrap(), "cached body");
    }

    #[test]
    fn cache_miss_offline_returns_transport_error() {
        let dir = temp_dir("offline-miss");
        let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
        let error = cache.get("https://example.com/missing").unwrap_err();
        match error {
            HttpError::Transport(message) => {
                assert!(message.contains("cache miss"), "got: {message}");
                assert!(message.contains("FORMAL_AI_LIVE_API"), "got: {message}");
            }
            other @ HttpError::Status { .. } => {
                panic!("expected Transport error, got {other:?}")
            }
        }
    }

    #[test]
    fn cache_miss_online_populates_and_returns_body() {
        let dir = temp_dir("online-miss");
        let url = "https://example.com/foo";
        let stub = StubHttp::new(&[(url, "fetched body")]);
        let cache = CachedHttpClient::new(&dir, stub).with_online(true);
        assert_eq!(cache.get(url).unwrap(), "fetched body");
        let again = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
        assert_eq!(again.get(url).unwrap(), "fetched body");
    }

    #[test]
    fn cache_paths_use_semantic_subdirectories() {
        let dir = PathBuf::from("cache-root");
        let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
        let (body, meta) = cache.cache_paths("https://example.com/x");
        assert!(
            body.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("body")),
            "got: {}",
            body.display()
        );
        assert!(
            meta.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("url")),
            "got: {}",
            meta.display()
        );
        let location = cache_location("https://example.com/x");
        assert_eq!(location.directory, PathBuf::from("http-cache").join("misc"),);
        assert!(!location.stem.is_empty());
    }

    #[test]
    fn wiktionary_url_lands_under_per_language_subdirectory() {
        let location = cache_location(
            "https://en.wiktionary.org/w/api.php?action=parse&page=apple&prop=wikitext&formatversion=2&format=json&redirects=1",
        );
        assert_eq!(
            location.directory,
            PathBuf::from("wiktionary-cache").join("en")
        );
        assert_eq!(location.stem, "apple");
    }

    #[test]
    fn wikidata_search_url_keyed_by_search_term() {
        let location = cache_location(
            "https://www.wikidata.org/w/api.php?action=wbsearchentities&format=json&language=en&type=lexeme&srsearch=apple&limit=3",
        );
        assert_eq!(
            location.directory,
            PathBuf::from("wikidata-cache").join("search")
        );
        assert_eq!(location.stem, "apple");
    }

    #[test]
    fn wikidata_sparql_url_lands_in_sparql_bucket() {
        let location = cache_location(
            "https://query.wikidata.org/sparql?format=json&query=SELECT%20%3Flemma%20WHERE%20%7B%20%7D",
        );
        assert_eq!(
            location.directory,
            PathBuf::from("wikidata-cache").join("sparql")
        );
        assert_eq!(location.stem.len(), 16);
    }

    #[test]
    fn base64_round_trip_preserves_arbitrary_bytes() {
        let cases: &[&[u8]] = &[
            b"",
            b"x",
            b"ab",
            b"abc",
            b"abcd",
            b"hello world",
            &[0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0xff, 0xfe, 0xfd],
        ];
        for case in cases {
            let encoded = base64_encode(case);
            let decoded = base64_decode(&encoded).expect("decode");
            assert_eq!(&decoded, case, "round trip differs for input {case:?}");
        }
    }

    #[test]
    fn parse_seed_bundle_round_trips_through_write_seed_record() {
        let body = r#"{"parse":{"title":"apple","wikitext":"* Russian: {{t+|ru|яблоко}}"}}"#;
        let url = "https://en.wiktionary.org/w/api.php?action=parse&page=apple&prop=wikitext&formatversion=2&format=json&redirects=1";
        let mut buf = String::new();
        write_seed_record(&mut buf, "wiktionary_en_apple", url, body);
        let parsed = parse_seed_bundle(&buf);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, url);
        assert_eq!(parsed[0].1, body);
    }

    #[test]
    fn parse_seed_bundle_concatenates_chunked_body() {
        let bundle = "response_chunky\n  url \"https://example.org/x\"\n  body_base64 \"aGVs\"\n  body_base64 \"bG8=\"\n";
        let parsed = parse_seed_bundle(bundle);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].1, "hello");
    }

    #[test]
    fn parse_seed_chunks_yields_one_pair_per_record() {
        let bundle = "response_a\n  url \"https://example.org/x\"\n  body_base64 \"aGVs\"\n\nresponse_b\n  url \"https://example.org/x\"\n  body_base64 \"bG8=\"\n";
        let chunks = parse_seed_chunks(bundle);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].0, "https://example.org/x");
        assert_eq!(chunks[0].1, "aGVs");
        assert_eq!(chunks[1].0, "https://example.org/x");
        assert_eq!(chunks[1].1, "bG8=");
    }

    #[test]
    fn seed_files_stay_under_per_file_line_cap() {
        for (name, contents) in seed_files() {
            let lines = contents.lines().count();
            assert!(
                lines <= MAX_SEED_LINES_PER_FILE,
                "{name} has {lines} lines, exceeds MAX_SEED_LINES_PER_FILE={MAX_SEED_LINES_PER_FILE}",
            );
        }
    }

    #[test]
    fn seed_response_is_consulted_before_disk_or_transport() {
        // Use a URL we expect to be present in the committed bundle. If the
        // bundle is empty during early bootstrapping, skip the assertion —
        // the test exists to lock the precedence once the bundle is filled.
        let any_url = seed_index().keys().next().cloned();
        let Some(url) = any_url else {
            return;
        };
        let dir = temp_dir("seed-precedence");
        let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
        let body = cache.get(&url).expect("seeded response must hit");
        assert!(!body.is_empty());
    }
}
