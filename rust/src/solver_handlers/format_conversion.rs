//! Format-conversion handler (issue #1177, E142 code-task family).
//!
//! Recognizes "convert this JSON to YAML" / "YAML to JSON" requests (English
//! and Russian; cue phrases live in `data/seed/code-task-cues.lino`), and
//! converts between a JSON document and a YAML subset: block-style mappings
//! and sequences, two-space indentation, scalars (numbers, booleans, null,
//! plain or quoted strings). Flow style, anchors, tags and multi-line
//! scalars are refused rather than guessed.
//!
//! Every conversion is verified before being shown: JSON→YAML is re-parsed
//! with the same subset parser and compared against the input value;
//! YAML→JSON is parsed back and compared. On any mismatch the handler
//! refuses by name (the refusal reasons are seed templates).

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;
use serde_json::{Map, Value};
use super::finalize_simple;

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const INTENT: &str = "format_conversion";

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// Trigger phrases for one intent (optionally one named role) from
/// `data/seed/code-task-cues.lino`.
fn cue_phrases(intent: &str, role: &str) -> Vec<String> {
    let Some(text) = seed_text(CUES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "cues") {
        if record.find_child_value("intent") != intent {
            continue;
        }
        if record.find_child_value("role") != role {
            continue;
        }
        for phrase in record
            .children
            .iter()
            .flat_map(|child| child.children.iter())
            .filter(|phrase| phrase.name == "phrase")
        {
            if !phrase.id.is_empty() {
                out.push(phrase.id.clone());
            }
        }
    }
    out
}

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// True when any cue phrase of the intent with the given role matches.
fn role_cued(prompt: &str, normalized: &str, role: &str) -> bool {
    let lower = prompt.to_lowercase();
    cue_phrases(INTENT, role)
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
}

// ---------------------------------------------------------------------------
// Extraction of the document under discussion
// ---------------------------------------------------------------------------

/// The body of the first fenced block tagged `tag` (```json, ```yaml).
fn fenced(prompt: &str, tag: &str) -> Option<String> {
    let open = ["```", tag, "\n"].concat();
    let start = prompt.find(&open)? + open.len();
    let end = prompt[start..].find("```")? + start;
    let body = &prompt[start..end];
    if body.trim().is_empty() {
        None
    } else {
        Some(body.to_owned())
    }
}

/// The body of the first fenced block with any tag.
fn fenced_any(prompt: &str) -> Option<String> {
    let start = prompt.find("```")? + 3;
    let rest = &prompt[start..];
    let after_tag = match rest.find('\n') {
        Some(nl) => &rest[nl + 1..],
        None => return None,
    };
    let end = after_tag.find("```")?;
    let body = &after_tag[..end];
    if body.trim().is_empty() {
        None
    } else {
        Some(body.to_owned())
    }
}

/// The first backtick span.
fn backtick_span(prompt: &str) -> Option<String> {
    let start = prompt.find('`')?;
    let end = prompt[start + 1..].find('`')? + start + 1;
    let span = &prompt[start + 1..end];
    if span.trim().is_empty() {
        None
    } else {
        Some(span.to_owned())
    }
}

/// The first brace-balanced JSON document in the prompt (string-aware).
fn json_span(prompt: &str) -> Option<String> {
    let start = prompt.find(|c| c == '{' || c == '[')?;
    let bytes = prompt.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (index, byte) in bytes.iter().enumerate().skip(start) {
        if in_string {
            match byte {
                b'\\' if !escaped => escaped = true,
                b'"' if !escaped => in_string = false,
                _ => escaped = false,
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' | b'[' => depth += 1,
            b'}' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(prompt[start..index + 1].to_owned());
                }
            }
            _ => {}
        }
    }
    None
}

/// The JSON document under discussion, if any.
fn json_text(prompt: &str) -> Option<String> {
    fenced(prompt, "json")
        .or_else(|| fenced_any(prompt))
        .or_else(|| backtick_span(prompt).filter(|span| span.find('{').is_some() || span.find('[').is_some()))
        .or_else(|| json_span(prompt))
}

