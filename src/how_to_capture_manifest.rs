//! The committed QA record of a real-service how-to capture — issue #991.
//!
//! A capture tree on its own is bytes without a claim: it says what was stored,
//! not what was promised. Issue #991 requires the QA captures to be committed
//! *with* timestamps, hashes, and the license/provenance each byte is quoted
//! under, so the offline replay can be checked against a written record and a
//! gated live refresh can report exactly which source drifted.
//!
//! The manifest is that record. It is derived from the capture cache itself —
//! never hand-written — joined with `data/seed/sources-registry.lino` for the
//! license, so a manifest can neither claim a capture that is missing nor a
//! license the registry does not declare.

use std::fs;
use std::io;
use std::path::Path;

use crate::links_format::push_lino_node;
use crate::seed::parser::parse_lino;
use crate::seed::{source_registry, SourceRecord};
use crate::source_fetch::sha256_hex;

/// File the manifest is committed as, next to the capture tree.
pub const CAPTURE_MANIFEST_FILE: &str = "capture-manifest.lino";

/// Root node name of the manifest projection.
const ROOT: &str = "capture_manifest";

/// One committed capture and everything a reviewer needs to trust it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureManifestEntry {
    /// The exact URL that was retrieved.
    pub url: String,
    /// Registry id of the service the URL belongs to, or `unregistered`.
    pub source_id: String,
    /// sha256 of the captured bytes, as stored in the cache metadata.
    pub sha256: String,
    /// When the bytes were retrieved (Unix seconds, as the cache records it).
    pub fetched_at: String,
    /// Size of the captured body in bytes.
    pub bytes: usize,
    /// License the bytes are quoted under, from the registry.
    pub license_name: String,
    /// Canonical URL of that license.
    pub license_url: String,
}

impl CaptureManifestEntry {
    /// Stable one-line payload used in traces and drift reports.
    #[must_use]
    pub fn trace_payload(&self) -> String {
        format!(
            "url={} source={} sha256={} fetched_at={} bytes={} license={}",
            self.url, self.source_id, self.sha256, self.fetched_at, self.bytes, self.license_name
        )
    }
}

/// How a manifest entry compares against the bytes on disk now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureDrift {
    /// The URL is in the manifest but no longer in the capture tree.
    Missing {
        /// The URL that vanished.
        url: String,
    },
    /// The URL is in the capture tree but not in the manifest.
    Unrecorded {
        /// The newly captured URL.
        url: String,
    },
    /// The same URL now yields different bytes.
    Changed {
        /// The URL whose bytes changed.
        url: String,
        /// sha256 the manifest recorded.
        recorded_sha256: String,
        /// sha256 of the bytes now stored.
        current_sha256: String,
    },
}

impl CaptureDrift {
    /// Stable one-line payload for the drift report.
    #[must_use]
    pub fn trace_payload(&self) -> String {
        match self {
            Self::Missing { url } => format!("drift=missing url={url}"),
            Self::Unrecorded { url } => format!("drift=unrecorded url={url}"),
            Self::Changed {
                url,
                recorded_sha256,
                current_sha256,
            } => format!(
                "drift=changed url={url} recorded={recorded_sha256} current={current_sha256}"
            ),
        }
    }
}

/// Read every capture stored under `cache_dir`, newest metadata first by URL.
///
/// `cache_dir` is the same directory [`crate::source_fetch::CachedSourceClient`]
/// was constructed with; the captures live in its `source-cache` subtree.
///
/// # Errors
///
/// Returns the underlying I/O error when the capture tree cannot be read.
pub fn read_captures(cache_dir: impl AsRef<Path>) -> io::Result<Vec<CaptureManifestEntry>> {
    let root = cache_dir.as_ref().join("source-cache");
    let mut entries: Vec<CaptureManifestEntry> = Vec::new();
    let registry = source_registry();
    let listing = match fs::read_dir(&root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(entries),
        Err(error) => return Err(error),
    };
    for item in listing {
        let path = item?.path();
        if path.extension().is_none_or(|extension| extension != "meta") {
            continue;
        }
        let metadata = fs::read_to_string(&path)?;
        let mut url = String::new();
        let mut fetched_at = String::new();
        let mut sha256 = String::new();
        for line in metadata.lines() {
            if let Some(value) = line.strip_prefix("url=") {
                value.clone_into(&mut url);
            } else if let Some(value) = line.strip_prefix("fetched_at=") {
                value.clone_into(&mut fetched_at);
            } else if let Some(value) = line.strip_prefix("sha256=") {
                value.clone_into(&mut sha256);
            }
        }
        if url.is_empty() || sha256.is_empty() {
            continue;
        }
        let body = root.join("objects").join(format!("{sha256}.body"));
        let bytes = fs::read(&body).unwrap_or_default();
        let record = registry_record_for(&registry, &url);
        entries.push(CaptureManifestEntry {
            url,
            source_id: record
                .map_or_else(|| String::from("unregistered"), |record| record.id.clone()),
            sha256,
            fetched_at,
            bytes: bytes.len(),
            license_name: record.map_or_else(String::new, |record| record.license_name.clone()),
            license_url: record.map_or_else(String::new, |record| record.license_url.clone()),
        });
    }
    entries.sort_by(|left, right| left.url.cmp(&right.url));
    Ok(entries)
}

