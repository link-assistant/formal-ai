//! Convert captured shared chat transcripts into the portable `demo_memory`
//! event log used by the browser demo and CLI.

use std::error::Error;
use std::fmt;

use serde_json::{Map, Value};

use crate::memory::{export_links_notation, MemoryEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedDialogFormat {
    Auto,
    ChatGptShareHtml,
    MarkdownTranscript,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SharedDialogMetadata {
    pub source_url: Option<String>,
    pub demo_label: Option<String>,
    pub conversation_id: Option<String>,
    pub conversation_title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedDialog {
    pub title: Option<String>,
    pub conversation_id: Option<String>,
    pub turns: Vec<SharedDialogTurn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedDialogTurn {
    pub id: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedDialogError {
    EmptyDialog,
    Parse(String),
    UnsupportedFormat(String),
}

impl fmt::Display for SharedDialogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDialog => formatter.write_str("no visible dialog turns were found"),
            Self::Parse(message) | Self::UnsupportedFormat(message) => formatter.write_str(message),
        }
    }
}

impl Error for SharedDialogError {}

pub fn convert_shared_dialog_to_demo_memory(
    input: &str,
    format: SharedDialogFormat,
    metadata: &SharedDialogMetadata,
) -> Result<String, SharedDialogError> {
    let dialog = parse_shared_dialog(input, format, metadata)?;
    let events = shared_dialog_to_memory_events(&dialog, metadata);
    Ok(export_links_notation(&events))
}

pub fn parse_shared_dialog(
    input: &str,
    format: SharedDialogFormat,
    metadata: &SharedDialogMetadata,
) -> Result<SharedDialog, SharedDialogError> {
    let concrete_format = match format {
        SharedDialogFormat::Auto => detect_format(input)?,
        other => other,
    };
    match concrete_format {
        SharedDialogFormat::Auto => unreachable!("auto format is resolved before parsing"),
        SharedDialogFormat::ChatGptShareHtml => parse_chatgpt_share_html(input, metadata),
        SharedDialogFormat::MarkdownTranscript => parse_markdown_transcript(input, metadata),
    }
}

#[must_use]
pub fn shared_dialog_to_memory_events(
    dialog: &SharedDialog,
    metadata: &SharedDialogMetadata,
) -> Vec<MemoryEvent> {
    let conversation_id = metadata
        .conversation_id
        .clone()
        .or_else(|| dialog.conversation_id.clone());
    let conversation_title = metadata
        .conversation_title
        .clone()
        .or_else(|| dialog.title.clone());
    let evidence: Vec<String> = metadata.source_url.iter().cloned().collect();

    dialog
        .turns
        .iter()
        .enumerate()
        .map(|(index, turn)| MemoryEvent {
            id: if turn.id.is_empty() {
                format!("shared-dialog-turn-{}", index + 1)
            } else {
                turn.id.clone()
            },
            role: Some(turn.role.clone()),
            content: Some(turn.content.clone()),
            demo_label: metadata.demo_label.clone(),
            conversation_id: conversation_id.clone(),
            conversation_title: conversation_title.clone(),
            evidence: evidence.clone(),
            ..MemoryEvent::default()
        })
        .collect()
}

fn detect_format(input: &str) -> Result<SharedDialogFormat, SharedDialogError> {
    if input.contains("__reactRouterContext.streamController.enqueue(")
        || input.contains("linear_conversation")
    {
        return Ok(SharedDialogFormat::ChatGptShareHtml);
    }
    if looks_like_google_ai_mode_interstitial(input) {
        return Err(SharedDialogError::UnsupportedFormat(String::from(
            "static Google AI Mode capture did not include a replayable transcript; use browser-backed web-capture support",
        )));
    }
    Ok(SharedDialogFormat::MarkdownTranscript)
}

fn parse_chatgpt_share_html(
    input: &str,
    metadata: &SharedDialogMetadata,
) -> Result<SharedDialog, SharedDialogError> {
    let table = extract_chatgpt_devalue_table(input)?;
    let resolved = resolve_devalue_root(&table)?;
    let data = find_object_with_array_key(&resolved, "linear_conversation").ok_or_else(|| {
        SharedDialogError::Parse(String::from(
            "ChatGPT share capture did not contain linear_conversation data",
        ))
    })?;
    let linear = data
        .get("linear_conversation")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            SharedDialogError::Parse(String::from(
                "ChatGPT linear_conversation field was not an array",
            ))
        })?;
    let mut turns = Vec::new();
    for item in linear {
        if let Some(turn) = chatgpt_turn_from_item(item) {
            turns.push(turn);
        }
    }
    if turns.is_empty() {
        return Err(SharedDialogError::EmptyDialog);
    }

    let title = metadata
        .conversation_title
        .clone()
        .or_else(|| string_field(data, "title"))
        .or_else(|| title_from_html(input));
    let conversation_id = metadata
        .conversation_id
        .clone()
        .or_else(|| string_field(data, "conversation_id"))
        .or_else(|| chatgpt_share_id(metadata));

    Ok(SharedDialog {
        title,
        conversation_id,
        turns,
    })
}

