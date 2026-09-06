use std::fmt::Write as _;
use std::path::Path;

use serde_json::{Map, Value};

use crate::protocol_policy::find_tool_definition;

const PATCH_BEGIN: &str = "*** Begin Patch";
const PATCH_ADD_FILE: &str = "*** Add File:";
const PATCH_END: &str = "*** End Patch";

/// Whether `tool_name` is a freeform custom tool on the Responses surface.
#[must_use]
pub fn is_custom_response_tool(tools: &[Value], tool_name: &str) -> bool {
    find_tool_definition(tools, tool_name)
        .and_then(|definition| definition.get("type"))
        .and_then(Value::as_str)
        == Some("custom")
}

/// Lower semantic planner arguments into the freeform input expected by a
/// Responses custom tool. Patch tools use Codex's native patch grammar; other
/// custom tools receive the planner's original freeform bytes unchanged.
#[must_use]
pub fn custom_response_tool_input(tools: &[Value], tool_name: &str, arguments: String) -> String {
    let Some(definition) = find_tool_definition(tools, tool_name) else {
        return arguments;
    };
    if definition.get("type").and_then(Value::as_str) != Some("custom") {
        return arguments;
    }
    let leaf = tool_name.rsplit("__").next().unwrap_or(tool_name);
    if !leaf.to_ascii_lowercase().contains("patch") {
        return arguments;
    }
    apply_patch_input(&arguments).unwrap_or(arguments)
}

fn apply_patch_input(arguments: &str) -> Option<String> {
    let arguments = serde_json::from_str::<Value>(arguments).ok()?;
    let path = ["path", "filePath", "file_path"]
        .iter()
        .find_map(|name| arguments.get(*name).and_then(Value::as_str))?;
    let content = arguments.get("content").and_then(Value::as_str)?;
    if path.is_empty() || path.contains(['\r', '\n']) {
        return None;
    }

    let mut rendered = String::new();
    let _ = writeln!(rendered, "{PATCH_BEGIN}");
    let _ = writeln!(rendered, "{PATCH_ADD_FILE} {path}");
    if content.is_empty() {
        rendered.push_str("+\n");
    } else {
        for line in content.lines() {
            let _ = writeln!(rendered, "+{line}");
        }
    }
    let _ = writeln!(rendered, "{PATCH_END}");
    Some(rendered)
}

/// Project the planner's capability-shaped arguments onto the exact JSON Schema
/// advertised by the selected client tool.
///
/// Agentic clients use different names for the same values (`command`/`cmd`,
/// `query`/`pattern`, and several path/edit variants). Planner code deliberately
/// carries those semantic aliases; this boundary removes undeclared aliases and
/// fills every required schema field before a call crosses the protocol.
///
/// `workspace` is the directory the client said it is running in, when it said
/// so; a path the client requires to be absolute is resolved against it rather
/// than against the server's own directory.
pub fn response_arguments_for_tool(
    tools: &[Value],
    tool_name: &str,
    arguments: String,
    user_prompt: &str,
    workspace: Option<&str>,
) -> String {
    let Some(definition) = find_tool_definition(tools, tool_name) else {
        return arguments;
    };
    let Some(schema) = tool_parameters_schema(definition) else {
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
    let creates_file = call_writes_bytes(properties, source);
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
                    .then(|| missing_required_value(property_schema, name, user_prompt))
                    .flatten()
            });
        if let Some(value) = value {
            let mut value = constrain_to_schema(value, property_schema, name, user_prompt);
            if let Some(path) = value
                .as_str()
                .filter(|_| demands_absolute_path(definition, name, property_schema))
            {
                // A write with no observed workspace keeps the request's own
                // spelling so the client resolves it in its own directory.
                if let Some(absolute) = absolute_path(path, workspace, creates_file) {
                    value = Value::String(absolute);
                }
            }
            projected.insert(name.clone(), value);
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
    aliases.iter().find_map(|alias| source.get(*alias).cloned())
}

/// Whether the client's own schema says this argument has to be an absolute
/// path (issue #671).
///
/// The planner names files the way the request did, which is usually relative.
/// Several real clients reject that outright — the agentic matrix caught
/// `agent` answering `Error: File not found: /alpha.txt` and `qwen` answering
/// `File path must be absolute, but was relative: alpha.txt`. The requirement is
/// advertised, so it is read rather than hardcoded per client: `gemini` names
/// the property `absolute_path`, `qwen` and `opencode` say so in the property
/// description, and `agent` says so in the tool description ("The filePath
/// parameter must be an absolute path, not a relative path"). A client that
/// accepts relative paths advertises nothing and keeps the request's own
/// spelling, because the absolute form is only correct while the server shares
/// the client's working directory.
fn demands_absolute_path(definition: &Value, name: &str, property_schema: &Value) -> bool {
    const PATH_PROPERTIES: &[&str] = &[
        "path",
        "filePath",
        "file_path",
        "absolute_path",
        "dir_path",
        "directory",
        "notebook_path",
    ];
    if !PATH_PROPERTIES.contains(&name) {
        return false;
    }
    if name == "absolute_path" {
        return true;
    }
    let says_absolute = |value: Option<&Value>| {
        value
            .and_then(Value::as_str)
            .is_some_and(|text| text.to_lowercase().contains("absolute"))
    };
    says_absolute(property_schema.get("description")) || says_absolute(tool_description(definition))
}

