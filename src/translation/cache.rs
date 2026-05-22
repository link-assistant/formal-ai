//! File-based cache for raw HTTP responses from Wikipedia / Wikidata /
//! Wiktionary, keyed by the **semantic identity** of the resource.
//!
//! Earlier revisions of this module hashed the full URL and dumped 3 500+
//! tiny `.body` / `.url` files into a single `data/translation-cache/`
//! directory. That made the pull request unreviewable and locked the
//! cache to one consumer (translation). Issue #221 reshaped the cache so
//! that:
//!
//! - **Wikidata** responses land under `data/wikidata-cache/` keyed by
//!   the Q-id or Lexeme-id mentioned in the URL.
//! - **Wiktionary** wikitext lands under `data/wiktionary-cache/<lang>/`
//!   keyed by the page title.
//! - **SPARQL** queries land under `data/wikidata-cache/sparql/` keyed by
//!   a short hash, because there is no natural semantic name for a SPARQL
//!   string.
//! - **Everything else** still uses a URL-hash filename, but in a
//!   dedicated `data/http-cache/misc/` bucket. Unrecognised hosts should
//!   be rare in practice — the cache is intentionally narrow.
//!
//! This layout lets other formalization paths reuse the same Wikidata or
//! Wiktionary lookups without duplicating bytes. It also keeps each
//! directory small and human-readable (`grep -R apple
//! data/wiktionary-cache/` works).
//!
//! Cache contents are gitignored: they are populated lazily by integration
//! runs (`FORMAL_AI_LIVE_API=1`) and serve only as a local accelerator.
//! Unit tests rely on the offline dictionary at
//! `data/seed/translations.lino`, not on cached HTTP responses, so the
//! committed repository stays light.

use std::fs;
use std::path::{Path, PathBuf};

use super::http::{HttpClient, HttpError};

/// Default cache root, relative to the crate root.
///
/// Per-source subfolders (`wikidata-cache/`, `wiktionary-cache/<lang>/`, …)
/// live as siblings of this directory so callers from other formalization
/// paths can share the same bytes.
pub const DEFAULT_CACHE_DIR: &str = "data";

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
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("action=") {
            action = Some(percent_decode(value));
        } else if let Some(value) = pair.strip_prefix("srsearch=") {
            srsearch = Some(percent_decode(value));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct StubHttp {
        responses: Mutex<HashMap<String, String>>,
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
        // Cheap deterministic-ish randomness: use the system time in nanos.
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
        // Pre-populate the cache by hand.
        let url = "https://example.com/cached";
        let (body_path, meta_path) = cache.cache_paths(url);
        fs::create_dir_all(body_path.parent().unwrap()).unwrap();
        fs::write(&body_path, "cached body").unwrap();
        fs::write(&meta_path, url).unwrap();
        assert_eq!(cache.get(url).unwrap(), "cached body");
        // Stub had no entry; if it had been called we'd see a 404.
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
        // Second call should be a cache hit even if the stub disappears.
        let again = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
        assert_eq!(again.get(url).unwrap(), "fetched body");
    }

    #[test]
    fn cache_paths_use_semantic_subdirectories() {
        let dir = PathBuf::from("/tmp/whatever");
        let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
        let (body, meta) = cache.cache_paths("https://example.com/x");
        let body_str = body.to_string_lossy().to_string();
        let meta_str = meta.to_string_lossy().to_string();
        assert!(
            std::path::Path::new(&body_str)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("body")),
            "got: {body_str}"
        );
        assert!(
            std::path::Path::new(&meta_str)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("url")),
            "got: {meta_str}"
        );
        // Unknown host → falls into the misc bucket so it is still cached.
        assert!(
            body_str.contains("http-cache/misc/"),
            "expected http-cache/misc subdir, got: {body_str}"
        );
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
}
