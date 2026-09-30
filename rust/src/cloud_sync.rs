//! Opt-in, vendor-neutral append-only memory synchronization (issue #669).
//!
//! Each remote object is one canonical event addressed by SHA-256. Transport
//! implementations must publish objects atomically and never replace an object.
//! The filesystem implementation is suitable for fixture remotes and mounted
//! user-owned storage. Git, WebDAV and S3 adapters can implement the same trait.
//! Local counters are not transferred: recalling an event must not create a new
//! remote event or overwrite another machine's usage accounting.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use sha2::{Digest, Sha256};
use crate::memory::{MemoryEvent, MemoryStore, export_links_notation, parse_links_notation};

const MAX_OBJECT_BYTES: u64 = 16 * 1024 * 1024;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncConfig {
    /// Explicit opt-in; constructing a backend alone never enables transfer.
    pub enabled: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncCursor {
    /// Only successfully acknowledged objects are remembered. Safe to discard:
    /// remote put-if-absent makes replay idempotent after a crash.
    pub uploaded: BTreeSet<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub pushed: usize,
    pub pulled: usize,
}

#[derive(Debug)]
pub enum SyncError {
    OptInRequired,
    Io(io::Error),
    InvalidKey(String),
    InvalidObject(String),
    IdentityConflict(String),
    ObjectTooLarge,
}
impl From<io::Error> for SyncError {
    fn from(error: io::Error) -> Self { Self::Io(error) }
}

/// Remote storage contract. `put_if_absent` must return true for a newly
/// published object, false for the same existing bytes, and error for different
/// bytes under the same key. `keys` may be eventually consistent: another sync
/// pass will pull objects published after this pass's listing.
pub trait Transport {
    fn keys(&mut self) -> Result<Vec<String>, SyncError>;
    fn read(&mut self, key: &str) -> Result<String, SyncError>;
    fn put_if_absent(&mut self, key: &str, contents: &str) -> Result<bool, SyncError>;
}

/// User-owned directory backend; no network, service or credentials required.
#[derive(Debug, Clone)]
pub struct DirectoryTransport { root: PathBuf }
impl DirectoryTransport {
    /// Does not create the remote or perform I/O before explicit sync opt-in.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    fn path(&self, key: &str) -> Result<PathBuf, SyncError> {
        validate_key(key)?;
        Ok(self.root.join(format!("{key}.lino")))
    }
}

impl Transport for DirectoryTransport {
    fn keys(&mut self) -> Result<Vec<String>, SyncError> {
        fs::create_dir_all(&self.root)?;
        let mut keys = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() { continue; }
            let name = entry.file_name();
            if let Some(key) = name.to_str().and_then(|name| name.strip_suffix(".lino")) {
                validate_key(key)?;
                keys.push(key.to_owned());
            }
        }
        keys.sort();
        Ok(keys)
    }
    fn read(&mut self, key: &str) -> Result<String, SyncError> {
        let path = self.path(key)?;
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(SyncError::InvalidObject(key.to_owned()));
        }
        if fs::metadata(&path)?.len() > MAX_OBJECT_BYTES { return Err(SyncError::ObjectTooLarge); }
        Ok(fs::read_to_string(path)?)
    }
    fn put_if_absent(&mut self, key: &str, contents: &str) -> Result<bool, SyncError> {
        let destination = self.path(key)?;
        if contents.len() as u64 > MAX_OBJECT_BYTES { return Err(SyncError::ObjectTooLarge); }
        if digest(contents) != key { return Err(SyncError::InvalidObject(key.to_owned())); }
        fs::create_dir_all(&self.root)?;
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = self.root.join(format!(".pending-{}-{sequence}", std::process::id()));
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        let result = (|| -> Result<bool, SyncError> {
            file.write_all(contents.as_bytes())?;
            file.sync_all()?;
            // Hard-link publication is atomic and cannot replace another writer's
            // object. Readers only enumerate final .lino names.
            match fs::hard_link(&temporary, &destination) {
                Ok(()) => Ok(true),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    if self.read(key)? == contents { Ok(false) }
                    else { Err(SyncError::InvalidObject(key.to_owned())) }
                }
                Err(error) => Err(error.into()),
            }
        })();
        drop(file);
        let _ = fs::remove_file(temporary);
        result
    }
}

