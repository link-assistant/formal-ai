//! Sweep replacement databases left behind by processes that no longer exist.
//!
//! `replace_memory_events_transactionally` builds its replacement at a
//! process-id-scoped sibling path and removes it on every path it can reach --
//! but a process that is killed mid-rebuild reaches none of them, and nothing
//! ever returned for the 67 MB file it left. Issue #1109 found 1.1 GB of these
//! beside one store, across seventeen dead process ids.
//!
//! The sweep runs when a store is opened, before its lock is taken: that is the
//! one moment the store is known to be quiescent, and it is the same point at
//! which the transition log is already repaired. A sibling is removed only when
//! its embedded process id is not alive, so a rebuild in progress in another
//! process keeps its files.

use std::path::Path;

/// Remove `.<name>.<label>.<pid>.<n>.tmp*` siblings whose `<pid>` is dead.
///
/// Returns how many files were removed. Errors are deliberately swallowed: a
/// sweep that cannot remove a file leaves exactly the situation it found, and
/// the open that follows does not depend on it.
pub(super) fn sweep_dead_replacements(database: &Path) -> usize {
    let (Some(parent), Some(name)) = (database.parent(), database.file_name()) else {
        return 0;
    };
    let Some(name) = name.to_str() else {
        return 0;
    };
    let prefix = format!(".{name}.");
    let Ok(entries) = std::fs::read_dir(if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    }) else {
        return 0;
    };
    let mut removed = 0;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let Some(rest) = file_name.strip_prefix(&prefix) else {
            continue;
        };
        // `<label>.<pid>.<n>.tmp`, optionally followed by `.lock`, or the
        // replacement's own transition log `<label>.<pid>.<n>.transitions.links`.
        let mut parts = rest.split('.');
        let (Some(_label), Some(pid), Some(_sequence)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let Ok(pid) = pid.parse::<u32>() else {
            continue;
        };
        if pid == std::process::id() || process_is_alive(pid) {
            continue;
        }
        if std::fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Whether a process with this id exists, without a `libc` dependency.
fn process_is_alive(pid: u32) -> bool {
    if cfg!(target_os = "linux") {
        return Path::new("/proc").join(pid.to_string()).exists();
    }
    // `kill -0` reports existence without signalling. EPERM also means the
    // process exists, so anything but a clean "no such process" keeps the file:
    // the sweep only ever removes what it is sure about.
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output()
            .is_ok_and(|output| {
                output.status.success()
                    || !String::from_utf8_lossy(&output.stderr).contains("No such process")
            })
    }
    #[cfg(not(unix))]
    {
        true
    }
}
