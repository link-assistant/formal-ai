//! Issue #669: interleaved stores converge, repeated sync is idempotent, privacy opt-in.
use formal_ai::cloud_sync::{DirectoryTransport, SyncConfig, SyncCursor, SyncError, Transport, sync_memory};
use formal_ai::memory::{MemoryEvent, MemoryStore};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("formal-ai-memory-sync-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))))
    }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
fn event(id: &str, text: &str) -> MemoryEvent {
    MemoryEvent { id: id.to_owned(), content: Some(text.to_owned()), ..MemoryEvent::default() }
}
fn projected(store: &MemoryStore) -> BTreeMap<String, Option<String>> {
    store.events().iter().map(|event| (event.id.clone(), event.content.clone())).collect()
}
#[test]
fn memory_sync_interleaved_appends_converge_and_preserve_prefix() {
    let fixture = Fixture::new();
    let mut remote = DirectoryTransport::new(&fixture.0);
    let config = SyncConfig { enabled: true };
    let mut a = MemoryStore::from_events(vec![event("a1", "first")]);
    let mut b = MemoryStore::from_events(vec![event("b1", "second")]);
    let mut ca = SyncCursor::default(); let mut cb = SyncCursor::default();
    sync_memory(&config, &mut a, &mut remote, &mut ca).unwrap();
    sync_memory(&config, &mut b, &mut remote, &mut cb).unwrap();
    a.append(event("a2", "third"));
    let prefix = a.events().to_vec();
    sync_memory(&config, &mut a, &mut remote, &mut ca).unwrap();
    assert_eq!(&a.events()[..prefix.len()], prefix.as_slice());
    b.append(event("b2", "fourth"));
    sync_memory(&config, &mut b, &mut remote, &mut cb).unwrap();
    sync_memory(&config, &mut a, &mut remote, &mut ca).unwrap();
    assert_eq!(projected(&a), projected(&b));
    let report = sync_memory(&config, &mut a, &mut remote, &mut ca).unwrap();
    assert_eq!(report.pushed, 0); assert_eq!(report.pulled, 0);
}
#[test]
fn memory_sync_disabled_does_not_touch_remote() {
    let fixture = Fixture::new();
    let mut remote = DirectoryTransport::new(&fixture.0);
    assert!(matches!(sync_memory(&SyncConfig::default(), &mut MemoryStore::new(), &mut remote, &mut SyncCursor::default()), Err(SyncError::OptInRequired)));
    assert!(!fixture.0.exists());
}
#[test]
fn memory_sync_rejects_conflicting_ids_before_local_mutation() {
    let fixture = Fixture::new(); let mut remote = DirectoryTransport::new(&fixture.0);
    let config = SyncConfig { enabled: true };
    let mut a = MemoryStore::from_events(vec![event("same", "a")]);
    sync_memory(&config, &mut a, &mut remote, &mut SyncCursor::default()).unwrap();
    let mut b = MemoryStore::from_events(vec![event("same", "b")]);
    let before = b.events().to_vec();
    assert!(matches!(sync_memory(&config, &mut b, &mut remote, &mut SyncCursor::default()), Err(SyncError::IdentityConflict(_))));
    assert_eq!(b.events(), before.as_slice());
}
#[test]
fn memory_sync_rejects_path_traversal() {
    let fixture = Fixture::new(); let mut remote = DirectoryTransport::new(&fixture.0);
    assert!(matches!(remote.read("../secret"), Err(SyncError::InvalidKey(_))));
    assert!(!fixture.0.exists());
}
