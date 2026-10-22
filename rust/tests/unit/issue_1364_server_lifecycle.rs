#![cfg(not(target_arch = "wasm32"))]
//! Hosted-only live-volume and signal acceptance; no original volume bytes are claimed.
use formal_ai::SyncStore;
use std::fs;
struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("formal-ai-1364-{label}-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn live_memory_migrates_released_schema_and_preserves_incompatible_original() {
    let home = Fixture::new("live-memory");
    let path = home.0.join("memory.lino");
    let original = include_bytes!("../fixtures/memory/schema-1.lino");
    fs::write(&path, original).unwrap();
    let store = SyncStore::open_live_at(&path).expect("live migration");
    assert_eq!(store.events().len(), 2);
    let migrated = fs::read(&path).unwrap();
    assert!(String::from_utf8_lossy(&migrated).contains("schema_version \"2\""));
    assert_eq!(
        fs::read_dir(&home.0)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".backup"))
            .count(),
        1
    );
    assert_eq!(SyncStore::open_live_at(&path).unwrap().events().len(), 2);
    assert_eq!(fs::read(&path).unwrap(), migrated);
    let invalid = home.0.join("incompatible.lino");
    let bytes =
        b"demo_memory\n  schema_version \"99\"\n  event \"future\"\n    content \"untouched\"\n";
    fs::write(&invalid, bytes).unwrap();
    let mut selected = SyncStore::open_live_at(&invalid).unwrap();
    assert_eq!(selected.events(), &[]);
    assert!(
        selected
            .record_chat_exchange("new prompt", "new answer")
            .unwrap()
            > 0
    );
    assert_eq!(fs::read(&invalid).unwrap(), bytes);
    assert_eq!(
        SyncStore::open_live_at(&invalid).unwrap().events(),
        selected.events()
    );
    assert_eq!(fs::read(&invalid).unwrap(), bytes);
}
#[cfg(unix)]
#[test]
fn live_memory_rejects_symlink_without_touching_target() {
    let home = Fixture::new("symlink");
    let target = home.0.join("target");
    let link = home.0.join("memory.lino");
    fs::write(&target, b"unchanged").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(SyncStore::open_live_at(&link).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"unchanged");
}

#[cfg(unix)]
#[test]
fn serve_handles_sigterm_and_exits_successfully_within_stop_budget() {
    const CHILD: &str = "FORMAL_AI_1364_SIGTERM_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let signal = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            for _ in 0..300 {
                if std::net::TcpStream::connect(address).is_ok() {
                    let mut command = std::process::Command::new("kill");
                    command.args(["-TERM", &std::process::id().to_string()]);
                    let output = formal_ai::bounded_process::command_output(
                        command,
                        std::time::Duration::from_secs(1),
                        4096,
                        4096,
                    )
                    .unwrap();
                    assert!(output.status.success());
                    return;
                }
                if std::time::Instant::now() >= deadline {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            panic!("serve did not accept before signal deadline");
        });
        formal_ai::serve(&address.to_string()).expect("SIGTERM clean shutdown");
        signal.join().unwrap();
        return;
    }
    let home = Fixture::new("signal-home");
    let name = std::thread::current().name().unwrap().to_owned();
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", &name, "--nocapture"])
        .env(CHILD, "1")
        .env("FORMAL_AI_MEMORY_PATH", home.0.join("memory.lino"))
        .env("HOME", &home.0);
    let output = formal_ai::bounded_process::command_output(
        command,
        std::time::Duration::from_secs(6),
        1_048_576,
        1_048_576,
    )
    .expect("bounded native signal acceptance");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
