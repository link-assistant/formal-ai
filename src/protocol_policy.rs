use serde_json::Value;

use crate::associative_package::{default_package_store, PackagePermissionDecision};
use crate::engine::{SymbolicAnswer, ThinkingStep};

pub fn tool_call_refusal_answer() -> SymbolicAnswer {
    SymbolicAnswer {
        intent: String::from("tool_call_refused"),
        answer: String::from(
            "Tool calls and function execution are not allowed without explicit agent mode. \
             Enable agent mode only for an isolated execution environment.",
        ),
        execution_recipe: None,
        confidence: 1.0,
        evidence_links: vec![String::from("policy:agent_mode_required_for_tools")],
        thinking_steps: policy_thinking_steps("agent_mode_required_for_tools"),
        links_notation: String::from(
            "tool_call_refusal\n  policy \"agent_mode_required_for_tools\"\n  thinking_step \"policy_refusal agent_mode_required_for_tools\"\n",
        ),
    }
}

pub fn tool_permission_refusal_answer(decision: &PackagePermissionDecision) -> SymbolicAnswer {
    let PackagePermissionDecision::Denied { capability, reason } = decision else {
        return tool_call_refusal_answer();
    };
    SymbolicAnswer {
        intent: String::from("tool_call_refused"),
        answer: format!(
            "Tool calls are not allowed for `{capability}`: {reason}. Install or import an \
             associative package that grants this capability before enabling the tool."
        ),
        execution_recipe: None,
        confidence: 1.0,
        evidence_links: vec![format!("policy:package_permission_required:{capability}")],
        thinking_steps: policy_thinking_steps(format!("package_permission_required:{capability}")),
        links_notation: format!(
            "tool_call_refusal\n  policy \"package_permission_required\"\n  capability \"{capability}\"\n  thinking_step \"policy_refusal package_permission_required:{capability}\"\n"
        ),
    }
}

fn policy_thinking_steps(detail: impl Into<String>) -> Vec<ThinkingStep> {
    vec![ThinkingStep::new(
        0,
        "policy_refusal",
        detail,
        "high",
        "policy",
    )]
}

/// Permission gate for the *agentic* path — an external CLI driving the server
/// over the OpenAI-compatible surface.
///
/// An agentic client executes tools in its own isolated sandbox and advertises
/// its *whole* toolset (often a dozen tools with CLI-specific names). Authorising
/// that by exact tool name would require an ever-growing per-CLI allowlist.
/// Instead each advertised tool is classified into a
/// [`Capability`](crate::agentic_coding::planner::Capability) — the same
/// classifier the planner uses to pick tools — and only the *capability class* is
/// checked. Tools the recipe never drives (list/grep/todo/…) are
/// unclassified and simply ignored: the client owns them. Returns the first
/// denial for a class no installed package grants, or [`None`] when every
/// classified tool's class is permitted. When no tools are advertised at all the
/// wildcard `tool:*` capability is consulted, matching the prior gate's behaviour.
#[must_use]
pub fn agentic_tool_permission_denial(names: &[String]) -> Option<PackagePermissionDecision> {
    use crate::agentic_coding::planner::tool_capability;
    let store = default_package_store();
    if names.is_empty() {
        let decision = store.permission_for_capability("tool:*");
        return matches!(decision, PackagePermissionDecision::Denied { .. }).then_some(decision);
    }
    names.iter().find_map(|name| {
        let capability = tool_capability(name)?;
        let decision = store.permission_for_capability(capability.permission_key());
        matches!(decision, PackagePermissionDecision::Denied { .. }).then_some(decision)
    })
}

pub fn is_tool_choice_request(value: &Value) -> bool {
    !matches_tool_choice_none(value)
}

