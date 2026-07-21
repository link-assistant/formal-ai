//! Issue #540 §6: background dreaming runtime — cooperative idleness, the
//! `serve()` startup wiring, the `FORMAL_AI_DREAMING` opt-out, and the locked
//! atomic writes that let the foreground server and the dreaming worker share
//! one memory log.

use std::time::Duration;

use formal_ai::{
    core_is_idle, dreaming_disabled, run_core_dreaming_once, write_locked_atomic,
    ForegroundActivity, MemoryStore,
};

#[test]
fn foreground_activity_blocks_idleness_until_released() {
    let guard = ForegroundActivity::begin();
    // While any foreground request is in flight the core is never idle, even
    // with a zero threshold.
    assert!(!core_is_idle(Duration::ZERO));
    drop(guard);
    // Releasing the guard stamps the idle clock, so a large threshold still
    // reports busy right after the request finishes...
    assert!(!core_is_idle(Duration::from_hours(1)));
    // ...while a zero threshold reports idle as soon as no request is active.
    // Other tests in this binary may briefly hold their own foreground guard,
    // so poll instead of asserting a single instant.
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while !core_is_idle(Duration::ZERO) {
        assert!(
            std::time::Instant::now() < deadline,
            "core never became idle after the guard was dropped"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn a_foreground_request_cancels_a_dreaming_run_mid_flight() {
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-dreaming-runtime-yield-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("memory.lino");
    MemoryStore::from_events(vec![formal_ai::MemoryEvent {
        id: String::from("req-1"),
        kind: Some(String::from("requirement")),
        role: Some(String::from("user")),
        content: Some(String::from(
            "latex proofs: Always include a LaTeX verification step.",
        )),
        conversation_title: Some(String::from("latex")),
        ..formal_ai::MemoryEvent::default()
    }])
    .save_to_file(&path)
    .expect("seed memory");
    let before = std::fs::read_to_string(&path).expect("seeded log");

    let _guard = ForegroundActivity::begin();
    let outcome = run_core_dreaming_once(&path).expect("run yields, not errors");

    // The mid-run cancellation point fires between planning and application:
    // nothing is learned, removed, or persisted while a request is active.
    assert_eq!(outcome.learned_amendments, 0);
    assert_eq!(outcome.removed_events, 0);
    assert_eq!(
        std::fs::read_to_string(&path).expect("log still present"),
        before,
        "a yielded run must not touch the memory log"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dreaming_env_opt_out_recognizes_only_explicit_off_values() {
    // Default-on: absent or affirmative values keep dreaming enabled; only an
    // explicit 0/off/false (any case) disables it.
    let previous = std::env::var_os("FORMAL_AI_DREAMING");
    for (value, disabled) in [
        (None, false),
        (Some("1"), false),
        (Some("on"), false),
        (Some("true"), false),
        (Some("0"), true),
        (Some("off"), true),
        (Some("OFF"), true),
        (Some("false"), true),
        (Some("False"), true),
    ] {
        match value {
            Some(value) => std::env::set_var("FORMAL_AI_DREAMING", value),
            None => std::env::remove_var("FORMAL_AI_DREAMING"),
        }
        assert_eq!(dreaming_disabled(), disabled, "value {value:?}");
    }
    match previous {
        Some(value) => std::env::set_var("FORMAL_AI_DREAMING", value),
        None => std::env::remove_var("FORMAL_AI_DREAMING"),
    }
}

#[test]
fn serve_startup_wires_the_dreaming_worker_and_requests_guard_the_idle_clock() {
    // The runtime wiring is a startup side effect that a unit test cannot
    // observe through a bound socket, so assert the wiring at the source
    // level, the same way the issue-540 traceability tests do. `serve()` speaks
    // sockets and lives in the transport submodule; the request path that opens
    // the idle gate stays in the protocol module beside it.
    let server = std::fs::read_to_string("src/server.rs").expect("read src/server.rs");
    let listener =
        std::fs::read_to_string("src/server/http_io.rs").expect("read src/server/http_io.rs");
    let serve_body = listener
        .split("pub fn serve(")
        .nth(1)
        .expect("serve() exists");
    assert!(
        serve_body
            .lines()
            .take(3)
            .any(|line| line.contains("dreaming_runtime::start_core_dreaming()")),
        "serve() must start the core dreaming worker before accepting connections"
    );
    assert!(
        server.contains("ForegroundActivity::begin()"),
        "every API request must register foreground activity for the idle gate"
    );

    let runtime =
        std::fs::read_to_string("src/dreaming_runtime.rs").expect("read src/dreaming_runtime.rs");
    assert!(
        runtime.contains("const DEFAULT_IDLE_SECONDS: u64 = 60;"),
        "the documented one-minute idle threshold must stay in force"
    );
    assert!(
        runtime.contains("thread_priority::ThreadPriority::Min"),
        "the core worker must drop to minimum OS scheduling priority"
    );
}

#[test]
fn write_locked_atomic_creates_parents_replaces_content_and_leaves_no_temp_files() {
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-write-locked-atomic-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("nested").join("memory.lino");

    write_locked_atomic(&path, "first document").expect("first write");
    assert_eq!(
        std::fs::read_to_string(&path).expect("written"),
        "first document"
    );

    write_locked_atomic(&path, "second document").expect("atomic replace");
    assert_eq!(
        std::fs::read_to_string(&path).expect("replaced"),
        "second document"
    );

    let entries: Vec<String> = std::fs::read_dir(path.parent().expect("parent"))
        .expect("list dir")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert!(
        entries.iter().all(|name| !name.contains(".tmp.")),
        "no temp files may survive a completed write: {entries:?}"
    );
    assert!(
        entries.iter().any(|name| name == "memory.lino.lock"),
        "the shared lock file guards concurrent writers: {entries:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
