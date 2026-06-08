//! JSON-to-LiNo cache encoding.
//!
//! The cache format maps JSON object keys to native Links Notation ids and keeps
//! source strings as scalar references. Checked-in Wikidata cache files are a
//! canonical projection of the raw JSON snapshot stored next to each `.lino`
//! file: the `LiNo` side keeps source labels, descriptions, aliases, lexeme
//! lemmas, and entity metadata without dragging the full transitive claim graph
//! into seed grounding.

use serde_json::{Map, Number, Value};

use crate::seed::parser::{parse_lino, LinoNode};

const LEGACY_ARRAY_MARKER: &str = "[]";
const LEGACY_EMPTY_OBJECT_MARKER: &str = "{}";
const LEGACY_ROOT_OBJECT: &str = "json-object";
const LEGACY_ROOT_ARRAY: &str = "json-array";
const LEGACY_ARRAY_ITEM_PREFIX: &str = "at-";
const ARRAY_ENTRY: &str = "entry";
const STRING_MARKER: &str = "string";

#[must_use]
pub fn json_to_lino(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Object(object) => write_object_entries(&mut out, 0, object, None),
        _ => write_named_value(&mut out, 0, "value", value),
    }
    out
}

#[must_use]
pub fn json_cache_file(root_id: &str, value: &Value) -> String {
    let mut out = String::new();
    if let Some(entity) = wikidata_entity_projection(value, root_id) {
        write_entity_cache_file(&mut out, root_id, &entity);
    } else {
        write_line(&mut out, 0, root_id, None, None);
        write_named_value(&mut out, 2, "value", value);
    }
    out
}

#[must_use]
pub fn json_cache_projection(root_id: &str, value: &Value) -> Value {
    if let Some(entity) = wikidata_entity_projection(value, root_id) {
        let mut entities = Map::new();
        entities.insert(root_id.to_string(), Value::Object(entity));
        let mut root = Map::new();
        root.insert(String::from("entities"), Value::Object(entities));
        return Value::Object(root);
    }
    value.clone()
}

pub fn lino_to_json(text: &str) -> Result<Value, String> {
    let tree = parse_lino(text);
    if let Some(node) = find_legacy_json_node(&tree) {
        return parse_legacy_json_value(node);
    }
    parse_compact_document(&tree)
}

fn wikidata_entity_projection(value: &Value, root_id: &str) -> Option<Map<String, Value>> {
    let entity = single_entity(value, root_id)?;
    let mut projected = Map::new();
    for key in [
        "type",
        "datatype",
        "labels",
        "descriptions",
        "aliases",
        "lemmas",
        "lexicalCategory",
        "language",
    ] {
        let Some(value) = entity.get(key) else {
            continue;
        };
        projected.insert(key.to_string(), project_entity_field(key, value));
    }
    Some(projected)
}

fn single_entity<'a>(value: &'a Value, root_id: &str) -> Option<&'a Map<String, Value>> {
    let object = value.as_object()?;
    let entities = object.get("entities")?.as_object()?;
    entities.get(root_id)?.as_object()
}

fn project_entity_field(key: &str, value: &Value) -> Value {
    match key {
        "labels" | "descriptions" | "lemmas" => {
            let Some(object) = value.as_object() else {
                return value.clone();
            };
            let mut projected = Map::new();
            for (language, entry) in object {
                if let Some(text) = language_value(entry, language) {
                    projected.insert(language.clone(), Value::String(text.to_string()));
                }
            }
            Value::Object(projected)
        }
        "aliases" => {
            let Some(object) = value.as_object() else {
                return value.clone();
            };
            let mut projected = Map::new();
            for (language, entries) in object {
                let Value::Array(entries) = entries else {
                    continue;
                };
                let aliases: Vec<Value> = entries
                    .iter()
                    .filter_map(|entry| language_value(entry, language))
                    .map(|text| Value::String(text.to_string()))
                    .collect();
                if !aliases.is_empty() {
                    projected.insert(language.clone(), Value::Array(aliases));
                }
            }
            Value::Object(projected)
        }
        _ => value.clone(),
    }
}

fn language_value<'a>(value: &'a Value, expected_language: &str) -> Option<&'a str> {
    let object = value.as_object()?;
    let language = object.get("language")?.as_str()?;
    if language != expected_language {
        return None;
    }
    object.get("value")?.as_str()
}

