//! Side-effect-free persisted-memory compatibility checks and explicit upgrades.
//!
//! Released Formal AI versions before this contract wrote an unversioned
//! `demo_memory` document. That shape is schema 1. Schema 2 adds a root-level
//! `schema_version` marker that schema-1 readers ignore, so rollback can either
//! reopen the migrated file directly or restore its verified byte backup.

use std::fmt;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fs2::FileExt as _;
use serde::Serialize;

use super::ROOT_HEADER;

pub const MINIMUM_READABLE_MEMORY_SCHEMA_VERSION: u32 = 1;
pub const MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION: u32 = 2;
pub const TARGET_MEMORY_SCHEMA_VERSION: u32 = 2;
const V1_TO_V2_MIGRATION_ID: &str = "demo_memory_v1_to_v2";
const SCHEMA_MARKER: &str = "schema_version";

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct ExclusiveFileLock(fs::File);

impl Drop for ExclusiveFileLock {
    fn drop(&mut self) {
        // Explicitly release the advisory lock before closing the descriptor.
        // A concurrently forked child can briefly inherit the open file
        // description even though the descriptor is close-on-exec.
        let _ = fs2::FileExt::unlock(&self.0);
    }
}

/// Machine-readable state returned by preflight and `/health`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryMigrationState {
    Missing,
    Ready,
    UpgradeRequired,
    Incompatible,
}

/// Complete, side-effect-free compatibility report for one memory path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
// These booleans are independent fields in the stable JSON operator contract,
// not mutually exclusive internal state.
#[allow(clippy::struct_excessive_bools)]
pub struct MemoryUpgradeStatus {
    pub binary_version: String,
    pub path_exists: bool,
    pub detected_schema_version: Option<u32>,
    pub minimum_readable_schema_version: u32,
    pub maximum_readable_schema_version: u32,
    pub target_schema_version: u32,
    pub compatible: bool,
    pub migration_required: bool,
    pub migration_id: Option<String>,
    pub rollback_supported: bool,
    pub migration_state: MemoryMigrationState,
    pub event_count: Option<usize>,
    pub source_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_reason: Option<String>,
}

impl MemoryUpgradeStatus {
    fn base(path_exists: bool) -> Self {
        Self {
            binary_version: String::from(env!("CARGO_PKG_VERSION")),
            path_exists,
            detected_schema_version: None,
            minimum_readable_schema_version: MINIMUM_READABLE_MEMORY_SCHEMA_VERSION,
            maximum_readable_schema_version: MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION,
            target_schema_version: TARGET_MEMORY_SCHEMA_VERSION,
            compatible: true,
            migration_required: false,
            migration_id: None,
            rollback_supported: true,
            migration_state: if path_exists {
                MemoryMigrationState::Ready
            } else {
                MemoryMigrationState::Missing
            },
            event_count: Some(0),
            source_sha256: None,
            refusal_code: None,
            refusal_reason: None,
        }
    }

    fn incompatible(
        path_exists: bool,
        detected: Option<u32>,
        code: &str,
        reason: impl Into<String>,
    ) -> Self {
        let mut status = Self::base(path_exists);
        status.detected_schema_version = detected;
        status.compatible = false;
        status.rollback_supported = false;
        status.migration_state = MemoryMigrationState::Incompatible;
        status.event_count = None;
        status.refusal_code = Some(code.to_owned());
        status.refusal_reason = Some(reason.into());
        status
    }

    fn incompatible_bytes(
        bytes: &[u8],
        path_exists: bool,
        detected: Option<u32>,
        code: &str,
        reason: impl Into<String>,
    ) -> Self {
        let mut status = Self::incompatible(path_exists, detected, code, reason);
        status.source_sha256 = Some(sha256(bytes));
        status
    }
}

/// A durable record of the migration transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MemoryMigrationReceipt {
    pub binary_version: String,
    pub changed: bool,
    pub migration_id: Option<String>,
    pub from_schema_version: Option<u32>,
    pub to_schema_version: u32,
    pub memory_path: String,
    pub backup_path: Option<String>,
    pub receipt_path: Option<String>,
    pub original_sha256: Option<String>,
    pub migrated_sha256: Option<String>,
    pub event_count: usize,
    pub rollback_supported: bool,
    pub rollback_strategy: String,
}

