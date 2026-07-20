//! Download-on-run payload cache for upstream benchmark slices (issue #698).
//!
//! Payloads land under `target/formal-ai-benchmarks`, are reused on the next
//! run, and are never written into `data/`. Downloads go through `curl` and
//! `gzip`, the same tools the issue #362 download-on-test benchmark already
//! depends on.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::manifest::{encode_component, SuiteManifest, SuiteSource, CACHE_DIR};

/// The datasets-server `rows` endpoint returns at most 100 rows per request.
const ROWS_PAGE_SIZE: usize = 100;

/// Fetch (or reuse) the payload for `manifest`, returning newline-delimited
/// JSON records — one upstream case per line.
///
/// `slice` is only used by the paged datasets-server sources, where fetching
/// the whole dataset would be wasteful; file-backed sources are cached whole.
pub fn fetch_records(
    manifest: &SuiteManifest,
    slice: usize,
    cache_root: &Path,
) -> Result<Vec<String>, String> {
    match &manifest.source {
        SuiteSource::JsonLines {
            url,
            cache_file,
            gzip,
        } => {
            let path = cache_root.join(cache_file);
            if !path.exists() {
                if *gzip {
                    download_gzip(url, &path)?;
                } else {
                    download(url, &path)?;
                }
            }
            let text = read_cached(&path)?;
            Ok(non_empty_lines(&text))
        }
        SuiteSource::BigBenchTask { url, cache_file } => {
            let path = cache_root.join(cache_file);
            if !path.exists() {
                download(url, &path)?;
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
        SuiteSource::DatasetsServerRows {
            dataset,
            config,
            split,
            cache_file,
        } => {
            let path = cache_root.join(format!("{cache_file}.{slice}"));
            if !path.exists() {
                let rows = download_rows(dataset, config, split, slice)?;
                write_cached(&path, &rows.join("\n"))?;
            }
            let text = read_cached(&path)?;
            Ok(non_empty_lines(&text))
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

fn write_cached(path: &Path, contents: &str) -> Result<(), String> {
    create_parent(path)?;
    fs::write(path, contents)
        .map_err(|error| format!("failed to write cache file {}: {error}", path.display()))
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

fn download_rows(
    dataset: &str,
    config: &str,
    split: &str,
    slice: usize,
) -> Result<Vec<String>, String> {
    let mut rows = Vec::new();
    let mut offset = 0;
    while rows.len() < slice {
        let length = ROWS_PAGE_SIZE.min(slice - rows.len());
        let url = format!(
            "https://datasets-server.huggingface.co/rows?dataset={}&config={}&split={}&offset={offset}&length={length}",
            encode_component(dataset),
            encode_component(config),
            encode_component(split),
        );
        let output = Command::new("curl")
            .args(["-fsSL", "--retry", "3", "--retry-delay", "2", &url])
            .output()
            .map_err(|error| format!("failed to start curl for {url}: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "datasets-server request failed for {dataset} ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim(),
            ));
        }
        let body: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|error| format!("datasets-server returned invalid JSON: {error}"))?;
        let page = body
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("datasets-server response for {dataset} has no `rows`"))?;
        if page.is_empty() {
            break;
        }
        for entry in page {
            let row = entry.get("row").unwrap_or(entry);
            rows.push(row.to_string());
        }
        offset += length;
    }
    Ok(rows)
}
