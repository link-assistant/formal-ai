//! File-based cache for raw HTTP responses from Wikipedia / Wikidata /
//! Wiktionary.
//!
//! The cache stores only **source data**, never the logic that computes a
//! translation. Each entry is the verbatim response body for a URL keyed
//! by a deterministic 64-bit hash of the URL, and the URL itself lives in
//! a sibling `.url` file for auditability.
//!
//! Why a file cache instead of `OnceLock<HashMap>`:
//!
//! 1. Integration tests run with `FORMAL_AI_LIVE_API=1` and write cache
//!    entries to disk. The next time the unit suite runs (offline), those
//!    entries make the same code paths exercise real wikitext — no
//!    hardcoded translation pairs, just real cached responses.
//! 2. The committed cache becomes part of the repo's pre-seed data
//!    (`data/translation-cache/`) so CI never hits the live network.
//! 3. The cache is content-addressable and idempotent — running the same
//!    test twice yields the same files on disk.

use std::fs;
use std::path::{Path, PathBuf};

use super::http::{HttpClient, HttpError};

/// Default cache directory, relative to the crate root.
pub const DEFAULT_CACHE_DIR: &str = "data/translation-cache";

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
        let key = cache_key(url);
        let mut body = self.cache_dir.clone();
        body.push(format!("{key}.body"));
        let mut meta = self.cache_dir.clone();
        meta.push(format!("{key}.url"));
        (body, meta)
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
        if let Err(error) = fs::create_dir_all(&self.cache_dir) {
            return Err(HttpError::Transport(format!(
                "failed to create cache directory {}: {error}",
                self.cache_dir.display(),
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
        fs::create_dir_all(&dir).unwrap();
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
    fn cache_paths_use_fnv_keyed_filenames() {
        let dir = PathBuf::from("/tmp/whatever");
        let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
        let (body, meta) = cache.cache_paths("https://example.com/x");
        assert!(body.to_string_lossy().ends_with(".body"));
        assert!(meta.to_string_lossy().ends_with(".url"));
    }
}