fn chatgpt_turn_from_item(item: &Value) -> Option<SharedDialogTurn> {
    let message = item.get("message").unwrap_or(item);
    let role = message
        .get("author")
        .and_then(|author| author.get("role"))
        .and_then(Value::as_str)?;
    if role != "user" && role != "assistant" {
        return None;
    }
    if message_is_hidden(message) {
        return None;
    }
    let content = message
        .get("content")
        .map(message_content_text)
        .unwrap_or_default()
        .trim()
        .to_owned();
    if content.is_empty() {
        return None;
    }
    let id = message
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    Some(SharedDialogTurn {
        id,
        role: role.to_owned(),
        content,
    })
}

fn message_is_hidden(message: &Value) -> bool {
    message
        .get("metadata")
        .and_then(|metadata| metadata.get("is_visually_hidden_from_conversation"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn message_content_text(content: &Value) -> String {
    if let Some(parts) = content.get("parts").and_then(Value::as_array) {
        let mut texts = Vec::new();
        for part in parts {
            if let Some(text) = part.as_str() {
                if !text.is_empty() {
                    texts.push(text.to_owned());
                }
            } else if let Some(text) = part.get("text").and_then(Value::as_str) {
                if !text.is_empty() {
                    texts.push(text.to_owned());
                }
            }
        }
        return texts.join("\n\n");
    }
    content
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn extract_chatgpt_devalue_table(input: &str) -> Result<Vec<Value>, SharedDialogError> {
    for chunk in extract_react_router_stream_chunks(input)? {
        let trimmed = chunk.trim_start();
        if !trimmed.starts_with('[') {
            continue;
        }
        let value = serde_json::from_str::<Value>(trimmed).map_err(|error| {
            SharedDialogError::Parse(format!(
                "failed to parse ChatGPT React Router payload as JSON: {error}"
            ))
        })?;
        if let Value::Array(table) = value {
            return Ok(table);
        }
    }
    Err(SharedDialogError::Parse(String::from(
        "ChatGPT share capture did not contain a JSON devalue table",
    )))
}

fn extract_react_router_stream_chunks(input: &str) -> Result<Vec<String>, SharedDialogError> {
    const ENQUEUE_MARKER: &str = "window.__reactRouterContext.streamController.enqueue(";

    let mut chunks = Vec::new();
    let mut cursor = 0;
    let bytes = input.as_bytes();
    while let Some(relative) = input[cursor..].find(ENQUEUE_MARKER) {
        let mut literal_start = cursor + relative + ENQUEUE_MARKER.len();
        while matches!(bytes.get(literal_start), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            literal_start += 1;
        }
        if bytes.get(literal_start) != Some(&b'"') {
            cursor = literal_start;
            continue;
        }
        let (literal, literal_len) = extract_json_string_literal(&input[literal_start..])?;
        let chunk = serde_json::from_str::<String>(literal).map_err(|error| {
            SharedDialogError::Parse(format!(
                "failed to decode ChatGPT React Router stream string: {error}"
            ))
        })?;
        chunks.push(chunk);
        cursor = literal_start + literal_len;
    }
    if chunks.is_empty() {
        return Err(SharedDialogError::Parse(String::from(
            "ChatGPT share capture did not contain React Router stream chunks",
        )));
    }
    Ok(chunks)
}

fn extract_json_string_literal(input: &str) -> Result<(&str, usize), SharedDialogError> {
    let bytes = input.as_bytes();
    if bytes.first() != Some(&b'"') {
        return Err(SharedDialogError::Parse(String::from(
            "expected a JSON string literal",
        )));
    }
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => return Ok((&input[..=index], index + 1)),
            _ => index += 1,
        }
    }
    Err(SharedDialogError::Parse(String::from(
        "unterminated JSON string literal",
    )))
}

fn resolve_devalue_root(table: &[Value]) -> Result<Value, SharedDialogError> {
    if table.is_empty() {
        return Err(SharedDialogError::Parse(String::from(
            "ChatGPT devalue table was empty",
        )));
    }
    Ok(resolve_table_index(table, 0, &mut Vec::new()))
}

fn resolve_table_index(table: &[Value], index: usize, stack: &mut Vec<usize>) -> Value {
    if index >= table.len() || stack.contains(&index) {
        return Value::Null;
    }
    stack.push(index);
    let value = resolve_table_value(table, &table[index], stack);
    stack.pop();
    value
}

fn resolve_table_value(table: &[Value], value: &Value, stack: &mut Vec<usize>) -> Value {
    match value {
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| resolve_encoded_reference(table, item, stack))
                .collect(),
        ),
        Value::Object(map) => {
            let mut resolved = Map::new();
            for (encoded_key, encoded_value) in map {
                let key = resolve_object_key(table, encoded_key, stack);
                let value = resolve_encoded_reference(table, encoded_value, stack);
                resolved.insert(key, value);
            }
            Value::Object(resolved)
        }
        other => other.clone(),
    }
}

