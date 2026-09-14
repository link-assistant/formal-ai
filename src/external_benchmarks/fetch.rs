//! Download-on-run payload cache for upstream benchmark slices (issue #698).
//!
//! Payloads land under `target/formal-ai-benchmarks`, are reused only while
//! their content and immutable upstream provenance still match, and are never
//! written into `data/`. Downloads go through `curl` and `gzip`, the same tools
//! the issue #362 download-on-test benchmark already depends on.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::{
    manifest::{CACHE_DIR, SuiteManifest, SuiteSource},
    vocabulary,
};

/// Fetch (or reuse) the payload for `manifest`, returning newline-delimited
/// JSON records — one upstream case per line.
///
/// File-backed sources are cached whole; `slice` is applied after parsing.
pub fn fetch_records(
    manifest: &SuiteManifest,
    _slice: usize,
    cache_root: &Path,
) -> Result<Vec<String>, String> {
    match &manifest.source {
        SuiteSource::JsonLines {
            url,
            cache_file,
            gzip,
        } => {
            let path = cache_root.join(cache_file);
            if !cache_is_valid(&path, manifest, url) {
                if *gzip {
                    download_gzip(url, &path)?;
                } else {
                    download(url, &path)?;
                }
                write_provenance(&path, manifest, url)?;
            }
            let text = read_cached(&path)?;
            Ok(non_empty_lines(&text))
        }
        SuiteSource::BigBenchTask { url, cache_file } => {
            let path = cache_root.join(cache_file);
            if !cache_is_valid(&path, manifest, url) {
                download(url, &path)?;
                write_provenance(&path, manifest, url)?;
            }
            let text = read_cached(&path)?;
            let document: serde_json::Value = serde_json::from_str(&text)
                .map_err(|error| format!("{} task.json is not valid JSON: {error}", manifest.id))?;
            let examples = document
                .get("examples")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| format!("{} task.json has no `examples` array", manifest.id))?;
            Ok(examples.iter().map(ToString::to_string).collect())
        }
        SuiteSource::RustSource { url, cache_file } => {
            let path = cache_root.join(cache_file);
            if !cache_is_valid(&path, manifest, url) {
                download(url, &path)?;
                write_provenance(&path, manifest, url)?;
            }
            let text = read_cached(&path)?;
            super::upstream_rust::adapt_records(manifest.id, &text)
        }
        SuiteSource::ParquetRows { url, cache_file } => {
            let path = cache_root.join(cache_file);
            if !cache_is_valid(&path, manifest, url) {
                download(url, &path)?;
                write_provenance(&path, manifest, url)?;
            }
            parquet_records(&path, manifest)
        }
        SuiteSource::Unavailable => Err(format!("{} has no fetchable payload", manifest.id)),
    }
}

/// Absolute cache directory for a repository root.
#[must_use]
pub fn cache_root(repository_root: &Path) -> PathBuf {
    repository_root.join(CACHE_DIR)
}

fn non_empty_lines(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(ToString::to_string)
        .collect()
}

fn read_cached(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read cached payload {}: {error}", path.display()))
}

fn create_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create cache directory {}: {error}",
                parent.display()
            )
        })?;
    }
    Ok(())
}

fn download(url: &str, destination: &Path) -> Result<(), String> {
    create_parent(destination)?;
    let partial = destination.with_extension("partial");
    let status = Command::new("curl")
        .args(["-fSL", "--retry", "3", "--retry-delay", "2", "-o"])
        .arg(&partial)
        .arg(url)
        .status()
        .map_err(|error| format!("failed to start curl for {url}: {error}"))?;
    if !status.success() {
        return Err(format!("curl failed for {url} with status {status}"));
    }
    fs::rename(&partial, destination)
        .map_err(|error| format!("failed to publish {}: {error}", destination.display()))
}

