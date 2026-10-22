//! Shared source-cache admission policy. Full caches refuse writes without evicting evidence.
use super::{FetchError, cache_diagnostic, cache_error};
use serde::Deserialize;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[cfg(not(target_arch = "wasm32"))]
use crate::provenance_snapshot::{open_observed, same_observed_file};

#[cfg(target_arch = "wasm32")]
fn open_observed(_path: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "source_cache_filesystem_unavailable",
    ))
}
#[cfg(target_arch = "wasm32")]
fn same_observed_file(_file: &File, _path: &Path) -> io::Result<bool> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "source_cache_filesystem_unavailable",
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Policy {
    format: String,
    pub(super) max_bytes: u64,
    pub(super) max_files: usize,
    pub(super) max_entries: usize,
    pub(super) max_body_bytes: usize,
    pub(super) max_metadata_bytes: usize,
}

pub(super) fn policy() -> Result<&'static Policy, FetchError> {
    static POLICY: OnceLock<Result<Policy, FetchError>> = OnceLock::new();
    POLICY
        .get_or_init(|| {
            let raw = include_str!("../../embedded/data/meta/source-cache-budget.json");
            let parsed: Policy =
                serde_json::from_str(raw).map_err(|error| FetchError::Cache(error.to_string()))?;
            if parsed.format != "source-cache-budget/v1"
                || parsed.max_body_bytes == 0
                || parsed.max_metadata_bytes == 0
                || parsed.max_entries == 0
                || parsed.max_files == 0
                || parsed.max_bytes == 0
                || parsed.max_body_bytes as u64 > parsed.max_bytes
            {
                return Err(FetchError::Cache("source_cache_policy_invalid".to_owned()));
            }
            Ok(parsed)
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn regular_directory(path: &Path) -> Result<(), FetchError> {
    if !fs::symlink_metadata(path)
        .map_err(|error| cache_error(path, &error))?
        .file_type()
        .is_dir()
    {
        return Err(cache_diagnostic("source_cache_directory_type", path));
    }
    Ok(())
}

pub(super) fn read_bounded(path: &Path, maximum: usize) -> Result<Vec<u8>, FetchError> {
    let observed = fs::symlink_metadata(path).map_err(|error| cache_error(path, &error))?;
    if !observed.file_type().is_file() || observed.len() > maximum as u64 {
        return Err(cache_diagnostic("source_cache_read_budget_or_type", path));
    }
    let mut file = open_observed(path).map_err(|error| cache_error(path, &error))?;
    let before = file.metadata().map_err(|error| cache_error(path, &error))?;
    if !before.is_file()
        || before.len() > maximum as u64
        || !fs::symlink_metadata(path)
            .map_err(|error| cache_error(path, &error))?
            .file_type()
            .is_file()
    {
        return Err(cache_diagnostic("source_cache_read_budget_or_type", path));
    }
    if !same_observed_file(&file, path).map_err(|error| cache_error(path, &error))? {
        return Err(cache_diagnostic("source_cache_read_changed", path));
    }
    let capacity = usize::try_from(before.len())
        .map_err(|_| cache_diagnostic("source_cache_read_budget_or_type", path))?;
    let mut bytes = Vec::with_capacity(capacity);
    let mut chunk = vec![0_u8; 64 * 1024];
    // Each positive read advances by at least one byte; EOF gets one final observation.
    for _ in 0..=maximum {
        let remaining = maximum.saturating_sub(bytes.len());
        let count = file
            .read(&mut chunk[..(remaining + 1).min(64 * 1024)])
            .map_err(|error| cache_error(path, &error))?;
        if count == 0 {
            let after = file.metadata().map_err(|error| cache_error(path, &error))?;
            let current = fs::symlink_metadata(path).map_err(|error| cache_error(path, &error))?;
            if !current.file_type().is_file()
                || after.len() != before.len()
                || bytes.len() as u64 != before.len()
                || after.modified().ok() != before.modified().ok()
            {
                return Err(cache_diagnostic("source_cache_read_changed", path));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if current.dev() != before.dev()
                    || current.ino() != before.ino()
                    || after.ctime() != before.ctime()
                    || after.ctime_nsec() != before.ctime_nsec()
                {
                    return Err(cache_diagnostic("source_cache_read_changed", path));
                }
            }
            if !same_observed_file(&file, path).map_err(|error| cache_error(path, &error))? {
                return Err(cache_diagnostic("source_cache_read_changed", path));
            }
            return Ok(bytes);
        }
        if count > remaining {
            return Err(cache_diagnostic("source_cache_read_budget_or_type", path));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    Err(cache_diagnostic("source_cache_read_progress_budget", path))
}

#[derive(Default)]
struct Usage {
    bytes: u64,
    files: usize,
    entries: usize,
}

fn visit(root: &Path, objects: bool, policy: &Policy, usage: &mut Usage) -> Result<(), FetchError> {
    regular_directory(root)?;
    for entry in fs::read_dir(root).map_err(|error| cache_error(root, &error))? {
        let entry = entry.map_err(|error| cache_error(root, &error))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| cache_error(&path, &error))?;
        if !objects && entry.file_name() == "objects" && metadata.file_type().is_dir() {
            visit(&path, true, policy, usage)?;
            continue;
        }
        if !metadata.file_type().is_file() {
            return Err(cache_diagnostic("source_cache_entry_type", &path));
        }
        if !objects && entry.file_name() == ".source-cache-write.lock" {
            continue;
        }
        usage.files += 1;
        usage.bytes = usage
            .bytes
            .checked_add(metadata.len())
            .ok_or_else(|| cache_diagnostic("source_cache_budget_exceeded", root))?;
        if !objects && path.extension().is_some_and(|value| value == "meta") {
            usage.entries += 1;
        }
        if usage.files > policy.max_files
            || usage.bytes > policy.max_bytes
            || usage.entries > policy.max_entries
        {
            return Err(cache_diagnostic("source_cache_budget_exceeded", root));
        }
    }
    Ok(())
}

pub(super) struct WriteAdmission {
    path: PathBuf,
    file: File,
    released: bool,
    pub(super) reuse_body: bool,
}
impl WriteAdmission {
    pub(super) fn finish(mut self) -> Result<(), FetchError> {
        if !same_observed_file(&self.file, &self.path)
            .map_err(|error| cache_error(&self.path, &error))?
        {
            return Err(cache_diagnostic("source_cache_lock_custody", &self.path));
        }
        fs::remove_file(&self.path).map_err(|error| cache_error(&self.path, &error))?;
        self.released = true;
        Ok(())
    }
}
impl Drop for WriteAdmission {
    fn drop(&mut self) {
        if !self.released && same_observed_file(&self.file, &self.path).unwrap_or(false) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// A single exclusive-create attempt. Shared lock names serialize native and Node writers.
pub(super) fn admit(
    root: &Path,
    metadata_path: &Path,
    body_path: &Path,
    bytes: &[u8],
    metadata_bytes: usize,
) -> Result<WriteAdmission, FetchError> {
    let policy = policy()?;
    let body_bytes = bytes.len();
    if body_bytes > policy.max_body_bytes || metadata_bytes > policy.max_metadata_bytes {
        return Err(cache_diagnostic(
            "source_cache_capture_budget_exceeded",
            root,
        ));
    }
    prepare_root(root)?;
    let path = root.join(".source-cache-write.lock");
    let lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| cache_error(&path, &error))?;
    let mut admission = WriteAdmission {
        path,
        file: lock,
        released: false,
        reuse_body: false,
    };
    let objects = root.join("objects");
    match fs::symlink_metadata(&objects) {
        Ok(_) => regular_directory(&objects)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(&objects).map_err(|error| cache_error(&objects, &error))?;
            regular_directory(&objects)?;
        }
        Err(error) => return Err(cache_error(&objects, &error)),
    }
    let mut usage = Usage::default();
    visit(root, false, policy, &mut usage)?;
    let existing_metadata = match fs::symlink_metadata(metadata_path) {
        Ok(metadata) if metadata.file_type().is_file() => true,
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Ok(_) => return Err(cache_diagnostic("source_cache_entry_type", metadata_path)),
        Err(error) => return Err(cache_error(metadata_path, &error)),
    };
    admission.reuse_body = match fs::symlink_metadata(body_path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            if read_bounded(body_path, policy.max_body_bytes)? != bytes {
                return Err(cache_diagnostic(
                    "source_cache_content_hash_mismatch",
                    body_path,
                ));
            }
            true
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Ok(_) => return Err(cache_diagnostic("source_cache_entry_type", body_path)),
        Err(error) => return Err(cache_error(body_path, &error)),
    };
    let additional_body = if admission.reuse_body { 0 } else { body_bytes };
    let peak_bytes = usage
        .bytes
        .checked_add(additional_body as u64)
        .and_then(|value| value.checked_add(metadata_bytes as u64));
    if usage.entries + usize::from(!existing_metadata) > policy.max_entries
        || usage.files + if admission.reuse_body { 1 } else { 2 } > policy.max_files
        || peak_bytes.is_none_or(|bytes| bytes > policy.max_bytes)
    {
        return Err(cache_diagnostic("source_cache_budget_exceeded", root));
    }
    Ok(admission)
}

/// Validate the cache root before creating its objects directory or staging captures.
fn prepare_root(root: &Path) -> Result<(), FetchError> {
    match fs::symlink_metadata(root) {
        Ok(_) => regular_directory(root),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder
                .create(root)
                .map_err(|error| cache_error(root, &error))?;
            regular_directory(root)
        }
        Err(error) => Err(cache_error(root, &error)),
    }
}

/// Own only exclusively-created temporary paths, and remove them on every unpublished exit.
pub(super) struct CaptureStage {
    path: PathBuf,
    file: File,
    published: bool,
}
impl CaptureStage {
    pub(super) fn write(path: PathBuf, bytes: &[u8]) -> Result<Self, FetchError> {
        use std::io::Write as _;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options
            .open(&path)
            .map_err(|error| cache_error(&path, &error))?;
        let mut stage = Self {
            path,
            file,
            published: false,
        };
        stage
            .file
            .write_all(bytes)
            .map_err(|error| cache_error(&stage.path, &error))?;
        stage
            .file
            .sync_all()
            .map_err(|error| cache_error(&stage.path, &error))?;
        Ok(stage)
    }
    pub(super) fn publish(mut self, destination: &Path) -> Result<(), FetchError> {
        if !same_observed_file(&self.file, &self.path)
            .map_err(|error| cache_error(&self.path, &error))?
        {
            return Err(cache_diagnostic(
                "source_cache_temporary_custody",
                &self.path,
            ));
        }
        fs::rename(&self.path, destination).map_err(|error| cache_error(destination, &error))?;
        self.published = true;
        Ok(())
    }
}
impl Drop for CaptureStage {
    fn drop(&mut self) {
        if !self.published && same_observed_file(&self.file, &self.path).unwrap_or(false) {
            let _ = fs::remove_file(&self.path);
        }
    }
}