/// The registry source whose API host serves `url`, if any.
fn registry_record_for<'a>(registry: &'a [SourceRecord], url: &str) -> Option<&'a SourceRecord> {
    let host = url
        .split_once("://")
        .map_or(url, |(_, rest)| rest.split('/').next().unwrap_or(rest));
    registry.iter().find(|record| record.host() == host)
}

/// Project entries into the committed Links Notation manifest.
#[must_use]
pub fn manifest_lino(entries: &[CaptureManifestEntry]) -> String {
    let mut out = String::from(ROOT);
    out.push('\n');
    for entry in entries {
        push_lino_node(&mut out, 2, "capture", Some(&entry.sha256));
        push_lino_node(&mut out, 4, "url", Some(&entry.url));
        push_lino_node(&mut out, 4, "source", Some(&entry.source_id));
        push_lino_node(&mut out, 4, "sha256", Some(&entry.sha256));
        push_lino_node(&mut out, 4, "fetched_at", Some(&entry.fetched_at));
        push_lino_node(&mut out, 4, "bytes", Some(&entry.bytes.to_string()));
        push_lino_node(&mut out, 4, "license_name", Some(&entry.license_name));
        push_lino_node(&mut out, 4, "license_url", Some(&entry.license_url));
    }
    out
}

/// Parse a committed manifest back into entries, in file order.
#[must_use]
pub fn parse_manifest(text: &str) -> Vec<CaptureManifestEntry> {
    let tree = parse_lino(text);
    let mut entries = Vec::new();
    for root in tree.children.iter().filter(|node| node.name == ROOT) {
        for capture in root.children.iter().filter(|node| node.name == "capture") {
            entries.push(CaptureManifestEntry {
                url: capture.find_child_value("url").to_owned(),
                source_id: capture.find_child_value("source").to_owned(),
                sha256: capture.find_child_value("sha256").to_owned(),
                fetched_at: capture.find_child_value("fetched_at").to_owned(),
                bytes: capture.find_child_value("bytes").parse().unwrap_or(0),
                license_name: capture.find_child_value("license_name").to_owned(),
                license_url: capture.find_child_value("license_url").to_owned(),
            });
        }
    }
    entries
}

/// Every difference between a committed manifest and the current capture tree.
///
/// An empty result means the replay is reproducing exactly the bytes the
/// manifest promises; anything else names the source that drifted.
#[must_use]
pub fn drift(
    recorded: &[CaptureManifestEntry],
    current: &[CaptureManifestEntry],
) -> Vec<CaptureDrift> {
    let mut differences = Vec::new();
    for entry in recorded {
        match current.iter().find(|other| other.url == entry.url) {
            None => differences.push(CaptureDrift::Missing {
                url: entry.url.clone(),
            }),
            Some(other) if other.sha256 != entry.sha256 => {
                differences.push(CaptureDrift::Changed {
                    url: entry.url.clone(),
                    recorded_sha256: entry.sha256.clone(),
                    current_sha256: other.sha256.clone(),
                });
            }
            Some(_) => {}
        }
    }
    for entry in current {
        if !recorded.iter().any(|other| other.url == entry.url) {
            differences.push(CaptureDrift::Unrecorded {
                url: entry.url.clone(),
            });
        }
    }
    differences
}

/// Verify that every body in the capture tree still hashes to the digest its
/// metadata (and the manifest) claims.
///
/// This is the tamper check the offline replay depends on: a committed capture
/// whose bytes were edited would otherwise replay silently as evidence.
///
/// # Errors
///
/// Returns the underlying I/O error when a recorded body cannot be read.
pub fn verify_bodies(
    cache_dir: impl AsRef<Path>,
    entries: &[CaptureManifestEntry],
) -> io::Result<Vec<String>> {
    let objects = cache_dir.as_ref().join("source-cache").join("objects");
    let mut invalid = Vec::new();
    for entry in entries {
        let path = objects.join(format!("{}.body", entry.sha256));
        let bytes = fs::read(&path)?;
        if sha256_hex(&bytes) != entry.sha256 {
            invalid.push(entry.url.clone());
        }
    }
    Ok(invalid)
}