fn download_gzip(url: &str, destination: &Path) -> Result<(), String> {
    let compressed = destination.with_extension("jsonl.gz");
    download(url, &compressed)?;
    let partial = destination.with_extension("partial");
    let output = fs::File::create(&partial)
        .map_err(|error| format!("failed to create {}: {error}", partial.display()))?;
    let status = Command::new("gzip")
        .arg("-dc")
        .arg(&compressed)
        .stdout(Stdio::from(output))
        .status()
        .map_err(|error| format!("failed to start gzip for {}: {error}", compressed.display()))?;
    if !status.success() {
        return Err(format!(
            "gzip failed for {} with status {status}",
            compressed.display()
        ));
    }
    fs::rename(&partial, destination)
        .map_err(|error| format!("failed to publish {}: {error}", destination.display()))
}

/// Why a payload cannot be decoded in this environment, if anything.
#[must_use]
pub fn unavailable_reason(manifest: &SuiteManifest) -> Option<String> {
    if matches!(manifest.source, SuiteSource::ParquetRows { .. })
        && !python_module_available("pyarrow.parquet")
    {
        return Some(vocabulary::text(
            "external_benchmark_parquet_module_unavailable",
        ));
    }
    None
}

fn python_module_available(module: &str) -> bool {
    let import = vocabulary::render("external_benchmark_python_import", &[("module", module)]);
    Command::new("python3")
        .args(["-c", &import])
        .output()
        .is_ok_and(|output| output.status.success())
}

fn parquet_records(path: &Path, manifest: &SuiteManifest) -> Result<Vec<String>, String> {
    let script = concat!(
        "import json, sys\n",
        "import pyarrow.parquet as parquet\n",
        "for row in parquet.read_table(sys.argv[1]).to_pylist():\n",
        "    print(json.dumps(row, default=str, separators=(',', ':')))\n",
    );
    let output = Command::new("python3")
        .args(["-c", script])
        .arg(path)
        .output()
        .map_err(|error| {
            vocabulary::render(
                "external_benchmark_parquet_start_error",
                &[
                    ("path", &path.display().to_string()),
                    ("error", &error.to_string()),
                ],
            )
        })?;
    if !output.status.success() {
        return Err(vocabulary::render(
            "external_benchmark_parquet_decode_error",
            &[
                ("suite", manifest.id),
                ("path", &path.display().to_string()),
                ("error", String::from_utf8_lossy(&output.stderr).trim()),
            ],
        ));
    }
    let text = String::from_utf8(output.stdout).map_err(|error| {
        vocabulary::render(
            "external_benchmark_parquet_utf8_error",
            &[("suite", manifest.id), ("error", &error.to_string())],
        )
    })?;
    Ok(non_empty_lines(&text))
}

fn provenance_path(payload: &Path) -> PathBuf {
    let mut name = payload.as_os_str().to_os_string();
    name.push(".provenance.lino");
    PathBuf::from(name)
}

fn cache_is_valid(path: &Path, manifest: &SuiteManifest, url: &str) -> bool {
    let Ok(payload) = fs::read(path) else {
        return false;
    };
    let Ok(provenance) = fs::read_to_string(provenance_path(path)) else {
        return false;
    };
    let payload_bytes = payload.len().to_string();
    let content_id = format!("{:016x}", fnv1a64(&payload));
    provenance
        == vocabulary::render(
            "external_benchmark_provenance",
            &[
                ("suite", manifest.id),
                ("source_ref", manifest.source_ref),
                ("url", url),
                ("payload_bytes", &payload_bytes),
                ("content_id", &content_id),
            ],
        )
}

fn write_provenance(path: &Path, manifest: &SuiteManifest, url: &str) -> Result<(), String> {
    let payload = fs::read(path)
        .map_err(|error| format!("failed to read cached payload {}: {error}", path.display()))?;
    let payload_bytes = payload.len().to_string();
    let content_id = format!("{:016x}", fnv1a64(&payload));
    let provenance = vocabulary::render(
        "external_benchmark_provenance",
        &[
            ("suite", manifest.id),
            ("source_ref", manifest.source_ref),
            ("url", url),
            ("payload_bytes", &payload_bytes),
            ("content_id", &content_id),
        ],
    );
    let destination = provenance_path(path);
    fs::write(&destination, provenance).map_err(|error| {
        vocabulary::render(
            "external_benchmark_provenance_write_error",
            &[
                ("path", &destination.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
