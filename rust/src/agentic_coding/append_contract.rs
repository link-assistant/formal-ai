//! Explicit schema-qualified atomic record append and provider-owned receipts.
use super::planner::{AgenticPlan, evidence_window_start, plan_one};
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

pub const APPEND_CONTRACT: &str = "atomic-record-append/v1";
pub const APPEND_MODE: &str = "atomic_record_append";

#[must_use]
pub fn general_change_plan_record_header_from(text: &str) -> Option<String> {
    let parsed = crate::seed::parser::parse_lino(text);
    let root = parsed.children.first()?;
    if parsed.children.len() != 1 || root.name != "agentic_tool_capabilities" {
        return None;
    }
    let records = root
        .children
        .iter()
        .filter(|node| node.name == "record-schema" && node.id == "general-change-plan")
        .collect::<Vec<_>>();
    if records.len() != 1 {
        return None;
    }
    let headers = records[0]
        .children
        .iter()
        .filter(|node| node.name == "header")
        .collect::<Vec<_>>();
    let header = headers.first()?.id.as_str();
    if headers.len() != 1
        || !header.as_bytes().first()?.is_ascii_lowercase()
        || !header
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return None;
    }
    Some(format!("{header}\n"))
}

#[must_use]
pub fn general_change_plan_record_header() -> Option<String> {
    general_change_plan_record_header_from(crate::seed::AGENTIC_TOOL_CAPABILITIES_LINO)
}

#[must_use]
pub fn declared_append_tool(definition: &Value) -> Option<&str> {
    let tool = definition.get("function").unwrap_or(definition);
    let parameters = tool.get("parameters")?;
    let properties = parameters.get("properties")?;
    if tool.get("x-formal-ai-contract")?.as_str()? != APPEND_CONTRACT
        || tool.get("x-formal-ai-receipt-contract")?.as_str()? != APPEND_CONTRACT
        || parameters.get("type")?.as_str()? != "object"
        || !parameters.get("required")?.as_array()?.iter().all(|key| {
            key.as_str().is_some_and(|key| {
                [
                    "path",
                    "content",
                    "record_id",
                    "append_mode",
                    "append_request_id",
                ]
                .contains(&key)
            })
        })
        || properties.get("append_mode")?.get("const")?.as_str()? != APPEND_MODE
        || !["path", "content", "record_id", "append_request_id"]
            .into_iter()
            .all(|key| {
                properties
                    .get(key)
                    .and_then(|property| property.get("type"))
                    .and_then(Value::as_str)
                    == Some("string")
            })
    {
        return None;
    }
    tool.get("name")?.as_str()
}

#[must_use]
pub fn project_append_contracts(
    messages: &[ChatMessage],
    definitions: &[Value],
) -> Vec<ChatMessage> {
    let mut projected = messages.to_vec();
    for message in &mut projected {
        message.append_contract = None;
    }
    let contracts: Vec<Value> = definitions
        .iter()
        .filter(|definition| {
            let Some(name) = declared_append_tool(definition) else {
                return false;
            };
            definitions
                .iter()
                .filter(|candidate| {
                    candidate
                        .get("function")
                        .unwrap_or(candidate)
                        .get("name")
                        .and_then(Value::as_str)
                        == Some(name)
                })
                .count()
                == 1
        })
        .cloned()
        .collect();
    if !contracts.is_empty()
        && let Some(user) = projected
            .iter_mut()
            .rev()
            .find(|message| message.role == "user")
    {
        user.append_contract = Some(Value::Array(contracts));
    }
    projected
}

#[must_use]
pub fn append_definition(name: &str) -> Value {
    json!({"type":"function","function":{
        "name":name,"x-formal-ai-contract":APPEND_CONTRACT,
        "x-formal-ai-receipt-contract":APPEND_CONTRACT,
        "parameters":{"type":"object","properties":{
            "path":{"type":"string"},"content":{"type":"string"},
            "record_id":{"type":"string"},"append_request_id":{"type":"string"},
            "append_mode":{"type":"string","const":APPEND_MODE}
        },"required":["path","content"]}
    }})
}

