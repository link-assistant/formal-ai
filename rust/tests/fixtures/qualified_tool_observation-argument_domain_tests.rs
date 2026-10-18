#![cfg(test)]

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

#[test]
fn optional_result_name_binds_unique_id_without_accepting_conflicts_or_duplicates() {
    let declaration = ChatMessage::assistant_tool_calls(vec![ToolCall {
        id: "optional-name".into(),
        kind: "function".into(),
        function: FunctionCall {
            name: "bash".into(),
            arguments: "{\"command\":\"true\"}".into(),
        },
    }]);
    let mut result = ChatMessage::tool_result("optional-name", "bash", "Exit code 0\n");
    result.name = None;
    let mut messages = vec![declaration, result];
    let frames = scan(&messages, 0);
    assert!(frames[0].binding == Binding::Exact && frames[0].succeeded);
    assert!(scan(&messages, 1).is_empty());
    messages[1].name = Some("write".into());
    assert!(scan(&messages, 0)[0].binding == Binding::Contradicted);
    messages[1].name = None;
    messages.push(messages[1].clone());
    assert!(scan(&messages, 0)[0].binding == Binding::Contradicted);
}
