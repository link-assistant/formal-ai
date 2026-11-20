//! Native ingress for the shared transactional dynamic meta host.

use std::io::{self, Read as _, Write as _};
use std::net::{SocketAddr, TcpStream};
use std::sync::LazyLock;
use std::time::Duration;

use crate::protocol::{ChatCompletionRequest, latest_user_request};

use super::{ApiAuthConfig, ApiHttpResponse, error_response};

pub(super) const PATH: &str = "/v1/dynamic-meta";
pub(super) const REQUEST_LIMIT: usize = 256 * 1024;
const RESPONSE_LIMIT: usize = 12 * 1024 * 1024;
const ADMIN_HEADER: &str = "x-formal-ai-meta-admin";
static RECIPE_PROMPT: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(?i)^(?:Run recipe \S+ on|Use recipe \S+ to rewrite) ("[^\n]*")\.?$"#)
        .expect("valid dynamic recipe prompt pattern")
});

pub(super) fn route(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
    auth: &ApiAuthConfig,
) -> Option<ApiHttpResponse> {
    if method != "POST" {
        return None;
    }
    let address = (!super::ephemeral_enabled())
        .then(|| std::env::var("FORMAL_AI_DYNAMIC_META_LOOPBACK").ok())
        .flatten();
    let recipe_chat = address.is_some()
        && matches!(
            path,
            "/v1/chat/completions" | "/api/openai/v1/chat/completions"
        )
        && auth.bearer_token.is_some()
        && is_recipe_chat(body);
    if path != PATH && !recipe_chat {
        return None;
    }
    if (recipe_chat || address.is_none()) && !auth.allows(headers) {
        return Some(error_response(401, "missing or invalid bearer token"));
    }
    let Some(address) = address else {
        return Some(error_response(404, "Dynamic meta endpoint unavailable"));
    };
    if body.len() > REQUEST_LIMIT {
        return Some(error_response(413, "Dynamic meta request is too large"));
    }
    let Ok(address) = address.parse::<SocketAddr>() else {
        return Some(error_response(503, "Invalid dynamic meta loopback address"));
    };
    if !address.ip().is_loopback() {
        return Some(error_response(503, "Invalid dynamic meta loopback address"));
    }
    let _activity = crate::dreaming_runtime::ForegroundActivity::begin();
    Some(
        forward(address, path, headers, body)
            .unwrap_or_else(|_| error_response(503, "Dynamic meta host unavailable")),
    )
}

fn is_recipe_chat(body: &str) -> bool {
    serde_json::from_str::<ChatCompletionRequest>(body)
        .ok()
        .and_then(|request| latest_user_request(&request.messages))
        .is_some_and(|prompt| {
            RECIPE_PROMPT
                .captures(prompt.trim())
                .is_some_and(|captures| serde_json::from_str::<String>(&captures[1]).is_ok())
        })
}

fn forward(
    address: SocketAddr,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> io::Result<ApiHttpResponse> {
    let mut socket = TcpStream::connect_timeout(&address, Duration::from_secs(1))?;
    socket.set_read_timeout(Some(Duration::from_secs(35)))?;
    socket.set_write_timeout(Some(Duration::from_secs(2)))?;
    write!(
        socket,
        "POST {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
        body.len()
    )?;
    for &(name, value) in headers {
        if [
            "authorization",
            "x-api-key",
            "x-goog-api-key",
            "anthropic-api-key",
            ADMIN_HEADER,
        ]
        .iter()
        .any(|known| name.eq_ignore_ascii_case(known))
            && !value.contains(['\r', '\n'])
        {
            write!(socket, "{name}: {value}\r\n")?;
        }
    }
    write!(socket, "\r\n{body}")?;
    let mut bytes = Vec::new();
    socket
        .take(RESPONSE_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > RESPONSE_LIMIT {
        return Err(invalid_response());
    }
    decode_response(&bytes)
}

fn invalid_response() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid dynamic meta host response",
    )
}

fn decode_response(bytes: &[u8]) -> io::Result<ApiHttpResponse> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid_response())?;
    let (head, body) = text.split_once("\r\n\r\n").ok_or_else(invalid_response)?;
    let mut lines = head.split("\r\n");
    let status_code = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .ok_or_else(invalid_response)?;
    let mut content_type = None;
    let mut length = None;
    for (name, value) in lines.filter_map(|line| line.split_once(':')) {
        if name.eq_ignore_ascii_case("content-type") {
            content_type = match value.trim() {
                "application/json" => Some("application/json"),
                "text/event-stream" => Some("text/event-stream"),
                _ => None,
            };
        } else if name.eq_ignore_ascii_case("content-length") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    if length != Some(body.len()) {
        return Err(invalid_response());
    }
    Ok(ApiHttpResponse {
        status_code,
        content_type: content_type.ok_or_else(invalid_response)?,
        body: body.to_owned(),
        deprecated: false,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/dynamic_meta_gateway.rs"]
mod tests;
