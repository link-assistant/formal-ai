//! Agentic route for evidence-weighted repository statement audits.
//!
//! The planner does not synthesize an audit document. It asks the client-owned
//! shell tool to execute the same public CLI that a person can replay, then
//! consumes the command result on the following turn. This keeps Agent CLI
//! evidence on the production boundary rather than a planner-only shortcut.

/// The artifact produced in the agent workspace.
pub const STATEMENT_AUDIT_PATH: &str = "statement-audit.lino";

/// The exact public CLI operation delegated to the client shell.
pub const STATEMENT_AUDIT_COMMAND: &str =
    "formal-ai statement-audit --root . --output statement-audit.lino";

/// Whether a task requests the generalized repository statement audit.
#[must_use]
pub fn is_statement_audit_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    if lower.contains(STATEMENT_AUDIT_PATH) || lower.contains("statement audit") {
        return true;
    }
    let repository_scope = lower.contains("repository") || lower.contains("repo");
    let statement_scope = lower.contains("statements") || lower.contains("requirements");
    let assessment = lower.contains("audit")
        || lower.contains("probability")
        || lower.contains("probabilities")
        || lower.contains("weigh");
    repository_scope && statement_scope && assessment
}
