//! Why the paused-response test reset under CI load (#715).
//!
//! Standalone probe kept for re-use. Run it with:
//!
//! ```sh
//! rust-script experiments/issue_715_rst_on_unread_request_probe.rs
//! ```
//!
//! `tests/integration/http_client.rs` stands up a fake server that answers one
//! request, goes quiet mid-body, then closes. It passed locally and failed on
//! CI with `ConnectionReset`, which reads like flakiness but is not.
//!
//! The client writes its request head and the terminating blank line as
//! separate syscalls, so whether they arrive as one TCP segment is a timing
//! accident — on an idle loopback they coalesce, under load they do not. The
//! fake server issued a single `read`, so the second segment stayed queued.
//! Closing a socket that still has unread bytes queued makes the kernel send
//! RST instead of FIN, and the reset discards the response already sitting in
//! the client's receive buffer. The pause is what widens the window: it holds
//! the connection open long enough for the client to be mid-read when the
//! close lands.
//!
//! This probe forces the two segments apart with a sleep, which turns the race
//! into a certainty and shows the failure is the fake server's, not the
//! client's:
//!
//! ```text
//! single read (current test)   ok=0/5  failures=["ConnectionReset", ...]
//! drain to header end (fix)    ok=5/5  failures=[]
//! ```

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

/// `drain_fully = false` reproduces the fake server before the fix.
fn serve(drain_fully: bool) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0u8; 1024];
        if drain_fully {
            let mut seen = Vec::new();
            loop {
                let count = stream.read(&mut request).unwrap();
                seen.extend_from_slice(&request[..count]);
                if count == 0 || seen.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
        } else {
            let _ = stream.read(&mut request);
        }
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n{\"half\":").unwrap();
        stream.flush().unwrap();
        thread::sleep(Duration::from_millis(300));
        let _ = stream.write_all(b"\"whole\"}");
        // The thread ends here, so `stream` drops and the socket closes.
    });
    port
}

fn client(port: u16) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    // Two write syscalls, exactly as `http_request`'s `write!` calls issue them.
    write!(stream, "GET /paused HTTP/1.1\r\nhost: x\r\nconnection: close\r\n")?;
    thread::sleep(Duration::from_millis(50));
    write!(stream, "\r\n")?;

    let mut raw = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => raw.extend_from_slice(&chunk[..count]),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(error) => return Err(error),
        }
    }
    Ok(String::from_utf8_lossy(&raw).into_owned())
}

fn main() {
    for (label, drain) in [
        ("single read (current test)", false),
        ("drain to header end (fix)", true),
    ] {
        let mut ok = 0;
        let mut failures = Vec::new();
        for _ in 0..5 {
            match client(serve(drain)) {
                Ok(response) if response.ends_with("{\"half\":\"whole\"}") => ok += 1,
                Ok(response) => failures.push(format!("truncated: {response:?}")),
                Err(error) => failures.push(format!("{:?}", error.kind())),
            }
        }
        println!("{label:<28} ok={ok}/5  failures={failures:?}");
    }
}
