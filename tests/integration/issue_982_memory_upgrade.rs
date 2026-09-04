use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;

use formal_ai::{
    MemoryEvent, MemoryStore, SyncStore, export_memory_links_notation, handle_api_request,
    migrate_memory_with_pre_commit,
};
use fs2::FileExt as _;

fn fixture_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-982-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("create fixture directory");
    path
}

fn run_memory_command(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .arg("memory")
        .args(arguments)
        .output()
        .expect("run formal-ai memory command")
}

const fn released_fixture() -> &'static str {
    include_str!("../fixtures/memory/schema-1.lino")
}

#[test]
fn fixtures_cover_every_readable_schema() {
    let fixtures = [
        (
            1_u64,
            include_bytes!("../fixtures/memory/schema-1.lino").as_slice(),
        ),
        (
            2_u64,
            include_bytes!("../fixtures/memory/schema-2.lino").as_slice(),
        ),
    ];

    for (expected_schema, fixture) in fixtures {
        let dir = fixture_dir(&format!("schema-{expected_schema}"));
        let memory_path = dir.join("memory.lino");
        std::fs::write(&memory_path, fixture).expect("write schema fixture");
        let output = run_memory_command(&[
            "upgrade-status",
            "--path",
            memory_path.to_str().expect("memory path"),
            "--format",
            "json",
        ]);
        assert!(output.status.success());
        let status: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("fixture status JSON");
        assert_eq!(status["detected_schema_version"], expected_schema);
        assert_eq!(status["compatible"], true);
        assert_eq!(
            status["migration_required"],
            expected_schema < status["target_schema_version"].as_u64().expect("target")
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn upgrade_status_detects_released_schema_without_mutating_memory() {
    let dir = fixture_dir("preflight");
    let memory_path = dir.join("memory.lino");
    let released = concat!(
        "demo_memory\n",
        "  event \"event-1\"\n",
        "    role \"user\"\n",
        "    content \"keep me\"\n",
        "    futureField \"preserve me\"\n",
    );
    std::fs::write(&memory_path, released).expect("write released fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "memory",
            "upgrade-status",
            "--path",
            memory_path.to_str().expect("utf-8 fixture path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run upgrade preflight");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let status: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("preflight stdout must be JSON");
    assert_eq!(status["detected_schema_version"], 1);
    assert_eq!(status["minimum_readable_schema_version"], 1);
    assert_eq!(status["maximum_readable_schema_version"], 2);
    assert_eq!(status["target_schema_version"], 2);
    assert_eq!(status["compatible"], true);
    assert_eq!(status["migration_required"], true);
    assert_eq!(status["migration_id"], "demo_memory_v1_to_v2");
    assert_eq!(status["rollback_supported"], true);
    assert_eq!(status["migration_state"], "upgrade_required");
    assert_eq!(
        std::fs::read_to_string(&memory_path).expect("read after preflight"),
        released,
        "preflight must be side-effect-free"
    );
    assert_eq!(
        std::fs::read_dir(&dir).expect("list fixture dir").count(),
        1,
        "preflight must not create a lock, backup, temp file, or receipt"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn released_writer_escapes_are_valid_upgrade_input_and_round_trip_losslessly() {
    let dir = fixture_dir("released-escapes");
    let memory_path = dir.join("memory.lino");
    let backup_path = dir.join("rollback.lino");
    let inputs = concat!(
        "{\"command\":\"set -eu\\n",
        "printf '%s\\\\n' \\\"migration canary\\\"\",\"description\":\"\"}"
    );
    let released = export_memory_links_notation(&[MemoryEvent {
        id: String::from("released-tool-event"),
        kind: Some(String::from("tool_call")),
        inputs: Some(String::from(inputs)),
        content: Some(String::from("quotes: \\\"double\\\" and \\ slash")),
        ..MemoryEvent::default()
    }]);
    std::fs::write(&memory_path, &released).expect("write released output");

    let status = formal_ai::preflight_memory_upgrade(&memory_path);
    assert!(status.compatible, "{status:?}");
    assert_eq!(status.detected_schema_version, Some(1));
    assert_eq!(status.event_count, Some(1));

    formal_ai::migrate_memory(&memory_path, Some(&backup_path), None)
        .expect("migrate released escaped scalars");
    assert_eq!(
        std::fs::read_to_string(&backup_path).expect("read rollback backup"),
        released
    );
    let migrated = std::fs::read_to_string(&memory_path).expect("read migrated memory");
    assert_eq!(
        migrated,
        released.replacen("demo_memory\n", "demo_memory\n  schema_version \"2\"\n", 1)
    );
    let loaded = MemoryStore::load_from_file(&memory_path).expect("load migrated memory");
    assert_eq!(loaded.events()[0].inputs.as_deref(), Some(inputs));
    assert_eq!(
        loaded.events()[0].content.as_deref(),
        Some("quotes: \\\"double\\\" and \\ slash")
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn upgrade_status_for_missing_path_creates_nothing() {
    let root = std::env::temp_dir().join(format!(
        "formal-ai-issue-982-missing-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let memory_path = root.join("nested/memory.lino");

    let output = run_memory_command(&[
        "upgrade-status",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--format",
        "json",
    ]);
    assert!(output.status.success());
    let status: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("missing status JSON");
    assert_eq!(status["path_exists"], false);
    assert_eq!(status["compatible"], true);
    assert_eq!(status["migration_required"], false);
    assert_eq!(status["migration_state"], "missing");
    assert!(!root.exists(), "preflight must not create the parent tree");
}

#[test]
fn released_zero_byte_store_is_readable_and_upgradeable() {
    let dir = fixture_dir("empty-released-store");
    let memory_path = dir.join("memory.lino");
    let backup_path = dir.join("rollback.lino");
    std::fs::write(&memory_path, []).expect("write released empty file");

    let status = formal_ai::preflight_memory_upgrade(&memory_path);
    assert_eq!(status.detected_schema_version, Some(1));
    assert_eq!(status.event_count, Some(0));
    assert!(status.compatible);
    assert!(status.migration_required);
    let store =
        MemoryStore::load_from_file(&memory_path).expect("released empty store remains readable");
    assert!(store.is_empty());
    assert_eq!(store.export_links_notation(), "demo_memory\n");

    let receipt = formal_ai::migrate_memory(&memory_path, Some(&backup_path), None)
        .expect("upgrade released empty store");
    assert!(receipt.changed);
    assert!(
        std::fs::read(&backup_path)
            .expect("read empty backup")
            .is_empty()
    );
    assert_eq!(
        std::fs::read_to_string(&memory_path).expect("read migrated empty store"),
        "demo_memory\n  schema_version \"2\"\n"
    );
}

#[test]
fn migration_is_atomic_lossless_receipted_idempotent_and_rollback_safe() {
    let dir = fixture_dir("whole-flow");
    let memory_path = dir.join("memory.lino");
    let backup_path = dir.join("rollback.lino");
    let receipt_path = dir.join("receipt.json");
    let export_path = dir.join("exported.lino");
    let released = released_fixture();
    std::fs::write(&memory_path, released).expect("write released fixture");
    #[cfg(unix)]
    std::fs::set_permissions(&memory_path, std::fs::Permissions::from_mode(0o600))
        .expect("make fixture private");

    let output = run_memory_command(&[
        "migrate",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--backup",
        backup_path.to_str().expect("backup path"),
        "--receipt",
        receipt_path.to_str().expect("receipt path"),
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("migration stdout must be JSON");
    assert_eq!(receipt["changed"], true);
    assert_eq!(receipt["migration_id"], "demo_memory_v1_to_v2");
    assert_eq!(receipt["from_schema_version"], 1);
    assert_eq!(receipt["to_schema_version"], 2);
    assert_eq!(receipt["event_count"], 2);
    assert_eq!(receipt["rollback_supported"], true);
    assert_eq!(
        std::fs::read_to_string(&backup_path).expect("read verified backup"),
        released,
        "rollback backup must contain the exact original bytes"
    );
    #[cfg(unix)]
    {
        assert_eq!(
            std::fs::metadata(&memory_path)
                .expect("migrated metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600,
            "atomic replacement must preserve memory permissions"
        );
        assert_eq!(
            std::fs::metadata(&backup_path)
                .expect("backup metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600,
            "rollback backup must preserve memory permissions"
        );
    }
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(&receipt_path).expect("read durable receipt")
        )
        .expect("receipt file must be JSON"),
        receipt
    );

    let migrated = std::fs::read_to_string(&memory_path).expect("read migrated memory");
    assert_eq!(
        migrated,
        released.replacen("demo_memory\n", "demo_memory\n  schema_version \"2\"\n", 1,),
        "migration may only add the schema marker"
    );
    let loaded = MemoryStore::load_from_file(&memory_path).expect("candidate loads migrated file");
    assert_eq!(
        loaded
            .events()
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>(),
        ["event-1", "event-2"],
        "event identifiers and ordering must survive"
    );
    assert_eq!(
        loaded.events()[0].unknown_fields,
        [(String::from("futureField"), String::from("preserve me"))]
    );
    assert_eq!(loaded.events()[1].evidence, ["source-a", "source-b"]);

    let query = run_memory_command(&[
        "query",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--prompt",
        "recall keep me",
    ]);
    assert!(
        query.status.success(),
        "query stderr: {}",
        String::from_utf8_lossy(&query.stderr)
    );
    let export = run_memory_command(&[
        "export",
        "--from",
        memory_path.to_str().expect("memory path"),
        "--path",
        export_path.to_str().expect("export path"),
        "--events-only",
    ]);
    assert!(
        export.status.success(),
        "export stderr: {}",
        String::from_utf8_lossy(&export.stderr)
    );
    let exported = std::fs::read_to_string(&export_path).expect("read candidate export");
    assert!(exported.starts_with("demo_memory\n  schema_version \"2\"\n"));
    assert!(exported.contains("futureField \"preserve me\""));
    assert!(exported.contains("evidence \"source-a|source-b\""));
    assert!(exported.find("event \"event-1\"") < exported.find("event \"event-2\""));

    let before_retry = std::fs::read(&memory_path).expect("memory before retry");
    let retry = run_memory_command(&[
        "migrate",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--backup",
        backup_path.to_str().expect("backup path"),
        "--receipt",
        receipt_path.to_str().expect("receipt path"),
        "--format",
        "json",
    ]);
    assert!(retry.status.success());
    let retry_receipt: serde_json::Value =
        serde_json::from_slice(&retry.stdout).expect("retry stdout must be JSON");
    assert_eq!(retry_receipt["changed"], false);
    assert_eq!(
        std::fs::read(&memory_path).expect("memory after retry"),
        before_retry,
        "a second migration must be a no-op"
    );

    std::fs::copy(&backup_path, &memory_path).expect("restore rollback backup");
    let rollback = run_memory_command(&[
        "upgrade-status",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--format",
        "json",
    ]);
    assert!(rollback.status.success());
    let rollback_status: serde_json::Value =
        serde_json::from_slice(&rollback.stdout).expect("rollback status JSON");
    assert_eq!(rollback_status["detected_schema_version"], 1);
    assert_eq!(rollback_status["event_count"], 2);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn interrupted_migration_keeps_original_byte_identical_and_retryable() {
    let dir = fixture_dir("interrupted");
    let memory_path = dir.join("memory.lino");
    let backup_path = dir.join("rollback.lino");
    let receipt_path = dir.join("receipt.json");
    let released = released_fixture().as_bytes();
    std::fs::write(&memory_path, released).expect("write released fixture");

    let error = migrate_memory_with_pre_commit(
        &memory_path,
        Some(&backup_path),
        Some(&receipt_path),
        |staged| {
            assert!(
                std::fs::read_to_string(staged)
                    .expect("read staged candidate")
                    .contains("schema_version \"2\"")
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "simulated stop",
            ))
        },
    )
    .expect_err("simulated interruption must abort migration");
    assert_eq!(error.code(), "migration_interrupted");
    assert_eq!(
        std::fs::read(&memory_path).expect("read original after stop"),
        released
    );
    assert_eq!(
        std::fs::read(&backup_path).expect("read backup after stop"),
        released
    );
    assert!(!receipt_path.exists());
    assert!(
        std::fs::read_dir(&dir)
            .expect("list interrupted fixture")
            .all(|entry| {
                let name = entry.expect("entry").file_name();
                let name = name.to_string_lossy();
                !name.contains(".migration.") && !name.contains(".receipt.")
            }),
        "interrupted memory and receipt staging files must be cleaned"
    );

    let retry = formal_ai::migrate_memory(&memory_path, Some(&backup_path), Some(&receipt_path))
        .expect("retry using verified existing backup");
    assert!(retry.changed);
    assert!(receipt_path.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn live_writer_lock_causes_machine_readable_refusal_without_modification() {
    let dir = fixture_dir("locked");
    let memory_path = dir.join("memory.lino");
    let lock_path = dir.join("memory.lino.lock");
    let released = released_fixture();
    std::fs::write(&memory_path, released).expect("write released fixture");
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .expect("open writer lock");
    lock.lock_exclusive().expect("hold writer lock");

    let output = run_memory_command(&[
        "migrate",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--format",
        "json",
    ]);
    assert!(!output.status.success(), "live writer must block migration");
    let refusal: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("lock refusal must be JSON");
    assert_eq!(refusal["error"]["code"], "memory_locked");
    assert_eq!(refusal["status"]["detected_schema_version"], 1);
    assert_eq!(
        std::fs::read_to_string(&memory_path).expect("memory after refusal"),
        released
    );
    lock.unlock().expect("release writer lock");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn future_schema_is_refused_nonzero_and_never_modified() {
    let dir = fixture_dir("future");
    let memory_path = dir.join("memory.lino");
    let future = concat!(
        "demo_memory\n",
        "  schema_version \"99\"\n",
        "  event \"future\"\n",
        "    content \"untouched\"\n",
    );
    std::fs::write(&memory_path, future).expect("write future fixture");

    let preflight = run_memory_command(&[
        "upgrade-status",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--format",
        "json",
    ]);
    assert!(!preflight.status.success());
    let status: serde_json::Value =
        serde_json::from_slice(&preflight.stdout).expect("future refusal must be JSON");
    assert_eq!(status["detected_schema_version"], 99);
    assert_eq!(status["compatible"], false);
    assert_eq!(status["refusal_code"], "schema_too_new");

    let migration = run_memory_command(&[
        "migrate",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--format",
        "json",
    ]);
    assert!(!migration.status.success());
    let refusal: serde_json::Value =
        serde_json::from_slice(&migration.stdout).expect("migration refusal must be JSON");
    assert_eq!(refusal["error"]["code"], "memory_incompatible");
    assert_eq!(refusal["status"]["refusal_code"], "schema_too_new");
    assert_eq!(
        std::fs::read_to_string(&memory_path).expect("future memory after refusal"),
        future
    );
    let mut server_store = SyncStore::open_at(&memory_path);
    assert!(
        server_store
            .record_chat_exchange("must not write", "future schema")
            .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(&memory_path).expect("future memory after server write refusal"),
        future
    );
    assert!(!dir.join("memory.lino.upgrade-receipt.json").exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn malformed_memory_is_refused_nonzero_and_never_modified() {
    let dir = fixture_dir("malformed");
    let memory_path = dir.join("memory.lino");
    let malformed = b"demo_memory\n  event \"broken\"\n    content \"unterminated\n";
    std::fs::write(&memory_path, malformed).expect("write malformed fixture");

    let preflight = run_memory_command(&[
        "upgrade-status",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--format",
        "json",
    ]);
    assert!(!preflight.status.success());
    let status: serde_json::Value =
        serde_json::from_slice(&preflight.stdout).expect("malformed refusal must be JSON");
    assert_eq!(status["compatible"], false);
    assert_eq!(status["refusal_code"], "memory_malformed");

    let migration = run_memory_command(&[
        "migrate",
        "--path",
        memory_path.to_str().expect("memory path"),
        "--format",
        "json",
    ]);
    assert!(!migration.status.success());
    let refusal: serde_json::Value =
        serde_json::from_slice(&migration.stdout).expect("migration refusal must be JSON");
    assert_eq!(refusal["error"]["code"], "memory_incompatible");
    assert_eq!(refusal["status"]["refusal_code"], "memory_malformed");
    assert_eq!(
        std::fs::read(&memory_path).expect("malformed memory after refusal"),
        malformed
    );
    assert!(!dir.join("memory.lino.upgrade-receipt.json").exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn health_exposes_schema_compatibility_without_triggering_migration() {
    let response = handle_api_request("GET", "/health", "");
    assert_eq!(response.status_code, 200);
    let health: serde_json::Value = serde_json::from_str(&response.body).expect("health JSON");
    assert_eq!(health["memory"]["minimum_readable_schema_version"], 1);
    assert_eq!(health["memory"]["maximum_readable_schema_version"], 2);
    assert_eq!(health["memory"]["target_schema_version"], 2);
    assert!(health["memory"]["compatible"].is_boolean());
    assert!(health["memory"]["migration_required"].is_boolean());
    assert!(health["memory"]["migration_state"].is_string());
}

#[test]
fn ordinary_server_write_preserves_released_schema_and_unknown_metadata() {
    let dir = fixture_dir("ordinary-write");
    let memory_path = dir.join("memory.lino");
    std::fs::write(&memory_path, released_fixture()).expect("write released fixture");

    let mut store = SyncStore::open_at(&memory_path);
    assert_eq!(
        store
            .record_chat_exchange("new prompt", "new answer")
            .expect("record chat exchange"),
        2
    );

    let after = std::fs::read_to_string(&memory_path).expect("read after ordinary write");
    assert!(after.starts_with("demo_memory\n  event"));
    assert!(!after.contains("schema_version"));
    assert!(after.contains("futureField \"preserve me\""));
    assert!(after.find("event \"event-1\"") < after.find("event \"event-2\""));
    let mut entries = std::fs::read_dir(&dir)
        .expect("list fixture dir")
        .map(|entry| {
            entry
                .expect("fixture entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    entries.sort();
    let mut expected = vec!["memory.lino", "memory.lino.lock"];
    #[cfg(feature = "doublets-native")]
    expected.extend([
        "memory.links",
        "memory.links.lock",
        "memory.transitions.links",
    ]);
    expected.sort_unstable();
    assert_eq!(
        entries, expected,
        "an ordinary write may create only the portable memory, its shared writer lock, and the documented link-cli persistence sidecars"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn first_write_to_a_new_store_uses_the_target_schema() {
    let dir = fixture_dir("new-store");
    let memory_path = dir.join("nested").join("memory.lino");
    let mut store = SyncStore::open_at(&memory_path);
    assert!(
        memory_path.is_file(),
        "an operational store retains the established eager-create contract"
    );
    assert!(
        std::fs::read(&memory_path)
            .expect("read newly initialized store")
            .is_empty(),
        "opening a missing store creates no synthetic event"
    );

    store
        .import_links_notation(
            "demo_memory\n  event \"new-event\"\n    role \"user\"\n    content \"new memory\"\n",
        )
        .expect("persist first event");
    let persisted = std::fs::read_to_string(&memory_path).expect("read new store");
    assert!(persisted.starts_with("demo_memory\n  schema_version \"2\"\n"));
    assert!(!formal_ai::preflight_memory_upgrade(&memory_path).migration_required);
    let _ = std::fs::remove_dir_all(dir);
}