fn write_entity_cache_file(out: &mut String, root_id: &str, entity: &Map<String, Value>) {
    write_line(out, 0, root_id, None, None);
    write_object_entries(out, 2, entity, None);
}

fn write_object_entries(
    out: &mut String,
    indent: usize,
    object: &Map<String, Value>,
    skip_key: Option<&str>,
) {
    for (key, value) in object {
        if skip_key.is_some_and(|skip_key| skip_key == key) {
            continue;
        }
        write_named_value(out, indent, key, value);
    }
}

fn write_named_value(out: &mut String, indent: usize, name: &str, value: &Value) {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            let token = scalar_value_token(value);
            write_line(out, indent, name, Some(&token), None);
        }
        Value::Array(values) => {
            if values.iter().all(is_scalar) {
                let token = scalar_array_token(values);
                write_line(out, indent, name, Some(&token), None);
            } else {
                write_line(out, indent, name, None, None);
                for value in values {
                    write_array_value(out, indent + 2, value);
                }
            }
        }
        Value::Object(object) if object.is_empty() => {
            write_line(out, indent, name, None, None);
        }
        Value::Object(object) => {
            write_line(out, indent, name, None, None);
            write_object_entries(out, indent + 2, object, None);
        }
    }
}

fn write_array_value(out: &mut String, indent: usize, value: &Value) {
    if let Value::String(value) = value {
        let token = quoted_string(value);
        write_line(out, indent, ARRAY_ENTRY, Some(&token), None);
    } else if is_scalar(value) {
        let token = scalar_value_token(value);
        write_line(out, indent, &token, None, None);
    } else if let Value::Object(object) = value {
        write_line(out, indent, ARRAY_ENTRY, None, None);
        if !object.is_empty() {
            write_object_entries(out, indent + 2, object, None);
        }
    } else {
        write_named_value(out, indent, ARRAY_ENTRY, value);
    }
}

fn write_line(
    out: &mut String,
    indent: usize,
    name: &str,
    value: Option<&str>,
    comment: Option<&str>,
) {
    out.extend(std::iter::repeat(' ').take(indent));
    out.push_str(name);
    if let Some(value) = value {
        out.push(' ');
        out.push_str(value);
    }
    if let Some(comment) = comment {
        out.push_str(" # ");
        out.push_str(comment);
    }
    out.push('\n');
}

const fn is_scalar(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

fn scalar_value_token(value: &Value) -> String {
    match value {
        Value::Null => String::from("null"),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => string_token(value),
        Value::Array(_) | Value::Object(_) => String::new(),
    }
}

fn scalar_array_token(values: &[Value]) -> String {
    let tokens: Vec<String> = values.iter().map(scalar_array_item_token).collect();
    format!("({})", tokens.join(" "))
}

fn scalar_array_item_token(value: &Value) -> String {
    match value {
        Value::Null => String::from("null"),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => quoted_string(value),
        Value::Array(_) | Value::Object(_) => String::new(),
    }
}

fn quoted_string(value: &str) -> String {
    if value.contains('"') {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
            .replace('\'', "''");
        return format!("'{escaped}'");
    }
    serde_json::to_string(value).unwrap_or_else(|_| String::from("\"\""))
}

fn string_token(value: &str) -> String {
    if is_bare_reference(value) {
        value.to_string()
    } else {
        quoted_string(value)
    }
}

fn is_bare_reference(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '$')
        })
}

fn find_legacy_json_node(node: &LinoNode) -> Option<&LinoNode> {
    if is_legacy_json_node(node) {
        return Some(node);
    }
    node.children.iter().find_map(find_legacy_json_node)
}

fn is_legacy_json_node(node: &LinoNode) -> bool {
    matches!(
        node.name.as_str(),
        LEGACY_ROOT_OBJECT
            | LEGACY_ROOT_ARRAY
            | "json-null"
            | "json-boolean"
            | "json-number"
            | "json-string"
    )
}

