/// Preserve the whole original literal task session while requiring the added
/// real plan reads and complete command-bound process observation.
fn assert_literal_capture_transition(legacy: &str, observed: &str) {
    let historical: serde_json::Value = serde_json::from_str(legacy).expect("legacy JSON");
    let mut actual: serde_json::Value = serde_json::from_str(observed).expect("observed JSON");
    let task = historical["task"].as_str().expect("original task");
    assert_eq!(actual["task"], task);
    let steps = actual["steps"].as_array_mut().expect("observed steps");
    assert_eq!(
        steps.len(),
        historical["steps"].as_array().unwrap().len() + 2
    );
    for index in [0, 2] {
        assert_eq!(steps[index]["tool"], "read_file");
        for key in ["path", "filePath", "file_path"] {
            assert_eq!(
                steps[index]["arguments"][key],
                ".formal-ai/general-change-plan.lino"
            );
        }
    }
    assert!(steps[2]["result"].as_str().unwrap().contains(task));
    let verification = steps.last_mut().expect("verification");
    let receipt: serde_json::Value =
        serde_json::from_str(verification["result"].as_str().unwrap()).expect("process receipt");
    assert_eq!(receipt["schema"], "command-execution-receipt/v1");
    assert_eq!(receipt["command"], verification["arguments"]["command"]);
    assert_eq!(receipt["exit_code"], 0);
    assert_eq!(receipt["complete"], true);
    assert_eq!(receipt["timed_out"], false);
    assert_eq!(receipt["truncated"], false);
    assert!(receipt["stderr"].is_string());
    let old_verification = historical["steps"].as_array().unwrap().last().unwrap();
    assert_eq!(receipt["stdout"], old_verification["result"]);
    verification["result"] = old_verification["result"].clone();
    steps.remove(2);
    steps.remove(0);
    let tools = actual["tools_advertised"].as_array_mut().unwrap();
    assert_eq!(tools.last().unwrap(), "read_file");
    tools.pop();
    assert_eq!(actual["turns"], historical["turns"].as_u64().unwrap() + 2);
    actual["turns"] = historical["turns"].clone();
    assert_eq!(
        actual, historical,
        "every original session field and target byte is preserved"
    );
}

/// Compare the unchanged historical request with its current target-owned effects.
/// Historical protocol bytes remain a separate, unchanged replay control above.
pub fn assert_current_literal_observation(legacy: &str, observed: &str) {
    use formal_ai::agentic_coding::general_planner::PLAN_PATH;
    let historical: serde_json::Value = serde_json::from_str(legacy).expect("legacy JSON");
    let actual: serde_json::Value = serde_json::from_str(observed).expect("current JSON");
    assert_eq!(actual["task"], historical["task"]);
    assert_eq!(actual["hit_turn_cap"], false);
    let original = historical["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["tool"] == "write_file" && step["arguments"]["path"] != PLAN_PATH)
        .expect("immutable original target write");
    let target = original["arguments"]["path"].as_str().unwrap();
    let content = original["arguments"]["content"].as_str().unwrap();
    let steps = actual["steps"].as_array().expect("current actual steps");
    assert!(
        !steps.is_empty() && steps.len() <= 8,
        "bounded current observation"
    );
    let mut written = false;
    let mut observed_after_write = false;
    for step in steps {
        let arguments = step["arguments"].as_object().expect("bound arguments");
        for key in ["path", "filePath", "file_path"] {
            assert_eq!(
                arguments.get(key).and_then(serde_json::Value::as_str),
                Some(target)
            );
        }
        match step["tool"].as_str().unwrap() {
            "write_file" => {
                assert!(
                    !written,
                    "exclusive creation cannot overwrite or retry a successful write"
                );
                assert_eq!(arguments.len(), 4);
                assert_eq!(arguments["content"], content);
                assert_eq!(
                    step["result"],
                    format!("wrote {} byte(s) to {target}", content.len())
                );
                written = true;
                observed_after_write = false;
            }
            "read_file" => {
                assert!(
                    written,
                    "a target read before creation has no verification authority"
                );
                assert_eq!(arguments.len(), 3);
                assert_eq!(
                    step["result"],
                    formal_ai::agentic_coding::tool_result::complete_owned_source_read_frame(
                        content
                    )
                );
                observed_after_write = true;
            }
            other => panic!("unrequested current literal effect: {other}"),
        }
    }
    assert!(
        written && observed_after_write,
        "actual target creation needs fresh exact-byte observation"
    );
    let final_answer =
        formal_ai::seed::localized_response("coding_workspace_effect_observed", "en")
            .expect("maintained observed-write response")
            .replace("{path}", target);
    assert_eq!(actual["final_answer"], final_answer);
    let mut expected_tools = historical["tools_advertised"].as_array().unwrap().clone();
    if !expected_tools.iter().any(|tool| tool == "read_file") {
        expected_tools.push(serde_json::json!("read_file"));
    }
    assert_eq!(
        actual["tools_advertised"],
        serde_json::json!(expected_tools)
    );
    let mut historical_identity = historical.clone();
    let mut current_identity = actual.clone();
    for field in ["steps", "final_answer", "turns", "tools_advertised"] {
        historical_identity.as_object_mut().unwrap().remove(field);
        current_identity.as_object_mut().unwrap().remove(field);
    }
    assert_eq!(
        current_identity, historical_identity,
        "all other original session identities remain exact"
    );
}