/// Structured failure used by the CLI to emit a JSON refusal before exiting
/// non-zero.
#[derive(Debug)]
pub struct MemoryUpgradeError {
    code: &'static str,
    message: String,
    status: Option<Box<MemoryUpgradeStatus>>,
}

impl MemoryUpgradeError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            status: None,
        }
    }

    fn with_status(mut self, status: MemoryUpgradeStatus) -> Self {
        self.status = Some(Box::new(status));
        self
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn status(&self) -> Option<&MemoryUpgradeStatus> {
        self.status.as_deref()
    }
}

impl fmt::Display for MemoryUpgradeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MemoryUpgradeError {}

/// Inspect a memory path without creating the file, its parent, or a lock.
#[must_use]
pub fn preflight_memory_upgrade(path: &Path) -> MemoryUpgradeStatus {
    match fs::read(path) {
        Ok(bytes) => inspect_memory_bytes(&bytes, true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => MemoryUpgradeStatus::base(false),
        Err(error) => MemoryUpgradeStatus::incompatible(
            true,
            None,
            "memory_unreadable",
            format!("memory_unreadable:path={}:error={error}", path.display()),
        ),
    }
}

pub(crate) fn inspect_memory_text(text: &str, path_exists: bool) -> MemoryUpgradeStatus {
    inspect_memory_bytes(text.as_bytes(), path_exists)
}

pub(crate) fn schema_version_for_loaded_document(text: &str) -> Option<u32> {
    let status = inspect_memory_text(text, true);
    status.compatible.then_some(
        status
            .detected_schema_version
            .unwrap_or(TARGET_MEMORY_SCHEMA_VERSION),
    )
}

fn inspect_memory_bytes(bytes: &[u8], path_exists: bool) -> MemoryUpgradeStatus {
    let mut status = MemoryUpgradeStatus::base(path_exists);
    status.source_sha256 = Some(sha256(bytes));
    if bytes.is_empty() {
        // Released binaries create the shared file before the first event, so
        // an existing zero-byte file is a valid schema-1 empty store.
        status.detected_schema_version = Some(MINIMUM_READABLE_MEMORY_SCHEMA_VERSION);
        status.migration_required = TARGET_MEMORY_SCHEMA_VERSION > 1;
        status.migration_id = status
            .migration_required
            .then(|| String::from(V1_TO_V2_MIGRATION_ID));
        status.migration_state = if status.migration_required {
            MemoryMigrationState::UpgradeRequired
        } else {
            MemoryMigrationState::Ready
        };
        return status;
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return MemoryUpgradeStatus::incompatible_bytes(
            bytes,
            path_exists,
            None,
            "memory_not_utf8",
            "persisted memory is not valid UTF-8",
        );
    };
    let Some(first) = text.lines().next() else {
        return MemoryUpgradeStatus::incompatible_bytes(
            bytes,
            path_exists,
            None,
            "memory_empty",
            "persisted memory is empty",
        );
    };
    if first != ROOT_HEADER {
        return MemoryUpgradeStatus::incompatible_bytes(
            bytes,
            path_exists,
            None,
            "memory_header_unknown",
            format!("memory_header_unknown:expected={ROOT_HEADER}"),
        );
    }

    let mut detected = None;
    for line in text.lines() {
        let indent = line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        if indent != 2 {
            continue;
        }
        let content = &line[indent..];
        let Some(value) = content.strip_prefix("schema_version ") else {
            continue;
        };
        let Some(value) = parse_persisted_quoted(value) else {
            return MemoryUpgradeStatus::incompatible_bytes(
                bytes,
                path_exists,
                None,
                "schema_version_invalid",
                "schema_version must be a quoted positive integer",
            );
        };
        let Ok(version) = value.parse::<u32>() else {
            return MemoryUpgradeStatus::incompatible_bytes(
                bytes,
                path_exists,
                None,
                "schema_version_invalid",
                "schema_version must be a quoted positive integer",
            );
        };
        if detected.replace(version).is_some() {
            return MemoryUpgradeStatus::incompatible_bytes(
                bytes,
                path_exists,
                Some(version),
                "schema_version_duplicate",
                "persisted memory contains more than one schema_version marker",
            );
        }
    }
    let detected = detected.unwrap_or(MINIMUM_READABLE_MEMORY_SCHEMA_VERSION);
    status.detected_schema_version = Some(detected);
    if detected < MINIMUM_READABLE_MEMORY_SCHEMA_VERSION {
        return MemoryUpgradeStatus::incompatible_bytes(
            bytes,
            path_exists,
            Some(detected),
            "schema_too_old",
            format!(
                "schema_too_old:detected={detected}:minimum={MINIMUM_READABLE_MEMORY_SCHEMA_VERSION}"
            ),
        );
    }
    if detected > MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION {
        return MemoryUpgradeStatus::incompatible_bytes(
            bytes,
            path_exists,
            Some(detected),
            "schema_too_new",
            format!(
                "schema_too_new:detected={detected}:maximum={MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION}"
            ),
        );
    }
    let event_count = match validate_persisted_memory_text(text) {
        Ok(event_count) => event_count,
        Err(error) => {
            return MemoryUpgradeStatus::incompatible_bytes(
                bytes,
                path_exists,
                Some(detected),
                "memory_malformed",
                format!("memory_malformed:error={error}"),
            );
        }
    };
    status.event_count = Some(event_count);
    if detected < TARGET_MEMORY_SCHEMA_VERSION {
        status.migration_required = true;
        status.migration_id = Some(String::from(V1_TO_V2_MIGRATION_ID));
        status.migration_state = MemoryMigrationState::UpgradeRequired;
    }
    status
}

/// Validate the exact line-oriented format emitted by released memory writers.
///
/// Persisted `demo_memory` is intentionally not passed through the canonical
/// Links Notation parser here. Released binaries escaped quoted scalars with
/// C-style backslashes, while canonical Links Notation doubles delimiters and
/// treats backslashes as data. Rejecting the released writer's own output would
/// make otherwise healthy memories impossible to upgrade. This validator is
/// strict about the released document shape while accepting precisely the
/// quoted-scalar convention its reader and writer already use.
fn validate_persisted_memory_text(text: &str) -> Result<usize, String> {
    let mut event_count = 0;
    let mut inside_event = false;

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let indent = line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        let content = &line[indent..];
        match indent {
            0 if line_number == 1 && content == ROOT_HEADER => {}
            2 => {
                inside_event = false;
                let Some((key, value)) = content.split_once(' ') else {
                    return Err(format!("line={line_number}:root_value_missing"));
                };
                if key.is_empty() || parse_persisted_quoted(value).is_none() {
                    return Err(format!("line={line_number}:root_value_invalid"));
                }
                if key == "event" {
                    event_count += 1;
                    inside_event = true;
                }
            }
            4 if inside_event => {
                let Some((key, value)) = content.split_once(' ') else {
                    return Err(format!("line={line_number}:event_value_missing"));
                };
                if key.is_empty() || parse_persisted_quoted(value).is_none() {
                    return Err(format!("line={line_number}:event_value_invalid"));
                }
            }
            _ => return Err(format!("line={line_number}:indent_or_parent_invalid")),
        }
    }

    Ok(event_count)
}

