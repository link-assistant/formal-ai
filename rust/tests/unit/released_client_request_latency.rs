//! Pure planning regressions from public released-client failure evidence.
use formal_ai::AnthropicMessagesRequest;
use formal_ai::protocol::{
    ChatCompletionRequest, ResponsesRequest, create_chat_completion_with_solver_and_memory,
};
use formal_ai::solver::{SolverConfig, UniversalSolver};
use std::time::{Duration, Instant};

#[test]
fn released_client_requests_plan_without_consuming_the_delivery_budget() {
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    for (client, source, expected_tool, issue) in [
        (
            "agent",
            include_str!("../fixtures/released-client-requests/agent.json"),
            "bash",
            "1411",
        ),
        (
            "codex",
            include_str!("../fixtures/released-client-requests/codex.json"),
            "exec_command",
            "1413",
        ),
        (
            "claude",
            include_str!("../fixtures/released-client-requests/claude.json"),
            "Bash",
            "1415",
        ),
    ] {
        let request: ChatCompletionRequest = match client {
            "codex" => serde_json::from_str::<ResponsesRequest>(source)
                .unwrap()
                .to_chat_completion_request(),
            "claude" => serde_json::from_str::<AnthropicMessagesRequest>(source)
                .unwrap()
                .to_chat_completion_request(),
            _ => serde_json::from_str(source).unwrap(),
        };
        let started = Instant::now();
        let completion = create_chat_completion_with_solver_and_memory(&request, &solver, &[]);
        let elapsed = started.elapsed();
        eprintln!(
            "released-client request client={client} planning_ms={}",
            elapsed.as_millis()
        );
        let calls = &completion.choices[0].message.tool_calls;
        if client == "codex" {
            assert!(
                calls.is_empty(),
                "the redacted schema cannot authorize a call"
            );
            assert!(
                completion.choices[0]
                    .message
                    .content
                    .plain_text()
                    .contains("required tool arguments could not be bound")
            );
        } else {
            assert_eq!(calls.len(), 1, "{client}: {completion:?}");
            assert_eq!(calls[0].function.name, expected_tool);
            assert!(calls[0].function.arguments.contains(&format!(
                "https://github.com/link-assistant/formal-ai/issues/{issue}"
            )));
        }
        assert!(
            elapsed < Duration::from_secs(15),
            "{client} planning consumed {elapsed:?}; released Agent requests took about 50s each"
        );
    }
}
