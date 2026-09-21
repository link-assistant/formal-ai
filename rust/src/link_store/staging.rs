//! Staging and publishing of a replacement link-cli database.
//!
//! A rebuild does not mutate the database being served. It builds a whole new
//! one under a process-unique name and publishes it with a `rename`, so the
//! served path is either the old graph or the new one and never a half-applied
//! mixture. These are the pieces that make that possible: naming the scratch
//! files, replacing one file with another where `rename` alone will not do,
//! sweeping the scratch away afterwards, and staging the replacement without
//! the per-append durability that only the served database needs.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{NativeLinkCliTransactions, server_link_transition_log_path};

/// Stop `fsync`ing every transition written to a *staged* replacement.
///
/// `FileTransitionLog` syncs each append so the write-ahead ordering of the
/// transactions layer survives a crash: a transition is durable before the
/// data-store write it describes. That guarantee is about recovering the
/// database being served. A replacement built by
/// [`LinkCliLinkStore::replace_memory_events_transactionally`] is not that
/// database. It is a scratch file under a process-unique name that is either
/// published whole by `rename` or deleted; a crash before the rename leaves it
/// orphaned and `sweep_dead_replacements` collects it, so no reader ever
/// replays its log. Per-entry syncing there buys nothing and costs one `fsync`
/// per doublet, which is what made a rebuild superlinear (issue #1106: 21
/// events cost 3.5 s, 56 events 13.6 s, while an append of the same events
/// costs 30 ms).
///
/// The staged log is still `fsync`ed once, by `flush_log` before publishing, so
/// what reaches the served path is exactly as durable as before.
#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub(super) fn stage_without_per_entry_sync(transactions: &mut NativeLinkCliTransactions) {
    transactions.log_store_mut().set_sync_on_append(false);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub(super) fn replacement_path(database: &Path, label: &str) -> PathBuf {
    static NEXT_REPLACEMENT: AtomicU64 = AtomicU64::new(0);
    let sequence = NEXT_REPLACEMENT.fetch_add(1, Ordering::Relaxed);
    let filename = database
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("memory.links");
    database.with_file_name(format!(
        ".{filename}.{label}.{}.{}.tmp",
        std::process::id(),
        sequence
    ))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub(super) fn replace_file(staged: &Path, destination: &Path) -> std::io::Result<()> {
    match std::fs::rename(staged, destination) {
        Ok(()) => Ok(()),
        Err(_rename_error) if destination.exists() => {
            // `rename` replaces atomically on Unix. Windows requires moving
            // the destination aside first, so retain a recoverable backup if
            // publishing the staged file fails.
            let backup = replacement_path(destination, "backup");
            std::fs::rename(destination, &backup)?;
            match std::fs::rename(staged, destination) {
                Ok(()) => {
                    let _ = std::fs::remove_file(backup);
                    Ok(())
                }
                Err(error) => {
                    let _ = std::fs::rename(&backup, destination);
                    Err(error)
                }
            }
        }
        Err(error) => Err(error),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub(super) fn cleanup_link_cli_files(database: &Path) {
    let _ = std::fs::remove_file(server_link_transition_log_path(database));
    let _ = std::fs::remove_file(link_cli::lock_file_path(database));
    let _ = std::fs::remove_file(database);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub(super) fn link_cli_debug_enabled() -> bool {
    std::env::var("FORMAL_AI_LINK_CLI_DEBUG").as_deref() == Ok("1")
}