/// Parse one released-writer quoted scalar and require no trailing tokens.
fn parse_persisted_quoted(value: &str) -> Option<String> {
    let value = value.trim_start();
    let bytes = value.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' if value[index + 1..].trim().is_empty() => {
                return super::parse_quoted(&value[..=index]);
            }
            b'"' => return None,
            _ => index += 1,
        }
    }
    None
}

/// Perform the supported migration while holding the shared writer lock.
pub fn migrate_memory(
    path: &Path,
    backup_path: Option<&Path>,
    receipt_path: Option<&Path>,
) -> Result<MemoryMigrationReceipt, MemoryUpgradeError> {
    migrate_memory_with_pre_commit(path, backup_path, receipt_path, |_| Ok(()))
}

/// Migration variant with a hook after staging/verification and before commit.
///
/// Orchestrators and tests can use the hook to simulate cancellation. A hook
/// error removes the staged candidate and leaves the original byte-identical;
/// the already-verified backup remains available for an idempotent retry.
pub fn migrate_memory_with_pre_commit<F>(
    path: &Path,
    backup_path: Option<&Path>,
    receipt_path: Option<&Path>,
    before_commit: F,
) -> Result<MemoryMigrationReceipt, MemoryUpgradeError>
where
    F: FnOnce(&Path) -> io::Result<()>,
{
    if path.as_os_str() == "-" {
        return Err(MemoryUpgradeError::new(
            "memory_path_invalid",
            "persisted-memory migration requires a filesystem path",
        ));
    }
    let lock_path = memory_lock_path(path);
    let lock_file = open_lock_file(&lock_path)?;
    lock_file.try_lock_exclusive().map_err(|error| {
        MemoryUpgradeError::new(
            "memory_locked",
            format!(
                "memory_locked:path={}:lock={}:error={error}",
                path.display(),
                lock_path.display()
            ),
        )
        .with_status(preflight_memory_upgrade(path))
    })?;
    let _lock = ExclusiveFileLock(lock_file);

    let original = fs::read(path).map_err(|error| {
        MemoryUpgradeError::new(
            "memory_unreadable",
            format!("memory_unreadable:path={}:error={error}", path.display()),
        )
    })?;
    let original_permissions = fs::metadata(path)
        .map_err(|error| {
            MemoryUpgradeError::new(
                "memory_metadata_unreadable",
                format!(
                    "memory_metadata_unreadable:path={}:error={error}",
                    path.display()
                ),
            )
        })?
        .permissions();
    let status = inspect_memory_bytes(&original, true);
    if !status.compatible {
        return Err(MemoryUpgradeError::new(
            "memory_incompatible",
            status
                .refusal_reason
                .clone()
                .unwrap_or_else(|| String::from("memory schema is incompatible")),
        )
        .with_status(status));
    }
    if !status.migration_required {
        return Ok(noop_receipt(path, &status));
    }
    if status.detected_schema_version != Some(1) {
        return Err(MemoryUpgradeError::new(
            "migration_unavailable",
            "no migration implementation matches the detected schema",
        )
        .with_status(status));
    }

    let original_sha256 = sha256(&original);
    let backup_path = backup_path.map_or_else(
        || default_backup_path(path, &original_sha256),
        Path::to_path_buf,
    );
    let receipt_path = receipt_path.map_or_else(|| default_receipt_path(path), Path::to_path_buf);
    validate_auxiliary_paths(path, &lock_path, &backup_path, &receipt_path)?;
    write_verified_backup(
        &backup_path,
        &original,
        &original_sha256,
        &original_permissions,
    )?;

    let migrated = migrate_v1_to_v2(&original)?;
    let migrated_status = inspect_memory_bytes(&migrated, true);
    if !migrated_status.compatible
        || migrated_status.detected_schema_version != Some(TARGET_MEMORY_SCHEMA_VERSION)
        || migrated_status.event_count != status.event_count
    {
        return Err(MemoryUpgradeError::new(
            "migration_validation_failed",
            "staged memory did not pass target-schema and event-count validation",
        )
        .with_status(migrated_status));
    }
    let migrated_sha256 = sha256(&migrated);
    let receipt = MemoryMigrationReceipt {
        binary_version: String::from(env!("CARGO_PKG_VERSION")),
        changed: true,
        migration_id: Some(String::from(V1_TO_V2_MIGRATION_ID)),
        from_schema_version: Some(1),
        to_schema_version: TARGET_MEMORY_SCHEMA_VERSION,
        memory_path: path.display().to_string(),
        backup_path: Some(backup_path.display().to_string()),
        receipt_path: Some(receipt_path.display().to_string()),
        original_sha256: Some(original_sha256),
        migrated_sha256: Some(migrated_sha256),
        event_count: status.event_count.unwrap_or_default(),
        rollback_supported: true,
        rollback_strategy: String::from("restore_backup_path"),
    };
    let receipt_bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| MemoryUpgradeError::new("receipt_serialize_failed", error.to_string()))?;
    let staged_path = temporary_path(path, "migration");
    write_new_file(&staged_path, &migrated, Some(&original_permissions)).map_err(|error| {
        MemoryUpgradeError::new(
            "migration_stage_failed",
            format!(
                "migration_stage_failed:path={}:error={error}",
                staged_path.display()
            ),
        )
    })?;
    let staged_receipt_path = temporary_path(&receipt_path, "receipt");
    if let Err(error) = write_new_file(
        &staged_receipt_path,
        &receipt_bytes,
        Some(&original_permissions),
    ) {
        let _ = fs::remove_file(&staged_path);
        return Err(MemoryUpgradeError::new(
            "receipt_stage_failed",
            format!(
                "receipt_stage_failed:path={}:error={error}",
                staged_receipt_path.display()
            ),
        ));
    }
    if let Err(error) = before_commit(&staged_path) {
        let _ = fs::remove_file(&staged_path);
        let _ = fs::remove_file(&staged_receipt_path);
        return Err(MemoryUpgradeError::new(
            "migration_interrupted",
            format!("migration_interrupted:error={error}"),
        )
        .with_status(status));
    }
    if let Err(error) = fs::rename(&staged_path, path) {
        let _ = fs::remove_file(&staged_path);
        let _ = fs::remove_file(&staged_receipt_path);
        return Err(MemoryUpgradeError::new(
            "migration_commit_failed",
            format!(
                "migration_commit_failed:path={}:error={error}",
                path.display()
            ),
        ));
    }
    if let Err(error) = sync_parent(path) {
        let _ = fs::remove_file(&staged_receipt_path);
        return Err(MemoryUpgradeError::new(
            "migration_sync_failed",
            format!(
                "migration_sync_failed:path={}:error={error}",
                path.display()
            ),
        ));
    }
    fs::rename(&staged_receipt_path, &receipt_path).map_err(|error| {
        let _ = fs::remove_file(&staged_receipt_path);
        MemoryUpgradeError::new(
            "receipt_write_failed",
            format!("receipt_write_failed:error={error}"),
        )
    })?;
    sync_parent(&receipt_path).map_err(|error| {
        MemoryUpgradeError::new(
            "receipt_sync_failed",
            format!(
                "receipt_sync_failed:path={}:error={error}",
                receipt_path.display()
            ),
        )
    })?;
    Ok(receipt)
}

