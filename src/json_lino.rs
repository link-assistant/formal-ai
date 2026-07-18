//! Lossless JSON ↔ Links Notation cache codec (issue #398).
//!
//! Checked-in Wikidata / Wiktionary cache files store the *entire* upstream
//! JSON snapshot as compact, native Links Notation — every key (labels,
//! descriptions, aliases, lemmas, lexical category, language, **and** the full
//! `forms`, `senses` and `claims` graph) survives the round trip. The earlier
//! codec kept only an eight-key projection, silently dropping forms / senses /
//! claims; this one is provably lossless:
//!
//! * [`json_cache_file`] encodes the whole document. Wikidata entity snapshots
//!   keep their readable *entity-rooted* shape — the `Q…` / `L…` / `P…` id is
//!   the top-level node, its fields are indented under it, and sibling
//!   top-level keys such as `success` stay at the root. `labels`,
//!   `descriptions`, `lemmas` and `aliases` keep the compact `lang value`
//!   form because the per-language `{language, value}` wrapper is fully
//!   reconstructible (the inner `language` always equals the map key).
//! * [`lino_to_json`] rebuilds the original JSON `Value` from the Links
//!   Notation text,
//!   re-expanding the language wrappers, so `lino_to_json(json_cache_file(v))`
//!   equals `v` for every cached document — modulo empty collections.
//! * Empty arrays / objects and JSON `null` are **never emitted**: an empty
//!   collection is an absent default (issue #398 review, defect #5).
//!   [`json_cache_projection`] returns the same empties-stripped normal form,
//!   which is exactly what `lino_to_json` reconstructs.
//!
//! The decoder uses its own quote-preserving tokenizer (not the seed
//! `parse_lino`, which decodes quotes
//! eagerly) so a string that *looks* like a number — e.g. the external-id
//! value `"146"` — is never silently retyped: ambiguous strings are quoted on
//! encode and recognised as strings on decode.

use serde_json::{Map, Number, Value};

const ARRAY_ENTRY: &str = "entry";
const ENTITIES_KEY: &str = "entities";

// ---------------------------------------------------------------------------
// Empty-collection normalization (defect #5)
// ---------------------------------------------------------------------------

/// Recursively drop JSON `null`, empty arrays and empty objects.
///
/// An empty collection is an absent default, so it never reaches the Links
/// Notation encoding and is normalised away on both sides of the round-trip
/// comparison.
#[must_use]
pub fn strip_empty(value: &Value) -> Option<Value> {
    match value {
        Value::Null => None,
        Value::Array(items) => {
            let kept: Vec<Value> = items.iter().filter_map(strip_empty).collect();
            (!kept.is_empty()).then_some(Value::Array(kept))
        }
        Value::Object(object) => {
            let mut kept = Map::new();
            for (key, value) in object {
                if let Some(value) = strip_empty(value) {
                    kept.insert(key.clone(), value);
                }
            }
            (!kept.is_empty()).then_some(Value::Object(kept))
        }
        scalar => Some(scalar.clone()),
    }
}

fn normal_form(value: &Value) -> Value {
    strip_empty(value).unwrap_or_else(|| Value::Object(Map::new()))
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

#[must_use]
pub fn json_to_lino(value: &Value) -> String {
    let value = normal_form(value);
    let mut out = String::new();
    write_document(&mut out, &value);
    out
}

/// Encode a cached source document.
///
/// Wikidata entity snapshots (`{entities: {<id>: …}, success: 1}`) keep their
/// entity-rooted form; any other document (e.g. the Wiktionary array) uses the
/// generic encoding.
#[must_use]
pub fn json_cache_file(root_id: &str, value: &Value) -> String {
    let value = normal_form(value);
    let mut out = String::new();
    if let Some((entity_id, entity)) = single_entity(&value, root_id) {
        write_line(&mut out, 0, entity_id, None);
        write_entity_fields(&mut out, 2, entity);
        if let Value::Object(object) = &value {
            for (key, field) in object {
                if key != ENTITIES_KEY {
                    write_named_value(&mut out, 0, key, field);
                }
            }
        }
    } else {
        write_document(&mut out, &value);
    }
    out
}

/// The empties-stripped JSON that [`lino_to_json`] reconstructs from
/// [`json_cache_file`]'s output.
///
/// Comparing a decoded cache file against this proves the codec is lossless up
/// to the empty-collection normal form.
#[must_use]
pub fn json_cache_projection(_root_id: &str, value: &Value) -> Value {
    normal_form(value)
}

fn single_entity<'a>(
    value: &'a Value,
    root_id: &'a str,
) -> Option<(&'a str, &'a Map<String, Value>)> {
    let entities = value.as_object()?.get(ENTITIES_KEY)?.as_object()?;
    let entity = entities.get(root_id)?.as_object()?;
    Some((root_id, entity))
}

fn write_document(out: &mut String, value: &Value) {
    match value {
        Value::Object(object) => write_object_entries(out, 0, object),
        Value::Array(items) => {
            for item in items {
                write_array_element(out, 0, item);
            }
        }
        scalar => write_line(out, 0, "value", Some(&scalar_token(scalar))),
    }
}

