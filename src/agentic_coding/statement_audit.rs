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

/// The replayable operation for a workspace that explicitly names the
/// conventional external evidence capture file.
pub const STATEMENT_AUDIT_WITH_EVIDENCE_COMMAND: &str =
    "formal-ai statement-audit --root . --evidence evidence.json --output statement-audit.lino";

/// Select the public CLI operation requested by an audit task.
///
/// Evidence intake remains fail closed: only an explicit reference to the
/// fixed workspace-local `evidence.json` path opts into external captures.
#[must_use]
pub fn command_for(prompt: &str) -> &'static str {
    if prompt.to_ascii_lowercase().contains("evidence.json") {
        STATEMENT_AUDIT_WITH_EVIDENCE_COMMAND
    } else {
        STATEMENT_AUDIT_COMMAND
    }
}

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
