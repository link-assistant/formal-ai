use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::{
    LinkCliLinkStore, LinkStore, LinkStoreBackend, MemoryEvent, SyncStore,
    server_link_database_path, server_link_transition_log_path,
};

fn temporary_directory(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-issue-1069-{label}-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn link_cli_rolls_back_an_entire_memory_event_projection() {
    let directory = temporary_directory("rollback");
    std::fs::create_dir_all(&directory).expect("create test directory");
    let database = directory.join("memory.links");

    let mut store = LinkCliLinkStore::open_at(&database).expect("open link-cli store");
    assert_eq!(store.backend(), LinkStoreBackend::LinkCli);
    assert_eq!(store.native_link_count(), 0);

    store.begin_transaction().expect("begin transaction");
    store
        .append_memory_event(MemoryEvent::user("must be rolled back"))
        .expect("append inside transaction");
    assert!(store.native_link_count() > 0);
    store.rollback_transaction().expect("rollback transaction");

    assert_eq!(store.native_link_count(), 0);
    assert!(store.records().is_empty());
    assert!(store.export_memory_links_notation().contains("demo_memory"));

    drop(store);
    let reopened = LinkCliLinkStore::open_at(&database).expect("reopen link-cli store");
    assert_eq!(reopened.native_link_count(), 0);
    drop(reopened);
    std::fs::remove_dir_all(directory).expect("remove test directory");
}

#[test]
fn committed_link_cli_transaction_survives_reopen() {
    let directory = temporary_directory("commit");
    std::fs::create_dir_all(&directory).expect("create test directory");
    let database = directory.join("memory.links");

    let mut store = LinkCliLinkStore::open_at(&database).expect("open link-cli store");
    store.begin_transaction().expect("begin transaction");
    store
        .append_memory_event(MemoryEvent::assistant("persist through link-cli"))
        .expect("append inside transaction");
    let committed_count = store.native_link_count();
    store.commit_transaction().expect("commit transaction");
    drop(store);

    let reopened = LinkCliLinkStore::open_at(&database).expect("reopen link-cli store");
    assert_eq!(reopened.native_link_count(), committed_count);
    assert!(server_link_transition_log_path(&database).is_file());
    drop(reopened);
    std::fs::remove_dir_all(directory).expect("remove test directory");
}

#[test]
fn sync_store_routes_server_writes_through_persistent_link_cli_transactions() {
    let directory = temporary_directory("server");
    let memory_path = directory.join("memory.lino");
    let database = server_link_database_path(&memory_path);
    let transition_log = server_link_transition_log_path(&database);

    let mut store = SyncStore::open_at(&memory_path);
    let added = store
        .record_chat_exchange("remember the link-cli boundary", "stored transactionally")
        .expect("persist chat exchange");
    assert_eq!(added, 2);
    assert!(
        memory_path.is_file(),
        "Links Notation projection remains portable"
    );
    assert!(database.is_file(), "server owns a binary link-cli sidecar");
    assert!(
        transition_log.is_file(),
        "server mutations have a recovery log"
    );

    let native = LinkCliLinkStore::open_at(&database).expect("inspect server link store");
    assert!(native.native_link_count() > 0);
    let log = std::fs::read_to_string(transition_log).expect("read transaction log");
    assert!(
        log.contains("__transactions:commit:"),
        "server projection was committed"
    );
    drop(native);

    let reopened = SyncStore::open_at(&memory_path);
    assert_eq!(reopened.events().len(), 2);
    std::fs::remove_dir_all(directory).expect("remove test directory");
}
