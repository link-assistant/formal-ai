//! Lifecycle and compatibility checks for wrapper-managed Formal AI servers.

use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{Read as _, Write as _};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

pub(super) struct ServerGuard {
    child: Child,
    pub(super) output_log: PathBuf,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(super) fn maybe_start_server(
    base_url: &str,
    port_override: Option<u16>,
) -> Result<Option<ServerGuard>, Box<dyn Error>> {
    let (host, port) = parse_host_port(base_url, port_override)?;
    let address = format!("{host}:{port}");
    if let Ok(stream) = TcpStream::connect(&address) {
        verify_server_version(stream, &address)?;
        return Ok(None);
    }
    let binary = formal_ai_binary_path()?;
    let output_log = temporary_server_output_log(port)?;
    let output = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&output_log)?;
    let mut child = Command::new(binary)
        .args([
            "serve",
            "--agent-mode",
            "--host",
            &host,
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::from(output.try_clone()?))
        .stderr(Stdio::from(output))
        .spawn()?;
    wait_for_server(&address, &mut child)?;
    Ok(Some(ServerGuard { child, output_log }))
}

fn verify_server_version(mut stream: TcpStream, address: &str) -> Result<(), Box<dyn Error>> {
    let timeout = Some(Duration::from_secs(2));
    stream.set_read_timeout(timeout)?;
    stream.set_write_timeout(timeout)?;
    stream.write_all(
        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAccept: application/json\r\n\r\n",
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .ok_or_else(|| render("client_server_health_invalid", &[("address", address)]))?;
    let health: Value = serde_json::from_str(body)?;
    let running_version = health
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| render("client_server_version_missing", &[("address", address)]))?;
    let expected_version = env!("CARGO_PKG_VERSION");
    if running_version != expected_version {
        return Err(render(
            "client_server_version_mismatch",
            &[
                ("address", address),
                ("running_version", running_version),
                ("expected_version", expected_version),
            ],
        )
        .into());
    }
    Ok(())
}

fn temporary_server_output_log(port: u16) -> Result<PathBuf, Box<dyn Error>> {
    let directory = crate::dialog_log::configured_directory()
        .unwrap_or_else(|| std::env::temp_dir().join("formal-ai-dialog-logs"));
    fs::create_dir_all(&directory)?;
    let pid = std::process::id();
    Ok(directory.join(format!("temporary-server-{pid}-{port}.log")))
}

fn parse_host_port(
    base_url: &str,
    port_override: Option<u16>,
) -> Result<(String, u16), Box<dyn Error>> {
    let (_, rest) = base_url
        .split_once("://")
        .ok_or("base URL must include a scheme, for example http://127.0.0.1:8080")?;
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, parsed_port) = if let Some(stripped) = authority.strip_prefix('[') {
        let (inside, after) = stripped
            .split_once(']')
            .ok_or("invalid bracketed IPv6 host in base URL")?;
        let port = after.strip_prefix(':').and_then(|value| value.parse().ok());
        (inside.to_string(), port)
    } else if let Some((host, port)) = authority.split_once(':') {
        (host.to_string(), port.parse().ok())
    } else {
        (authority.to_string(), None)
    };
    let port = port_override.or(parsed_port).unwrap_or(8080);
    Ok((host, port))
}

fn formal_ai_binary_path() -> Result<PathBuf, Box<dyn Error>> {
    let current = std::env::current_exe()?;
    let stem = current.file_stem().and_then(|value| value.to_str());
    if stem == Some("formal-ai") {
        return Ok(current);
    }
    let sibling = current.with_file_name(format!("formal-ai{}", std::env::consts::EXE_SUFFIX));
    if sibling.exists() {
        return Ok(sibling);
    }
    Ok(PathBuf::from("formal-ai"))
}

fn wait_for_server(address: &str, child: &mut Child) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(render(
                "client_server_exited_before_listening",
                &[("status", &status.to_string())],
            )
            .into());
        }
        if TcpStream::connect(address).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(render("client_server_did_not_listen", &[("address", address)]).into())
}

fn render(key: &str, values: &[(&str, &str)]) -> String {
    let template = crate::seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned());
    values.iter().fold(template, |text, (name, value)| {
        text.replace(&format!("{{{name}}}"), value)
    })
}
