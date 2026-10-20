//! Physical file observation for canonical source snapshots; no source policy is inferred from contents.
use super::{
    SourceFile, cpu_backend, failure, inventory_leaf, language_extensions, merkle_root,
    source_snapshot,
};
use crate::provenance_snapshot_generated::snapshot_path_eligible;
use serde::Serialize;
use serde_json::{Value, json};
use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

const INDEX_BYTES: u64 = 4 * 1024 * 1024;
const GIT_TIMEOUT: Duration = Duration::from_secs(15);
/// Resource budgets match the JavaScript repository snapshot defaults.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotLimits {
    /// Maximum tracked files; no excluded files are silently omitted.
    pub max_files: usize,
    /// Maximum bytes delivered by an individual source file.
    pub max_file_bytes: u64,
    /// Maximum total source bytes, checked before the first payload read.
    pub max_total_bytes: u64,
}
impl Default for SnapshotLimits {
    fn default() -> Self {
        Self {
            max_files: 50_000,
            max_file_bytes: 4 * 1024 * 1024,
            max_total_bytes: 32 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Identity {
    volume: u64,
    object: u64,
    length: u64,
    modified: SystemTime,
}
struct Entry {
    relative: String,
    path: PathBuf,
    mode: String,
    object: String,
    identity: Identity,
}
#[cfg(unix)]
pub(crate) fn open_observed(path: &Path) -> io::Result<File> {
    rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC
            | rustix::fs::OFlags::NONBLOCK,
        rustix::fs::Mode::empty(),
    )
    .map(File::from)
    .map_err(io::Error::from)
}
#[cfg(windows)]
pub(crate) fn open_observed(path: &Path) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
    fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}
#[cfg(not(any(unix, windows)))]
pub(crate) fn open_observed(_path: &Path) -> io::Result<File> {
    Err(failure("snapshot_platform_unsupported"))
}
#[cfg(unix)]
fn file_identity(_file: &File, metadata: &Metadata) -> io::Result<Identity> {
    use std::os::unix::fs::MetadataExt;
    Ok(Identity {
        volume: metadata.dev(),
        object: metadata.ino(),
        length: metadata.len(),
        modified: metadata.modified()?,
    })
}
#[cfg(windows)]
fn file_identity(file: &File, metadata: &Metadata) -> io::Result<Identity> {
    let information = winapi_util::file::information(file)?;
    Ok(Identity {
        volume: information.volume_serial_number(),
        object: information.file_index(),
        length: metadata.len(),
        modified: metadata.modified()?,
    })
}
#[cfg(not(any(unix, windows)))]
fn file_identity(_file: &File, _metadata: &Metadata) -> io::Result<Identity> {
    Err(failure("snapshot_platform_unsupported"))
}
fn regular(metadata: &Metadata) -> bool {
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return false;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return false;
        }
    }
    true
}
fn ancestors(root: &Path, relative: &str) -> io::Result<PathBuf> {
    let mut path = root.to_path_buf();
    let components: Vec<_> = relative.split('/').collect();
    for (index, component) in components.iter().enumerate() {
        path.push(component);
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(failure("snapshot_symlink_refused"));
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(failure("snapshot_symlink_refused"));
            }
        }
        if index + 1 < components.len() && !metadata.is_dir() {
            return Err(failure("snapshot_parent_not_directory"));
        }
    }
    Ok(path)
}
/// Compare descriptor and current no-follow path identities on every supported host.
pub(crate) fn same_observed_file(file: &File, path: &Path) -> io::Result<bool> {
    let metadata = file.metadata()?;
    if !regular(&metadata) {
        return Ok(false);
    }
    let current = open_observed(path)?;
    let observed = current.metadata()?;
    if !regular(&observed) {
        return Ok(false);
    }
    Ok(file_identity(file, &metadata)? == file_identity(&current, &observed)?)
}
fn inspect(path: &Path) -> io::Result<Identity> {
    let file = open_observed(path)?;
    let metadata = file.metadata()?;
    if !regular(&metadata) {
        return Err(failure("snapshot_regular_file_required"));
    }
    file_identity(&file, &metadata)
}
fn git(root: &Path, arguments: &[&str]) -> io::Result<Vec<u8>> {
    let mut child = Command::new("git")
        .arg("-c")
        .arg("core.fsmonitor=false")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| failure("snapshot_git_stdout_missing"))?;
    let reader = std::thread::spawn(move || {
        let mut output = Vec::new();
        stdout
            .take(INDEX_BYTES + 1)
            .read_to_end(&mut output)
            .map(|_| output)
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= GIT_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err(failure("snapshot_git_timeout"));
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let output = reader
        .join()
        .map_err(|_| failure("snapshot_git_reader_failed"))??;
    if !status.success()
        || u64::try_from(output.len()).map_err(|_| failure("snapshot_index_too_large"))?
            > INDEX_BYTES
    {
        return Err(failure("snapshot_git_failed_or_index_too_large"));
    }
    Ok(output)
}
fn preflight(root: &Path, limits: SnapshotLimits) -> io::Result<(Vec<Entry>, u64)> {
    let output = git(root, &["ls-files", "--cached", "--stage", "-z"])?;
    let listing =
        std::str::from_utf8(&output).map_err(|_| failure("snapshot_index_utf8_required"))?;
    if !listing.is_empty() && !listing.ends_with('\0') {
        return Err(failure("snapshot_index_truncated"));
    }
    let mut entries = Vec::new();
    let mut total = 0_u64;
    let mut seen = std::collections::HashSet::new();
    for row in listing.split_terminator('\0') {
        let (metadata, relative) = row
            .split_once('\t')
            .ok_or_else(|| failure("snapshot_index_row_invalid"))?;
        let fields: Vec<_> = metadata.split(' ').collect();
        if fields.len() != 3
            || fields[2] != "0"
            || !matches!(fields[0], "100644" | "100755")
            || !matches!(fields[1].len(), 40 | 64)
            || !fields[1]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(failure("snapshot_index_mode_or_stage_refused"));
        }
        if !snapshot_path_eligible(
            relative.to_owned(),
            relative.split('/').map(str::to_owned).collect(),
        ) || !seen.insert(relative)
            || entries.len() >= limits.max_files
        {
            return Err(failure("snapshot_path_or_file_budget_refused"));
        }
        let path = ancestors(root, relative)?;
        let identity = inspect(&path)?;
        total = total
            .checked_add(identity.length)
            .ok_or_else(|| failure("snapshot_size_overflow"))?;
        if identity.length > limits.max_file_bytes || total > limits.max_total_bytes {
            return Err(failure("snapshot_byte_budget_refused"));
        }
        entries.push(Entry {
            relative: relative.to_owned(),
            path,
            mode: fields[0].to_owned(),
            object: fields[1].to_owned(),
            identity,
        });
    }
    Ok((entries, total))
}
fn payload(root: &Path, entry: &Entry) -> io::Result<Vec<u8>> {
    let path = ancestors(root, &entry.relative)?;
    if path != entry.path {
        return Err(failure("snapshot_source_path_changed"));
    }
    let file = open_observed(&path)?;
    let metadata = file.metadata()?;
    if !regular(&metadata) || file_identity(&file, &metadata)? != entry.identity {
        return Err(failure("snapshot_source_changed"));
    }
    let mut content = Vec::new();
    (&file)
        .take(
            entry
                .identity
                .length
                .checked_add(1)
                .ok_or_else(|| failure("snapshot_size_overflow"))?,
        )
        .read_to_end(&mut content)?;
    if u64::try_from(content.len()).map_err(|_| failure("snapshot_source_size_changed"))?
        != entry.identity.length
        || file_identity(&file, &file.metadata()?)? != entry.identity
        || inspect(&path)? != entry.identity
    {
        return Err(failure("snapshot_source_changed"));
    }
    Ok(content)
}
/// Snapshot all tracked working bytes; preflight excludes nothing and refuses unsafe inventory rows.
pub fn snapshot_repository(
    directory: &Path,
    requested_backend: &str,
    limits: SnapshotLimits,
) -> io::Result<Value> {
    const MAXIMUM_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
    if limits.max_file_bytes > MAXIMUM_SAFE_INTEGER
        || limits.max_total_bytes > MAXIMUM_SAFE_INTEGER
        || u64::try_from(limits.max_files).map_err(|_| failure("invalid_snapshot_limits"))?
            > MAXIMUM_SAFE_INTEGER
    {
        return Err(failure("invalid_snapshot_limits"));
    }
    let hashing = cpu_backend(requested_backend)?;
    let root = fs::canonicalize(directory)?;
    let observed_root = git(&root, &["rev-parse", "--show-toplevel"])?;
    let observed_root =
        std::str::from_utf8(&observed_root).map_err(|_| failure("snapshot_root_utf8_required"))?;
    let observed_root = observed_root.strip_suffix('\n').unwrap_or(observed_root);
    if fs::canonicalize(observed_root)? != root {
        return Err(failure("snapshot_git_root_required"));
    }
    let (entries, total) = preflight(&root, limits)?;
    let mut files = Vec::new();
    let mut inventory = Vec::new();
    for entry in &entries {
        files.push(SourceFile {
            path: entry.relative.clone(),
            content: payload(&root, entry)?,
        });
        inventory.push(inventory_leaf(
            &entry.relative,
            &entry.mode,
            &entry.object,
            hashing,
        ));
    }
    let extensions = language_extensions(include_str!(
        "../../embedded/data/seed/page-formalization-rules.lino"
    ))?;
    let mut result = serde_json::to_value(source_snapshot(&files, &extensions, hashing)?)
        .map_err(io::Error::other)?;
    result["inventory"] = json!({"kind":"git-index-tracked-working-bytes", "file_count":entries.len(),
        "total_bytes":total,"root":blake3::Hash::from(merkle_root(&inventory, hashing)?).to_hex().to_string(),
        "exclusions":[],"symlinks":"refused","gitlinks":"refused","unmerged":"refused"});
    result["limits"] = serde_json::to_value(limits).map_err(io::Error::other)?;
    Ok(result)
}
