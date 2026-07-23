//! User-visible explanations emitted immediately before agentic tool calls.

use serde_json::Value;

use super::capability_router::classify_tool;
use super::planner::Capability;
use super::planner::PlannedToolCall;

/// Render a concise, localized explanation for an imminent agentic action.
///
/// Natural language stays in the seed catalog. The planner selects the concrete
/// tool; this layer classifies it into a capability and picks a natural, spoken
/// sentence for that capability. It never echoes the raw command that will run —
/// `OpenCode` already prints the command itself when the step executes, so the
/// narration only needs to say, in plain words, *what* it is about to do and
/// *why* (issue #819).
pub fn tool_action_narration(prompt: &str, calls: &[PlannedToolCall]) -> String {
    const TARGET_SLOT: &str = concat!("{", "target", "}");
    const SUBJECT_SLOT: &str = concat!("{", "subject", "}");

    let language = crate::language::detect(prompt).slug();
    let Some(first) = calls.first() else {
        return String::new();
    };
    let capability = classify_tool(&first.tool);

    // A local filesystem lookup gets a location-aware, command-free sentence
    // ("Let me look on your Desktop for …") built from the same extraction the
    // `find` command builder uses, so narration and action always agree.
    if capability == Some(Capability::Run) {
        if let Some(found) = super::shell_command::local_path_search_narration(prompt) {
            let intent = format!("agentic_action_find_{}", found.scope);
            if let Some(template) = localized(&intent, language) {
                return template.replace(SUBJECT_SLOT, found.subject.trim());
            }
        }
    }

    let intent = capability.map_or("agentic_action_generic", capability_intent);
    let template = localized(intent, language).unwrap_or_default();
    let target = tool_action_target(&first.arguments);
    template.replace(TARGET_SLOT, &target)
}

/// The seed intent that carries the natural sentence for a capability.
///
/// Capabilities that share a spoken shape reuse one sentence: fetching and
/// reading both "open" a resource, and every write-like capability "updates" a
/// file. Capabilities whose sentence would otherwise expose the raw command
/// (`Run`) or take no target (`AskUser`) map to argument-free phrasings.
const fn capability_intent(capability: Capability) -> &'static str {
    match capability {
        Capability::Search => "agentic_action_search",
        Capability::Fetch | Capability::Read | Capability::ReadMany => "agentic_action_read",
        Capability::Grep => "agentic_action_search_code",
        Capability::Write | Capability::Edit | Capability::MultiEdit => "agentic_action_edit",
        Capability::Run => "agentic_action_run",
        Capability::AskUser => "agentic_action_ask_user",
        Capability::Glob | Capability::ListDir | Capability::Todo | Capability::Subagent => {
            "agentic_action_generic"
        }
    }
}

/// Look up `intent` in the requested language, falling back to English so a
/// missing translation still yields a sentence rather than silence.
fn localized(intent: &str, language: &str) -> Option<String> {
    crate::seed::response_for(intent, language).or_else(|| crate::seed::response_for(intent, "en"))
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
