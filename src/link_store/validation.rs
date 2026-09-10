//! Strict validation of the `demo_memory` Links Notation document shape.
//!
//! Split out of `link_store.rs` when that file passed the 1000-line cap the
//! `check_file_size` gate enforces. These five functions form one closed group:
//! `validate_demo_memory_document` is the only entry point, and the other four
//! are reached only from it, so the seam adds no coupling that was not already
//! a call.

use super::LinkStoreError;
use crate::memory::ROOT_HEADER;

pub(super) fn validate_demo_memory_document(text: &str) -> Result<(), LinkStoreError> {
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let indent = line.chars().take_while(|ch| *ch == ' ').count();
        let content = &line[indent..];
        match indent {
            0 if content == ROOT_HEADER => {}
            2 if content.starts_with("schema_version ") => validate_schema_version_line(content)?,
            2 => validate_event_line(content)?,
            4 => validate_field_line(content)?,
            _ => {
                return Err(LinkStoreError::IllFormedLinksNotation(format!(
                    "unexpected indentation or record line: {content}"
                )));
            }
        }
    }
    Ok(())
}

fn validate_schema_version_line(content: &str) -> Result<(), LinkStoreError> {
    let Some(rest) = content.strip_prefix("schema_version ") else {
        return Err(LinkStoreError::IllFormedLinksNotation(String::from(
            "invalid schema version marker",
        )));
    };
    validate_strict_quoted(rest)?;
    let value = crate::memory::parse_quoted(rest).unwrap_or_default();
    if value.parse::<u32>().is_err() {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "invalid_schema_version:value={value}"
        )));
    }
    Ok(())
}

fn validate_event_line(content: &str) -> Result<(), LinkStoreError> {
    let Some(rest) = content.strip_prefix("event ") else {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "expected event record, got {content}"
        )));
    };
    validate_strict_quoted(rest)
}

fn validate_field_line(content: &str) -> Result<(), LinkStoreError> {
    let Some((key, rest)) = content.split_once(' ') else {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "expected field value, got {content}"
        )));
    };
    if !key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "invalid field name {key}"
        )));
    }
    validate_strict_quoted(rest)
}

fn validate_strict_quoted(rest: &str) -> Result<(), LinkStoreError> {
    let trimmed = rest.trim_start();
    let bytes = trimmed.as_bytes();
    if bytes.first() != Some(&b'"') {
        return Err(LinkStoreError::IllFormedLinksNotation(format!(
            "expected quoted value, got {rest}"
        )));
    }
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => {
                if trimmed[index + 1..].trim().is_empty() {
                    return Ok(());
                }
                return Err(LinkStoreError::IllFormedLinksNotation(format!(
                    "unexpected trailing content after quoted value: {}",
                    &trimmed[index + 1..]
                )));
            }
            _ => index += 1,
        }
    }
    Err(LinkStoreError::IllFormedLinksNotation(String::from(
        "unterminated quoted value",
    )))
}
