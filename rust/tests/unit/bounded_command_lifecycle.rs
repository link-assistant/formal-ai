//! Hosted tests of the actual shared command supervisor; never run native products locally.
use formal_ai::bounded_process::{
    command_output, command_output_with_input, command_status, stop_child,
};
use std::error::Error as _;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "formal-ai-native-command-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn node(source: &str) -> Command {
    let mut command = Command::new("node");
    command.args(["-e", source]);
    command
}

#[test]
fn binary_capture_preserves_both_streams_and_nonzero_status() {
    let command = node(
        "process.stdout.write(Buffer.from([0,255,195,169]));process.stderr.write(Buffer.from([1,0,254]));process.exitCode=7;",
    );
    let output = command_output(command, Duration::from_secs(5), 128, 128).unwrap();
    assert_eq!(output.stdout, [0, 255, 195, 169]);
    assert_eq!(output.stderr, [1, 0, 254]);
    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn blocked_input_and_nonterminating_child_share_the_same_deadline() {
    let command = node(
        "process.stdin.pause();process.stdout.write('started');process.on('SIGTERM',()=>{});setInterval(()=>{},1000);",
    );
    let started = Instant::now();
    let error = command_output_with_input(
        command,
        vec![0; 8 * 1024 * 1024],
        Duration::from_secs(2),
        128,
        128,
    )
    .unwrap_err();
    assert!(error.timed_out());
    assert!(error.cleanup_complete());
    assert_eq!(error.partial_stdout(), b"started");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[cfg(unix)]
#[test]
fn successful_parent_exit_does_not_hide_descendant_held_pipes() {
    let command = node(
        "const{spawn}=require('node:child_process');spawn(process.execPath,['-e','setTimeout(()=>process.exit(0),10000);'],{stdio:['ignore',process.stdout,process.stderr]});process.stdout.write('parent-finished');process.exit(0);",
    );
    let started = Instant::now();
    let error = command_output(command, Duration::from_secs(2), 128, 128).unwrap_err();
    assert!(error.timed_out());
    assert!(error.cleanup_complete());
    assert_eq!(
        error.observed_status().and_then(|status| status.code()),
        Some(0)
    );
    assert_eq!(error.partial_stdout(), b"parent-finished");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn capture_overflow_is_an_error_with_bounded_partial_bytes() {
    let error = command_output(
        node("process.stdout.write(Buffer.alloc(8192,65));"),
        Duration::from_secs(5),
        1024,
        0,
    )
    .unwrap_err();
    assert!(!error.timed_out());
    assert!(error.cleanup_complete());
    assert_eq!(error.partial_stdout(), vec![65; 1024]);
    assert!(error.partial_stderr().is_empty());
}

#[test]
fn status_only_commands_preserve_owned_file_stdio() {
    let workspace = Workspace::new();
    let log = workspace.0.join("stdout");
    let mut command = node("process.stdout.write('preserved');process.exitCode=9;");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(fs::File::create(&log).unwrap()))
        .stderr(Stdio::null());
    let status = command_status(command, Duration::from_secs(5)).unwrap();
    assert_eq!(status.code(), Some(9));
    assert_eq!(fs::read(log).unwrap(), b"preserved");
}

#[test]
fn invalid_time_or_size_policy_refuses_before_child_effects() {
    let workspace = Workspace::new();
    let marker = workspace.0.join("must-not-exist");
    let program = format!(
        "require('node:fs').writeFileSync({},'unexpected');",
        serde_json::to_string(&marker.to_string_lossy()).unwrap()
    );
    for (timeout, stdout, stderr) in [
        (Duration::ZERO, 128, 128),
        (Duration::MAX, 128, 128),
        (Duration::from_secs(1), usize::MAX, 1),
    ] {
        let error = command_output(node(&program), timeout, stdout, stderr).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(error.cleanup_complete());
        assert!(!marker.exists());
    }
}

#[test]
fn spawn_failure_retains_original_io_kind_and_cause() {
    let workspace = Workspace::new();
    let command = Command::new(workspace.0.join("missing-command"));
    let error = command_output(command, Duration::from_secs(1), 128, 128).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(error.source().is_some());
    assert!(error.cleanup_complete());
}

#[test]
fn owned_child_stop_has_a_finite_reap_boundary() {
    let mut command = node("setInterval(()=>{},1000);");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command.spawn().unwrap();
    assert!(stop_child(&mut child, Duration::from_millis(250)).unwrap());
    assert!(child.try_wait().unwrap().is_some());
}
