//! Real storage measurement and persisted auto-free-space consent (issue #494).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::dreaming::{
    apply_dreaming_plan, plan_memory_dreaming, DreamingConfig, DreamingOutcome, DreamingPlan,
};
use crate::memory::MemoryStore;

/// Capacity/free-space values measured for the filesystem containing memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageSnapshot {
    pub capacity_bytes: u64,
    pub free_bytes: u64,
}

/// Measure the filesystem that contains `memory_path`. For a not-yet-created
/// memory file the nearest existing ancestor is measured.
pub fn measure_storage(memory_path: &Path) -> io::Result<StorageSnapshot> {
    let measurement_path = existing_ancestor(memory_path);
    Ok(StorageSnapshot {
        capacity_bytes: fs2::total_space(&measurement_path)?,
        free_bytes: fs2::available_space(&measurement_path)?,
    })
}

/// The sidecar storing the user's explicit auto-free-space choice.
#[must_use]
pub fn auto_free_space_preference_path(memory_path: &Path) -> PathBuf {
    let filename = memory_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("formal-ai-memory.lino");
    memory_path.with_file_name(format!("{filename}.auto-free-space"))
}

/// The user's persisted auto-free-space decision.
///
/// Distinguishes "was never asked" from "was asked and declined" (issue #540
/// §4): a surface that prompts for consent must not re-prompt a user who
/// already said no.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoFreeSpaceChoice {
    /// No sidecar exists — the user has never been asked.
    NeverAsked,
    /// The user explicitly declined; do not re-prompt, do not delete.
    Declined,
    /// The user explicitly consented to automatic freeing.
    Enabled,
}

/// Read the persisted tri-state choice.
#[must_use]
pub fn auto_free_space_choice(memory_path: &Path) -> AutoFreeSpaceChoice {
    match fs::read_to_string(auto_free_space_preference_path(memory_path)) {
        Ok(value) if value.trim() == "enabled" => AutoFreeSpaceChoice::Enabled,
        Ok(value) if value.trim() == "disabled" => AutoFreeSpaceChoice::Declined,
        _ => AutoFreeSpaceChoice::NeverAsked,
    }
}

/// Read the persisted choice. Missing/invalid files mean "not enabled" so the
/// default can never silently delete data.
#[must_use]
pub fn auto_free_space_enabled(memory_path: &Path) -> bool {
    auto_free_space_choice(memory_path) == AutoFreeSpaceChoice::Enabled
}

/// Persist an explicit user choice atomically enough for this tiny sidecar.
pub fn persist_auto_free_space_choice(memory_path: &Path, enabled: bool) -> io::Result<()> {
    let preference = auto_free_space_preference_path(memory_path);
    if let Some(parent) = preference.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(preference, if enabled { "enabled\n" } else { "disabled\n" })
}

/// Build a pressure plan from the real filesystem and the exact size of the
/// caller's next write. Tests may still use `DreamingConfig` overrides directly.
pub fn plan_for_real_storage(
    store: &MemoryStore,
    memory_path: &Path,
    incoming_bytes: u64,
) -> io::Result<DreamingPlan> {
    let snapshot = measure_storage(memory_path)?;
    Ok(plan_memory_dreaming(
        store.events(),
        &DreamingConfig {
            storage_capacity_bytes: Some(snapshot.capacity_bytes),
            free_bytes: Some(snapshot.free_bytes),
            incoming_bytes,
            ..DreamingConfig::default()
        },
    ))
}

/// Apply only when the user has persisted consent. The caller supplies the
/// real next-write size, ensuring reclaiming is minimal and write-driven.
pub fn apply_auto_free_space_for_write(
    store: &mut MemoryStore,
    memory_path: &Path,
    incoming_bytes: u64,
) -> io::Result<Option<(DreamingPlan, DreamingOutcome)>> {
    if !auto_free_space_enabled(memory_path) {
        return Ok(None);
    }
    let snapshot = measure_storage(memory_path)?;
    Ok(apply_auto_free_space_with_snapshot(
        store,
        memory_path,
        incoming_bytes,
        snapshot,
    ))
}

/// The consent-gated freeing step with an explicit storage snapshot.
///
/// [`apply_auto_free_space_for_write`] measures the real filesystem and
/// delegates here; tests inject a synthetic snapshot to exercise the
/// stop-at-target behavior deterministically (issue #540 §4).
#[must_use]
pub fn apply_auto_free_space_with_snapshot(
    store: &mut MemoryStore,
    memory_path: &Path,
    incoming_bytes: u64,
    snapshot: StorageSnapshot,
) -> Option<(DreamingPlan, DreamingOutcome)> {
    if !auto_free_space_enabled(memory_path) {
        return None;
    }
    let plan = plan_memory_dreaming(
        store.events(),
        &DreamingConfig {
            storage_capacity_bytes: Some(snapshot.capacity_bytes),
            free_bytes: Some(snapshot.free_bytes),
            incoming_bytes,
            ..DreamingConfig::default()
        },
    );
    let outcome = apply_dreaming_plan(store, &plan);
    Some((plan, outcome))
}

fn existing_ancestor(path: &Path) -> PathBuf {
    let mut candidate = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };
    while !candidate.exists() {
        if !candidate.pop() {
            return PathBuf::from(".");
        }
    }
    candidate
}