fn parse_legacy_json_value(node: &LinoNode) -> Result<Value, String> {
    match node.name.as_str() {
        "json-null" => Ok(Value::Null),
        "json-boolean" => match node.id.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            other => Err(format!("invalid json boolean `{other}`")),
        },
        "json-number" => {
            let number: Number = serde_json::from_str(&node.id)
                .map_err(|error| format!("invalid json number `{}`: {error}", node.id))?;
            Ok(Value::Number(number))
        }
        "json-string" => Ok(Value::String(decode_text(&node.id)?)),
        LEGACY_ROOT_ARRAY => parse_legacy_array(node),
        LEGACY_ROOT_OBJECT => parse_legacy_object(node),
        other => Err(format!("unknown json node `{other}`")),
    }
}

fn parse_legacy_array(node: &LinoNode) -> Result<Value, String> {
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            continue;
        }
        let value_node = child
            .children
            .iter()
            .find(|candidate| is_legacy_json_node(candidate))
            .ok_or_else(|| format!("array item `{}` has no json value", child.id))?;
        items.push(parse_legacy_json_value(value_node)?);
    }
    Ok(Value::Array(items))
}

fn parse_legacy_object(node: &LinoNode) -> Result<Value, String> {
    let mut object = Map::new();
    for child in &node.children {
        if child.name != "member" {
            continue;
        }
        let key = decode_text(&child.id)?;
        let value_node = child
            .children
            .iter()
            .find(|candidate| is_legacy_json_node(candidate))
            .ok_or_else(|| format!("object member `{key}` has no json value"))?;
        object.insert(key, parse_legacy_json_value(value_node)?);
    }
    Ok(Value::Object(object))
}

fn parse_compact_document(tree: &LinoNode) -> Result<Value, String> {
    if tree.children.len() == 1 {
        let node = &tree.children[0];
        if is_entity_cache_root(node) {
            let mut entity = parse_object_children(&node.children)?;
            if !node.id.is_empty() {
                entity.insert(String::from("type"), parse_scalar_token(&node.id)?);
            }
            let mut entities = Map::new();
            entities.insert(node.name.clone(), Value::Object(entity));
            let mut root = Map::new();
            root.insert(String::from("entities"), Value::Object(entities));
            return Ok(Value::Object(root));
        }
    }
    Ok(Value::Object(parse_object_children(&tree.children)?))
}

fn is_entity_cache_root(node: &LinoNode) -> bool {
    let mut characters = node.name.chars();
    let Some(prefix) = characters.next() else {
        return false;
    };
    matches!(prefix, 'L' | 'P' | 'Q') && characters.all(|character| character.is_ascii_digit())
}

fn parse_compact_value(node: &LinoNode) -> Result<Value, String> {
    match node.id.trim() {
        LEGACY_EMPTY_OBJECT_MARKER if node.children.is_empty() => Ok(Value::Object(Map::new())),
        LEGACY_ARRAY_MARKER => parse_compact_array(node),
        id if is_array_item_link(id, &node.children) => parse_compact_array(node),
        id if is_parenthesized(id) => parse_scalar_array(id),
        "" if is_compact_array(&node.children) => parse_compact_array(node),
        "" if !node.children.is_empty() => {
            Ok(Value::Object(parse_object_children(&node.children)?))
        }
        "" => Ok(Value::Object(Map::new())),
        id => parse_scalar_token(id),
    }
}

fn parse_object_children(children: &[LinoNode]) -> Result<Map<String, Value>, String> {
    let mut object = Map::new();
    for child in children {
        object.insert(child.name.clone(), parse_compact_value(child)?);
    }
    Ok(object)
}

fn parse_compact_array(node: &LinoNode) -> Result<Value, String> {
    let mut values = Vec::new();
    for child in &node.children {
        let value =
            if child.name == ARRAY_ENTRY && !child.id.is_empty() && child.children.is_empty() {
                Value::String(child.id.clone())
            } else if child.id.is_empty() && child.children.is_empty() {
                parse_scalar_token(&child.name)?
            } else {
                parse_compact_value(child)?
            };
        values.push(value);
    }
    Ok(Value::Array(values))
}

fn parse_scalar_array(raw: &str) -> Result<Value, String> {
    let body = raw
        .trim()
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| format!("invalid scalar array `{raw}`"))?;
    let tokens = split_scalar_tokens(body)?;
    let mut values = Vec::new();
    for token in tokens {
        values.push(parse_scalar_token(&token)?);
    }
    Ok(Value::Array(values))
}

