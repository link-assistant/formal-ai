use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir() -> std::path::PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir =
        std::env::temp_dir().join(format!("formal-ai-issue-832-{}-{seq}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

fn write_fake_agent(bin_dir: &Path) {
    let path = bin_dir.join("agent");
    std::fs::write(
        &path,
        "#!/bin/sh\nprintf 'invoked\\n' > \"$FORMAL_AI_CAPTURE\"\n",
    )
    .expect("write fake agent");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make fake agent executable");
}

fn serve_health_once(
    listener: TcpListener,
    version: Option<&'static str>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept health probe");
        let mut request = [0_u8; 1024];
        let size = stream.read(&mut request).unwrap_or_default();
        if size == 0 {
            return;
        }
        let body = version.map_or_else(
            || r#"{"status":"ok"}"#.to_owned(),
            |version| format!(r#"{{"status":"ok","version":"{version}"}}"#),
        );
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write health response");
    })
}

fn run_with_server(version: Option<&'static str>) -> (std::process::Output, bool) {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_agent(&bin_dir);
    let capture = dir.join("capture.txt");
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("address").port();
    let server = serve_health_once(listener, version);
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--port", &port.to_string(), "agent", "-p", "hi"])
        .env("HOME", &home)
        .env("PATH", path)
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run formal-ai with existing server");
    server.join().expect("health server");
    let invoked = capture.exists();
    let _ = std::fs::remove_dir_all(&dir);
    (output, invoked)
}

#[test]
fn matching_existing_server_is_reused() {
    let (output, invoked) = run_with_server(Some(env!("CARGO_PKG_VERSION")));

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("started a temporary server"));
    assert!(invoked, "wrapped CLI was not invoked");
}

#[test]
fn incompatible_existing_server_is_rejected() {
    let (output, invoked) = run_with_server(Some("0.301.0"));

    assert!(!output.status.success(), "stale server was silently reused");
    assert!(!invoked, "wrapped CLI ran against a stale server");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("0.301.0"), "stderr: {stderr}");
    assert!(
        stderr.contains(env!("CARGO_PKG_VERSION")),
        "stderr: {stderr}"
    );
}

#[test]
fn legacy_server_without_a_version_is_rejected() {
    let (output, invoked) = run_with_server(None);

    assert!(
        !output.status.success(),
        "legacy server was silently reused"
    );
    assert!(!invoked, "wrapped CLI ran against a legacy server");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not report its version"),
        "stderr: {stderr}"
    );
}
