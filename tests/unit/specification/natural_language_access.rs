//! E25: natural-language access to memory, APIs, and code execution.

use formal_ai::{SolverConfig, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::default().solve(prompt)
}

fn agent_answer(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
    .solve(prompt)
}

#[test]
fn natural_language_memory_query_reads_link_network() {
    let response = answer("What do you know about 'greeting'?");

    assert!(response.answer.contains("greeting"));
    assert!(response.answer.contains("intent"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "intent:concept_introspection_greeting"));
}

#[test]
fn natural_language_api_call_requires_agent_mode() {
    let response = answer("Call the calculator API with `2 + 2`");

    assert_eq!(response.intent, "tool_call_refused");
    assert!(response.answer.contains("Execution status: refused"));
    assert!(response.answer.contains("agent mode"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "policy:agent_mode_required_for_tools:tool:calculator"));
}

#[test]
fn natural_language_api_call_invokes_allowed_tool_and_records_trace() {
    let response = agent_answer("Call the calculator API with `2 + 2`");

    assert_eq!(response.intent, "natural_language_api_call");
    assert!(response.answer.contains("Execution status: executed"));
    assert!(response.answer.contains("Result: 4"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "tool_call:calculator"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("tool_parameter:expression=2:+:2")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "tool_result:4"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("tool_permission:allowed:tool:calculator")));
}

#[test]
fn natural_language_web_search_api_call_records_parameters_and_result() {
    let response = agent_answer("Call the web_search API with query `Rust ownership`");

    assert_eq!(response.intent, "natural_language_api_call");
    assert!(response.answer.contains("Execution status: executed"));
    assert!(response.answer.contains("Tool call: web_search"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "web_search:request:Rust ownership"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "tool_result:search_plan_recorded"));
}

#[test]
fn natural_language_code_execution_requires_agent_mode() {
    let response = answer("Please execute this javascript:\n```js\nconsole.log(1 + 2);\n```");

    assert_eq!(response.intent, "tool_call_refused");
    assert!(response.answer.contains("Execution status: refused"));
    assert!(response.answer.contains("tool:javascript_execution"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "policy:agent_mode_required_for_tools:tool:javascript_execution"));
}

#[test]
fn natural_language_code_execution_gate_is_stable_across_supported_language_contexts() {
    let prompts = [
        r#"language: "en" English
Please execute this javascript:
```js
console.log(1 + 2);
```"#,
        r#"language: "ru" Russian
Please execute this javascript:
```js
console.log(1 + 2);
```"#,
        r#"language: "hi" Hindi
Please execute this javascript:
```js
console.log(1 + 2);
```"#,
        r#"language: "zh" Chinese
Please execute this javascript:
```js
console.log(1 + 2);
```"#,
    ];

    for prompt in prompts {
        let response = answer(prompt);

        assert_eq!(response.intent, "tool_call_refused");
        assert!(response.answer.contains("Execution status: refused"));
        assert!(response
            .evidence_links
            .iter()
            .any(|link| link == "policy:agent_mode_required_for_tools:tool:javascript_execution"));
    }
}

#[test]
fn natural_language_code_execution_runs_bounded_javascript_when_allowed() {
    let response = agent_answer("Please execute this javascript:\n```js\nconsole.log(1 + 2);\n```");

    assert_eq!(response.intent, "javascript_execution");
    assert!(response.answer.contains("Execution status: executed"));
    assert!(response.answer.contains("Output: 3"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "tool_call:javascript_execution"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("tool_permission:allowed:tool:javascript_execution")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("execution_status:")));
}

#[test]
fn natural_language_tool_without_package_permission_is_refused() {
    let response = agent_answer("Call the local_shell tool with `ls`");

    assert_eq!(response.intent, "tool_call_refused");
    assert!(response.answer.contains("Execution status: refused"));
    assert!(response.answer.contains("tool:local_shell"));
    assert!(response.answer.contains("associative package"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "policy:package_permission_required:tool:local_shell"));
}
