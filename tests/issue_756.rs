use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use formal_ai::{export_links_notation, resolve_memory_path_from, MemoryEvent, SyncStore};

fn memory_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

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
    let _guard = memory_env_lock();
    let root = std::env::temp_dir().join(format!("formal-ai-issue-756-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("temp home");

    let previous_memory = std::env::var_os("FORMAL_AI_MEMORY_PATH");
    let previous_home = std::env::var_os("HOME");
    std::env::remove_var("FORMAL_AI_MEMORY_PATH");
    std::env::set_var("HOME", &root);

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

    match previous_memory {
        Some(value) => std::env::set_var("FORMAL_AI_MEMORY_PATH", value),
        None => std::env::remove_var("FORMAL_AI_MEMORY_PATH"),
    }
    match previous_home {
        Some(value) => std::env::set_var("HOME", value),
        None => std::env::remove_var("HOME"),
    }
    let _ = std::fs::remove_dir_all(&root);
}