pub fn tool_choice_function_name(value: &Value) -> Option<String> {
    match value {
        Value::Object(object) => object
            .get("function")
            .and_then(|function| function.get("name"))
            .or_else(|| object.get("name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| hosted_tool_type_name(object).map(ToOwned::to_owned)),
        _ => None,
    }
}

pub fn tool_definition_name(value: &Value) -> Option<String> {
    match value {
        Value::Object(object) => object
            .get("function")
            .and_then(|function| function.get("name"))
            .or_else(|| object.get("name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| hosted_tool_type_name(object).map(ToOwned::to_owned)),
        _ => None,
    }
}

/// Executable names advertised by one tool definition.
///
/// The Responses API can group MCP functions beneath a `namespace` definition.
/// Codex addresses those children as `<namespace>__<function>`, so the planner
/// must see the qualified child names rather than the non-executable namespace
/// container.
pub fn tool_definition_names(value: &Value) -> Vec<String> {
    let mut names = Vec::new();
    append_qualified_tool_definition_names(value, None, &mut names);
    names
}

/// Find the concrete definition (and therefore schema) for an executable tool
/// name, including a child inside a Responses API namespace.
pub fn find_tool_definition<'a>(definitions: &'a [Value], tool_name: &str) -> Option<&'a Value> {
    definitions
        .iter()
        .find_map(|definition| find_qualified_tool_definition(definition, None, tool_name))
}

/// Convert the planner's qualified identity back to the Responses wire shape.
/// Namespace-aware clients route MCP calls by the `(namespace, name)` pair; a
/// flat qualified name is treated as an unrelated built-in function.
pub fn response_tool_call_identity(
    definitions: &[Value],
    tool_name: &str,
) -> (String, Option<String>) {
    definitions
        .iter()
        .find_map(|definition| namespace_tool_call_identity(definition, None, tool_name))
        .unwrap_or_else(|| (tool_name.to_owned(), None))
}

fn append_qualified_tool_definition_names(
    value: &Value,
    prefix: Option<&str>,
    names: &mut Vec<String>,
) {
    let Some(object) = value.as_object() else {
        return;
    };
    if object.get("type").and_then(Value::as_str) == Some("namespace") {
        let Some(namespace) = object.get("name").and_then(Value::as_str) else {
            return;
        };
        let qualified_namespace = qualify_tool_name(prefix, namespace);
        if let Some(children) = object.get("tools").and_then(Value::as_array) {
            for child in children {
                append_qualified_tool_definition_names(child, Some(&qualified_namespace), names);
            }
        }
        return;
    }
    if let Some(name) = tool_definition_name(value) {
        names.push(qualify_tool_name(prefix, &name));
    }
}

fn find_qualified_tool_definition<'a>(
    value: &'a Value,
    prefix: Option<&str>,
    tool_name: &str,
) -> Option<&'a Value> {
    let object = value.as_object()?;
    if object.get("type").and_then(Value::as_str) == Some("namespace") {
        let namespace = object.get("name").and_then(Value::as_str)?;
        let qualified_namespace = qualify_tool_name(prefix, namespace);
        return object
            .get("tools")
            .and_then(Value::as_array)?
            .iter()
            .find_map(|child| {
                find_qualified_tool_definition(child, Some(&qualified_namespace), tool_name)
            });
    }
    let name = tool_definition_name(value)?;
    (qualify_tool_name(prefix, &name) == tool_name).then_some(value)
}

fn namespace_tool_call_identity(
    value: &Value,
    prefix: Option<&str>,
    tool_name: &str,
) -> Option<(String, Option<String>)> {
    let object = value.as_object()?;
    if object.get("type").and_then(Value::as_str) != Some("namespace") {
        return None;
    }
    let namespace = object.get("name").and_then(Value::as_str)?;
    let qualified_namespace = qualify_tool_name(prefix, namespace);
    for child in object.get("tools").and_then(Value::as_array)? {
        if let Some(identity) =
            namespace_tool_call_identity(child, Some(&qualified_namespace), tool_name)
        {
            return Some(identity);
        }
        let Some(child_name) = tool_definition_name(child) else {
            continue;
        };
        if qualify_tool_name(Some(&qualified_namespace), &child_name) == tool_name {
            return Some((child_name, Some(qualified_namespace)));
        }
    }
    None
}

fn qualify_tool_name(prefix: Option<&str>, name: &str) -> String {
    prefix.map_or_else(
        || name.to_owned(),
        |prefix| {
            let separator = if prefix.ends_with("__") { "" } else { "__" };
            if name.starts_with(&format!("{prefix}{separator}")) {
                name.to_owned()
            } else {
                format!("{prefix}{separator}{name}")
            }
        },
    )
}

/// Canonical capability name for OpenAI/Anthropic hosted tools whose wire
/// definition carries only `type`. Function tools are deliberately excluded:
/// their executable name must still come from `function.name` or top-level
/// `name`.
fn hosted_tool_type_name(object: &serde_json::Map<String, Value>) -> Option<&'static str> {
    match object.get("type").and_then(Value::as_str)? {
        "web_search" | "web_search_preview" => Some("web_search"),
        "file_search" => Some("file_search"),
        "computer_use" | "computer_use_preview" => Some("computer_use"),
        "code_interpreter" => Some("code_interpreter"),
        kind if kind.starts_with("web_search_") => Some("web_search"),
        _ => None,
    }
}

/// Whether this definition is an OpenAI hosted tool (as opposed to a function
/// tool whose implementation lives in the client).
#[must_use]
pub fn is_hosted_tool_definition(value: &Value, capability: &str) -> bool {
    value
        .as_object()
        .and_then(hosted_tool_type_name)
        .is_some_and(|name| name == capability)
}

pub fn matches_tool_choice_none(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(choice) => choice.eq_ignore_ascii_case("none"),
        Value::Object(object) => object
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind.eq_ignore_ascii_case("none")),
        _ => false,
    }
}