/// Write the fields of a Wikidata entity, keeping the readable compact form for
/// the per-language sections whose `{language, value}` wrapper is fully
/// reconstructible from the map key.
fn write_entity_fields(out: &mut String, indent: usize, entity: &Map<String, Value>) {
    for (key, value) in entity {
        match key.as_str() {
            "labels" | "descriptions" | "lemmas" => {
                write_language_strings(out, indent, key, value);
            }
            "aliases" => write_language_aliases(out, indent, key, value),
            _ => write_named_value(out, indent, key, value),
        }
    }
}

fn write_language_strings(out: &mut String, indent: usize, name: &str, value: &Value) {
    let Some(object) = value.as_object() else {
        write_named_value(out, indent, name, value);
        return;
    };
    write_line(out, indent, name, None);
    for (language, entry) in object {
        if let Some(text) = language_text(entry) {
            write_line(out, indent + 2, language, Some(&string_token(text)));
        }
    }
}

fn write_language_aliases(out: &mut String, indent: usize, name: &str, value: &Value) {
    let Some(object) = value.as_object() else {
        write_named_value(out, indent, name, value);
        return;
    };
    write_line(out, indent, name, None);
    for (language, entries) in object {
        let Some(entries) = entries.as_array() else {
            continue;
        };
        let tokens: Vec<String> = entries
            .iter()
            .filter_map(language_text)
            .map(string_token)
            .collect();
        if !tokens.is_empty() {
            write_line(
                out,
                indent + 2,
                language,
                Some(&format!("({})", tokens.join(" "))),
            );
        }
    }
}

fn language_text(entry: &Value) -> Option<&str> {
    entry.as_object()?.get("value")?.as_str()
}

fn write_object_entries(out: &mut String, indent: usize, object: &Map<String, Value>) {
    for (key, value) in object {
        write_named_value(out, indent, key, value);
    }
}

fn write_named_value(out: &mut String, indent: usize, name: &str, value: &Value) {
    match value {
        Value::Array(items) if items.iter().all(is_scalar) => {
            write_line(out, indent, name, Some(&scalar_array_token(items)));
        }
        Value::Array(items) => {
            write_line(out, indent, name, None);
            for item in items {
                write_array_element(out, indent + 2, item);
            }
        }
        Value::Object(object) => {
            write_line(out, indent, name, None);
            write_object_entries(out, indent + 2, object);
        }
        scalar => write_line(out, indent, name, Some(&scalar_token(scalar))),
    }
}

fn write_array_element(out: &mut String, indent: usize, value: &Value) {
    match value {
        Value::Array(items) if items.iter().all(is_scalar) => {
            write_line(out, indent, ARRAY_ENTRY, Some(&scalar_array_token(items)));
        }
        Value::Array(items) => {
            write_line(out, indent, ARRAY_ENTRY, None);
            for item in items {
                write_array_element(out, indent + 2, item);
            }
        }
        Value::Object(object) => {
            write_line(out, indent, ARRAY_ENTRY, None);
            write_object_entries(out, indent + 2, object);
        }
        scalar => write_line(out, indent, ARRAY_ENTRY, Some(&scalar_token(scalar))),
    }
}

fn write_line(out: &mut String, indent: usize, name: &str, value: Option<&str>) {
    out.extend(std::iter::repeat_n(' ', indent));
    out.push_str(name);
    if let Some(value) = value {
        out.push(' ');
        out.push_str(value);
    }
    out.push('\n');
}

const fn is_scalar(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

fn scalar_token(value: &Value) -> String {
    match value {
        Value::Null => String::from("null"),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => string_token(value),
        Value::Array(_) | Value::Object(_) => String::new(),
    }
}

fn scalar_array_token(values: &[Value]) -> String {
    let tokens: Vec<String> = values.iter().map(scalar_token).collect();
    format!("({})", tokens.join(" "))
}

/// A bare token for an unambiguous reference, or a quoted scalar otherwise.
/// Strings that *look* like a JSON literal (`true`, `42`, …) are always quoted
/// so the decoder never retypes them.
fn string_token(value: impl AsRef<str>) -> String {
    let value = value.as_ref();
    if is_bare_reference(value) && !is_scalar_literal(value) {
        value.to_string()
    } else {
        quoted_string(value)
    }
}

fn is_scalar_literal(value: &str) -> bool {
    matches!(value, "null" | "true" | "false") || serde_json::from_str::<Number>(value).is_ok()
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

fn is_bare_reference(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '$')
        })
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/// A raw, quote-preserving Links Notation node: the value token is kept exactly
/// as written so the decoder can tell a quoted string from a bare reference.
#[derive(Debug, Default)]
struct RawNode {
    name: String,
    value: Option<String>,
    children: Vec<Self>,
}

