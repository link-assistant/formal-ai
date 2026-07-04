use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Runtime configuration for `formal-ai proxy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyConfig {
    pub listen: String,
    pub upstream: String,
    pub log_path: PathBuf,
    pub log_bodies: bool,
}

/// One JSONL row describing a proxied request/response exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyExchangeLog {
    pub method: String,
    pub path: String,
    pub request_model: Option<String>,
    pub request_tools: Vec<String>,
    pub status: u16,
    pub response_model: Option<String>,
    pub response_tool_calls: Vec<ProxyToolCallLog>,
    pub response_content_preview: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
}

/// A normalized tool/function call emitted by a proxied model response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyToolCallLog {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Upstream {
    authority: String,
    base_path: String,
}

#[derive(Debug)]
struct HttpHeader {
    name: String,
    value: String,
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: Vec<HttpHeader>,
    body: Vec<u8>,
}

#[derive(Debug)]
struct HttpResponseHead {
    status_line: String,
    status_code: u16,
    headers: Vec<HttpHeader>,
}

#[derive(Debug, Default)]
struct ResponseSummary {
    model: Option<String>,
    tool_calls: Vec<ProxyToolCallLog>,
    content: String,
}

#[derive(Debug, Default)]
struct StreamingChatAccumulator {
    model: Option<String>,
    content: String,
    tool_calls: BTreeMap<u64, StreamingToolCall>,
}

#[derive(Debug, Default)]
struct StreamingToolCall {
    name: String,
    arguments: String,
}

#[derive(Debug)]
struct SseEvent {
    data: String,
}

/// Run the blocking logging proxy until the process is terminated.
///
/// The proxy accepts HTTP/1.1 requests, forwards them to the configured HTTP
/// upstream, streams response bytes back to the client, and appends a structured
/// JSON row per completed exchange.
pub fn run_proxy(config: &ProxyConfig) -> io::Result<()> {
    let upstream = Upstream::parse(&config.upstream)?;
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.log_path)?;
    let logger = Arc::new(Mutex::new(log_file));
    let listener = TcpListener::bind(&config.listen)?;
    eprintln!(
        "formal-ai proxy listening on http://{} -> http://{}, logging to {}",
        config.listen,
        upstream.display_target(),
        config.log_path.display()
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let upstream = upstream.clone();
                let logger = Arc::clone(&logger);
                let log_bodies = config.log_bodies;
                thread::spawn(move || {
                    if let Err(error) =
                        handle_proxy_connection(stream, &upstream, &logger, log_bodies)
                    {
                        eprintln!("proxy request failed: {error}");
                    }
                });
            }
            Err(error) => eprintln!("proxy connection failed: {error}"),
        }
    }

    Ok(())
}

/// Build the JSONL exchange summary without opening sockets.
#[must_use]
pub fn summarize_proxy_exchange(
    method: &str,
    path: &str,
    request_body: &[u8],
    status: u16,
    response_content_type: &str,
    response_body: &[u8],
    log_bodies: bool,
) -> ProxyExchangeLog {
    let request_json = serde_json::from_slice::<Value>(request_body).ok();
    let response_text = String::from_utf8_lossy(response_body);
    let response_summary = if response_content_type
        .split(';')
        .next()
        .is_some_and(|content_type| {
            content_type
                .trim()
                .eq_ignore_ascii_case("text/event-stream")
        }) {
        summarize_sse_response(&response_text)
    } else {
        serde_json::from_slice::<Value>(response_body).map_or_else(
            |_| ResponseSummary {
                content: response_text.chars().take(160).collect(),
                ..ResponseSummary::default()
            },
            |value| summarize_response_value(&value),
        )
    };

    ProxyExchangeLog {
        method: method.to_owned(),
        path: path.to_owned(),
        request_model: request_json
            .as_ref()
            .and_then(|value| value.get("model"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        request_tools: request_json
            .as_ref()
            .map_or_else(Vec::new, collect_request_tool_names),
        status,
        response_model: response_summary.model,
        response_tool_calls: response_summary.tool_calls,
        response_content_preview: response_summary.content.chars().take(160).collect(),
        request_body: log_bodies.then(|| String::from_utf8_lossy(request_body).into_owned()),
        response_body: log_bodies.then(|| response_text.into_owned()),
    }
}

impl Upstream {
    fn parse(raw: &str) -> io::Result<Self> {
        let trimmed = raw.trim();
        let without_scheme = if let Some(rest) = trimmed.strip_prefix("http://") {
            rest
        } else if trimmed.contains("://") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "formal-ai proxy currently supports only http:// upstream URLs",
            ));
        } else {
            trimmed
        };
        let (authority, path) = without_scheme
            .split_once('/')
            .map_or((without_scheme, ""), |(authority, path)| (authority, path));
        if authority.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "proxy upstream must include a host and port",
            ));
        }
        let base_path = if path.trim().is_empty() {
            String::new()
        } else {
            format!("/{}", path.trim_matches('/'))
        };
        Ok(Self {
            authority: authority.to_owned(),
            base_path,
        })
    }

    fn target_path(&self, request_path: &str) -> String {
        if self.base_path.is_empty() {
            request_path.to_owned()
        } else if request_path.starts_with('/') {
            format!("{}{}", self.base_path, request_path)
        } else {
            format!("{}/{}", self.base_path, request_path)
        }
    }

    fn display_target(&self) -> String {
        if self.base_path.is_empty() {
            self.authority.clone()
        } else {
            format!("{}{}", self.authority, self.base_path)
        }
    }
}

fn handle_proxy_connection(
    client: TcpStream,
    upstream: &Upstream,
    logger: &Arc<Mutex<File>>,
    log_bodies: bool,
) -> io::Result<()> {
    let mut client_reader = BufReader::new(client.try_clone()?);
    let mut client_writer = client;
    let Some(request) = read_request(&mut client_reader)? else {
        return Ok(());
    };

    let mut upstream_stream = match TcpStream::connect(&upstream.authority) {
        Ok(stream) => stream,
        Err(error) => {
            let response_body = format!("proxy upstream connection failed: {error}");
            write_error_response(&mut client_writer, 502, &response_body)?;
            let log = summarize_proxy_exchange(
                &request.method,
                &request.path,
                &request.body,
                502,
                "text/plain",
                response_body.as_bytes(),
                log_bodies,
            );
            write_log(logger, &log)?;
            return Ok(());
        }
    };
    write_upstream_request(&mut upstream_stream, upstream, &request)?;

    let mut upstream_reader = BufReader::new(upstream_stream);
    let Some(response_head) = read_response_head(&mut upstream_reader)? else {
        let response_body = "proxy upstream closed without a response";
        write_error_response(&mut client_writer, 502, response_body)?;
        let log = summarize_proxy_exchange(
            &request.method,
            &request.path,
            &request.body,
            502,
            "text/plain",
            response_body.as_bytes(),
            log_bodies,
        );
        write_log(logger, &log)?;
        return Ok(());
    };

    write_response_head(&mut client_writer, &response_head)?;
    let mut response_body = Vec::new();
    forward_response_body(
        &mut upstream_reader,
        &mut client_writer,
        &response_head,
        &mut response_body,
    )?;
    client_writer.flush()?;

    let content_type = response_head
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case("content-type"))
        .map_or("", |header| header.value.as_str());
    let log = summarize_proxy_exchange(
        &request.method,
        &request.path,
        &request.body,
        response_head.status_code,
        content_type,
        &response_body,
        log_bodies,
    );
    write_log(logger, &log)
}

fn write_log(logger: &Arc<Mutex<File>>, log: &ProxyExchangeLog) -> io::Result<()> {
    let line = serde_json::to_string(log)
        .map_err(|error| io::Error::other(format!("failed to serialize proxy log: {error}")))?;
    let mut file = logger
        .lock()
        .map_err(|_| io::Error::other("proxy log mutex poisoned"))?;
    writeln!(file, "{line}")?;
    file.flush()
}

fn read_request(reader: &mut BufReader<TcpStream>) -> io::Result<Option<HttpRequest>> {
    let Some(request_line) = read_http_line(reader)? else {
        return Ok(None);
    };
    let request_line = trim_http_line(&request_line);
    if request_line.is_empty() {
        return Ok(None);
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let path = parts.next().unwrap_or_default().to_owned();
    let headers = read_headers(reader)?;
    let body = read_body(reader, &headers)?;
    Ok(Some(HttpRequest {
        method,
        path,
        headers,
        body,
    }))
}

fn read_response_head(reader: &mut BufReader<TcpStream>) -> io::Result<Option<HttpResponseHead>> {
    let Some(status_line) = read_http_line(reader)? else {
        return Ok(None);
    };
    let status_line = trim_http_line(&status_line);
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or_default();
    let headers = read_headers(reader)?;
    Ok(Some(HttpResponseHead {
        status_line,
        status_code,
        headers,
    }))
}

fn read_http_line(reader: &mut BufReader<TcpStream>) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let bytes = reader.read_until(b'\n', &mut line)?;
    if bytes == 0 {
        Ok(None)
    } else {
        Ok(Some(line))
    }
}

fn trim_http_line(line: &[u8]) -> String {
    String::from_utf8_lossy(line)
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_owned()
}

fn read_headers(reader: &mut BufReader<TcpStream>) -> io::Result<Vec<HttpHeader>> {
    let mut headers = Vec::new();
    while let Some(line) = read_http_line(reader)? {
        if line == b"\r\n" || line == b"\n" {
            break;
        }
        let line = trim_http_line(&line);
        if let Some((name, value)) = line.split_once(':') {
            headers.push(HttpHeader {
                name: name.trim().to_owned(),
                value: value.trim().to_owned(),
            });
        }
    }
    Ok(headers)
}

fn read_body(reader: &mut BufReader<TcpStream>, headers: &[HttpHeader]) -> io::Result<Vec<u8>> {
    if is_chunked(headers) {
        return read_chunked_body(reader);
    }
    let Some(length) = content_length(headers) else {
        return Ok(Vec::new());
    };
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body)?;
    Ok(body)
}

fn read_chunked_body(reader: &mut BufReader<TcpStream>) -> io::Result<Vec<u8>> {
    let mut body = Vec::new();
    while let Some(size_line) = read_http_line(reader)? {
        let size = parse_chunk_size(&size_line)?;
        if size == 0 {
            discard_chunk_trailers(reader)?;
            break;
        }
        let mut chunk = vec![0_u8; size];
        reader.read_exact(&mut chunk)?;
        body.extend_from_slice(&chunk);
        let mut crlf = [0_u8; 2];
        reader.read_exact(&mut crlf)?;
    }
    Ok(body)
}

fn discard_chunk_trailers(reader: &mut BufReader<TcpStream>) -> io::Result<()> {
    while let Some(line) = read_http_line(reader)? {
        if line == b"\r\n" || line == b"\n" {
            break;
        }
    }
    Ok(())
}

fn write_upstream_request(
    upstream_stream: &mut TcpStream,
    upstream: &Upstream,
    request: &HttpRequest,
) -> io::Result<()> {
    write!(
        upstream_stream,
        "{} {} HTTP/1.1\r\n",
        request.method,
        upstream.target_path(&request.path)
    )?;
    write!(upstream_stream, "host: {}\r\n", upstream.authority)?;
    for header in &request.headers {
        if skip_request_header(&header.name) {
            continue;
        }
        write!(upstream_stream, "{}: {}\r\n", header.name, header.value)?;
    }
    write!(upstream_stream, "accept-encoding: identity\r\n")?;
    write!(upstream_stream, "connection: close\r\n")?;
    if !request.body.is_empty() {
        write!(
            upstream_stream,
            "content-length: {}\r\n",
            request.body.len()
        )?;
    }
    write!(upstream_stream, "\r\n")?;
    upstream_stream.write_all(&request.body)?;
    upstream_stream.flush()
}

fn write_response_head(client: &mut TcpStream, response: &HttpResponseHead) -> io::Result<()> {
    write!(client, "{}\r\n", response.status_line)?;
    for header in &response.headers {
        if skip_response_header(&header.name) {
            continue;
        }
        write!(client, "{}: {}\r\n", header.name, header.value)?;
    }
    write!(client, "connection: close\r\n\r\n")
}

fn write_error_response(client: &mut TcpStream, status: u16, body: &str) -> io::Result<()> {
    let status_text = match status {
        502 => "502 Bad Gateway",
        _ => "500 Internal Server Error",
    };
    write!(
        client,
        "HTTP/1.1 {status_text}\r\n\
         content-type: text/plain\r\n\
         content-length: {}\r\n\
         connection: close\r\n\
         \r\n{body}",
        body.len()
    )
}

fn forward_response_body(
    reader: &mut BufReader<TcpStream>,
    client: &mut TcpStream,
    response: &HttpResponseHead,
    captured_body: &mut Vec<u8>,
) -> io::Result<()> {
    if is_chunked(&response.headers) {
        return forward_chunked_body(reader, client, captured_body);
    }
    if let Some(length) = content_length(&response.headers) {
        return forward_fixed_body(reader, client, captured_body, length);
    }
    forward_until_eof(reader, client, captured_body)
}

fn forward_fixed_body(
    reader: &mut BufReader<TcpStream>,
    client: &mut TcpStream,
    captured_body: &mut Vec<u8>,
    length: usize,
) -> io::Result<()> {
    let mut remaining = length;
    let mut buffer = [0_u8; 8192];
    while remaining > 0 {
        let take = remaining.min(buffer.len());
        reader.read_exact(&mut buffer[..take])?;
        client.write_all(&buffer[..take])?;
        captured_body.extend_from_slice(&buffer[..take]);
        remaining -= take;
    }
    Ok(())
}

fn forward_until_eof(
    reader: &mut BufReader<TcpStream>,
    client: &mut TcpStream,
    captured_body: &mut Vec<u8>,
) -> io::Result<()> {
    let mut buffer = [0_u8; 8192];
    loop {
        let bytes = reader.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        client.write_all(&buffer[..bytes])?;
        captured_body.extend_from_slice(&buffer[..bytes]);
    }
    Ok(())
}

fn forward_chunked_body(
    reader: &mut BufReader<TcpStream>,
    client: &mut TcpStream,
    captured_body: &mut Vec<u8>,
) -> io::Result<()> {
    while let Some(size_line) = read_http_line(reader)? {
        client.write_all(&size_line)?;
        let size = parse_chunk_size(&size_line)?;
        if size == 0 {
            forward_chunk_trailers(reader, client)?;
            break;
        }
        let mut chunk = vec![0_u8; size];
        reader.read_exact(&mut chunk)?;
        client.write_all(&chunk)?;
        captured_body.extend_from_slice(&chunk);
        let mut crlf = [0_u8; 2];
        reader.read_exact(&mut crlf)?;
        client.write_all(&crlf)?;
    }
    Ok(())
}

fn forward_chunk_trailers(
    reader: &mut BufReader<TcpStream>,
    client: &mut TcpStream,
) -> io::Result<()> {
    while let Some(line) = read_http_line(reader)? {
        client.write_all(&line)?;
        if line == b"\r\n" || line == b"\n" {
            break;
        }
    }
    Ok(())
}

fn parse_chunk_size(line: &[u8]) -> io::Result<usize> {
    let text = String::from_utf8_lossy(line);
    let size_text = text.trim().split(';').next().unwrap_or_default().trim();
    usize::from_str_radix(size_text, 16).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid chunk size `{size_text}`: {error}"),
        )
    })
}

const fn skip_request_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("host")
        || name.eq_ignore_ascii_case("connection")
        || name.eq_ignore_ascii_case("proxy-connection")
        || name.eq_ignore_ascii_case("keep-alive")
        || name.eq_ignore_ascii_case("transfer-encoding")
        || name.eq_ignore_ascii_case("content-length")
        || name.eq_ignore_ascii_case("accept-encoding")
}

const fn skip_response_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("connection")
        || name.eq_ignore_ascii_case("proxy-connection")
        || name.eq_ignore_ascii_case("keep-alive")
}

fn content_length(headers: &[HttpHeader]) -> Option<usize> {
    headers.iter().find_map(|header| {
        header
            .name
            .eq_ignore_ascii_case("content-length")
            .then(|| header.value.parse::<usize>().ok())
            .flatten()
    })
}

fn is_chunked(headers: &[HttpHeader]) -> bool {
    headers.iter().any(|header| {
        header.name.eq_ignore_ascii_case("transfer-encoding")
            && header
                .value
                .split(',')
                .any(|part| part.trim().eq_ignore_ascii_case("chunked"))
    })
}

fn collect_request_tool_names(value: &Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(tools) = value.get("tools").and_then(Value::as_array) {
        for tool in tools {
            append_tool_definition_names(tool, &mut names);
        }
    }
    if let Some(functions) = value.get("functions").and_then(Value::as_array) {
        for function in functions {
            if let Some(name) = function.get("name").and_then(Value::as_str) {
                names.push(name.to_owned());
            }
        }
    }
    for choice_key in ["tool_choice", "function_call"] {
        if let Some(name) = value
            .get(choice_key)
            .and_then(|choice| choice.get("function").or(Some(choice)))
            .and_then(|function| function.get("name"))
            .and_then(Value::as_str)
        {
            names.push(name.to_owned());
        }
    }
    names.sort();
    names.dedup();
    names
}

fn append_tool_definition_names(value: &Value, names: &mut Vec<String>) {
    if let Some(name) = value
        .get("function")
        .and_then(|function| function.get("name"))
        .or_else(|| value.get("name"))
        .and_then(Value::as_str)
    {
        names.push(name.to_owned());
    }
    if let Some(declarations) = value.get("functionDeclarations").and_then(Value::as_array) {
        for declaration in declarations {
            if let Some(name) = declaration.get("name").and_then(Value::as_str) {
                names.push(name.to_owned());
            }
        }
    }
}

fn summarize_sse_response(body: &str) -> ResponseSummary {
    let events = parse_sse_events(body);
    for event in &events {
        let data = event.data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) == Some("response.completed") {
            if let Some(response) = value.get("response") {
                return summarize_response_value(response);
            }
        }
    }

    let mut chat = StreamingChatAccumulator::default();
    let mut summary = ResponseSummary::default();
    for event in events {
        let data = event.data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        if chat.apply_chunk(&value) {
            continue;
        }
        merge_response_summary(&mut summary, summarize_response_value(&value));
    }
    merge_response_summary(&mut summary, chat.finish());
    summary
}

fn parse_sse_events(body: &str) -> Vec<SseEvent> {
    let normalized = body.replace("\r\n", "\n");
    normalized
        .split("\n\n")
        .filter_map(|block| {
            let mut data = String::new();
            for line in block.lines() {
                if let Some(value) = line.strip_prefix("data:") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(value.trim_start());
                }
            }
            (!data.is_empty()).then_some(SseEvent { data })
        })
        .collect()
}

impl StreamingChatAccumulator {
    fn apply_chunk(&mut self, value: &Value) -> bool {
        let Some(choices) = value.get("choices").and_then(Value::as_array) else {
            return false;
        };
        if self.model.is_none() {
            self.model = value
                .get("model")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
        }
        for choice in choices {
            let Some(delta) = choice.get("delta") else {
                continue;
            };
            if let Some(content) = delta.get("content").and_then(Value::as_str) {
                self.content.push_str(content);
            }
            if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
                for call in tool_calls {
                    let index = call.get("index").and_then(Value::as_u64).unwrap_or(0);
                    let entry = self.tool_calls.entry(index).or_default();
                    if let Some(name) = call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                    {
                        entry.name.push_str(name);
                    }
                    if let Some(arguments) = call
                        .get("function")
                        .and_then(|function| function.get("arguments"))
                        .and_then(Value::as_str)
                    {
                        entry.arguments.push_str(arguments);
                    }
                }
            }
        }
        true
    }

    fn finish(self) -> ResponseSummary {
        ResponseSummary {
            model: self.model,
            tool_calls: self
                .tool_calls
                .into_values()
                .filter(|call| !call.name.is_empty())
                .map(|call| ProxyToolCallLog {
                    name: call.name,
                    arguments: arguments_from_str(&call.arguments),
                })
                .collect(),
            content: self.content,
        }
    }
}

fn summarize_response_value(value: &Value) -> ResponseSummary {
    let mut summary = ResponseSummary::default();
    apply_response_value(value, &mut summary);
    summary
}

fn merge_response_summary(target: &mut ResponseSummary, source: ResponseSummary) {
    if target.model.is_none() {
        target.model = source.model;
    }
    target.tool_calls.extend(source.tool_calls);
    target.content.push_str(&source.content);
}

fn apply_response_value(value: &Value, summary: &mut ResponseSummary) {
    set_model(summary, value.get("model").and_then(Value::as_str));
    set_model(summary, value.get("modelVersion").and_then(Value::as_str));

    if let Some(response) = value.get("response") {
        apply_response_value(response, summary);
    }
    if let Some(item) = value.get("item") {
        apply_response_item(item, summary);
    }
    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        for choice in choices {
            if let Some(message) = choice.get("message") {
                apply_chat_message(message, summary);
            }
        }
    }
    if let Some(output) = value.get("output").and_then(Value::as_array) {
        for item in output {
            apply_response_item(item, summary);
        }
    }
    if value.get("type").and_then(Value::as_str) == Some("function_call") {
        apply_response_item(value, summary);
    }
    if let Some(candidates) = value.get("candidates").and_then(Value::as_array) {
        for candidate in candidates {
            if let Some(parts) = candidate
                .get("content")
                .and_then(|content| content.get("parts"))
                .and_then(Value::as_array)
            {
                for part in parts {
                    apply_gemini_part(part, summary);
                }
            }
        }
    }
    if let Some(content) = value.get("content").and_then(Value::as_array) {
        for block in content {
            apply_anthropic_content_block(block, summary);
        }
    }
}

fn set_model(summary: &mut ResponseSummary, model: Option<&str>) {
    if summary.model.is_none() {
        summary.model = model.map(ToOwned::to_owned);
    }
}

fn apply_chat_message(message: &Value, summary: &mut ResponseSummary) {
    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in tool_calls {
            if let Some(function) = call.get("function") {
                append_function_call(function, summary);
            }
        }
    }
    if let Some(function) = message.get("function_call") {
        append_function_call(function, summary);
    }
    append_content_value(message.get("content"), summary);
}

fn apply_response_item(item: &Value, summary: &mut ResponseSummary) {
    match item.get("type").and_then(Value::as_str) {
        Some("function_call") => {
            if let Some(name) = item.get("name").and_then(Value::as_str) {
                summary.tool_calls.push(ProxyToolCallLog {
                    name: name.to_owned(),
                    arguments: arguments_from_value(item.get("arguments")),
                });
            }
        }
        Some("message") => {
            if let Some(content) = item.get("content").and_then(Value::as_array) {
                for part in content {
                    append_content_value(part.get("text"), summary);
                }
            }
        }
        _ => {}
    }
}

fn apply_gemini_part(part: &Value, summary: &mut ResponseSummary) {
    if let Some(call) = part.get("functionCall") {
        if let Some(name) = call.get("name").and_then(Value::as_str) {
            summary.tool_calls.push(ProxyToolCallLog {
                name: name.to_owned(),
                arguments: arguments_from_value(call.get("args")),
            });
        }
    }
    append_content_value(part.get("text"), summary);
}

fn apply_anthropic_content_block(block: &Value, summary: &mut ResponseSummary) {
    match block.get("type").and_then(Value::as_str) {
        Some("tool_use") => {
            if let Some(name) = block.get("name").and_then(Value::as_str) {
                summary.tool_calls.push(ProxyToolCallLog {
                    name: name.to_owned(),
                    arguments: arguments_from_value(block.get("input")),
                });
            }
        }
        Some("text") => append_content_value(block.get("text"), summary),
        _ => {}
    }
}

fn append_function_call(function: &Value, summary: &mut ResponseSummary) {
    if let Some(name) = function.get("name").and_then(Value::as_str) {
        summary.tool_calls.push(ProxyToolCallLog {
            name: name.to_owned(),
            arguments: arguments_from_value(function.get("arguments")),
        });
    }
}

fn append_content_value(value: Option<&Value>, summary: &mut ResponseSummary) {
    match value {
        Some(Value::String(text)) => summary.content.push_str(text),
        Some(Value::Array(parts)) => {
            for part in parts {
                append_content_value(part.get("text"), summary);
            }
        }
        _ => {}
    }
}

fn arguments_from_value(value: Option<&Value>) -> Value {
    match value {
        Some(Value::String(arguments)) => arguments_from_str(arguments),
        Some(value) => value.clone(),
        None => Value::Null,
    }
}

fn arguments_from_str(arguments: &str) -> Value {
    serde_json::from_str(arguments).unwrap_or_else(|_| Value::String(arguments.to_owned()))
}
