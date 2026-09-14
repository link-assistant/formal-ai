//! Bring the native projection beside a memory file in line with its events.
//!
//! The HTTP server opens a fresh store per request, so how much of the memory
//! the projection already holds cannot live in a field: a marker beside the
//! database records it. When the marker proves a prefix, only the remaining
//! events are appended; otherwise the graph is rebuilt, which is correct
//! whatever the database contains. Issue #1106 measured the difference on a
//! 400-event store: 51.8 s per request rebuilding, 0.35 s appending.
//!
//! The marker is removed before the write and rewritten only after it
//! succeeds, so an interrupted synchronization leaves no claim standing and
//! `.lino` stays the deterministic recovery source it is documented to be.

use std::path::{Path, PathBuf};

use super::{LinkCliLinkStore, LinkStoreError};
use crate::memory::MemoryEvent;

/// The marker recording how many leading events the projection holds.
fn projection_marker_path(database: &Path) -> PathBuf {
    let mut name = database.as_os_str().to_os_string();
    name.push(".projected");
    PathBuf::from(name)
}

/// Make the projection at `database` hold exactly `events`.
///
/// Appending is an optimization, never a correctness requirement: a marker
/// that does not describe the database (a `.lino` replaced underneath it, a
/// half-written address map) makes the append refuse, and the rebuild runs
/// instead, reporting nothing and losing no write.
///
/// # Errors
///
/// Returns the backend error when neither path can bring the database in line.
pub fn synchronize_memory_events(
    database: &Path,
    events: &[MemoryEvent],
) -> Result<(), LinkStoreError> {
    let marker = projection_marker_path(database);
    let already_projected = std::fs::read_to_string(&marker)
        .ok()
        .and_then(|text| text.trim().parse::<usize>().ok())
        .filter(|projected| *projected <= events.len());
    let _ = std::fs::remove_file(&marker);
    let mut store = LinkCliLinkStore::open_at(database)?;
    let appended = already_projected.is_some_and(|projected| {
        store
            .append_memory_events_transactionally(events, projected)
            .is_ok()
    });
    if !appended {
        store.replace_memory_events_transactionally(events)?;
    }
    drop(store);
    crate::memory::write_locked_atomic(&marker, &format!("{}\n", events.len()))
        .map_err(|error| LinkStoreError::Backend(error.to_string()))
}
