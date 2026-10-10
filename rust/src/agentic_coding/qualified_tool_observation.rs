//! Immutable current-window transcript binding, never execution or approval authority.

use super::capability_router::classify_tool;
use super::planner::Capability;
use super::tool_result::failure_message;
use crate::protocol::ChatMessage;
use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Binding {
    Exact,
    Unknown,
    Contradicted,
}

pub(super) struct QualifiedToolAttempt {
    capability: Capability,
    declared_tool: String,
    call_id: String,
    arguments_value: Value,
    pub(super) binding: Binding,
    duplicate: bool,
    receipt_present: bool,
    pub(super) succeeded: bool,
    pub(super) detail: String,
}

#[derive(PartialEq, Eq)]
enum ExactArgumentNumber {
    Integer(i64),
    Fraction(u64),
}

fn exact_argument_number(number: &serde_json::Number) -> Option<ExactArgumentNumber> {
    const MAXIMUM: i64 = 9_007_199_254_740_991;
    if number.is_i64() {
        return number
            .as_i64()
            .filter(|value| (-MAXIMUM..=MAXIMUM).contains(value))
            .map(ExactArgumentNumber::Integer);
    }
    if number.is_u64() {
        return number
            .as_u64()
            .filter(|value| *value <= MAXIMUM as u64)
            .map(|value| ExactArgumentNumber::Integer(value as i64));
    }
    if !number.is_f64() {
        return None;
    }
    // This reads an already stored float; integer kinds never pass through float conversion.
    let value = number.as_f64()?;
    if !value.is_finite() {
        return None;
    }
    if value.fract() == 0.0 {
        if !(-(MAXIMUM as f64)..=MAXIMUM as f64).contains(&value) {
            return None;
        }
        return Some(ExactArgumentNumber::Integer(value as i64));
    }
    Some(ExactArgumentNumber::Fraction(value.to_bits()))
}

/// Closed JSON equality over exactly representable safe integers and stored finite fractions.
/// Normalizes integer/float representations of one and signed zero without rounding integer kinds.
/// This compares transcript arguments only; it grants no provider execution or effect authority.
pub(super) fn argument_values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(left), Value::Bool(right)) => left == right,
        (Value::String(left), Value::String(right)) => left == right,
        (Value::Number(left), Value::Number(right)) => {
            let Some(left) = exact_argument_number(left) else {
                return false;
            };
            let Some(right) = exact_argument_number(right) else {
                return false;
            };
            left == right
        }
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| argument_values_equal(left, right))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, left)| {
                    right
                        .get(key)
                        .is_some_and(|right| argument_values_equal(left, right))
                })
        }
        _ => false,
    }
}

#[cfg(test)]
mod argument_domain_tests {
    use super::*;
    use crate::protocol::{FunctionCall, ToolCall};

    #[test]
    fn exact_numeric_domain_normalizes_without_lossy_integer_conversion() {
        for (left, right, expected) in [
            ("1", "1.0", true),
            ("-0.0", "0", true),
            ("0.125", "0.125", true),
            ("0.125", "0.25", false),
            ("9007199254740991", "9007199254740991.0", true),
            ("-9007199254740991", "-9007199254740991.0", true),
            ("9007199254740992", "9007199254740992", false),
            ("9007199254740992", "9007199254740993", false),
            ("-9007199254740992", "-9007199254740992", false),
            ("18446744073709551615", "18446744073709551615", false),
            ("{\"a\":[1,null]}", "{\"a\":[1.0,null]}", true),
            (
                "{\"a\":[9007199254740993]}",
                "{\"a\":[9007199254740993]}",
                false,
            ),
        ] {
            let left: Value = serde_json::from_str(left).expect("left JSON");
            let right: Value = serde_json::from_str(right).expect("right JSON");
            assert_eq!(argument_values_equal(&left, &right), expected);
        }
    }

