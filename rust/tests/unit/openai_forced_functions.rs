//! Native coverage for the forced client function requests exercised by the JS server.
use formal_ai::{
    ChatCompletionRequest, ResponsesRequest, SolverConfig, UniversalSolver,
    create_chat_completion_with_solver, create_response_with_solver,
};
use serde_json::{Value, json};

fn request_body(row: &Value, chat: bool) -> Value {
    let sentiment = row["schema"] != "extraction";
    let mut parameters = if sentiment {
        json!({
            "type":"object","required":["label"],"additionalProperties":false,
            "properties":{"label":{"type":"string","enum":["positive","neutral","negative"]}}
        })
    } else {
        json!({
            "type":"object","required":["count"],"additionalProperties":false,
            "properties":{"count":{"type":"integer","enum":[1,2]}}
        })
    };
    if let Some(constraint) = row["unsupported"].as_str() {
        parameters[constraint] = json!(2);
    }
    let name = row["name"].as_str().unwrap_or("submit_label");
    let description = row["description"].as_str().unwrap_or(if sentiment {
        "Submit the sentiment label of the message"
    } else {
        "Record the count"
    });
    let mut function = json!({"name": name, "description": description, "parameters": parameters});
    let (tool, forced) = if let Some(hosted) = row["hosted"].as_str() {
        (json!({"type": hosted}), json!({"type": hosted}))
    } else if chat {
        (
            json!({"type":"function","function":function}),
            json!({"type":"function","function":{"name":name}}),
        )
    } else {
        function["type"] = json!("function");
        function["strict"] = json!(true);
        (function, json!({"type":"function","name":name}))
    };
    let mut tools = vec![tool];
    if row["extra_function"] == true {
        let mut customer = json!({
            "name":"record_customer",
            "parameters":{
                "type":"object","required":["first","last"],
                "properties":{"first":{"type":"string"},"last":{"type":"string"}}
            }
        });
        let extra = if chat {
            json!({"type":"function","function":customer})
        } else {
            customer["type"] = json!("function");
            customer
        };
        tools.insert(0, extra);
    }
    if row["extra_hosted"] == true {
        tools.insert(0, json!({"type":"code_interpreter"}));
    }
    let mut body = json!({
        "model":"formal-ai", "tools":tools, "tool_choice":row.get("choice").unwrap_or(&forced),
    });
    if chat {
        body["messages"] = json!([{"role":"user","content":row["prompt"]}]);
    } else {
        body["input"] = row["prompt"].clone();
    }
    body
}

#[test]
fn both_openai_protocols_handle_forced_functions_in_either_agent_mode() {
    let rows: Vec<Value> = serde_json::from_str(include_str!(
        "../../../data/parity/issue-1590-forced-functions.json"
    ))
    .unwrap();
    for agent_mode in [false, true] {
        let solver = UniversalSolver::new(SolverConfig {
            agent_mode,
            ..SolverConfig::default()
        });
        for row in &rows {
            if agent_mode && row["gated"] == true {
                continue;
            }
            for chat in [true, false] {
                let request = request_body(row, chat);
                let result = if chat {
                    let request: ChatCompletionRequest = serde_json::from_value(request).unwrap();
                    serde_json::to_value(create_chat_completion_with_solver(&request, &solver))
                        .unwrap()
                } else {
                    let request: ResponsesRequest = serde_json::from_value(request).unwrap();
                    serde_json::to_value(create_response_with_solver(&request, &solver)).unwrap()
                };
                let id = &row["id"];
                let message = &result["choices"][0]["message"];
                let output = &result["output"];
                if let Some(expected) = row.get("expected") {
                    let call = if chat {
                        assert_eq!(
                            result["choices"][0]["finish_reason"], "tool_calls",
                            "{id}: {result}"
                        );
                        assert_eq!(message["tool_calls"].as_array().unwrap().len(), 1);
                        &message["tool_calls"][0]["function"]
                    } else {
                        assert_eq!(output.as_array().unwrap().len(), 1, "{id}: {result}");
                        assert_eq!(output[0]["type"], "function_call", "{id}: {result}");
                        &output[0]
                    };
                    assert_eq!(call["name"], row["name"].as_str().unwrap_or("submit_label"));
                    let arguments: Value =
                        serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
                    assert_eq!(&arguments, expected, "{id}: {result}");
                } else if row["gated"] == true {
                    let text = if chat {
                        &message["content"]
                    } else {
                        &output[0]["content"][0]["text"]
                    };
                    assert!(
                        text.as_str()
                            .unwrap()
                            .contains("without explicit agent mode"),
                        "{id}: {result}"
                    );
                } else {
                    let reason = if chat {
                        assert_eq!(message["content"], "");
                        &message["refusal"]
                    } else {
                        assert_eq!(output[0]["content"][0]["type"], "refusal");
                        assert!(output[0]["content"][0].get("text").is_none());
                        &output[0]["content"][0]["refusal"]
                    };
                    assert!(
                        reason
                            .as_str()
                            .unwrap()
                            .contains("Could not derive arguments"),
                        "{id}: {result}"
                    );
                }
            }
        }
    }
}
