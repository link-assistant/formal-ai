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
            .map(ToOwned::to_owned),
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
            .map(ToOwned::to_owned),
        _ => None,
    }
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
