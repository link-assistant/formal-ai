//! The harness's own HTTP client, against a deliberately slow server.
//!
//! The suite runs many debug builds of the server at once, so a response that
//! goes quiet under load is ordinary rather than broken. These pin the
//! distinction the client has to draw: a pause is not a failure, a hang is.

use std::io::{ErrorKind, Read as _, Write as _};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use super::http_server::{http_request, http_request_with_timeout};

/// Serve one HTTP response, going quiet for `pause` midway through the body.
///
/// Returns the port. The connection closes when the thread drops the stream,
/// which is the EOF the client reads up to.
fn serve_once_pausing_mid_body(pause: Duration) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake server");
    let port = listener.local_addr().expect("read fake server port").port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept client connection");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"half\":")
            .expect("write the first half");
        stream.flush().expect("flush the first half");
        thread::sleep(pause);
        let _ = stream.write_all(b"\"whole\"}");
    });
    port
}

#[test]
fn a_response_that_pauses_mid_body_is_read_in_full() {
    // Longer than the 2s window a per-syscall read timeout used to allow, which
    // failed the request outright and discarded the half already read.
    let port = serve_once_pausing_mid_body(Duration::from_millis(2_500));

    let response = http_request("GET", port, "/paused", None, None)
        .expect("a paused response should still complete");

    assert_eq!(response.status_code, 200);
    assert_eq!(
        response.body, "{\"half\":\"whole\"}",
        "both halves should survive the pause"
    );
}

#[test]
fn a_response_that_never_arrives_still_fails() {
    // The pause outlasts the deadline, so this is a hang as far as the client
    // can tell: waiting through a quiet stretch must not mean waiting forever.
    let port = serve_once_pausing_mid_body(Duration::from_secs(2));

    let error = http_request_with_timeout(
        "GET",
        port,
        "/wedged",
        None,
        None,
        Duration::from_millis(300),
    )
    .expect_err("a response past the deadline should fail");

    assert!(
        matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut),
        "should fail as a timeout, got {error:?}"
    );
}
