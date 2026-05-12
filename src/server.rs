use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use serde::Serialize;
use serde_json::json;

use crate::engine::DEFAULT_MODEL;
use crate::protocol::{
    create_chat_completion, create_response, ChatCompletionRequest, ResponsesRequest,
};
use crate::telegram::handle_telegram_webhook;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiHttpResponse {
    pub status_code: u16,
    pub content_type: &'static str,
    pub body: String,
}

#[must_use]
pub fn handle_api_request(method: &str, path: &str, body: &str) -> ApiHttpResponse {
    let normalized_path = path.split('?').next().unwrap_or(path);

    match (method, normalized_path) {
        ("OPTIONS", _) => ApiHttpResponse {
            status_code: 204,
            content_type: "application/json",
            body: String::new(),
        },
        ("GET", "/health") => json_response(
            200,
            &json!({
                "status": "ok",
                "model": DEFAULT_MODEL,
            }),
        ),
        ("GET", "/v1/models") => json_response(
            200,
            &json!({
                "object": "list",
                "data": [{
                    "id": DEFAULT_MODEL,
                    "object": "model",
                    "created": 0,
                    "owned_by": "link-assistant"
                }]
            }),
        ),
        ("POST", "/v1/chat/completions") => {
            match serde_json::from_str::<ChatCompletionRequest>(body) {
                Ok(request) => json_response(200, &create_chat_completion(&request)),
                Err(error) => error_response(400, &format!("invalid chat request: {error}")),
            }
        }
        ("POST", "/v1/responses") => match serde_json::from_str::<ResponsesRequest>(body) {
            Ok(request) => json_response(200, &create_response(&request)),
            Err(error) => error_response(400, &format!("invalid responses request: {error}")),
        },
        ("POST", "/telegram/webhook") => match handle_telegram_webhook(body) {
            Ok(Some(reply)) => json_response(200, &reply),
            Ok(None) => json_response(200, &json!({ "ok": true })),
            Err(error) => error_response(400, &error.to_string()),
        },
        _ => error_response(404, "route not found"),
    }
}

pub fn serve(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    eprintln!("formal-ai server listening on http://{address}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_connection(&mut stream) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }

    Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> std::io::Result<()> {
    let Some((method, path, body)) = read_request(stream)? else {
        return Ok(());
    };
    let response = handle_api_request(&method, &path, &body);
    write_response(stream, &response)
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Option<(String, String, String)>> {
    let mut buffer = [0_u8; 8192];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(None);
    }

    let mut request_bytes = buffer[..bytes_read].to_vec();
    let header_end = loop {
        if let Some(position) = find_header_end(&request_bytes) {
            break position;
        }
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        request_bytes.extend_from_slice(&buffer[..bytes_read]);
    };

    let header_text = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
    let content_length = content_length(&header_text);
    let body_start = header_end + 4;

    while request_bytes.len() < body_start.saturating_add(content_length) {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        request_bytes.extend_from_slice(&buffer[..bytes_read]);
    }

    let request_line = header_text.lines().next().unwrap_or_default();
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default().to_owned();
    let path = request_parts.next().unwrap_or_default().to_owned();
    let body_end = body_start
        .saturating_add(content_length)
        .min(request_bytes.len());
    let body = String::from_utf8_lossy(&request_bytes[body_start..body_end]).to_string();

    Ok(Some((method, path, body)))
}

fn write_response(stream: &mut TcpStream, response: &ApiHttpResponse) -> std::io::Result<()> {
    let status_text = match response.status_code {
        200 => "200 OK",
        204 => "204 No Content",
        400 => "400 Bad Request",
        404 => "404 Not Found",
        _ => "500 Internal Server Error",
    };

    write!(
        stream,
        "HTTP/1.1 {status_text}\r\n\
         content-type: {}\r\n\
         content-length: {}\r\n\
         access-control-allow-origin: *\r\n\
         access-control-allow-methods: GET,POST,OPTIONS\r\n\
         access-control-allow-headers: content-type,authorization\r\n\
         connection: close\r\n\
         \r\n{}",
        response.content_type,
        response.body.len(),
        response.body
    )
}

fn json_response<T: Serialize>(status_code: u16, value: &T) -> ApiHttpResponse {
    match serde_json::to_string_pretty(value) {
        Ok(body) => ApiHttpResponse {
            status_code,
            content_type: "application/json",
            body,
        },
        Err(error) => error_response(500, &format!("failed to serialize response: {error}")),
    }
}

fn error_response(status_code: u16, message: &str) -> ApiHttpResponse {
    ApiHttpResponse {
        status_code,
        content_type: "application/json",
        body: json!({
            "error": {
                "message": message,
                "type": "formal_ai_error"
            }
        })
        .to_string(),
    }
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length(headers: &str) -> usize {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}
