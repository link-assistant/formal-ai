//! Real source-client filesystem boundaries; only the response transport is controlled.
use ::formal_ai::{CachedSourceClient, FetchError, SourceTransport};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);
struct CacheDirectory(PathBuf);
impl CacheDirectory {
    fn new() -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "formal-ai-source-budget-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("new exclusive fixture directory");
        Self(path)
    }
    fn root(&self) -> PathBuf {
        self.0.join("source-cache")
    }
}
impl Drop for CacheDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[derive(Clone)]
struct BytesTransport(Vec<u8>);
impl SourceTransport for BytesTransport {
    fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
        Ok(self.0.clone())
    }
}
fn policy_limit(key: &str) -> usize {
    let policy: serde_json::Value = serde_json::from_str(include_str!(
        "../../embedded/data/meta/source-cache-budget.json"
    ))
    .unwrap();
    usize::try_from(policy[key].as_u64().unwrap()).unwrap()
}
fn client(directory: &CacheDirectory) -> CachedSourceClient<BytesTransport> {
    CachedSourceClient::new(
        &directory.0,
        BytesTransport(b"retained exact source".to_vec()),
    )
    .with_online(true)
}
fn key(url: &str) -> String {
    ::formal_ai::translation::cache::cache_key(url)
}

#[test]
fn shared_body_reuses_one_object_and_preserves_offline_captures() {
    let directory = CacheDirectory::new();
    let online = client(&directory);
    let first = online.fetch("https://budget.invalid/first").unwrap();
    let second = online.fetch("https://budget.invalid/second").unwrap();
    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(
        fs::read_dir(directory.root().join("objects"))
            .unwrap()
            .count(),
        1
    );
    let offline = CachedSourceClient::new(&directory.0, BytesTransport(Vec::new()));
    assert!(
        offline
            .fetch("https://budget.invalid/first")
            .unwrap()
            .cached()
    );
    assert_eq!(
        offline
            .fetch("https://budget.invalid/second")
            .unwrap()
            .bytes(),
        first.bytes()
    );
}
#[test]
fn full_entry_budget_refuses_additional_capture_without_eviction() {
    let directory = CacheDirectory::new();
    fs::create_dir_all(directory.root().join("objects")).unwrap();
    let limit = policy_limit("maxEntries");
    for index in 0..limit {
        fs::write(
            directory.root().join(format!("retained-{index}.meta")),
            b"retained",
        )
        .unwrap();
    }
    let url = "https://budget.invalid/full";
    let error = client(&directory).fetch(url).unwrap_err();
    assert!(
        error.to_string().contains("source_cache_budget_exceeded"),
        "{error}"
    );
    assert!(!directory.root().join(format!("{}.meta", key(url))).exists());
    assert_eq!(fs::read_dir(directory.root()).unwrap().count(), limit + 1);
    assert_eq!(
        fs::read(directory.root().join("retained-0.meta")).unwrap(),
        b"retained"
    );
}
#[test]
fn existing_shared_lock_is_a_single_immediate_refusal() {
    let directory = CacheDirectory::new();
    fs::create_dir_all(directory.root()).unwrap();
    let lock = directory.root().join(".source-cache-write.lock");
    fs::write(&lock, b"another writer").unwrap();
    let started = Instant::now();
    assert!(
        client(&directory)
            .fetch("https://budget.invalid/locked")
            .is_err()
    );
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(fs::read(lock).unwrap(), b"another writer");
}
#[test]
fn body_larger_than_policy_is_never_published() {
    let directory = CacheDirectory::new();
    let oversized = BytesTransport(vec![b'x'; policy_limit("maxBodyBytes") + 1]);
    let error = CachedSourceClient::new(&directory.0, oversized)
        .with_online(true)
        .fetch("https://budget.invalid/oversized")
        .unwrap_err();
    assert!(matches!(
        error,
        FetchError::Cache(_) | FetchError::Transport(_)
    ));
    assert!(!directory.root().join(".source-cache-write.lock").exists());
    if directory.root().join("objects").exists() {
        assert_eq!(
            fs::read_dir(directory.root().join("objects"))
                .unwrap()
                .count(),
            0
        );
    }
}
#[test]
fn oversized_sparse_metadata_is_refused_before_payload_allocation() {
    let directory = CacheDirectory::new();
    fs::create_dir_all(directory.root()).unwrap();
    let url = "https://budget.invalid/sparse";
    let file = fs::File::create(directory.root().join(format!("{}.meta", key(url)))).unwrap();
    file.set_len(u64::try_from(policy_limit("maxMetadataBytes")).unwrap() + 1)
        .unwrap();
    let error = client(&directory).fetch(url).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("source_cache_read_budget_or_type"),
        "{error}"
    );
}
#[cfg(unix)]
#[test]
fn symlinked_root_does_not_create_objects_in_external_directory() {
    use std::os::unix::fs::symlink;
    let directory = CacheDirectory::new();
    let external = directory.0.join("unrelated");
    fs::create_dir(&external).unwrap();
    fs::write(external.join("keep"), b"keep").unwrap();
    symlink(&external, directory.root()).unwrap();
    assert!(
        client(&directory)
            .fetch("https://budget.invalid/symlink")
            .is_err()
    );
    assert_eq!(fs::read_dir(&external).unwrap().count(), 1);
    assert!(!external.join("objects").exists());
}
#[cfg(unix)]
#[test]
fn fifo_metadata_cannot_block_open_indefinitely() {
    let directory = CacheDirectory::new();
    fs::create_dir_all(directory.root()).unwrap();
    let url = "https://budget.invalid/fifo";
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        directory.root().join(format!("{}.meta", key(url))),
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
    .unwrap();
    let source = client(&directory);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let _ = sender.send(source.fetch(url));
    });
    assert!(
        receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("FIFO refusal deadline")
            .is_err()
    );
    worker.join().unwrap();
}