fn noop_receipt(path: &Path, status: &MemoryUpgradeStatus) -> MemoryMigrationReceipt {
    MemoryMigrationReceipt {
        binary_version: String::from(env!("CARGO_PKG_VERSION")),
        changed: false,
        migration_id: None,
        from_schema_version: status.detected_schema_version,
        to_schema_version: status
            .detected_schema_version
            .unwrap_or(TARGET_MEMORY_SCHEMA_VERSION),
        memory_path: path.display().to_string(),
        backup_path: None,
        receipt_path: None,
        original_sha256: status.source_sha256.clone(),
        migrated_sha256: status.source_sha256.clone(),
        event_count: status.event_count.unwrap_or_default(),
        rollback_supported: true,
        rollback_strategy: String::from("not_required"),
    }
}

fn migrate_v1_to_v2(original: &[u8]) -> Result<Vec<u8>, MemoryUpgradeError> {
    if original.is_empty() {
        let mut migrated = String::from(ROOT_HEADER);
        migrated.push('\n');
        migrated.push_str("  ");
        migrated.push_str(SCHEMA_MARKER);
        migrated.push_str(" \"");
        migrated.push_str(&TARGET_MEMORY_SCHEMA_VERSION.to_string());
        migrated.push_str("\"\n");
        return Ok(migrated.into_bytes());
    }
    let text = std::str::from_utf8(original)
        .map_err(|_| MemoryUpgradeError::new("memory_not_utf8", "memory is not UTF-8"))?;
    let suffix = text.strip_prefix(ROOT_HEADER).ok_or_else(|| {
        MemoryUpgradeError::new(
            "memory_header_unknown",
            "memory root header is not recognized",
        )
    })?;
    let (newline, remainder) = if let Some(rest) = suffix.strip_prefix("\r\n") {
        ("\r\n", rest)
    } else if let Some(rest) = suffix.strip_prefix('\n') {
        ("\n", rest)
    } else if suffix.is_empty() {
        ("\n", "")
    } else {
        return Err(MemoryUpgradeError::new(
            "memory_header_invalid",
            "memory root header must occupy its own line",
        ));
    };
    let mut migrated = String::from(ROOT_HEADER);
    migrated.push_str(newline);
    migrated.push_str("  ");
    migrated.push_str(SCHEMA_MARKER);
    migrated.push_str(" \"");
    migrated.push_str(&TARGET_MEMORY_SCHEMA_VERSION.to_string());
    migrated.push('"');
    migrated.push_str(newline);
    migrated.push_str(remainder);
    Ok(migrated.into_bytes())
}

