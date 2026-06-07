//! Lossless JSON-to-LiNo cache encoding.
//!
//! The cache format stores JSON structure as links and stores string/key text
//! as Unicode scalar code points encoded into safe references. No quoted JSON
//! string is embedded in the `.lino` output.

use std::fmt::Write as _;

use serde_json::{Map, Number, Value};

use crate::seed::parser::{parse_lino, LinoNode};

const ROOT_OBJECT: &str = "json-object";
const ROOT_ARRAY: &str = "json-array";

#[must_use]
pub fn json_to_lino(value: &Value) -> String {
    let mut out = String::new();
    write_json_value(&mut out, 0, value);
    out
}

#[must_use]
pub fn json_cache_file(root_id: &str, value: &Value) -> String {
    let mut out = String::new();
    out.push_str(root_id);
    out.push_str(" # cached json source ");
    out.push_str(root_id);
    out.push('\n');
    write_json_value(&mut out, 2, value);
    out
}

pub fn lino_to_json(text: &str) -> Result<Value, String> {
    let tree = parse_lino(text);
    let node = find_json_node(&tree).ok_or_else(|| String::from("missing json value node"))?;
    parse_json_value(node)
}

fn write_json_value(out: &mut String, indent: usize, value: &Value) {
    match value {
        Value::Null => write_line(out, indent, "json-null", ""),
        Value::Bool(value) => write_line(out, indent, "json-boolean", &value.to_string()),
        Value::Number(value) => write_line(out, indent, "json-number", &value.to_string()),
        Value::String(value) => write_line(out, indent, "json-string", &encode_text(value)),
        Value::Array(values) => {
            write_line(out, indent, ROOT_ARRAY, "");
            for (index, value) in values.iter().enumerate() {
                write_line(out, indent + 2, "item", &index.to_string());
                write_json_value(out, indent + 4, value);
            }
        }
        Value::Object(values) => {
            write_line(out, indent, ROOT_OBJECT, "");
            for (key, value) in values {
                write_line(out, indent + 2, "member", &encode_text(key));
                write_json_value(out, indent + 4, value);
            }
        }
    }
}

fn write_line(out: &mut String, indent: usize, name: &str, id: &str) {
    out.extend(std::iter::repeat(' ').take(indent));
    out.push_str(name);
    if !id.is_empty() {
        out.push(' ');
        out.push_str(id);
    }
    out.push('\n');
}

fn find_json_node(node: &LinoNode) -> Option<&LinoNode> {
    if is_json_node(node) {
        return Some(node);
    }
    node.children.iter().find_map(find_json_node)
}

fn is_json_node(node: &LinoNode) -> bool {
    matches!(
        node.name.as_str(),
        ROOT_OBJECT | ROOT_ARRAY | "json-null" | "json-boolean" | "json-number" | "json-string"
    )
}

fn parse_json_value(node: &LinoNode) -> Result<Value, String> {
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
        ROOT_ARRAY => parse_array(node),
        ROOT_OBJECT => parse_object(node),
        other => Err(format!("unknown json node `{other}`")),
    }
}

fn parse_array(node: &LinoNode) -> Result<Value, String> {
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            continue;
        }
        let value_node = child
            .children
            .iter()
            .find(|candidate| is_json_node(candidate))
            .ok_or_else(|| format!("array item `{}` has no json value", child.id))?;
        items.push(parse_json_value(value_node)?);
    }
    Ok(Value::Array(items))
}

fn parse_object(node: &LinoNode) -> Result<Value, String> {
    let mut object = Map::new();
    for child in &node.children {
        if child.name != "member" {
            continue;
        }
        let key = decode_text(&child.id)?;
        let value_node = child
            .children
            .iter()
            .find(|candidate| is_json_node(candidate))
            .ok_or_else(|| format!("object member `{key}` has no json value"))?;
        object.insert(key, parse_json_value(value_node)?);
    }
    Ok(Value::Object(object))
}

fn encode_text(value: &str) -> String {
    let mut out = String::from("u");
    for character in value.chars() {
        let _ = write!(out, "-{:X}", character as u32);
    }
    out
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