// Include the maintained pure reader to exercise its exact production implementation.
#[allow(dead_code)]
#[path = "../../src/source_fetch/transport.rs"]
mod source_transport;

#[test]
fn bounded_transport_reader_accepts_exact_limit_and_positive_short_reads() {
    struct ShortRead(std::io::Cursor<Vec<u8>>);
    impl std::io::Read for ShortRead {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            let length = output.len().min(1);
            std::io::Read::read(&mut self.0, &mut output[..length])
        }
    }
    assert_eq!(
        source_transport::read_limited(ShortRead(std::io::Cursor::new(b"abcd".to_vec())), 4)
            .unwrap(),
        b"abcd"
    );
    assert!(
        source_transport::read_limited(std::io::Cursor::new(Vec::<u8>::new()), 0)
            .unwrap()
            .is_empty()
    );
}
#[test]
fn bounded_transport_reader_refuses_before_consuming_full_oversized_stream() {
    struct Infinite {
        reads: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl std::io::Read for Infinite {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            self.reads.set(self.reads.get() + 1);
            output.fill(b'x');
            Ok(output.len())
        }
    }
    let reads = std::rc::Rc::new(std::cell::Cell::new(0));
    let error = source_transport::read_limited(
        Infinite {
            reads: std::rc::Rc::clone(&reads),
        },
        7,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("source_transport_output_budget_exceeded")
    );
    assert_eq!(reads.get(), 1);
}
#[test]
fn bounded_transport_reader_does_not_retry_interrupted_or_failed_action() {
    struct Failing {
        calls: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl std::io::Read for Failing {
        fn read(&mut self, _output: &mut [u8]) -> std::io::Result<usize> {
            self.calls.set(self.calls.get() + 1);
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "original interrupted read",
            ))
        }
    }
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let error = source_transport::read_limited(
        Failing {
            calls: std::rc::Rc::clone(&calls),
        },
        8,
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Interrupted);
    assert_eq!(error.to_string(), "original interrupted read");
    assert_eq!(calls.get(), 1);
}
#[test]
fn bounded_transport_reader_refuses_overflowed_capacity_before_reading() {
    let error = source_transport::read_limited(std::io::empty(), usize::MAX).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("source_transport_invalid_output_limit")
    );
}

#[cfg(unix)]
#[test]
fn actual_bounded_child_capture_keeps_stdout_stderr_and_unsuccessful_status_separate() {
    let mut command = std::process::Command::new("sh");
    command.args([
        "-c",
        "printf 'source-body'; printf 'diagnostic' >&2; exit 7",
    ]);
    let observed = source_transport::bounded_output(&mut command, 32, 32).unwrap();
    assert_eq!(observed.stdout, b"source-body");
    assert_eq!(observed.stderr, b"diagnostic");
    assert_eq!(observed.status.code(), Some(7));
}
#[cfg(unix)]
#[test]
fn actual_child_output_overflow_is_refused_and_child_is_observably_reaped() {
    let mut command = std::process::Command::new("sh");
    command.args([
        "-c",
        "for item in 1 2 3 4 5; do printf 'source-body-beyond-bound'; done",
    ]);
    let started = Instant::now();
    let error = source_transport::bounded_output(&mut command, 8, 32).unwrap_err();
    assert!(started.elapsed() < Duration::from_secs(2));
    let message = error.to_string();
    assert!(
        message.contains("source_transport_output_budget_exceeded"),
        "{message}"
    );
    assert!(message.contains("cleanup_complete=true"), "{message}");
}