fn memory_lock_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("memory.lino");
    path.with_file_name(format!("{file_name}.lock"))
}

fn open_lock_file(path: &Path) -> Result<fs::File, MemoryUpgradeError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            MemoryUpgradeError::new(
                "lock_open_failed",
                format!("lock_parent_create_failed:error={error}"),
            )
        })?;
    }
    fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .map_err(|error| {
            MemoryUpgradeError::new(
                "lock_open_failed",
                format!("lock_open_failed:path={}:error={error}", path.display()),
            )
        })
}

fn default_backup_path(path: &Path, digest: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("memory.lino");
    path.with_file_name(format!("{file_name}.schema-1.{}.backup", &digest[..12]))
}

fn default_receipt_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("memory.lino");
    path.with_file_name(format!("{file_name}.upgrade-receipt.json"))
}

fn validate_auxiliary_paths(
    memory: &Path,
    lock: &Path,
    backup: &Path,
    receipt: &Path,
) -> Result<(), MemoryUpgradeError> {
    if backup == memory
        || backup == lock
        || receipt == memory
        || receipt == lock
        || backup == receipt
    {
        return Err(MemoryUpgradeError::new(
            "migration_path_collision",
            "memory, lock, backup, and receipt paths must be distinct",
        ));
    }
    Ok(())
}

