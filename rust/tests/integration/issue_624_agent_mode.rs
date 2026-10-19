use crate::http_server::{
    http_post_json, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

#[test]
fn agent_mode_routes_natural_language_directory_listing_to_bash_tool_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    for prompt in [
        "what files are in this folder?",
        "show me the contents of this directory",
        "can you check which files exist in the current folder?",
        "print a directory listing of the current working directory",
    ] {
        let response = http_post_json(
            port,
            "/api/openai/v1/chat/completions",
            Some("sk-local-agentic-tools"),
            &serde_json::json!({
                "model": "formal-ai",
                "stream": false,
                "messages": [{"role": "user", "content": prompt}],
                "tools": [{
                    "type": "function",
                    "function": {
                        "name": "bash",
                        "description": "Execute a shell command",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "command": {"type": "string"}
                            },
                            "required": ["command"]
                        }
                    }
                }]
            }),
        );

        assert_eq!(
            response["choices"][0]["finish_reason"], "tool_calls",
            "{prompt}"
        );
        let call = &response["choices"][0]["message"]["tool_calls"][0];
        assert_eq!(call["function"]["name"], "bash", "{prompt}");
        let arguments: serde_json::Value =
            serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
        assert_eq!(arguments["command"], "ls", "{prompt}");
    }
}