/// The client-authored prose for a tool, under either the flat or the
/// `function`-wrapped shape.
fn tool_description(definition: &Value) -> Option<&Value> {
    definition.get("description").or_else(|| {
        definition
            .get("function")
            .and_then(|function| function.get("description"))
    })
}

/// Resolve a relative path against the client's directory.
///
/// An already-absolute path is preserved verbatim: the planner or the request
/// already named a place, and rewriting it would move the effect.
///
/// With no observed workspace the answer depends on what the call does. A read
/// resolved against the server's own directory is the shared-directory case the
/// agentic matrix runs, and when the guess is wrong the read simply fails and
/// says so. A *write* resolved the same way is the issue-#1075 defect: the Scala
/// session wrote `/home/box/.formal-ai/general-change-plan.lino` into the
/// server's sidecar while the task workspace was
/// `/tmp/gh-issue-solver-1788563504540`, and the plan looked written while the
/// pull request stayed empty. A path the server cannot ground is therefore left
/// as the request spelled it, relative, so the one process that does know its
/// own directory — the client — resolves it. Returning `None` says exactly that:
/// there is no grounded absolute form of this path.
fn absolute_path(path: &str, workspace: Option<&str>, creates_file: bool) -> Option<String> {
    let path = Path::new(path);
    if path.is_absolute() {
        return Some(path.to_string_lossy().into_owned());
    }
    if let Some(workspace) = workspace.map(Path::new).filter(|root| root.is_absolute()) {
        return Some(workspace.join(path).to_string_lossy().into_owned());
    }
    if creates_file {
        return None;
    }
    Some(
        std::path::absolute(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .into_owned(),
    )
}

/// Whether this call carries bytes to put in the file it names.
///
/// Read the two together: the schema advertises a content-shaped property and
/// the planner filled it. That is what separates "write this file" from "look at
/// this file", without asking what the tool is called.
fn call_writes_bytes(properties: &Map<String, Value>, source: &Map<String, Value>) -> bool {
    const CONTENT_PROPERTIES: &[&str] = &["content", "contents", "file_text", "text", "new_string"];
    CONTENT_PROPERTIES
        .iter()
        .any(|name| properties.contains_key(*name) && source.contains_key(*name))
}

/// The value for a required property the planner did not supply.
///
/// [`None`] means the call cannot be grounded on this argument, and the argument
/// is left out rather than invented (issue #1075). Two kinds are never invented.
///
/// An *identity* argument names a resource: `repository_full_name`, `owner`,
/// `channel_id`. The empty string that used to fill it is not a neutral
/// placeholder — `github.create_file` with `"repository_full_name": ""`
/// addresses no repository and answered 404, having reported a file creation
/// that never happened. The request's own URLs are read for a real identity
/// first; when they say nothing, neither does the call.
///
/// A *choice* argument is a required enum with several options and no declared
/// default. Picking the first is picking an action — the order in a schema is
/// not a preference — so the choice is left to fail loudly instead.
fn missing_required_value(schema: &Value, name: &str, user_prompt: &str) -> Option<Value> {
    if schema.get("default").is_none() && crate::tool_scope::is_identity_argument(name) {
        return crate::tool_scope::grounded_identity_argument(name, user_prompt);
    }
    if schema.get("default").is_none()
        && schema
            .get("enum")
            .and_then(Value::as_array)
            .is_some_and(|options| options.len() > 1)
    {
        return None;
    }
    Some(schema_default(schema, name, user_prompt))
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
        // A required free-text field describes the work, and the work is what
        // the request said. `github.create_file` was called with
        // `"message": ""` beside its empty repository; a commit message that
        // says nothing is a fabrication too, and this one is already known.
        _ if matches!(
            name,
            "prompt" | "instruction" | "message" | "commit_message" | "commitMessage"
        ) =>
        {
            Value::String(user_prompt.to_owned())
        }
        _ => Value::String(String::new()),
    }
}

fn constrain_to_schema(value: Value, schema: &Value, name: &str, user_prompt: &str) -> Value {
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array)
        && !allowed.contains(&value)
    {
        return allowed.first().cloned().unwrap_or(value);
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
