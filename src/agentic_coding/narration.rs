//! User-visible explanations emitted immediately before agentic tool calls.

use serde_json::Value;

use super::planner::PlannedToolCall;

/// Render a concise, localized explanation for an imminent agentic action.
///
/// Natural language stays in the seed catalog. The planner only selects the
/// concrete tool and target, keeping this useful for every advertised CLI tool
/// rather than a fixed list of product-specific names.
pub(crate) fn tool_action_narration(prompt: &str, calls: &[PlannedToolCall]) -> String {
    let language = crate::language::detect(prompt).slug();
    let template = crate::seed::response_for("agentic_action_before_tool", language)
        .or_else(|| crate::seed::response_for("agentic_action_before_tool", "en"))
        .unwrap_or_default();
    let tool = calls
        .iter()
        .map(|call| call.tool.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let target = calls
        .first()
        .map(|call| tool_action_target(&call.arguments))
        .unwrap_or_default();
    template
        .replace("{tool}", &tool)
        .replace("{target}", &target)
}

fn tool_action_target(arguments: &str) -> String {
    const TARGET_FIELDS: &[&str] = &[
        "url",
        "query",
        "path",
        "file_path",
        "filePath",
        "command",
        "prompt",
        "pattern",
        "target",
        "title",
        "name",
    ];

    fn first_text(value: &Value) -> Option<&str> {
        match value {
            Value::String(text) if !text.trim().is_empty() => Some(text),
            Value::Array(values) => values.iter().find_map(first_text),
            Value::Object(values) => TARGET_FIELDS
                .iter()
                .find_map(|field| values.get(*field).and_then(first_text))
                .or_else(|| values.values().find_map(first_text)),
            _ => None,
        }
    }

    let parsed = serde_json::from_str::<Value>(arguments).ok();
    let target = parsed
        .as_ref()
        .and_then(first_text)
        .unwrap_or(arguments)
        .trim();
    let mut shortened = target.chars().take(160).collect::<String>();
    if target.chars().count() > 160 {
        shortened.push('…');
    }
    shortened
}
