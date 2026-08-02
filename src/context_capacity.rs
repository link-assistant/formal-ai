use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::storage_policy::measure_storage;

pub const DEFAULT_AVG_UTF8_BYTES_PER_CHAR: u64 = 2;
pub const AVG_UTF8_BYTES_PER_CHAR_ENV: &str = "FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR";

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ContextCapacity {
    pub context_window_tokens: u64,
    pub context_used_tokens: u64,
    pub context_used_fraction: f64,
    pub disk_free_bytes: u64,
    pub memory_used_bytes: u64,
    pub avg_utf8_bytes_per_char: u64,
}

impl ContextCapacity {
    pub fn current() -> io::Result<Self> {
        let memory_path = context_memory_path();
        let disk_free_bytes = measure_storage(&memory_path)?.free_bytes;
        let memory_used_bytes = memory_used_bytes(&memory_path)?;
        Ok(Self::from_bytes(
            disk_free_bytes,
            memory_used_bytes,
            avg_utf8_bytes_per_char(),
        ))
    }

    #[must_use]
    pub fn from_bytes(
        disk_free_bytes: u64,
        memory_used_bytes: u64,
        avg_utf8_bytes_per_char: u64,
    ) -> Self {
        let average = avg_utf8_bytes_per_char.max(1);
        let context_window_tokens = disk_free_bytes / average;
        let context_used_tokens = memory_used_bytes / average;
        // This public ratio is an estimate, so f64 precision is sufficient even
        // when filesystem counts exceed its exactly represented integer range.
        #[allow(clippy::cast_precision_loss)]
        let context_used_fraction = if context_window_tokens == 0 {
            0.0
        } else {
            context_used_tokens as f64 / context_window_tokens as f64
        };
        Self {
            context_window_tokens,
            context_used_tokens,
            context_used_fraction,
            disk_free_bytes,
            memory_used_bytes,
            avg_utf8_bytes_per_char: average,
        }
    }
}

#[must_use]
pub fn avg_utf8_bytes_per_char() -> u64 {
    std::env::var(AVG_UTF8_BYTES_PER_CHAR_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_AVG_UTF8_BYTES_PER_CHAR)
}

#[must_use]
pub fn context_memory_path() -> PathBuf {
    crate::shared_memory::shared_memory_path()
}

fn memory_used_bytes(path: &Path) -> io::Result<u64> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if !metadata.is_dir() {
        return Ok(0);
    }
    memory_directory_bytes(path)
}

fn memory_directory_bytes(path: &Path) -> io::Result<u64> {
    let mut total = 0_u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let entry_path = entry.path();
        if file_type.is_dir() {
            total = total.saturating_add(memory_directory_bytes(&entry_path)?);
        } else if file_type.is_file() && is_memory_log(&entry_path) {
            total = total.saturating_add(entry.metadata()?.len());
        }
    }
    Ok(total)
}

fn is_memory_log(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "lino")
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains("event-log"))
}
