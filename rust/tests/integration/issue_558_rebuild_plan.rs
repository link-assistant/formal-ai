#[path = "../fixtures/issue-558/repair_fixture.rs"]
mod repair_fixture;
use repair_fixture::REBUILD_TASK;

use std::time::Duration;

use crate::http_server::{
    http_post_json_with_read_timeout, reserve_loopback_port, spawn_formal_ai_server_with_args,
};

/// End-to-end proof that issue #558's *rebuild-and-reattach loop* — recompiling Formal AI
/// and reattaching the improved WebAssembly worker to the UI once a change is accepted
/// (`R558-06`) — is reachable *through the agentic server*: an agentic CLI (`Codex` /
/// `OpenCode` / `Gemini` / `Agent CLI`) advertises its `write_file` + `bash` tools, asks
/// Formal AI to rebuild and reattach the improved worker, and the server drives the loop
/// by emitting a tool call that writes the generated rebuild-and-reattach plan. The plan
/// is ordered, observable, and reversible — nothing is rebuilt or restarted
/// automatically.
#[test]
fn agent_mode_routes_rebuild_to_a_write() {
    let port = reserve_loopback_port();
    let request = repair_fixture::canonical_change_request();
    let report = formal_ai::BenchmarkGateReport::issue_362_from_counts(4, 0);
    let receipt = request
        .review(
            &report,
            &formal_ai::HumanApproval::granted("integration-reviewer"),
        )
        .unwrap();
    let temporary = std::env::temp_dir().join(format!(
        "formal-ai-issue-558-rebuild-{}-{port}",
        std::process::id()
    ));
    std::fs::create_dir_all(&temporary).unwrap();
    let path = temporary.join("accepted-changes.json");
    std::fs::write(&path, serde_json::to_vec(&[&receipt]).unwrap()).unwrap();
    let _server = spawn_formal_ai_server_with_args(
        port,
        &["--agent-mode"],
        &[("FORMAL_AI_ACCEPTED_CHANGES", path.to_str().unwrap())],
    );

    for prompt in [
        REBUILD_TASK,
        "An improvement was accepted — reattach the improved worker to the UI and give me the steps.",
        "Rebuild the wasm worker and hot-swap the local server so the UI uses the accepted version.",
    ] {
        let prompt = repair_fixture::task_with_json(
            prompt,
            serde_json::json!({
                "change_id": receipt.change_id,
            }),
        );
        let response = http_post_json_with_read_timeout(
            port,
            "/api/openai/v1/chat/completions",
            Some("sk-local-agentic-tools"),
            &serde_json::json!({
                "model": "formal-ai",
                "stream": false,
                "messages": [{"role": "user", "content": prompt}],
                "tools": [
                    {
                        "type": "function",
                        "function": {
                            "name": "write_file",
                            "description": "Create or overwrite a file",
                            "parameters": {
                                "type": "object",
                                "properties": {
                                    "path": {"type": "string"},
                                    "content": {"type": "string"}
                                },
                                "required": ["path", "content"]
                            }
                        }
                    },
                    {
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
                    }
                ]
            }),
            // Composing the plan grounds the artifacts against the embedded repository,
            // which is cheap, but keep a generous window to match the sibling recipe
            // integration tests. Later responses are memoised.
            Duration::from_secs(90),
        );

        assert_eq!(
            response["choices"][0]["finish_reason"], "tool_calls",
            "{prompt}"
        );
        let call = &response["choices"][0]["message"]["tool_calls"][0];
        assert_eq!(call["function"]["name"], "write_file", "{prompt}");
        let arguments: serde_json::Value =
            serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
        assert_eq!(
            arguments["path"],
            formal_ai::agentic_coding::rebuild_plan::REBUILD_PATH,
            "{prompt}"
        );
        // The written document is the generated rebuild-and-reattach plan.
        let content = arguments["content"].as_str().unwrap();
        assert_eq!(
            content,
            formal_ai::agentic_coding::rebuild_plan::render_document(
                &formal_ai::RebuildPlan::for_accepted_change(&receipt)
            ),
            "{prompt}"
        );
        assert!(content.contains("rebuild_plan"), "{prompt}");
        assert!(content.contains("reattached_artifacts"), "{prompt}");
        assert!(
            content.contains("rebuild_and_reattach_pipeline"),
            "{prompt}"
        );
        assert!(content.contains("reversible"), "{prompt}");
        assert!(content.contains("integration-reviewer"), "{prompt}");
    }
    let _ = std::fs::remove_dir_all(&temporary);
}