fn write_verified_backup(
    path: &Path,
    original: &[u8],
    expected_sha256: &str,
    permissions: &fs::Permissions,
) -> Result<(), MemoryUpgradeError> {
    if path.exists() {
        let existing = fs::read(path).map_err(|error| {
            MemoryUpgradeError::new(
                "backup_read_failed",
                format!("backup_read_failed:path={}:error={error}", path.display()),
            )
        })?;
        if sha256(&existing) != expected_sha256 || existing != original {
            return Err(MemoryUpgradeError::new(
                "backup_conflict",
                format!("backup_conflict:path={}", path.display()),
            ));
        }
        return Ok(());
    }
    write_atomic_with_permissions(path, original, Some(permissions)).map_err(|error| {
        MemoryUpgradeError::new(
            "backup_write_failed",
            format!("backup_write_failed:path={}:error={error}", path.display()),
        )
    })?;
    let verified = fs::read(path).map_err(|error| {
        MemoryUpgradeError::new(
            "backup_read_failed",
            format!("backup_read_failed:path={}:error={error}", path.display()),
        )
    })?;
    if verified != original || sha256(&verified) != expected_sha256 {
        return Err(MemoryUpgradeError::new(
            "backup_verification_failed",
            format!("backup_verification_failed:path={}", path.display()),
        ));
    }
    Ok(())
}

fn write_atomic_with_permissions(
    path: &Path,
    bytes: &[u8],
    permissions: Option<&fs::Permissions>,
) -> io::Result<()> {
    let staged = temporary_path(path, "atomic");
    write_new_file(&staged, bytes, permissions)?;
    if let Err(error) = fs::rename(&staged, path) {
        let _ = fs::remove_file(&staged);
        return Err(error);
    }
    sync_parent(path)
}

fn write_new_file(
    path: &Path,
    bytes: &[u8],
    permissions: Option<&fs::Permissions>,
) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions.clone())?;
    }
    file.write_all(bytes)?;
    file.sync_all()
}

fn temporary_path(path: &Path, purpose: &str) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("memory.lino");
    path.with_file_name(format!(
        ".{file_name}.{purpose}.{}.{}",
        std::process::id(),
        sequence
    ))
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    crate::source_fetch::sha256_hex(bytes)
}
