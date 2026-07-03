use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct FormalAiServer {
    child: Child,
}

impl Drop for FormalAiServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

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
    let child = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["serve", "--host", "127.0.0.1", "--port", &port.to_string()])
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
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut last_error = String::from("server was not probed");

    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("check server process") {
            panic!("formal-ai server exited before becoming healthy: {status}");
        }

        match http_request("GET", port, "/health", None, None) {
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

pub fn http_request(
    method: &str,
    port: u16,
    path: &str,
    bearer_token: Option<&str>,
    body: Option<&str>,
) -> std::io::Result<HttpResponse> {
    let address = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect_timeout(
        &address.parse().expect("loopback address should parse"),
        Duration::from_secs(2),
    )?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
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

    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;
    Ok(parse_http_response(&raw))
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
