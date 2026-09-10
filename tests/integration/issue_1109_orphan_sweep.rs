//! Opening a store removes replacement databases left by dead processes (#1109).
//!
//! A rebuild that is killed partway leaves a 67 MB `.<name>.database.<pid>.<n>.tmp`
//! beside the store, and nothing came back for it: 1.1 GB across seventeen dead
//! process ids beside one store. `open_at` now sweeps siblings whose process id
//! is not alive -- and only those, so a rebuild running in another process keeps
//! its files.

use formal_ai::link_store::LinkCliLinkStore;

#[test]
fn opening_a_store_removes_orphans_of_dead_processes_and_keeps_live_ones() {
    let dir = std::env::temp_dir().join(format!("formal-ai-issue-1109-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let database = dir.join("memory.links");

    // Above any pid_max in use (Linux 4194304, macOS 99998), so certainly dead.
    let dead = 4_194_305_u32;
    let live = std::process::id();
    let dead_files = [
        format!(".memory.links.database.{dead}.0.tmp"),
        format!(".memory.links.database.{dead}.0.tmp.lock"),
        format!(".memory.links.database.{dead}.0.transitions.links"),
    ];
    let live_files = [
        format!(".memory.links.database.{live}.3.tmp"),
        format!(".memory.links.database.{live}.3.tmp.lock"),
    ];
    for name in dead_files.iter().chain(live_files.iter()) {
        std::fs::write(dir.join(name), b"x").expect("write fixture");
    }
    // Not a replacement file at all: a different store's sibling stays.
    std::fs::write(dir.join(".other.links.database.1.0.tmp"), b"x").expect("write fixture");

    let store = LinkCliLinkStore::open_at(&database).expect("open");
    drop(store);

    for name in &dead_files {
        assert!(!dir.join(name).exists(), "{name} should have been swept");
    }
    for name in &live_files {
        assert!(
            dir.join(name).exists(),
            "{name} belongs to a live process and must stay"
        );
    }
    assert!(dir.join(".other.links.database.1.0.tmp").exists());
    let _ = std::fs::remove_dir_all(&dir);
}
