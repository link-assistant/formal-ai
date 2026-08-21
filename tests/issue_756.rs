use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use formal_ai::{MemoryEvent, SyncStore, export_links_notation, resolve_memory_path_from};

fn event(id: &str, content: &str) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        content: Some(content.to_owned()),
        ..MemoryEvent::default()
    }
}

#[test]
fn shared_memory_path_honors_override_and_platform_defaults() {
    assert_eq!(
        resolve_memory_path_from(Some(OsStr::new("/custom/shared.lino")), None, None, false),
        PathBuf::from("/custom/shared.lino")
    );
    assert_eq!(
        resolve_memory_path_from(None, Some(OsStr::new("/users/alice")), None, false),
        PathBuf::from("/users/alice/.formal-ai/memory.lino")
    );
    assert_eq!(
        resolve_memory_path_from(None, None, Some(OsStr::new("/appdata/alice")), true),
        PathBuf::from("/appdata/alice/formal-ai/memory.lino")
    );
    assert_eq!(
        resolve_memory_path_from(
            Some(OsStr::new("  ")),
            Some(OsStr::new("/users/alice")),
            None,
            false
        ),
        PathBuf::from("/users/alice/.formal-ai/memory.lino")
    );
}

#[test]
fn fresh_default_store_is_created_securely_and_shared_between_surfaces() {
    let root = std::env::temp_dir().join(format!("formal-ai-issue-756-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("temp home");

    // The default store is found through the environment, so the test has to
    // move `HOME` and clear the override to see it. Edition 2024 made
    // `std::env::set_var` unsafe -- rightly, since `cargo test` runs these on
    // threads that share one environment -- and this crate forbids unsafe code,
    // so the two variables are scoped by `temp-env` instead. Its reentrant lock
    // is held for the closure, which is what now keeps two tests from reading
    // each other's `HOME`, in place of the mutex this file used to hold by hand.
    let overrides: [(&str, Option<&Path>); 2] = [
        ("FORMAL_AI_MEMORY_PATH", None),
        ("HOME", Some(root.as_path())),
    ];
    temp_env::with_vars(overrides, || {
        let expected = root.join(".formal-ai/memory.lino");
        let mut desktop_surface = SyncStore::open();
        assert!(
            expected.is_file(),
            "first open must create {}",
            expected.display()
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(expected.parent().unwrap())
                .expect("memory directory metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o700);
        }

        let lino = export_links_notation(&[event("shared-fact", "remember me everywhere")]);
        desktop_surface
            .import_links_notation(&lino)
            .expect("desktop write");
        let vscode_surface = SyncStore::open();
        assert_eq!(vscode_surface.events().len(), 1);
        assert_eq!(vscode_surface.events()[0].id, "shared-fact");
    });

    let _ = std::fs::remove_dir_all(&root);
}