fn validate_key(key: &str) -> Result<(), SyncError> {
    if key.len() == 64 && key.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        Ok(())
    } else { Err(SyncError::InvalidKey(key.to_owned())) }
}
fn digest(text: &str) -> String { format!("{:x}", Sha256::digest(text.as_bytes())) }
fn canonical(event: &MemoryEvent) -> MemoryEvent {
    let mut event = event.clone();
    event.access_count = 0;
    event.write_count = 1;
    event
}
fn encode(event: &MemoryEvent) -> String { export_links_notation(&[canonical(event)]) }
fn decode(key: &str, text: &str) -> Result<MemoryEvent, SyncError> {
    validate_key(key)?;
    if text.len() as u64 > MAX_OBJECT_BYTES { return Err(SyncError::ObjectTooLarge); }
    if digest(text) != key { return Err(SyncError::InvalidObject(key.to_owned())); }
    let events = parse_links_notation(text);
    if events.len() != 1 || events[0].id.is_empty() || encode(&events[0]) != text {
        return Err(SyncError::InvalidObject(key.to_owned()));
    }
    Ok(events.into_iter().next().expect("validated one event"))
}

/// Merge remote events without shrinking or reordering the local prefix. All
/// remote objects and identities are validated before any local append or push.
/// A transport failure can leave acknowledged immutable remote objects behind;
/// rerunning safely retries them and the local store remains unchanged.
pub fn sync_memory(
    config: &SyncConfig,
    local: &mut MemoryStore,
    transport: &mut impl Transport,
    cursor: &mut SyncCursor,
) -> Result<SyncReport, SyncError> {
    if !config.enabled { return Err(SyncError::OptInRequired); }
    let mut by_identity: BTreeMap<String, String> = BTreeMap::new();
    let mut outgoing: BTreeMap<String, String> = BTreeMap::new();
    for event in local.events() {
        if event.id.is_empty() { return Err(SyncError::InvalidObject(String::from("missing_event_id"))); }
        let canonical = canonical(event);
        let bytes = encode(&canonical);
        if let Some(previous) = by_identity.insert(canonical.id.clone(), bytes.clone()) {
            if previous != bytes { return Err(SyncError::IdentityConflict(canonical.id)); }
        }
        outgoing.insert(digest(&bytes), bytes);
    }
    let mut incoming = Vec::new();
    let mut keys = transport.keys()?;
    keys.sort();
    keys.dedup();
    let remote_keys: BTreeSet<String> = keys.iter().cloned().collect();
    for key in keys {
        let bytes = transport.read(&key)?;
        let event = decode(&key, &bytes)?;
        match by_identity.get(&event.id) {
            Some(previous) if previous != &bytes => return Err(SyncError::IdentityConflict(event.id)),
            Some(_) => {},
            None => {
                by_identity.insert(event.id.clone(), bytes);
                incoming.push(event);
            }
        }
    }
    let mut report = SyncReport::default();
    for (key, bytes) in outgoing {
        // Recheck remote presence even with a cursor: it may belong to a new or
        // restored remote. Cursor state is an optimization, never authority.
        if !remote_keys.contains(&key) && transport.put_if_absent(&key, &bytes)? {
            report.pushed += 1;
        }
        cursor.uploaded.insert(key);
    }
    report.pulled = incoming.len();
    local.import(&incoming);
    Ok(report)
}

/// A directory path is the supported backend today. URL transport selection is
/// left to the CLI/server integration rather than silently treating a URL as a
/// local directory or claiming that WebDAV/S3 authentication is implemented.
#[must_use]
pub fn directory_remote(path: &Path) -> DirectoryTransport { DirectoryTransport::new(path) }