pub fn lino_to_json(text: &str) -> Result<Value, String> {
    let root = parse_raw_tree(text);
    let tops = &root.children;
    if !tops.is_empty() && tops.iter().all(|node| node.name == ARRAY_ENTRY) {
        let mut items = Vec::new();
        for node in tops {
            items.push(value_of(node)?);
        }
        return Ok(Value::Array(items));
    }

    let mut root_object = Map::new();
    let mut entities = Map::new();
    for node in tops {
        if is_entity_id(&node.name) {
            entities.insert(node.name.clone(), entity_value(node)?);
        } else {
            root_object.insert(node.name.clone(), value_of(node)?);
        }
    }
    if !entities.is_empty() {
        root_object.insert(String::from(ENTITIES_KEY), Value::Object(entities));
    }
    Ok(Value::Object(root_object))
}

fn parse_raw_tree(text: &str) -> RawNode {
    let mut root = RawNode::default();
    let mut stack: Vec<(Option<usize>, Vec<usize>)> = vec![(None, Vec::new())];
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        let content = line[indent..].trim_end();
        if content.is_empty() {
            continue;
        }
        let node = parse_raw_line(content);
        while stack.len() > 1
            && stack
                .last()
                .and_then(|frame| frame.0)
                .is_some_and(|top| top >= indent)
        {
            stack.pop();
        }
        let parent_path = stack
            .last()
            .map(|frame| frame.1.clone())
            .unwrap_or_default();
        let parent = navigate_mut(&mut root, &parent_path);
        parent.children.push(node);
        let new_index = parent.children.len() - 1;
        let mut new_path = parent_path;
        new_path.push(new_index);
        stack.push((Some(indent), new_path));
    }
    root
}

fn parse_raw_line(content: &str) -> RawNode {
    content.find(char::is_whitespace).map_or_else(
        || RawNode {
            name: content.to_string(),
            value: None,
            children: Vec::new(),
        },
        |pos| RawNode {
            name: content[..pos].to_string(),
            value: Some(content[pos..].trim().to_string()),
            children: Vec::new(),
        },
    )
}

fn navigate_mut<'a>(root: &'a mut RawNode, path: &[usize]) -> &'a mut RawNode {
    let mut current = root;
    for &index in path {
        current = &mut current.children[index];
    }
    current
}

fn value_of(node: &RawNode) -> Result<Value, String> {
    if !node.children.is_empty() {
        if node.children.iter().all(|child| child.name == ARRAY_ENTRY) {
            let mut items = Vec::new();
            for child in &node.children {
                items.push(value_of(child)?);
            }
            return Ok(Value::Array(items));
        }
        return Ok(Value::Object(object_of(&node.children)?));
    }
    node.value
        .as_ref()
        .map_or_else(|| Ok(Value::Object(Map::new())), |token| parse_token(token))
}

fn object_of(children: &[RawNode]) -> Result<Map<String, Value>, String> {
    let mut object = Map::new();
    for child in children {
        object.insert(child.name.clone(), value_of(child)?);
    }
    Ok(object)
}

/// Decode a Wikidata entity node, re-expanding the compact per-language
/// sections back into their `{language, value}` JSON wrappers.
fn entity_value(node: &RawNode) -> Result<Value, String> {
    let mut entity = Map::new();
    for child in &node.children {
        let value = match child.name.as_str() {
            "labels" | "descriptions" | "lemmas" => reexpand_language_strings(child)?,
            "aliases" => reexpand_language_aliases(child)?,
            _ => value_of(child)?,
        };
        entity.insert(child.name.clone(), value);
    }
    Ok(Value::Object(entity))
}

fn reexpand_language_strings(node: &RawNode) -> Result<Value, String> {
    let mut object = Map::new();
    for language in &node.children {
        let text = value_of(language)?;
        let text = text
            .as_str()
            .ok_or_else(|| format!("language `{}` value is not a string", language.name))?;
        object.insert(language.name.clone(), language_entry(&language.name, text));
    }
    Ok(Value::Object(object))
}

fn reexpand_language_aliases(node: &RawNode) -> Result<Value, String> {
    let mut object = Map::new();
    for language in &node.children {
        let values = value_of(language)?;
        let values = values
            .as_array()
            .ok_or_else(|| format!("aliases `{}` is not a list", language.name))?;
        let entries: Vec<Value> = values
            .iter()
            .filter_map(Value::as_str)
            .map(|text| language_entry(&language.name, text))
            .collect();
        object.insert(language.name.clone(), Value::Array(entries));
    }
    Ok(Value::Object(object))
}

fn language_entry(language: &str, value: &str) -> Value {
    let mut entry = Map::new();
    entry.insert(
        String::from("language"),
        Value::String(language.to_string()),
    );
    entry.insert(String::from("value"), Value::String(value.to_string()));
    Value::Object(entry)
}

fn is_entity_id(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some('L' | 'P' | 'Q'))
        && name.len() > 1
        && characters.all(|character| character.is_ascii_digit())
}

fn parse_token(raw: &str) -> Result<Value, String> {
    let raw = raw.trim();
    if is_parenthesized(raw) {
        return parse_scalar_array(raw);
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

fn parse_scalar_array(raw: &str) -> Result<Value, String> {
    let body = raw
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| format!("invalid scalar array `{raw}`"))?;
    let mut values = Vec::new();
    for token in split_scalar_tokens(body)? {
        values.push(parse_token(&token)?);
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
