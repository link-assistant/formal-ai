//! Issue #1154 R1: a tool's command property, as its own schema declares it.
//!
//! Split out of `tool_result.rs` (the 1000-line Rust ceiling); the parent
//! re-exports both functions, so callers keep their paths.

use serde_json::Value;

use crate::protocol::ChatMessage;

/// The property name a tool definition's own schema gives the shell command.
///
/// A definition such as Codex's `exec_command` declares
/// `parameters.properties.cmd` with `"description": "The bash command to
/// execute"`; the property whose description or title names a command is that
/// tool's command key, whatever it is called. Callers that hold the request's
/// tool definitions resolve the key once and pass it in ahead of the fallback
/// list, so a client that renames the property keeps working without a Rust
/// change (requirement 1 of issue #1154).
#[must_use]
pub fn command_argument_key(definition: &Value) -> Option<String> {
    // A function tool nests its parameters under `function`; a custom tool
    // declares them at the top level.
    let parameters = definition
        .get("function")
        .or(Some(definition))?
        .get("parameters")?;
    let object = parameters.get("properties")?.as_object()?;
    object
        .iter()
        .find(|(_, property)| {
            let describes_command = ["description", "title", "name"].iter().any(|field| {
                property
                    .get(*field)
                    .and_then(Value::as_str)
                    .is_some_and(|text| {
                        let lowered = text.to_lowercase();
                        lowered.contains("command") && !lowered.contains("working directory")
                    })
            });
            describes_command
                && matches!(property.get("type"), Some(Value::String(kind)) if kind == "string")
        })
        .map(|(key, _)| key.clone())
}

/// The argument keys `command_argument` reads on its own.
const FALLBACK_COMMAND_KEYS: [&str; 3] = ["command", "cmd", "script"];

/// The transcript as the planner reads it (requirement 1 of issue #1154).
///
/// Every call to a tool whose own schema names its command property `K` — outside
/// `command`/`cmd`/`script` — also carries that value under `command`, so the
/// shared `command_argument` reads it. A pure projection of the request:
/// nothing is stored between requests, and calls to tools without such a
/// declaration (or already carrying `command`) are returned unchanged.
#[must_use]
pub fn project_declared_command_keys(
    messages: &[ChatMessage],
    tools: &[Value],
) -> Vec<ChatMessage> {
    if tools.is_empty() {
        return messages.to_vec();
    }
    let declared_key = |name: &str| {
        crate::protocol_policy::find_tool_definition(tools, name)
            .and_then(command_argument_key)
            .filter(|key| !FALLBACK_COMMAND_KEYS.contains(&key.as_str()))
    };
    messages
        .iter()
        .map(|message| {
            let mut message = message.clone();
            for call in &mut message.tool_calls {
                let Some(key) = declared_key(&call.function.name) else {
                    continue;
                };
                let Ok(Value::Object(mut object)) =
                    serde_json::from_str::<Value>(&call.function.arguments)
                else {
                    continue;
                };
                if object.contains_key("command") {
                    continue;
                }
                let Some(command) = object.get(&key).cloned() else {
                    continue;
                };
                object.insert(String::from("command"), command);
                call.function.arguments = Value::Object(object).to_string();
            }
            message
        })
        .collect()
}
