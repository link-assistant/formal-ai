use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long a response may take in total before the harness calls it a hang.
///
/// This is a liveness guard, not a latency assertion: the suite runs many debug
/// builds of the server at once, so a slow response is ordinary and only a
/// wedged one is a failure. Call sites that need longer still say so through
/// [`http_post_json_with_read_timeout`].
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

/// How long a single read may block before the deadline is re-checked.
const POLL_SLICE: Duration = Duration::from_millis(100);

const HEALTH_TIMEOUT: Duration = Duration::from_secs(5);

pub struct FormalAiServer {
    child: Child,
}

impl Drop for FormalAiServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find_map(|(header_name, value)| {
            header_name
                .eq_ignore_ascii_case(name)
                .then_some(value.as_str())
        })
    }
}

pub fn reserve_loopback_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("reserve loopback port")
        .local_addr()
        .expect("read reserved port")
        .port()
}

pub fn spawn_formal_ai_server(port: u16) -> FormalAiServer {
    spawn_formal_ai_server_with_args(port, &[])
}

pub fn spawn_formal_ai_server_agent_mode(port: u16) -> FormalAiServer {
    spawn_formal_ai_server_with_args(port, &["--agent-mode"])
}

fn spawn_formal_ai_server_with_args(port: u16, extra_args: &[&str]) -> FormalAiServer {
    let child = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["serve", "--host", "127.0.0.1", "--port", &port.to_string()])
        .args(extra_args)
        .env("FORMAL_AI_API_BEARER_TOKEN", "sk-local-agentic-tools")
        .env_remove("FORMAL_AI_HTTP_BEARER_TOKEN")
        .env_remove("FORMAL_AI_API_TOKEN")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn formal-ai server");
    let mut server = FormalAiServer { child };
    wait_for_health(port, &mut server.child);
    server
}

fn wait_for_health(port: u16, child: &mut Child) {
    let deadline = Instant::now() + HEALTH_TIMEOUT;
    let mut last_error = String::from("server was not probed");

    // Each probe gets whatever is left of the overall budget, so a probe can
    // never outlive the deadline it is being raced against.
    while let Some(remaining) = deadline
        .checked_duration_since(Instant::now())
        .filter(|left| !left.is_zero())
    {
        if let Some(status) = child.try_wait().expect("check server process") {
            panic!("formal-ai server exited before becoming healthy: {status}");
        }

        match http_request_with_timeout("GET", port, "/health", None, None, remaining) {
            Ok(response) if response.status_code == 200 => return,
            Ok(response) => {
                last_error = format!(
                    "GET /health returned HTTP {} with body {}",
                    response.status_code, response.body
                );
            }
            Err(error) => {
                last_error = error.to_string();
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    panic!("formal-ai server did not become healthy on port {port}: {last_error}");
}

pub fn http_get_json(port: u16, path: &str, bearer_token: Option<&str>) -> serde_json::Value {
    let response =
        http_request("GET", port, path, bearer_token, None).expect("GET request should complete");
    assert_eq!(
        response.status_code, 200,
        "GET {path} should return 200, got {} with body {}",
        response.status_code, response.body
    );
    serde_json::from_str(&response.body).expect("GET response should be JSON")
}

pub fn http_post_json(
    port: u16,
    path: &str,
    bearer_token: Option<&str>,
    body: &serde_json::Value,
) -> serde_json::Value {
    let body = body.to_string();
    let response =
        http_request("POST", port, path, bearer_token, Some(&body)).expect("POST should complete");
    assert_eq!(
        response.status_code, 200,
        "POST {path} should return 200, got {} with body {}",
        response.status_code, response.body
    );
    serde_json::from_str(&response.body).expect("POST response should be JSON")
}

/// POST a JSON body, allowing longer than [`RESPONSE_TIMEOUT`] for the response.
///
/// Used by recipes whose *first* response performs genuinely heavy deterministic
/// work — e.g. the issue-#558 self-healing recipe parses a real module through the
/// CST/AST engine (a source → links round-trip) to build its repair case. That first
/// parse takes a few seconds in a debug build; subsequent responses are memoised and
/// instant.
pub fn http_post_json_with_read_timeout(
    port: u16,
    path: &str,
    bearer_token: Option<&str>,
    body: &serde_json::Value,
    response_timeout: Duration,
) -> serde_json::Value {
    let body = body.to_string();
    let response = http_request_with_timeout(
        "POST",
        port,
        path,
        bearer_token,
        Some(&body),
        response_timeout,
    )
    .expect("POST should complete");
    assert_eq!(
        response.status_code, 200,
        "POST {path} should return 200, got {} with body {}",
        response.status_code, response.body
    );
    serde_json::from_str(&response.body).expect("POST response should be JSON")
}

pub fn http_request(
    method: &str,
    port: u16,
    path: &str,
    bearer_token: Option<&str>,
    body: Option<&str>,
) -> std::io::Result<HttpResponse> {
    http_request_with_timeout(method, port, path, bearer_token, body, RESPONSE_TIMEOUT)
}

pub fn http_request_with_timeout(
    method: &str,
    port: u16,
    path: &str,
    bearer_token: Option<&str>,
    body: Option<&str>,
    response_timeout: Duration,
) -> std::io::Result<HttpResponse> {
    let address = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect_timeout(
        &address.parse().expect("loopback address should parse"),
        Duration::from_secs(2),
    )?;
    // The socket timeout bounds one read syscall, not the whole response, so it
    // is only the polling slice; `read_response` owns the actual deadline.
    stream.set_read_timeout(Some(POLL_SLICE))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;

    let body = body.unwrap_or_default();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\n\
         host: 127.0.0.1:{port}\r\n\
         connection: close\r\n"
    )?;
    if let Some(token) = bearer_token {
        write!(stream, "authorization: Bearer {token}\r\n")?;
    }
    if method == "POST" {
        write!(
            stream,
            "content-type: application/json\r\n\
             content-length: {}\r\n",
            body.len()
        )?;
    }
    write!(stream, "\r\n{body}")?;

    let raw = read_response(&mut stream, response_timeout)?;
    Ok(parse_http_response(&String::from_utf8_lossy(&raw)))
}

/// Read until the server closes the connection or the deadline passes.
///
/// A socket read timeout fires per syscall, so reading to EOF through one is a
/// bound on silence, not on the response: a busy server that pauses longer than
/// the timeout between chunks looks identical to a wedged one, and the bytes
/// already read are discarded with it. Polling against a single deadline instead
/// makes a quiet stretch cost nothing and keeps only a genuine hang fatal.
fn read_response(stream: &mut TcpStream, timeout: Duration) -> std::io::Result<Vec<u8>> {
    let deadline = Instant::now() + timeout;
    let mut raw = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return Ok(raw),
            Ok(count) => raw.extend_from_slice(&chunk[..count]),
            // `WouldBlock`/`TimedOut` mean the slice elapsed with nothing to
            // read; `Interrupted` means a signal landed. Neither ends the
            // response, so both just re-check the deadline.
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) =>
            {
                if Instant::now() >= deadline {
                    return Err(error);
                }
            }
            Err(error) => return Err(error),
        }
    }
}

fn parse_http_response(raw: &str) -> HttpResponse {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let status_code = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or_default();
    let content_type = head
        .lines()
        .skip(1)
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-type")
                .then(|| value.trim().to_owned())
        })
        .unwrap_or_default();
    let headers = head
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect();
    HttpResponse {
        status_code,
        content_type,
        headers,
        body: body.to_owned(),
    }
}