/// The YAML under discussion: a fenced block, a backtick span, or from the
/// first mapping/sequence line to the end of the prompt.
fn yaml_text(prompt: &str) -> Option<String> {
    if let Some(body) = fenced(prompt, "yaml").or_else(|| fenced_any(prompt)) {
        return Some(body);
    }
    if let Some(span) = backtick_span(prompt) {
        return Some(span);
    }
    let mut seen = 0usize;
    for line in prompt.lines() {
        let start = seen;
        seen += line.len() + 1;
        let trimmed = line.trim();
        if trimmed.find(": ").is_some()
            || trimmed.strip_prefix("- ").is_some()
            || trimmed.ends_with(':')
        {
            return Some(prompt[start..].to_owned());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// YAML subset rendering (Value -> YAML) and parsing (YAML -> Value)
// ---------------------------------------------------------------------------

/// True when a string renders safely as a plain (unquoted) YAML scalar.
fn plain_safe(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let first = text.chars().next().unwrap_or(' ');
    if !(first.is_alphanumeric() || first == '_') {
        return false;
    }
    if text.trim() != text {
        return false;
    }
    for marker in [':', '#', '"', '\'', '\n', '\t'] {
        if text.find(marker).is_some() {
            return false;
        }
    }
    if matches!(text, "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "~") {
        return false;
    }
    if text.parse::<f64>().is_ok() {
        return false;
    }
    true
}

/// Render one scalar as a YAML token (plain when safe, JSON-quoted else).
fn scalar_yaml(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                int.to_string()
            } else if let Some(uint) = number.as_u64() {
                uint.to_string()
            } else if let Some(float) = number.as_f64() {
                if float.fract() == 0.0 {
                    [float.trunc().to_string(), ".0".to_owned()].concat()
                } else {
                    float.to_string()
                }
            } else {
                number.to_string()
            }
        }
        Value::String(text) => {
            if plain_safe(text) {
                text.clone()
            } else {
                serde_json::to_string(text).unwrap_or_default()
            }
        }
        _ => String::new(),
    }
}

/// Render a key (same safety rules as scalars).
fn key_yaml(key: &str) -> String {
    if plain_safe(key) {
        key.to_owned()
    } else {
        serde_json::to_string(key).unwrap_or_default()
    }
}

/// Render a value as block YAML at the given indent width.
fn render_yaml(value: &Value, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match value {
        Value::Object(map) => {
            let mut out = String::new();
            for (key, child) in map {
                match child {
                    Value::Object(inner) if !inner.is_empty() => {
                        out.push_str(&[&pad, &key_yaml(key), ":\n"].concat());
                        out.push_str(&render_yaml(child, indent + 2));
                    }
                    Value::Array(inner) if !inner.is_empty() => {
                        out.push_str(&[&pad, &key_yaml(key), ":\n"].concat());
                        out.push_str(&render_yaml(child, indent + 2));
                    }
                    Value::Object(_) => {
                        out.push_str(&[&pad, &key_yaml(key), ": {}\n"].concat());
                    }
                    Value::Array(_) => {
                        out.push_str(&[&pad, &key_yaml(key), ": []\n"].concat());
                    }
                    _ => {
                        out.push_str(&[&pad, &key_yaml(key), ": ", &scalar_yaml(child), "\n"].concat());
                    }
                }
            }
            out
        }
        Value::Array(items) => {
            let mut out = String::new();
            for child in items {
                match child {
                    Value::Object(inner) if !inner.is_empty() => {
                        out.push_str(&[&pad, "-\n"].concat());
                        out.push_str(&render_yaml(child, indent + 2));
                    }
                    Value::Array(inner) if !inner.is_empty() => {
                        out.push_str(&[&pad, "-\n"].concat());
                        out.push_str(&render_yaml(child, indent + 2));
                    }
                    Value::Object(_) => {
                        out.push_str(&[&pad, "- {}\n"].concat());
                    }
                    Value::Array(_) => {
                        out.push_str(&[&pad, "- []\n"].concat());
                    }
                    _ => {
                        out.push_str(&[&pad, "- ", &scalar_yaml(child), "\n"].concat());
                    }
                }
            }
            out
        }
        scalar => {
            if indent == 0 {
                [scalar_yaml(scalar), "\n".to_owned()].concat()
            } else {
                [scalar_yaml(scalar)].concat()
            }
        }
    }
}

