//! Reusable associative package and permission model tests.
//!
//! Issue #281 / R65 requires Deep.Foundation-inspired packages that can carry
//! handlers, permissions, and trigger-style computation as reviewable Links
//! Notation rather than one-off Rust edits.

use formal_ai::{
    compile_natural_language_skill, create_chat_completion_with_solver, handle_api_request,
    AssociativePackage, ChatCompletionRequest, PackagePermissionDecision, PackageStore,
    SolverConfig, UniversalSolver,
};
use lino_objects_codec::format::parse_indented;

const SKILL: &str = "When the user says `checksum status`, answer `checksum cache is valid.`";
const PERMISSIONED_PROCEDURE_SKILL: &str = r"
Skill `filesystem audit`
Input `path`: path
Tool `local_shell`
Permission `tool:local_shell`: user-approved shell runner
Step `run local_shell to list files`
Expected test `audit /tmp` -> `filesystem audit requires shell permission.`
";

#[test]
fn package_definition_exports_handlers_triggers_and_permissions() {
    let skill = compile_natural_language_skill(SKILL).expect("skill should compile");
    let package = AssociativePackage::from_compiled_skill(
        "pkg_checksum_skill",
        "Checksum skill package",
        "1.0.0",
        &skill,
    )
    .with_permission("tool:checksum_status", "compiled checksum status handler");

    let notation = package.links_notation();
    parse_indented(&notation).expect("package export must be valid Links Notation");
    assert!(notation.contains("type \"associative_package\""));
    assert!(notation.contains("handler"));
    assert!(notation.contains("trigger"));
    assert!(notation.contains("permission"));
    assert!(package
        .link_records()
        .iter()
        .any(|record| record.record_type == "PackagePermission"));
}

#[test]
fn package_store_validates_dependencies_before_install() {
    let base = AssociativePackage::new("pkg_base", "Base package", "1.0.0");
    let dependent = AssociativePackage::new("pkg_dependent", "Dependent package", "1.0.0")
        .with_dependency("pkg_base", "1.0.0");

    let mut store = PackageStore::new();
    let missing = store
        .install(dependent.clone())
        .expect_err("missing dependency must be rejected");
    assert!(missing.to_string().contains("pkg_base"));

    store.install(base).expect("base package installs");
    store
        .install(dependent)
        .expect("dependency should validate after base install");
}

#[test]
fn imported_compiled_skill_package_replays_without_rust_edits() {
    let skill = compile_natural_language_skill(SKILL).expect("skill should compile");
    let original = AssociativePackage::from_compiled_skill(
        "pkg_checksum_skill",
        "Checksum skill package",
        "1.0.0",
        &skill,
    );
    let exported = original.links_notation();
    let imported =
        AssociativePackage::from_links_notation(&exported).expect("exported package should import");

    let mut store = PackageStore::new();
    store.install(imported).expect("imported package installs");
    let replay = store
        .replay("Checksum status")
        .expect("imported package should replay");

    assert_eq!(replay.package_id, "pkg_checksum_skill");
    assert_eq!(replay.answer, "checksum cache is valid.");
    assert!(replay.trigger_id.starts_with("compiled_skill_rule_"));
    assert!(replay.handler_id.starts_with("compiled_skill_handler_"));
}

#[test]
fn generalized_compiled_skill_package_carries_generated_tests_and_permissions() {
    let skill =
        compile_natural_language_skill(PERMISSIONED_PROCEDURE_SKILL).expect("skill should compile");
    let package = AssociativePackage::from_compiled_skill(
        "pkg_filesystem_audit",
        "Filesystem audit package",
        "1.0.0",
        &skill,
    );
    let notation = package.links_notation();
    parse_indented(&notation).expect("package export must be valid Links Notation");

    assert!(package
        .permissions
        .iter()
        .any(|permission| permission.capability == "tool:local_shell"));
    assert!(notation.contains("permission"));
    assert!(notation.contains("tool:local_shell"));

    let replay = package
        .replay("Audit /tmp")
        .expect("generated expected test trigger should replay");
    assert_eq!(replay.answer, "filesystem audit requires shell permission.");
}

#[test]
fn permission_gate_denies_tools_without_package_grant() {
    let mut store = PackageStore::new();
    store
        .install(
            AssociativePackage::new("pkg_safe_math", "Safe math package", "1.0.0")
                .with_permission("tool:calculator", "local deterministic calculator"),
        )
        .expect("package installs");

    assert!(matches!(
        store.permission_for_tool("calculator"),
        PackagePermissionDecision::Allowed { .. }
    ));
    assert!(matches!(
        store.permission_for_tool("local_shell"),
        PackagePermissionDecision::Denied { .. }
    ));
}

#[test]
fn chat_tool_gate_requires_agent_mode_and_package_permission() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-symbolic-production",
        "messages": [{
            "role": "user",
            "content": "What is 2 + 2?"
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "calculator",
                "description": "Evaluate local deterministic math",
                "parameters": {"type": "object"}
            }
        }],
        "tool_choice": {
            "type": "function",
            "function": {"name": "calculator"}
        }
    }))
    .unwrap();

    let denied = create_chat_completion_with_solver(&request, &UniversalSolver::default());
    assert!(denied.choices[0]
        .message
        .content
        .plain_text()
        .contains("agent mode"));

    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let allowed = create_chat_completion_with_solver(&request, &solver);
    let body = allowed.choices[0].message.content.plain_text();
    assert!(body.contains('4'));
    assert!(!body.contains("not allowed"));

    let shell_request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-symbolic-production",
        "messages": [{
            "role": "user",
            "content": "List local files"
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "local_shell",
                "description": "Run a local shell command",
                "parameters": {"type": "object"}
            }
        }],
        "tool_choice": {
            "type": "function",
            "function": {"name": "local_shell"}
        }
    }))
    .unwrap();
    let denied_by_package = create_chat_completion_with_solver(&shell_request, &solver);
    let denial = denied_by_package.choices[0].message.content.plain_text();
    assert!(denial.contains("tool:local_shell"));
    assert!(denial.contains("associative package"));
}

#[test]
fn graph_endpoint_exposes_package_handler_trigger_and_permission_links() {
    let response = handle_api_request("GET", "/v1/graph", "");
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    let nodes = json["nodes"].as_array().expect("nodes array");
    let edges = json["edges"].as_array().expect("edges array");

    assert!(nodes.iter().any(|node| {
        node["id"] == "pkg_formal_ai_core"
            && node["links_notation"]
                .as_str()
                .is_some_and(|text| text.contains("associative_package"))
    }));
    for role in ["package_handler", "package_trigger", "package_permission"] {
        assert!(
            edges.iter().any(|edge| edge["role"] == role),
            "graph should expose {role} edge"
        );
    }
}