    #[test]
    fn unsafe_arguments_never_accept_an_exact_success_receipt() {
        let messages = vec![
            ChatMessage::assistant_tool_calls(vec![ToolCall {
                id: "unsafe-call".into(),
                kind: "function".into(),
                function: FunctionCall {
                    name: "bash".into(),
                    arguments: "{\"command\":\"true\",\"nested\":[9007199254740993]}".into(),
                },
            }]),
            ChatMessage::tool_result("unsafe-call", "bash", "Output: done\nExit Code: 0"),
        ];
        assert!(scan(&messages, 0).is_empty());
    }
}

pub(super) fn scan(messages: &[ChatMessage], start: usize) -> Vec<QualifiedToolAttempt> {
    let mut frames: Vec<QualifiedToolAttempt> = Vec::new();
    let mut declared_ids = std::collections::HashSet::new();
    for message in messages.iter().skip(start) {
        if message.role.eq_ignore_ascii_case("assistant") {
            for call in &message.tool_calls {
                if call.id.is_empty() {
                    continue;
                }
                let Some(capability) = classify_tool(&call.function.name) else {
                    continue;
                };
                let prior_declaration = !declared_ids.insert(call.id.clone());
                if prior_declaration {
                    for frame in frames.iter_mut().filter(|frame| frame.call_id == call.id) {
                        frame.binding = Binding::Contradicted;
                        frame.succeeded = false;
                        frame.duplicate = true;
                    }
                }
                let Ok(arguments_value) = serde_json::from_str::<Value>(&call.function.arguments)
                else {
                    continue;
                };
                if !argument_values_equal(&arguments_value, &arguments_value) {
                    continue;
                }
                let duplicate =
                    prior_declaration || frames.iter().any(|frame| frame.call_id == call.id);
                if duplicate {
                    for frame in frames.iter_mut().filter(|frame| frame.call_id == call.id) {
                        frame.binding = Binding::Contradicted;
                        frame.succeeded = false;
                        frame.duplicate = true;
                    }
                }
                frames.push(QualifiedToolAttempt {
                    capability,
                    declared_tool: call.function.name.clone(),
                    call_id: call.id.clone(),
                    arguments_value,
                    binding: if duplicate {
                        Binding::Contradicted
                    } else {
                        Binding::Unknown
                    },
                    duplicate,
                    receipt_present: false,
                    succeeded: false,
                    detail: String::new(),
                });
            }
            continue;
        }
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let Some(id) = message.tool_call_id.as_deref() else {
            continue;
        };
        let count = frames.iter().filter(|frame| frame.call_id == id).count();
        if count != 1 {
            for frame in frames.iter_mut().filter(|frame| frame.call_id == id) {
                frame.binding = Binding::Contradicted;
                frame.succeeded = false;
                frame.duplicate = true;
            }
            continue;
        }
        let Some(frame) = frames.iter_mut().find(|frame| frame.call_id == id) else {
            continue;
        };
        if frame.duplicate {
            continue;
        }
        if frame.receipt_present {
            frame.binding = Binding::Contradicted;
            frame.succeeded = false;
            frame.duplicate = true;
            continue;
        }
        frame.receipt_present = true;
        let raw = message.content.plain_text();
        frame.detail = raw.clone();
        frame.binding = match message.name.as_deref() {
            None => Binding::Unknown,
            Some(name) if name == frame.declared_tool => Binding::Exact,
            Some(_) => Binding::Contradicted,
        };
        frame.succeeded = frame.binding == Binding::Exact
            && failure_message(&raw, message.is_error, frame.capability != Capability::Run)
                .is_none();
    }
    frames
}

pub(super) fn latest<'a>(
    frames: &'a [QualifiedToolAttempt],
    capability: Capability,
    arguments: &Value,
) -> Option<&'a QualifiedToolAttempt> {
    frames.iter().rev().find(|frame| {
        frame.capability == capability && argument_values_equal(&frame.arguments_value, arguments)
    })
}