fn split_scalar_tokens(raw: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut characters = raw.chars().peekable();
    while let Some(character) = characters.next() {
        if let Some(quote_character) = quote {
            current.push(character);
            if escaped {
                escaped = false;
                continue;
            }
            if quote_character == '"' && character == '\\' {
                escaped = true;
                continue;
            }
            if quote_character == '\''
                && character == '\''
                && characters.peek().is_some_and(|next| *next == '\'')
            {
                current.push(characters.next().expect("peeked quote should exist"));
                continue;
            }
            if character == quote_character {
                quote = None;
            }
            continue;
        }
        match character {
            '"' | '\'' => {
                quote = Some(character);
                current.push(character);
            }
            character if character.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }
    if quote.is_some() {
        return Err(String::from("unterminated quoted scalar in array"));
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    Ok(tokens)
}

fn parse_scalar_token(raw: &str) -> Result<Value, String> {
    let raw = raw.trim();
    if let Some(string) = raw.strip_prefix(STRING_MARKER) {
        let string = string.trim_start();
        return parse_scalar_token(string).and_then(|value| match value {
            Value::String(_) => Ok(value),
            other => Err(format!("invalid string scalar `{other:?}`")),
        });
    }
    if raw.starts_with('"') {
        let value: String = serde_json::from_str(raw)
            .map_err(|error| format!("invalid quoted string `{raw}`: {error}"))?;
        return Ok(Value::String(value));
    }
    if raw.starts_with('\'') {
        return parse_single_quoted_string(raw).map(Value::String);
    }
    match raw {
        "null" => Ok(Value::Null),
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        _ => serde_json::from_str::<Number>(raw)
            .map(Value::Number)
            .or_else(|_| Ok(Value::String(raw.to_string()))),
    }
}

fn parse_single_quoted_string(raw: &str) -> Result<String, String> {
    let body = raw
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .ok_or_else(|| format!("invalid quoted string `{raw}`"))?;
    let mut out = String::new();
    let mut characters = body.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\'' && characters.peek().is_some_and(|next| *next == '\'') {
            out.push('\'');
            characters.next();
            continue;
        }
        if character == '\\' {
            match characters.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('\\') | None => out.push('\\'),
                Some('\'') => out.push('\''),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
            continue;
        }
        out.push(character);
    }
    Ok(out)
}

fn is_parenthesized(raw: &str) -> bool {
    raw.starts_with('(') && raw.ends_with(')')
}

fn is_array_item_link(raw: &str, children: &[LinoNode]) -> bool {
    let Some(body) = raw
        .trim()
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    else {
        return false;
    };
    if children.is_empty() {
        return false;
    }
    let ids: Vec<&str> = body.split_whitespace().collect();
    ids.len() == children.len()
        && ids.iter().zip(children).all(|(id, child)| {
            (child.name == ARRAY_ENTRY || child.name.starts_with(LEGACY_ARRAY_ITEM_PREFIX))
                && *id == child.name
        })
}

fn is_compact_array(children: &[LinoNode]) -> bool {
    !children.is_empty()
        && children
            .iter()
            .all(|child| child.name == ARRAY_ENTRY || is_non_string_scalar_node(child))
}

fn is_non_string_scalar_node(node: &LinoNode) -> bool {
    node.id.is_empty()
        && node.children.is_empty()
        && (matches!(node.name.as_str(), "null" | "true" | "false")
            || serde_json::from_str::<Number>(&node.name).is_ok())
}

fn decode_text(value: &str) -> Result<String, String> {
    let encoded = value
        .strip_prefix('u')
        .ok_or_else(|| format!("encoded text `{value}` does not start with u"))?;
    if encoded.is_empty() {
        return Ok(String::new());
    }
    let mut out = String::new();
    for chunk in encoded.split('-').filter(|chunk| !chunk.is_empty()) {
        let codepoint = u32::from_str_radix(chunk, 16)
            .map_err(|error| format!("invalid codepoint `{chunk}`: {error}"))?;
        let character =
            char::from_u32(codepoint).ok_or_else(|| format!("invalid unicode scalar `{chunk}`"))?;
        out.push(character);
    }
    Ok(out)
}