#[must_use]
pub fn append_receipt_valid(receipt: &Value, append_arguments: &Value) -> bool {
    let Some(before) = receipt.get("before").and_then(Value::as_str) else {
        return false;
    };
    let Some(after) = receipt.get("after").and_then(Value::as_str) else {
        return false;
    };
    let Some(content) = append_arguments.get("content").and_then(Value::as_str) else {
        return false;
    };
    let Some(identity) = append_arguments.get("record_id").and_then(Value::as_str) else {
        return false;
    };
    if receipt.get("schema").and_then(Value::as_str) != Some(APPEND_CONTRACT)
        || receipt.get("complete").and_then(Value::as_bool) != Some(true)
        || receipt.get("success").and_then(Value::as_bool) != Some(true)
        || ["path", "record_id", "content", "append_request_id"]
            .into_iter()
            .any(|key| receipt.get(key) != append_arguments.get(key))
        || receipt.get("before_bytes") != Some(&Value::from(before.len()))
        || receipt.get("after_bytes") != Some(&Value::from(after.len()))
    {
        return false;
    }
    match receipt.get("operation").and_then(Value::as_str) {
        Some("appended") => {
            let separator = if !before.is_empty() && !before.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            !before.split('\n').any(|line| line == identity)
                && after == format!("{before}{separator}{content}")
        }
        Some("already_present") => {
            if after != before || before.split('\n').filter(|line| *line == identity).count() != 1 {
                return false;
            }
            let Some(header) = general_change_plan_record_header() else {
                return false;
            };
            before.match_indices(content).any(|(start, _)| {
                let following = &before[start + content.len()..];
                (start == 0 || before.as_bytes().get(start - 1) == Some(&b'\n'))
                    && (following.is_empty() || following.starts_with(header.as_str()))
            })
        }
        _ => false,
    }
}

pub enum AppendRecordStep {
    Pending(AgenticPlan),
    Observed,
    Refused,
}

#[must_use]
pub fn append_record_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    path: &str,
    content: &str,
    identity: &str,
) -> Option<AppendRecordStep> {
    let current = &messages[evidence_window_start(messages)..];
    let user = messages
        .iter()
        .rev()
        .find(|message| message.role == "user")?;
    let contracts = user.append_contract.as_ref()?.as_array()?;
    let names: Vec<&str> = contracts
        .iter()
        .filter_map(declared_append_tool)
        .filter(|name| tool_names.contains(name))
        .collect();
    if names.len() != 1 {
        return None;
    }
    let tool = names[0];
    let mut append_arguments = json!({"path":path,"content":content,"record_id":identity,"append_mode":APPEND_MODE,
        "append_request_id":format!("{identity}/{}", messages.len())});
    let mut matched = None;
    for (index, message) in current.iter().enumerate() {
        if message.role != "assistant" {
            continue;
        }
        for call in &message.tool_calls {
            let mut expected = append_arguments.clone();
            expected["append_request_id"] = Value::String(format!(
                "{identity}/{}",
                evidence_window_start(messages) + index
            ));
            if call.function.name != tool
                || serde_json::from_str::<Value>(&call.function.arguments)
                    .ok()
                    .as_ref()
                    != Some(&expected)
            {
                continue;
            }
            let results: Vec<&ChatMessage> = current[index + 1..]
                .iter()
                .filter(|result| {
                    result.role == "tool"
                        && result.tool_call_id.as_deref() == Some(call.id.as_str())
                        && result.name.as_deref() == Some(tool)
                })
                .collect();
            append_arguments = expected;
            matched = Some(if results.len() == 1 {
                Some(results[0])
            } else {
                None
            });
        }
    }
    let Some(result) = matched else {
        return Some(AppendRecordStep::Pending(plan_one(
            tool,
            append_arguments.to_string(),
        )));
    };
    Some(
        if result.is_some_and(|message| {
            !message.is_error
                && message
                    .append_receipt
                    .as_ref()
                    .is_some_and(|receipt| append_receipt_valid(receipt, &append_arguments))
        }) {
            AppendRecordStep::Observed
        } else {
            AppendRecordStep::Refused
        },
    )
}
