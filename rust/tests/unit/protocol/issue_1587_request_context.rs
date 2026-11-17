use formal_ai::protocol::{
    ChatCompletionRequest, ResponsesRequest, create_chat_completion, create_response,
};

#[test]
fn request_constraints_and_facts_match_shared_fixtures() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../data/parity/issue-1587-request-context.json"
    ))
    .unwrap();
    for row in rows.as_array().unwrap() {
        let request: ChatCompletionRequest =
            serde_json::from_value(serde_json::json!({"messages":row["messages"]})).unwrap();
        let result = create_chat_completion(&request);
        let message = &result.choices[0].message;
        let text = message.content.plain_text();
        if let Some(expected) = row["answer"].as_str() {
            assert_eq!(text, expected, "{}", row["id"]);
        }
        if let Some(expected) = row["contains"].as_str() {
            assert!(text.contains(expected), "{}: {text}", row["id"]);
        }
        if let Some(expected) = row["constraint"].as_str() {
            assert!(message.thinking_steps.iter().any(|step| step.step == "request_constraint" && step.detail.contains(expected)), "{}: missing constraint trace", row["id"]);
        }
        if row["fact"].as_bool() == Some(true) {
            assert!(
                message
                    .thinking_steps
                    .iter()
                    .any(|step| step.step == "request_fact"),
                "{}: missing fact trace",
                row["id"]
            );
        }
        if row["no_constraint"].as_bool() == Some(true) {
            assert!(
                !message
                    .thinking_steps
                    .iter()
                    .any(|step| step.step == "request_constraint"),
                "{}: operational instruction mistaken for output",
                row["id"]
            );
        }
    }
}

#[test]
fn request_context_does_not_leak_and_responses_use_instructions() {
    for _ in 0..2 {
        let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({"messages":[{"role":"system","content":"Always answer in French."},{"role":"user","content":"What is 2 + 2?"}]})).unwrap();
        assert!(
            create_chat_completion(&request).choices[0]
                .message
                .content
                .plain_text()
                .contains("Cannot satisfy language constraint")
        );
        let request: ChatCompletionRequest = serde_json::from_value(
            serde_json::json!({"messages":[{"role":"user","content":"What is 2 + 2?"}]}),
        )
        .unwrap();
        assert_eq!(
            create_chat_completion(&request).choices[0]
                .message
                .content
                .plain_text(),
            "2 + 2 = 4"
        );
    }
    let request: ResponsesRequest = serde_json::from_value(
        serde_json::json!({"instructions":"Answer in JSON.","input":"What is 9 + 4?"}),
    )
    .unwrap();
    let result = create_response(&request);
    let output = serde_json::to_value(result).unwrap();
    assert_eq!(
        output["output"][0]["content"][0]["text"],
        "{\"answer\":\"9 + 4 = 13\"}"
    );
}