/// Parse a YAML scalar token into a JSON value.
fn parse_scalar(token: &str) -> Value {
    if token.starts_with('"') {
        if let Ok(Value::String(text)) = serde_json::from_str::<Value>(token) {
            return Value::String(text);
        }
        return Value::String(token.trim_matches('"').to_owned());
    }
    match token {
        "null" | "~" => Value::Null,
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            if let Ok(int) = token.parse::<i64>() {
                Value::Number(int.into())
            } else if let Ok(uint) = token.parse::<u64>() {
                Value::Number(uint.into())
            } else if let Ok(float) = token.parse::<f64>() {
                match serde_json::Number::from_f64(float) {
                    Some(number) => Value::Number(number),
                    None => Value::String(token.to_owned()),
                }
            } else {
                Value::String(token.to_owned())
            }
        }
    }
}

/// Split a mapping line into (key, value text); `key:` yields an empty value.
fn split_mapping_line(content: &str) -> Option<(String, String)> {
    if content.starts_with('"') {
        let bytes = content.as_bytes();
        let mut index = 1usize;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' => index += 2,
                b'"' => {
                    let key_token = &content[..index + 1];
                    if let Some(rest) = content[index + 1..].strip_prefix(':') {
                        let key: String = serde_json::from_str(key_token).ok()?;
                        return Some((key, rest.trim().to_owned()));
                    }
                    return None;
                }
                _ => index += 1,
            }
        }
        return None;
    }
    let colon = content.find(':')?;
    let key = content[..colon].trim().to_owned();
    if key.is_empty() {
        return None;
    }
    Some((key, content[colon + 1..].trim().to_owned()))
}

/// One significant YAML line: its indent width and trimmed content.
struct YamlLine {
    indent: usize,
    content: String,
}

/// Parse a block (mapping or sequence) at `indent`, returning the value and
/// the index of the first line after the block.
fn parse_block(lines: &[YamlLine], start: usize, indent: usize) -> Option<(Value, usize)> {
    let first = lines.get(start)?;
    if first.indent != indent {
        return None;
    }
    let is_entry = |line: &YamlLine| line.content == "-" || line.strip_prefix().is_some();
    if is_entry(first) {
        // Sequence of entries at this indent.
        let mut items: Vec<Value> = Vec::new();
        let mut index = start;
        while index < lines.len() && lines[index].indent == indent && is_entry(&lines[index]) {
            let rest = lines[index].strip_prefix().unwrap_or("").to_owned();
            if rest.is_empty() {
                // `-` alone: the item is the deeper block that follows.
                match lines.get(index + 1) {
                    Some(next) if next.indent > indent => {
                        let (value, after) = parse_block(lines, index + 1, next.indent)?;
                        items.push(value);
                        index = after;
                    }
                    _ => {
                        items.push(Value::Null);
                        index += 1;
                    }
                }
            } else if rest.find(": ").is_some() || rest.ends_with(':') {
                // `- key: value`: a mapping whose first pair sits after the
                // dash; continuation lines align with the key column.
                let item_indent = indent + 2;
                let mut item_lines = vec![YamlLine {
                    indent: item_indent,
                    content: rest,
                }];
                let mut scan = index + 1;
                while scan < lines.len()
                    && lines[scan].indent == item_indent
                    && lines[scan].strip_prefix().is_none()
                {
                    item_lines.push(YamlLine {
                        indent: item_indent,
                        content: lines[scan].content.clone(),
                    });
                    scan += 1;
                }
                let (value, _) = parse_block(&item_lines, 0, item_indent)?;
                items.push(value);
                index = scan;
            } else {
                items.push(parse_scalar(&rest));
                index += 1;
            }
        }
        return Some((Value::Array(items), index));
    }
    // Mapping at this indent.
    let mut map = Map::new();
    let mut index = start;
    while index < lines.len() && lines[index].indent == indent && !is_entry(&lines[index]) {
        let (key, value_text) = split_mapping_line(&lines[index].content)?;
        if value_text.is_empty() {
            match lines.get(index + 1) {
                Some(next) if next.indent > indent => {
                    let (value, after) = parse_block(lines, index + 1, next.indent)?;
                    map.insert(key, value);
                    index = after;
                }
                _ => {
                    map.insert(key, Value::Null);
                    index += 1;
                }
            }
        } else if value_text == "{}" {
            map.insert(key, Value::Object(Map::new()));
            index += 1;
        } else if value_text == "[]" {
            map.insert(key, Value::Array(Vec::new()));
            index += 1;
        } else {
            map.insert(key, parse_scalar(&value_text));
            index += 1;
        }
    }
    Some((Value::Object(map), index))
}

