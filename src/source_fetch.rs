//! Cached, content-addressed retrieval for external evidence.
//!
//! Live access is opt-in. Every successful result owns the retrieved bytes and
//! provenance calculated from those bytes; callers never construct `source:`
//! or `cache_hit:` events from a request alone.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::event_log::EventLog;
use crate::translation::cache::cache_key;

const CACHE_FORMAT_VERSION: &str = "source-capture-v2";
const LEGACY_CACHE_FORMAT_VERSION: &str = "source-capture-v1";
const DEFAULT_TTL_SECONDS: u64 = 60 * 60 * 24 * 60;
static TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);
const USER_AGENT: &str = concat!(
    "formal-ai/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/link-assistant/formal-ai; source retrieval)"
);

/// A transport capable of returning the exact response bytes.
pub trait SourceTransport: Send + Sync {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>;
}

/// curl-backed transport used by opt-in live retrieval.
#[derive(Debug, Clone, Copy, Default)]
pub struct CurlSourceTransport;

impl SourceTransport for CurlSourceTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let output = Command::new("curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                // Stack Exchange always gzips its API responses; without this
                // the transport would hand callers compressed bytes that no
                // extractor can read (issue #991).
                "--compressed",
                "--max-time",
                "30",
                "--user-agent",
                USER_AGENT,
                url,
            ])
            .output()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    FetchError::Transport(String::from("curl is required for live retrieval"))
                } else {
                    FetchError::Transport(error.to_string())
                }
            })?;
        if !output.status.success() {
            return Err(FetchError::Transport(
                [
                    "source_transport_curl_exit",
                    &output.status.to_string(),
                    String::from_utf8_lossy(&output.stderr).trim(),
                ]
                .join(":"),
            ));
        }
        Ok(output.stdout)
    }
}

/// Why a source could not be captured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    InvalidUrl(String),
    OfflineCacheMiss(String),
    Transport(String),
    Cache(String),
}

impl Display for FetchError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(url) => write!(formatter, "unsupported source URL: {url}"),
            Self::OfflineCacheMiss(url) => write!(formatter, "no cached capture for {url}"),
            Self::Transport(message) => write!(formatter, "source transport error: {message}"),
            Self::Cache(message) => write!(formatter, "source cache error: {message}"),
        }
    }
}

impl Error for FetchError {}

/// Exact fetched bytes plus provenance derived from the successful retrieval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCapture {
    source_url: String,
    fetched_at: String,
    sha256: String,
    cached: bool,
    bytes: Vec<u8>,
}

impl SourceCapture {
    #[must_use]
    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    #[must_use]
    pub fn fetched_at(&self) -> &str {
        &self.fetched_at
    }

    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    #[must_use]
    pub const fn cached(&self) -> bool {
        self.cached
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn trace_payload(&self) -> String {
        [
            self.source_url.as_str(),
            " fetched_at=",
            self.fetched_at.as_str(),
            " sha256=",
            self.sha256.as_str(),
            " cached=",
            if self.cached { "true" } else { "false" },
        ]
        .concat()
    }

    /// Append evidence events that are backed by this capture.
    pub fn record(&self, log: &mut EventLog) {
        log.append("source:http", self.trace_payload());
        if self.cached {
            log.append("cache_hit", self.source_url.clone());
        }
    }
}

/// Cache-backed source client. Network fallback is disabled unless `online` is
/// explicitly enabled.
pub struct CachedSourceClient<T: SourceTransport> {
    cache_dir: PathBuf,
    transport: T,
    online: bool,
    now: fn() -> u64,
    ttl_seconds: u64,
}

impl<T: SourceTransport> CachedSourceClient<T> {
    #[must_use]
    pub fn new(cache_dir: impl Into<PathBuf>, transport: T) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            transport,
            online: false,
            now: unix_now,
            ttl_seconds: DEFAULT_TTL_SECONDS,
        }
    }

    #[must_use]
    pub const fn with_online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    #[must_use]
    pub const fn with_clock(mut self, now: fn() -> u64) -> Self {
        self.now = now;
        self
    }

    #[must_use]
    pub const fn with_ttl_seconds(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = ttl_seconds;
        self
    }

    pub fn fetch(&self, url: &str) -> Result<SourceCapture, FetchError> {
        validate_url(url)?;
        let (cache_root, metadata_path) = self.paths(url);
        match read_capture(url, &cache_root, &metadata_path) {
            Ok(Some(mut capture)) => {
                let fetched_at = capture.fetched_at.parse::<u64>().unwrap_or(0);
                let age = (self.now)().saturating_sub(fetched_at);
                if !self.online || age <= self.ttl_seconds {
                    capture.cached = true;
                    return Ok(capture);
                }
            }
            Ok(None) => {}
            Err(error) => return Err(error),
        }
        if !self.online {
            return Err(FetchError::OfflineCacheMiss(url.to_owned()));
        }

        let bytes = self.transport.get(url)?;
        let capture = SourceCapture {
            source_url: url.to_owned(),
            fetched_at: format_timestamp((self.now)()),
            sha256: sha256_hex(&bytes),
            cached: false,
            bytes,
        };
        write_capture(&capture, &cache_root, &metadata_path)?;
        Ok(capture)
    }

    fn paths(&self, url: &str) -> (PathBuf, PathBuf) {
        let root = self.cache_dir.join("source-cache");
        let key = cache_key(url);
        let metadata = root.join(format!("{key}.meta"));
        (root, metadata)
    }
}

