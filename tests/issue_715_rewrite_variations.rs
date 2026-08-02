//! Structural NL variation and safety coverage for issue #715 rewrites.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::normal_markov::quoted_segments;
use formal_ai::protocol::{ChatMessage, ToolCall};

fn one_call(messages: &[ChatMessage]) -> PlannedToolCall {
    match plan_chat_step(messages, &["read_file", "write_file"]) {
        Some(AgenticPlan::ToolCalls(mut calls)) if calls.len() == 1 => calls.remove(0),
        other => panic!("expected one tool call, got {other:?}"),
    }
}

fn arguments(call: &PlannedToolCall) -> serde_json::Value {
    serde_json::from_str(&call.arguments).expect("tool arguments")
}

fn prior_artifact(path: &str, source: &str, prompt: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage::user("Create the active workspace artifact."),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "write-prior".to_owned(),
            "write_file".to_owned(),
            serde_json::json!({"path": path, "content": source}).to_string(),
        )]),
        ChatMessage::tool_result("write-prior", "write_file", format!("Wrote {path}")),
        ChatMessage::assistant("The artifact is in the client workspace."),
        ChatMessage::user(prompt),
    ]
}

fn rewrite(path: &str, source: &str, prompt: &str) -> String {
    let mut messages = prior_artifact(path, source, prompt);
    let read = one_call(&messages);
    assert_eq!(read.tool, "read_file", "{prompt}");
    assert_eq!(arguments(&read)["path"], path, "{prompt}");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-current".to_owned(),
        "read_file".to_owned(),
        read.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "read-current",
        "read_file",
        source,
    ));

    let write = one_call(&messages);
    assert_eq!(write.tool, "write_file", "{prompt}");
    assert_eq!(arguments(&write)["path"], path, "{prompt}");
    arguments(&write)["content"]
        .as_str()
        .expect("written content")
        .to_owned()
}

#[test]
fn creation_and_deletion_execute_across_ten_phrasings_in_each_natural_language() {
    let groups = [
        [
            "Map '' to 'EN' at the start.",
            "Where '' occurs, create 'EN'.",
            "Insert by substituting '' with 'EN'.",
            "Apply the empty-to-content rule '' then 'EN'.",
            "Use the creation rewrite '' → 'EN'.",
            "Map 'drop' to '' in the active file.",
            "Delete by substituting 'drop' with ''.",
            "Apply the content-to-empty rule 'drop' then ''.",
            "Remove the sequence using 'drop' → ''.",
            "Use the deletion rewrite 'drop' as ''.",
        ],
        [
            "Отобрази '' в 'RU' в начале.",
            "Там, где встречается '', создай 'RU'.",
            "Вставь, заменив '' на 'RU'.",
            "Примени правило из пустоты '' в 'RU'.",
            "Используй создание '' → 'RU'.",
            "Отобрази 'drop' в '' в активном файле.",
            "Удали, заменив 'drop' на ''.",
            "Примени правило из 'drop' в пустоту ''.",
            "Убери последовательность через 'drop' → ''.",
            "Используй удаление 'drop' как ''.",
        ],
        [
            "शुरुआत में '' को 'HI' में मैप करें।",
            "जहाँ '' है वहाँ 'HI' बनाएँ।",
            "'' को 'HI' से बदलकर जोड़ें।",
            "खाली '' से 'HI' वाला नियम लगाएँ।",
            "निर्माण नियम '' → 'HI' उपयोग करें।",
            "सक्रिय फ़ाइल में 'drop' को '' में मैप करें।",
            "'drop' को '' से बदलकर हटाएँ।",
            "'drop' से खाली '' वाला नियम लगाएँ।",
            "'drop' → '' से क्रम हटाएँ।",
            "हटाने का नियम 'drop' से '' उपयोग करें।",
        ],
        [
            "在开头把''映射为'ZH'。",
            "在''出现处创建'ZH'。",
            "将''改写为'ZH'来插入。",
            "应用从空串''到'ZH'的规则。",
            "使用创建重写'' → 'ZH'。",
            "在活动文件中把'drop'映射为''。",
            "将'drop'改写为''来删除。",
            "应用从'drop'到空串''的规则。",
            "通过'drop' → ''移除序列。",
            "使用删除重写'drop'为''。",
        ],
    ];

    for prompt in groups.into_iter().flatten() {
        let slots = quoted_segments(prompt);
        assert_eq!(slots.len(), 2, "{prompt}: {slots:?}");
        assert!(
            slots[0].is_empty() || slots[0] == "drop",
            "{prompt}: unexpected source slot {slots:?}"
        );
        assert!(
            slots[0].is_empty() || slots[1].is_empty(),
            "{prompt}: deletion must retain an empty destination {slots:?}"
        );
        let (source, expected) = if slots[0].is_empty() {
            ("body", format!("{}body", slots[1]))
        } else {
            ("keepdrop", String::from("keep"))
        };
        assert_eq!(
            rewrite("src/active.rs", source, prompt),
            expected,
            "{prompt}"
        );
    }
}

#[test]
fn active_artifacts_are_not_restricted_by_a_programming_extension_catalog() {
    let updated = rewrite(
        "notes.custom-format",
        "alpha beta alpha",
        "Transform 'alpha' into 'omega'.",
    );
    assert_eq!(updated, "omega beta alpha");
}

#[test]
fn a_single_self_containing_substitution_is_terminal() {
    let updated = rewrite("main.rs", "foo", "Transform 'foo' into 'foobar'.");
    assert_eq!(updated, "foobar");
}

#[test]
fn a_concise_quoted_rule_is_not_mistaken_for_transport_framing() {
    let updated = rewrite("notes.txt", "can't wait", "'can't' → 'can'");
    assert_eq!(updated, "can wait");
}

#[test]
fn a_cyclic_program_reaches_the_bound_without_writing_partial_bytes() {
    let prompt = "Apply the ordered rules 'a' to 'b', then 'b' to 'a'.";
    let mut messages = prior_artifact("cycle.txt", "a", prompt);
    let read = one_call(&messages);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-cycle".to_owned(),
        "read_file".to_owned(),
        read.arguments,
    )]));
    messages.push(ChatMessage::tool_result("read-cycle", "read_file", "a"));

    match plan_chat_step(&messages, &["read_file", "write_file"]) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("100000-step safety bound"));
            assert!(answer.contains("no partial bytes were written"));
            assert!(answer.contains("halt StepLimit"));
            assert!(answer.contains("omitted_steps 99936"));
            assert!(
                answer.len() < 20_000,
                "bounded trace response was {} bytes",
                answer.len()
            );
        }
        other => panic!("cyclic rewrite must stop safely, got {other:?}"),
    }
}
