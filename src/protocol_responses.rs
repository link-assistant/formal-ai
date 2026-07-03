use serde_json::Value;

use crate::agentic_coding::planner::{tool_capability, Capability};
use crate::protocol_policy::tool_definition_name;

pub(crate) fn response_arguments_for_tool(
    tools: &[Value],
    tool_name: &str,
    arguments: String,
) -> String {
    if !responses_tool_prefers_cmd_argument(tools, tool_name) {
        return arguments;
    }

    let Ok(mut value) = serde_json::from_str::<Value>(&arguments) else {
        return arguments;
    };
    let Some(object) = value.as_object_mut() else {
        return arguments;
    };
    if object.contains_key("cmd") {
        return value.to_string();
    }
    let Some(command) = object.remove("command") else {
        return arguments;
    };
    object.insert(String::from("cmd"), command);
    value.to_string()
}

fn responses_tool_prefers_cmd_argument(tools: &[Value], tool_name: &str) -> bool {
    if tool_capability(tool_name) != Some(Capability::Run) {
        return false;
    }
    tools.iter().any(|tool| {
        tool_definition_name(tool).as_deref() == Some(tool_name) && tool_schema_prefers_cmd(tool)
    })
}

fn tool_schema_prefers_cmd(tool: &Value) -> bool {
    let Some(schema) = tool_parameters_schema(tool) else {
        return false;
    };
    if !schema_has_property(schema, "cmd") {
        return false;
    }
    schema_required_contains(schema, "cmd")
        || (!schema_has_property(schema, "command") && !schema_required_contains(schema, "command"))
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

fn schema_has_property(schema: &Value, property: &str) -> bool {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .is_some_and(|properties| properties.contains_key(property))
}

fn schema_required_contains(schema: &Value, property: &str) -> bool {
    schema
        .get("required")
        .and_then(Value::as_array)
        .is_some_and(|required| {
            required
                .iter()
                .any(|entry| entry.as_str() == Some(property))
        })
}