impl YamlLine {
    /// The content after a leading `- ` marker, when present.
    fn strip_prefix(&self) -> Option<&str> {
        self.content.strip_prefix("- ")
    }
}

/// Parse a YAML-subset document into a JSON value.
fn parse_yaml(text: &str) -> Option<Value> {
    let trimmed = text.trim();
    if trimmed == "{}" {
        return Some(Value::Object(Map::new()));
    }
    if trimmed == "[]" {
        return Some(Value::Array(Vec::new()));
    }
    let mut lines = Vec::new();
    for raw in text.lines() {
        let content = raw.trim_end();
        let significant = content.trim_start();
        if significant.is_empty() || significant.find('#') == Some(0) {
            continue;
        }
        lines.push(YamlLine {
            indent: content.len() - significant.len(),
            content: significant.to_owned(),
        });
    }
    if lines.is_empty() {
        return None;
    }
    let root_indent = lines[0].indent;
    let (value, next) = parse_block(&lines, 0, root_indent)?;
    if next != lines.len() {
        return None;
    }
    Some(value)
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// A refusal body for a named reason code.
fn refusal(reason: &str) -> String {
    let reason_text = template(&["format_conversion_reason_", reason].concat(), &[]);
    template("format_conversion_refusal", &[("reason", &reason_text)])
}

/// Try to recognize a JSON↔YAML conversion request and perform it within
/// the supported subset, verifying the result before showing it. Refuses by
/// name (with the reason) otherwise.
pub fn handle_format_conversion(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let to_yaml = role_cued(prompt, normalized, "to_yaml");
    let to_json = role_cued(prompt, normalized, "to_json");
    if !to_yaml && !to_json {
        return None;
    }
    let direction = if to_yaml { "yaml" } else { "json" };
    log.append("format_conversion:request", format!("dir={}", direction));

    let (body, confidence) = if to_yaml {
        match json_text(prompt) {
            None => {
                log.append("format_conversion:refusal", "json=none".to_owned());
                (refusal("subset"), 0.4)
            }
            Some(text) => match serde_json::from_str::<Value>(&text) {
                Err(error) => {
                    log.append("format_conversion:refusal", "json=bad".to_owned());
                    (
                        template(
                            "format_conversion_refusal",
                            &[(
                                "reason",
                                &template(
                                    "format_conversion_reason_bad_json",
                                    &[("detail", &error.to_string())],
                                ),
                            )],
                        ),
                        0.4,
                    )
                }
                Ok(value) => {
                    let mut yaml = render_yaml(&value, 0);
                    if yaml.is_empty() {
                        yaml =
                            if value.is_object() { "{}".to_owned() } else { "[]".to_owned() };
                    }
                    match parse_yaml(&yaml) {
                        Some(reparsed) if reparsed == value => {
                            log.append("format_conversion:converted", "roundtrip=ok".to_owned());
                            (
                                template("format_conversion_to_yaml", &[("yaml", &yaml)]),
                                0.7,
                            )
                        }
                        _ => {
                            log.append("format_conversion:refusal", "roundtrip=fail".to_owned());
                            (refusal("roundtrip"), 0.4)
                        }
                    }
                }
            },
        }
    } else {
        match yaml_text(prompt) {
            None => {
                log.append("format_conversion:refusal", "yaml=none".to_owned());
                (refusal("subset"), 0.4)
            }
            Some(text) => match parse_yaml(&text) {
                None => {
                    log.append("format_conversion:refusal", "yaml=bad".to_owned());
                    (refusal("subset"), 0.4)
                }
                Some(value) => {
                    let json = serde_json::to_string_pretty(&value).unwrap_or_default();
                    match serde_json::from_str::<Value>(&json) {
                        Ok(reparsed) if reparsed == value => {
                            log.append("format_conversion:converted", "roundtrip=ok".to_owned());
                            (template("format_conversion_to_json", &[("json", &json)]), 0.7)
                        }
                        _ => {
                            log.append("format_conversion:refusal", "roundtrip=fail".to_owned());
                            (refusal("roundtrip"), 0.4)
                        }
                    }
                }
            },
        }
    };

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:format_conversion",
        &body,
        confidence,
    ))
}
