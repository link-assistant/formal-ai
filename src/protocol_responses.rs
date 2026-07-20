use std::path::Path;

use serde_json::{Map, Value};

use crate::protocol_policy::find_tool_definition;

/// Project the planner's capability-shaped arguments onto the exact JSON Schema
/// advertised by the selected client tool.
///
/// Agentic clients use different names for the same values (`command`/`cmd`,
/// `query`/`pattern`, and several path/edit variants). Planner code deliberately
/// carries those semantic aliases; this boundary removes undeclared aliases and
/// fills every required schema field before a call crosses the protocol.
pub fn response_arguments_for_tool(
    tools: &[Value],
    tool_name: &str,
    arguments: String,
    user_prompt: &str,
) -> String {
    let Some(schema) = find_tool_definition(tools, tool_name).and_then(tool_parameters_schema)
    else {
        return arguments;
    };
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return arguments;
    };
    // An empty/unspecified object schema is intentionally permissive. Preserve
    // the planner shape for older clients that advertise no property metadata.
    if properties.is_empty() {
        return arguments;
    }
    let Ok(source) = serde_json::from_str::<Value>(&arguments) else {
        return arguments;
    };
    let Some(source) = source.as_object() else {
        return arguments;
    };

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut projected = Map::new();
    for (name, property_schema) in properties {
        let value = source
            .get(name)
            .cloned()
            .or_else(|| semantic_alias(name, source, user_prompt))
            .or_else(|| {
                required
                    .iter()
                    .any(|entry| entry.as_str() == Some(name))
                    .then(|| schema_default(property_schema, name, user_prompt))
            });
        if let Some(value) = value {
            projected.insert(
                name.clone(),
                constrain_to_schema(value, property_schema, name, user_prompt),
            );
        }
    }
    let projected = Value::Object(projected).to_string();
    if std::env::var("FORMAL_AI_TRACE_REQUESTS").as_deref() == Ok("1") && projected != arguments {
        eprintln!(
            "[trace] tool_schema_projection: tool={tool_name} planned={arguments} emitted={projected}"
        );
    }
    projected
}

fn tool_parameters_schema(tool: &Value) -> Option<&Value> {
    let object = tool.as_object()?;
    object
        .get("parameters")
        .or_else(|| object.get("input_schema"))
        .or_else(|| {
            object
                .get("function")
                .and_then(|function| function.get("parameters"))
        })
        .or_else(|| {
            object
                .get("function")
                .and_then(|function| function.get("input_schema"))
        })
}

fn semantic_alias(name: &str, source: &Map<String, Value>, user_prompt: &str) -> Option<Value> {
    let aliases: &[&str] = match name {
        "path" | "filePath" | "file_path" | "absolute_path" => {
            &["path", "filePath", "file_path", "absolute_path"]
        }
        "command" | "cmd" => &["command", "cmd"],
        "query" | "pattern" => &["query", "pattern"],
        "paths" | "file_paths" | "files" => &["paths", "file_paths", "files"],
        "old" | "oldString" | "old_string" | "old_str" => {
            &["old", "oldString", "old_string", "old_str"]
        }
        "new" | "newString" | "new_string" | "new_str" => {
            &["new", "newString", "new_string", "new_str"]
        }
        "prompt" | "instruction" => return Some(Value::String(user_prompt.to_owned())),
        _ => return None,
    };
    let mut value = aliases
        .iter()
        .find_map(|alias| source.get(*alias).cloned())?;
    if name == "absolute_path" {
        if let Some(path) = value.as_str() {
            value = Value::String(absolute_path(path));
        }
    }
    Some(value)
}

fn absolute_path(path: &str) -> String {
    let path = Path::new(path);
    if path.is_absolute() {
        return path.to_string_lossy().into_owned();
    }
    std::path::absolute(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

fn schema_default(schema: &Value, name: &str, user_prompt: &str) -> Value {
    if let Some(default) = schema.get("default") {
        return default.clone();
    }
    if let Some(first) = schema
        .get("enum")
        .and_then(Value::as_array)
        .and_then(|e| e.first())
    {
        return first.clone();
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("boolean") => Value::Bool(name == "login"),
        Some("array") => Value::Array(Vec::new()),
        Some("object") => Value::Object(Map::new()),
        Some("integer" | "number") => Value::from(0),
        Some("null") => Value::Null,
        _ if matches!(name, "prompt" | "instruction") => Value::String(user_prompt.to_owned()),
        _ => Value::String(String::new()),
    }
}

fn constrain_to_schema(value: Value, schema: &Value, name: &str, user_prompt: &str) -> Value {
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.contains(&value) {
            return allowed.first().cloned().unwrap_or(value);
        }
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("object") => {
            let Some(source) = value.as_object() else {
                return schema_default(schema, name, user_prompt);
            };
            let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
                return value;
            };
            let required = schema
                .get("required")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let mut projected = Map::new();
            for (child_name, child_schema) in properties {
                let child = source.get(child_name).cloned().or_else(|| {
                    required
                        .iter()
                        .any(|entry| entry.as_str() == Some(child_name))
                        .then(|| schema_default(child_schema, child_name, user_prompt))
                });
                if let Some(child) = child {
                    projected.insert(
                        child_name.clone(),
                        constrain_to_schema(child, child_schema, child_name, user_prompt),
                    );
                }
            }
            Value::Object(projected)
        }
        Some("array") => {
            let Some(values) = value.as_array() else {
                return schema_default(schema, name, user_prompt);
            };
            let mut values = values.clone();
            if let Some(item_schema) = schema.get("items") {
                values = values
                    .into_iter()
                    .map(|item| constrain_to_schema(item, item_schema, name, user_prompt))
                    .collect();
                let minimum = schema
                    .get("minItems")
                    .and_then(Value::as_u64)
                    .and_then(|minimum| usize::try_from(minimum).ok())
                    // Client-provided schemas must not be able to force an
                    // unbounded allocation while defaults are projected.
                    .unwrap_or(0)
                    .min(64);
                while values.len() < minimum {
                    values.push(schema_default(item_schema, name, user_prompt));
                }
            }
            Value::Array(values)
        }
        Some("string") if !value.is_string() => schema_default(schema, name, user_prompt),
        Some("boolean") if !value.is_boolean() => schema_default(schema, name, user_prompt),
        Some("integer") if !value.is_i64() && !value.is_u64() => {
            schema_default(schema, name, user_prompt)
        }
        Some("number") if !value.is_number() => schema_default(schema, name, user_prompt),
        Some("null") if !value.is_null() => Value::Null,
        _ => value,
    }
}
