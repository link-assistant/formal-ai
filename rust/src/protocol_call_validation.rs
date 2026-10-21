//! Refuse unbound required values at the protocol boundary on every planner route.
use crate::agentic_coding::planner::AgenticPlan;
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

pub fn invalid_arguments(value: &Value, schema: &Value, depth: usize) -> bool {
    if depth > 32 || schema == &Value::Bool(false) || schema.get("$ref").is_some() {
        return true;
    }
    if let Some(parts) = schema.get("allOf").and_then(Value::as_array)
        && parts
            .iter()
            .any(|part| invalid_arguments(value, part, depth + 1))
    {
        return true;
    }
    if let Some(parts) = schema.get("anyOf").and_then(Value::as_array)
        && !parts
            .iter()
            .any(|part| !invalid_arguments(value, part, depth + 1))
    {
        return true;
    }
    if let Some(parts) = schema.get("oneOf").and_then(Value::as_array)
        && parts
            .iter()
            .filter(|part| !invalid_arguments(value, part, depth + 1))
            .count()
            != 1
    {
        return true;
    }
    if let Some(part) = schema.get("not")
        && !invalid_arguments(value, part, depth + 1)
    {
        return true;
    }
    let matches_type = |kind: &str| match kind.to_ascii_lowercase().as_str() {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value
            .as_f64()
            .is_some_and(|number| number.is_finite() && number.fract() == 0.0),
        _ => false,
    };
    if let Some(kind) = schema.get("type") {
        let valid = kind.as_str().map_or_else(
            || {
                kind.as_array()
                    .is_some_and(|types| types.iter().filter_map(Value::as_str).any(matches_type))
            },
            matches_type,
        );
        if !valid {
            return true;
        }
    }
    if schema
        .get("enum")
        .and_then(Value::as_array)
        .is_some_and(|items| !items.contains(value))
        || schema.get("const").is_some_and(|item| item != value)
    {
        return true;
    }
    if let Some(text) = value.as_str() {
        let length = text.chars().count() as u64;
        if schema
            .get("minLength")
            .and_then(Value::as_u64)
            .is_some_and(|n| length < n)
            || schema
                .get("maxLength")
                .and_then(Value::as_u64)
                .is_some_and(|n| length > n)
        {
            return true;
        }
    }
    if let (Some(text), Some(pattern)) = (
        value.as_str(),
        schema.get("pattern").and_then(Value::as_str),
    ) && (pattern.chars().count() > 128
        || text.chars().count() > 16384
        || pattern.contains(['(', ')', '|', '{', '}', '\\'])
        || pattern
            .chars()
            .filter(|ch| matches!(ch, '+' | '*' | '?'))
            .count()
            > 1
        || regex::Regex::new(pattern).map_or(true, |pattern| !pattern.is_match(text)))
    {
        return true;
    }
    if let Some(number) = value.as_f64()
        && (schema
            .get("minimum")
            .and_then(Value::as_f64)
            .is_some_and(|n| number < n)
            || schema
                .get("maximum")
                .and_then(Value::as_f64)
                .is_some_and(|n| number > n))
    {
        return true;
    }
    if let Some(items) = value.as_array() {
        let length = items.len() as u64;
        if schema
            .get("minItems")
            .and_then(Value::as_u64)
            .is_some_and(|n| length < n)
            || schema
                .get("maxItems")
                .and_then(Value::as_u64)
                .is_some_and(|n| length > n)
        {
            return true;
        }
        if let Some(item_schema) = schema.get("items")
            && items
                .iter()
                .any(|item| invalid_arguments(item, item_schema, depth + 1))
        {
            return true;
        }
    }
    if let Some(object) = value.as_object() {
        if let Some(required) = schema.get("required").and_then(Value::as_array)
            && required.iter().filter_map(Value::as_str).any(|key| {
                object
                    .get(key)
                    .is_none_or(|item| item.is_null() || item.as_str() == Some(""))
            })
        {
            return true;
        }
        for (key, item) in object {
            if let Some(property) = schema
                .get("properties")
                .and_then(|fields| fields.get(key))
                .or_else(|| schema.get("additionalProperties"))
                && invalid_arguments(item, property, depth + 1)
            {
                return true;
            }
        }
    }
    false
}

pub fn bind_plan(
    plan: AgenticPlan,
    tools: &[Value],
    messages: &[ChatMessage],
    prompt: &str,
    workspace: Option<&str>,
) -> AgenticPlan {
    let AgenticPlan::ToolCalls(mut calls) = plan else {
        if let AgenticPlan::Final(answer) = &plan {
            crate::dialog_log::structured_log::event(
                "debug",
                "planner",
                || "planning_terminal".into(),
                || json!({"answer":answer.chars().take(4096).collect::<String>(),"state":"terminal","repeat_limit":crate::bounded_retry::MAXIMUM_FAILED_ATTEMPTS}),
            );
        }
        return plan;
    };
    for call in &mut calls {
        call.arguments = crate::protocol_responses::response_arguments_for_tool(
            tools,
            &call.tool,
            call.arguments.clone(),
            prompt,
            workspace,
        );
        if let Some(definition) = crate::protocol_policy::find_tool_definition(tools, &call.tool)
            && let Some(schema) = crate::protocol_responses::tool_parameters_schema(definition)
        {
            let value = serde_json::from_str::<Value>(&call.arguments);
            if value
                .as_ref()
                .map_or(true, |value| invalid_arguments(value, schema, 0))
            {
                crate::dialog_log::structured_log::event(
                    "error",
                    "planner",
                    || "tool_argument_binding_failed".into(),
                    || json!({"tool":call.tool,"arguments":call.arguments,"schema":schema}),
                );
                return AgenticPlan::Final(crate::seed::agentic_message(
                    "tool-argument-binding-incomplete",
                    &[("tool", &call.tool)],
                ));
            }
        }
        crate::dialog_log::structured_log::event(
            "debug",
            "planner",
            || "tool_call_bound".into(),
            || json!({"tool":call.tool,"arguments":call.arguments,"repeat_limit":crate::agentic_coding::repetition_guard::MAXIMUM_IDENTICAL_ACTION_ATTEMPTS,"state":"awaiting_client_receipt"}),
        );
    }
    if let Some(reason) =
        crate::agentic_coding::repetition_guard::repeated_action_stop_for_candidates(
            messages,
            Some(&calls),
        )
    {
        return AgenticPlan::Final(reason);
    }
    AgenticPlan::ToolCalls(calls)
}