fn validate_url(url: &str) -> Result<(), FetchError> {
    let authority = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    if authority.is_some_and(|rest| {
        !rest.is_empty()
            && !rest.starts_with('/')
            && !rest.chars().any(char::is_control)
            && !rest.chars().any(char::is_whitespace)
    }) {
        Ok(())
    } else {
        Err(FetchError::InvalidUrl(url.to_owned()))
    }
}

fn read_capture(
    expected_url: &str,
    cache_root: &Path,
    metadata_path: &Path,
) -> Result<Option<SourceCapture>, FetchError> {
    let metadata = match fs::read_to_string(metadata_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(cache_error(metadata_path, &error)),
    };
    let mut lines = metadata.lines();
    let version = lines.next();
    if !matches!(
        version,
        Some(CACHE_FORMAT_VERSION | LEGACY_CACHE_FORMAT_VERSION)
    ) {
        return Err(cache_diagnostic(
            "source_cache_unsupported_metadata",
            metadata_path,
        ));
    }
    let source_url = field(lines.next(), "url", metadata_path)?;
    let fetched_at = field(lines.next(), "fetched_at", metadata_path)?;
    let recorded_sha256 = field(lines.next(), "sha256", metadata_path)?;
    if source_url != expected_url {
        return Err(cache_diagnostic("source_cache_url_mismatch", metadata_path));
    }
    if !is_sha256_hex(&recorded_sha256) {
        return Err(cache_diagnostic(
            "source_cache_invalid_content_hash",
            metadata_path,
        ));
    }
    let body_path = if version == Some(CACHE_FORMAT_VERSION) {
        content_path(cache_root, &recorded_sha256)
    } else {
        metadata_path.with_extension("body")
    };
    let bytes = fs::read(&body_path).map_err(|error| cache_error(&body_path, &error))?;
    let actual_sha256 = sha256_hex(&bytes);
    if actual_sha256 != recorded_sha256 {
        return Err(cache_diagnostic(
            "source_cache_content_hash_mismatch",
            &body_path,
        ));
    }
    Ok(Some(SourceCapture {
        source_url,
        fetched_at,
        sha256: actual_sha256,
        cached: true,
        bytes,
    }))
}

fn field(line: Option<&str>, name: &str, path: &Path) -> Result<String, FetchError> {
    line.and_then(|line| line.strip_prefix(&format!("{name}=")))
        .map(str::to_owned)
        .ok_or_else(|| {
            FetchError::Cache(
                [
                    "source_cache_missing_field",
                    name,
                    &path.display().to_string(),
                ]
                .join(":"),
            )
        })
}

fn write_capture(
    capture: &SourceCapture,
    cache_root: &Path,
    metadata_path: &Path,
) -> Result<(), FetchError> {
    let body_path = content_path(cache_root, &capture.sha256);
    let object_root = body_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(object_root).map_err(|error| cache_error(object_root, &error))?;
    let body_tmp = unique_temp_path(&body_path);
    let metadata_tmp = unique_temp_path(metadata_path);
    fs::write(&body_tmp, &capture.bytes).map_err(|error| cache_error(&body_tmp, &error))?;
    let metadata = [
        CACHE_FORMAT_VERSION,
        "\nurl=",
        capture.source_url.as_str(),
        "\nfetched_at=",
        capture.fetched_at.as_str(),
        "\nsha256=",
        capture.sha256.as_str(),
        "\n",
    ]
    .concat();
    fs::write(&metadata_tmp, metadata).map_err(|error| cache_error(&metadata_tmp, &error))?;
    fs::rename(&body_tmp, &body_path).map_err(|error| cache_error(&body_path, &error))?;
    fs::rename(&metadata_tmp, metadata_path).map_err(|error| cache_error(metadata_path, &error))
}

fn content_path(cache_root: &Path, sha256: &str) -> PathBuf {
    cache_root.join("objects").join(format!("{sha256}.body"))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn unique_temp_path(path: &Path) -> PathBuf {
    let id = TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    path.with_extension(format!("{extension}.{}.{}.tmp", std::process::id(), id))
}

fn cache_error(path: &Path, error: &io::Error) -> FetchError {
    FetchError::Cache(format!("{}: {error}", path.display()))
}

fn cache_diagnostic(code: &str, path: &Path) -> FetchError {
    FetchError::Cache([code, &path.display().to_string()].join(":"))
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn format_timestamp(seconds: u64) -> String {
    // Unix seconds are real, timezone-independent, and losslessly replayable.
    format!("{seconds}")
}