fn resolve_encoded_reference(table: &[Value], value: &Value, stack: &mut Vec<usize>) -> Value {
    if let Some(index) = value.as_i64() {
        if index < 0 {
            return Value::Null;
        }
        let Ok(index) = usize::try_from(index) else {
            return Value::Null;
        };
        return resolve_table_index(table, index, stack);
    }
    resolve_table_value(table, value, stack)
}

fn resolve_object_key(table: &[Value], encoded_key: &str, stack: &mut Vec<usize>) -> String {
    if let Some(raw_index) = encoded_key.strip_prefix('_') {
        if let Ok(index) = raw_index.parse::<usize>() {
            if let Value::String(key) = resolve_table_index(table, index, stack) {
                return key;
            }
        }
    }
    encoded_key.to_owned()
}

fn find_object_with_array_key<'a>(value: &'a Value, key: &str) -> Option<&'a Map<String, Value>> {
    match value {
        Value::Object(map) => {
            if map.get(key).and_then(Value::as_array).is_some() {
                return Some(map);
            }
            map.values()
                .find_map(|child| find_object_with_array_key(child, key))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|child| find_object_with_array_key(child, key)),
        _ => None,
    }
}

fn parse_markdown_transcript(
    input: &str,
    metadata: &SharedDialogMetadata,
) -> Result<SharedDialog, SharedDialogError> {
    let mut turns = Vec::new();
    let mut current_role: Option<&'static str> = None;
    let mut current_lines = Vec::new();

    for line in input.lines() {
        if let Some((role, rest)) = markdown_turn_prefix(line) {
            push_markdown_turn(&mut turns, current_role.take(), &current_lines);
            current_role = Some(role);
            current_lines.clear();
            current_lines.push(rest.trim_start().to_owned());
        } else if current_role.is_some() {
            current_lines.push(line.to_owned());
        }
    }
    push_markdown_turn(&mut turns, current_role.take(), &current_lines);

    if turns.is_empty() {
        return Err(SharedDialogError::EmptyDialog);
    }

    Ok(SharedDialog {
        title: metadata.conversation_title.clone(),
        conversation_id: metadata.conversation_id.clone(),
        turns,
    })
}

fn markdown_turn_prefix(line: &str) -> Option<(&'static str, &str)> {
    let trimmed = line.trim_start();
    for (prefix, role) in [
        ("U:", "user"),
        ("User:", "user"),
        ("A:", "assistant"),
        ("Assistant:", "assistant"),
    ] {
        if trimmed.len() >= prefix.len() && trimmed[..prefix.len()].eq_ignore_ascii_case(prefix) {
            return Some((role, &trimmed[prefix.len()..]));
        }
    }
    None
}

fn push_markdown_turn(
    turns: &mut Vec<SharedDialogTurn>,
    role: Option<&'static str>,
    lines: &[String],
) {
    let Some(role) = role else {
        return;
    };
    let content = trimmed_content_lines(lines);
    if content.is_empty() {
        return;
    }
    let id = format!("markdown-turn-{}", turns.len() + 1);
    turns.push(SharedDialogTurn {
        id,
        role: role.to_owned(),
        content,
    });
}

fn trimmed_content_lines(lines: &[String]) -> String {
    let Some(start) = lines.iter().position(|line| !line.trim().is_empty()) else {
        return String::new();
    };
    let end = lines
        .iter()
        .rposition(|line| !line.trim().is_empty())
        .unwrap_or(start);
    lines[start..=end].join("\n").trim().to_owned()
}

fn string_field(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(ToOwned::to_owned)
}

fn title_from_html(input: &str) -> Option<String> {
    let start = input.find("<title>")? + "<title>".len();
    let end = input[start..].find("</title>")? + start;
    let title = &input[start..end];
    Some(
        title
            .strip_prefix("ChatGPT - ")
            .unwrap_or(title)
            .replace("&#x27;", "'"),
    )
}

fn chatgpt_share_id(metadata: &SharedDialogMetadata) -> Option<String> {
    let url = metadata.source_url.as_deref()?;
    let marker = "/share/";
    let start = url.find(marker)? + marker.len();
    let tail = &url[start..];
    let end = tail.find(['?', '/', '#']).unwrap_or(tail.len());
    Some(tail[..end].to_owned())
}

fn looks_like_google_ai_mode_interstitial(input: &str) -> bool {
    input.contains("share.google/aimode")
        || (input.contains("Google Search")
            && input.contains("If you're having trouble accessing Google Search"))
        || (input.contains("/search?q=") && input.contains("enablejs"))
}