// Genuine native HTTP/Anthropic routing with synthetic tool receipts; no installer is executed here.
fn missing_compiler_messages() -> serde_json::Value {
    serde_json::json!([
        {"role":"user","content":"Compile the requested program."},
        {"role":"assistant","content":[{"type":"tool_use","id":"compile-observed","name":"Bash","input":{"command":"kotlinc Main.kt -include-runtime -d Main.jar"}}]},
        {"role":"user","content":[{"type":"tool_result","tool_use_id":"compile-observed","is_error":true,"content":"Exit code 127\n/bin/bash: line 1: kotlinc: command not found"}]}
    ])
}
fn compiler_recovery_response(port: u16, messages: &serde_json::Value) -> serde_json::Value {
    http_post_json(
        port,
        "/api/anthropic/v1/messages",
        Some("sk-local-agentic-tools"),
        &serde_json::json!({"model":"formal-ai","max_tokens":4096,"messages":messages,"tools":[
            {"name":"Bash","description":"Execute an authorized shell command","input_schema":{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}},
            {"name":"Write","description":"Write a workspace file","input_schema":{"type":"object","properties":{"file_path":{"type":"string"},"content":{"type":"string"}},"required":["file_path","content"]}}
        ]}),
    )
}
fn compiler_recovery_call(response: &serde_json::Value) -> &serde_json::Value {
    response["content"]
        .as_array()
        .expect("Anthropic content")
        .iter()
        .find(|block| block["type"] == "tool_use")
        .expect("real routed tool call")
}
fn append_compiler_receipt(
    messages: &mut serde_json::Value,
    response: &serde_json::Value,
    receipt: &str,
    failed: bool,
) {
    let call = compiler_recovery_call(response);
    let turns = messages.as_array_mut().expect("declared transcript");
    turns.push(serde_json::json!({"role":"assistant","content":response["content"]}));
    turns.push(serde_json::json!({"role":"user","content":[{"type":"tool_result","tool_use_id":call["id"],"is_error":failed,"content":receipt}]}));
}
#[test]
fn caller_granted_native_anthropic_route_recovers_exact_observed_command() {
    let port = reserve_loopback_port();
    let _server = super::http_server::spawn_formal_ai_server_with_env(
        port,
        &[
            ("FORMAL_AI_AGENT_MODE", "1"),
            ("FORMAL_AI_INSTALL_GRANT", "workspace"),
        ],
    );
    let mut messages = missing_compiler_messages();
    let write = compiler_recovery_response(port, &messages);
    let call = compiler_recovery_call(&write);
    assert_eq!(call["name"], "Write");
    let written_path = call["input"]["file_path"]
        .as_str()
        .or_else(|| call["input"]["path"].as_str());
    assert_eq!(
        written_path,
        Some(".formal-ai/bootstrap/prerequisite-installer.py")
    );
    assert_eq!(
        call["input"]["content"],
        include_str!("../../../data/meta/prerequisite-installer.py")
    );
    append_compiler_receipt(&mut messages, &write, "written", false);
    let run = compiler_recovery_response(port, &messages);
    let call = compiler_recovery_call(&run);
    assert_eq!(call["name"], "Bash");
    let command = call["input"]["command"]
        .as_str()
        .expect("authored transaction");
    assert!(command.contains("https://kotlinlang.org/docs/command-line.html"));
    assert!(command.ends_with(
        "\n# __formal_ai_prerequisite_retry\nkotlinc Main.kt -include-runtime -d Main.jar"
    ));
    let mut failed = messages.clone();
    append_compiler_receipt(
        &mut failed,
        &run,
        "Exit code 1\nsynthetic digest mismatch",
        true,
    );
    let failure = compiler_recovery_response(port, &failed);
    assert!(
        failure["content"]
            .as_array()
            .unwrap()
            .iter()
            .all(|block| block["type"] != "tool_use")
    );
    assert!(failure.to_string().contains("synthetic digest mismatch"));
    append_compiler_receipt(
        &mut messages,
        &run,
        "{\"exit_code\":0,\"stdout\":\"synthetic observed retry success\",\"stderr\":\"\"}",
        false,
    );
    let final_response = compiler_recovery_response(port, &messages);
    assert!(
        final_response["content"]
            .as_array()
            .unwrap()
            .iter()
            .all(|block| block["type"] != "tool_use")
    );
    assert!(
        final_response
            .to_string()
            .contains("synthetic observed retry success")
    );
}
#[test]
fn denied_native_anthropic_grant_never_adopts_publisher_or_transcript_authority() {
    let port = reserve_loopback_port();
    let denied =
        r#"{"schema":"formal-ai-workspace-prerequisites-v1","scope":"workspace","programs":[]}"#;
    let _server = super::http_server::spawn_formal_ai_server_with_env(
        port,
        &[
            ("FORMAL_AI_AGENT_MODE", "1"),
            ("FORMAL_AI_PREREQUISITE_GRANT_JSON", denied),
        ],
    );
    let response = compiler_recovery_response(port, &missing_compiler_messages());
    assert!(
        response["content"]
            .as_array()
            .unwrap()
            .iter()
            .all(|block| block["type"] != "tool_use")
    );
    assert!(response.to_string().contains("command not found"));
    let answer = response["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|block| block["text"].as_str())
        .collect::<String>();
    assert!(answer.starts_with("The command failed:"));
    assert!(answer.contains("Exit code 127"));
    assert!(answer.contains("FORMAL_AI_INSTALL_GRANT=workspace"));
    assert!(!answer.to_ascii_lowercase().contains("would you like"));
    assert!(!answer.to_ascii_lowercase().contains("report issue"));
}

#[test]
fn native_owned_recipe_retains_agent_error_wrapper_exit_status() {
    let port = reserve_loopback_port();
    let _server = super::http_server::spawn_formal_ai_server_with_env(
        port,
        &[
            ("FORMAL_AI_AGENT_MODE", "1"),
            ("FORMAL_AI_INSTALL_GRANT", "workspace"),
        ],
    );
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/prerequisite-client-receipts.json"
    ))
    .expect("shared process receipts");
    for entry in cases.as_array().expect("receipt array") {
        let mut messages = serde_json::json!([{"role":"user","content":"Write a Scala program that prints \"Hello, World!\" to stdout."}]);
        let mut response = compiler_recovery_response(port, &messages);
        let mut verifier = None;
        for _ in 0..8 {
            if compiler_recovery_call(&response)["name"] != "Write" {
                break;
            }
            let call = compiler_recovery_call(&response);
            let written_path = call["input"]["file_path"]
                .as_str()
                .or_else(|| call["input"]["path"].as_str());
            if written_path == Some("tests/verify-output.sh") {
                assert!(verifier.is_none(), "one owned verifier write");
                verifier = Some(
                    call["input"]["content"]
                        .as_str()
                        .expect("verifier bytes")
                        .to_owned(),
                );
            }
            append_compiler_receipt(&mut messages, &response, "written", false);
            response = compiler_recovery_response(port, &messages);
        }
        assert_eq!(
            compiler_recovery_call(&response)["input"]["command"],
            "sh tests/verify-output.sh"
        );
        let verifier = verifier.expect("the actual recipe emits its owned verifier");
        assert!(verifier.contains(entry["command"].as_str().expect("catalog compiler command")));
        assert!(verifier.contains("__formal_ai_owned_process_failed_0__"));
        let mut receipt = entry["receipt"].as_str().expect("receipt text").to_owned();
        if entry["exit"].as_i64().is_some_and(|status| status != 0) {
            receipt.push_str("\n__formal_ai_owned_process_failed_0__");
        }
        append_compiler_receipt(
            &mut messages,
            &response,
            &receipt,
            entry["exit"].as_i64() != Some(0),
        );
        let next = compiler_recovery_response(port, &messages);
        let installer = next["content"]
            .as_array()
            .expect("content blocks")
            .iter()
            .any(|block| {
                block["type"] == "tool_use"
                    && block["name"] == "Write"
                    && block["input"]["file_path"]
                        .as_str()
                        .or_else(|| block["input"]["path"].as_str())
                        == Some(".formal-ai/bootstrap/prerequisite-installer.py")
            });
        assert_eq!(
            installer,
            entry["installer"].as_bool().expect("declared result")
        );
    }
}
