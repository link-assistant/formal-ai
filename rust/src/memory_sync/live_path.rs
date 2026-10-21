//! Live memory selection preserves incompatible bytes and uses the existing lossless upgrade.
use crate::memory::{TARGET_MEMORY_SCHEMA_VERSION, export_links_notation_with_schema};
use crate::provenance_snapshot::{open_observed, same_observed_file};
use std::fs::{self, OpenOptions};
use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};
const MAXIMUM_BYTES: u64 = 64 * 1024 * 1024;
fn observed(path: &Path) -> io::Result<Vec<u8>> {
    if !fs::symlink_metadata(path)?.is_file() {
        return Err(io::Error::other("memory_live_regular_file_required"));
    }
    let mut file = open_observed(path)?;
    let before = file.metadata()?;
    if !before.is_file() || before.len() > MAXIMUM_BYTES {
        return Err(io::Error::other("memory_live_identity_or_size"));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(before.len()).map_err(io::Error::other)?);
    let mut buffer = vec![0_u8; 65_536];
    for _ in 0..=MAXIMUM_BYTES {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if u64::try_from(bytes.len())
            .map_err(io::Error::other)?
            .saturating_add(count as u64)
            > MAXIMUM_BYTES
        {
            return Err(io::Error::other("memory_live_size_changed"));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    if !same_observed_file(&file, path)?
        || file.metadata()?.len() != before.len()
        || file.metadata()?.modified()? != before.modified()?
        || bytes.len() as u64 != before.len()
    {
        return Err(io::Error::other("memory_live_changed"));
    }
    Ok(bytes)
}
fn current_private(path: &Path) -> io::Result<()> {
    let bytes = observed(path)?;
    let text = std::str::from_utf8(&bytes).map_err(io::Error::other)?;
    let status = crate::memory::upgrade::inspect_memory_text(text, true);
    if !status.compatible || status.detected_schema_version != Some(TARGET_MEMORY_SCHEMA_VERSION) {
        return Err(io::Error::other("memory_live_sibling_incompatible"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        let metadata = open_observed(path)?.metadata()?;
        if metadata.mode() & 0o077 != 0 || metadata.uid() != rustix::process::getuid().as_raw() {
            return Err(io::Error::other("memory_live_sibling_not_private"));
        }
    }
    Ok(())
}
fn create_private(path: &Path) -> io::Result<bool> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(false),
        Err(error) => return Err(error),
    };
    let bytes = export_links_notation_with_schema(&[], TARGET_MEMORY_SCHEMA_VERSION).into_bytes();
    let mut offset = 0;
    for _ in 0..bytes.len() {
        if offset == bytes.len() {
            break;
        }
        let count = file.write(&bytes[offset..])?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "memory_live_write_no_progress",
            ));
        }
        offset += count;
    }
    if offset != bytes.len() {
        return Err(io::Error::other("memory_live_write_incomplete"));
    }
    file.sync_all()?;
    if !same_observed_file(&file, path)? {
        return Err(io::Error::other("memory_live_changed"));
    }
    Ok(true)
}
/// Only supported schemas migrate; unknown documents remain byte-identical beside a new private log.
pub(super) fn resolve(path: &Path) -> io::Result<PathBuf> {
    let bytes = match observed(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(path.to_path_buf()),
        Err(error) => return Err(error),
    };
    let source_sha256 = crate::source_fetch::sha256_hex(&bytes);
    let status = std::str::from_utf8(&bytes)
        .ok()
        .map(|text| crate::memory::upgrade::inspect_memory_text(text, true));
    if let Some(status) = status.as_ref().filter(|status| status.compatible) {
        if status.migration_required {
            let receipt = crate::memory::migrate_memory_with_pre_commit(path, None, None, |_| {
                if crate::source_fetch::sha256_hex(&observed(path)?) != source_sha256 {
                    return Err(io::Error::other("memory_live_preimage_changed"));
                }
                Ok(())
            })
            .map_err(io::Error::other)?;
            crate::dialog_log::structured_log::event(
                "info",
                "memory",
                || "memory.live_migration".to_owned(),
                || serde_json::json!({"path":path,"source_sha256":source_sha256,"changed":receipt.changed}),
            );
        }
        return Ok(path.to_path_buf());
    }
    let filename = path
        .file_name()
        .ok_or_else(|| io::Error::other("memory_live_filename_required"))?;
    let mut name = filename.to_os_string();
    name.push(format!(
        ".schema-{TARGET_MEMORY_SCHEMA_VERSION}.{source_sha256}.lino"
    ));
    let selected = path.with_file_name(name);
    let created = create_private(&selected)?;
    current_private(&selected)?;
    if crate::source_fetch::sha256_hex(&observed(path)?) != source_sha256 {
        return Err(io::Error::other("memory_live_preimage_changed"));
    }
    crate::dialog_log::structured_log::event(
        "warn",
        "memory",
        || "memory.live_replacement".to_owned(),
        || serde_json::json!({"original_path":path,"path":selected,"source_sha256":source_sha256,"reason":status.and_then(|status|status.refusal_code).unwrap_or_else(||"memory_not_utf8".to_owned()),"created":created,"original_preserved":true}),
    );
    Ok(selected)
}

/// Revalidate every selected source through a descriptor; health never calls this mutating path.
pub(super) fn read_selected(path: &Path, private: bool) -> io::Result<String> {
    if private {
        current_private(path)?;
    }
    let bytes = observed(path)?;
    let text = String::from_utf8(bytes).map_err(io::Error::other)?;
    let status = crate::memory::upgrade::inspect_memory_text(&text, true);
    if !status.compatible {
        return Err(io::Error::other("memory_live_active_incompatible"));
    }
    Ok(text)
}
