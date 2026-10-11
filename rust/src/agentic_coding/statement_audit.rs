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
pub fn command_for(prompt: &str) -> String {
    if let Some(frame) = audit_role_frame(prompt)
        && frame.explicit_output
    {
        return frame
            .output
            .filter(|_| frame.complete)
            .and_then(|output| registered_audit_command(&output))
            .unwrap_or_default();
    }
    if prompt.to_ascii_lowercase().contains("evidence.json") {
        STATEMENT_AUDIT_WITH_EVIDENCE_COMMAND.to_owned()
    } else {
        STATEMENT_AUDIT_COMMAND.to_owned()
    }
}

fn registered_audit_command(output: &str) -> Option<String> {
    if !super::write_request::safe_relative_path(output) {
        return None;
    }
    let document = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/seed/statement-audit-registry.lino"
    ));
    let roots = document
        .children
        .iter()
        .filter(|node| node.name == "statement_audit_registry")
        .collect::<Vec<_>>();
    let [root] = roots.as_slice() else {
        return None;
    };
    let contracts = root
        .children
        .iter()
        .filter(|node| node.name == "request-contract")
        .collect::<Vec<_>>();
    let [contract] = contracts.as_slice() else {
        return None;
    };
    if contract.id != "repository-audit" {
        return None;
    }
    let templates = contract
        .children
        .iter()
        .filter(|node| node.name == "command-template")
        .collect::<Vec<_>>();
    let [template] = templates.as_slice() else {
        return None;
    };
    if !template.children.is_empty() || template.id.matches("{output}").count() != 1 {
        return None;
    }
    Some(template.id.replace("{output}", output))
}

/// Whether a task requests the generalized repository statement audit.
#[must_use]
pub fn is_statement_audit_task(prompt: &str) -> bool {
    if let Some(frame) = audit_role_frame(prompt) {
        if frame.explicit_output {
            return frame.complete
                && frame
                    .output
                    .as_deref()
                    .is_some_and(|output| registered_audit_command(output).is_some());
        }
    }
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

/// Every clause must bind one distinct source-owned audit operation role.
pub(super) fn owned_statement_audit_output(prompt: &str) -> Option<String> {
    let frame = audit_role_frame(prompt)?;
    frame
        .output
        .filter(|value| frame.complete && super::write_request::safe_relative_path(value))
}

struct AuditRoleFrame {
    output: Option<String>,
    complete: bool,
    explicit_output: bool,
}

fn audit_role_frame(prompt: &str) -> Option<AuditRoleFrame> {
    if !crate::normal_markov::quoted_segment_spans(prompt).is_empty() {
        return None;
    }
    let document = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/seed/statement-audit-registry.lino"
    ));
    let roots = if document.name == "statement_audit_registry" {
        vec![&document]
    } else {
        document
            .children
            .iter()
            .filter(|node| node.name == "statement_audit_registry")
            .collect()
    };
    let [root] = roots.as_slice() else {
        return None;
    };
    let contracts = root
        .children
        .iter()
        .filter(|node| node.name == "request-contract")
        .collect::<Vec<_>>();
    let [contract] = contracts.as_slice() else {
        return None;
    };
    if contract.id != "repository-audit" {
        return None;
    }
    let roles = contract
        .children
        .iter()
        .filter(|node| node.name == "role")
        .collect::<Vec<_>>();
    let clauses = super::shell_command_policy::sentences(prompt);
    if roles.is_empty() {
        return None;
    }
    let mut complete = clauses.len() == roles.len();
    let mut seen = std::collections::BTreeSet::new();
    let mut output = None;
    for clause in clauses {
        let mut matches = Vec::new();
        for role in &roles {
            let patterns = role
                .children
                .iter()
                .filter(|node| node.name == "pattern")
                .collect::<Vec<_>>();
            let [pattern] = patterns.as_slice() else {
                return None;
            };
            let expression = regex::RegexBuilder::new(&pattern.id)
                .case_insensitive(true)
                .build()
                .ok()?;
            if let Some(matched) = expression.captures(clause.text) {
                if matched.get(0)?.as_str() == clause.text {
                    matches.push((
                        *role,
                        matched.get(1).map(|capture| capture.as_str().to_owned()),
                    ));
                }
            }
        }
        let [(role, value)] = matches.as_slice() else {
            complete = false;
            continue;
        };
        if !seen.insert(role.id.as_str()) {
            complete = false;
            continue;
        }
        if role.id == "deliver" {
            output.clone_from(value);
        }
    }
    Some(AuditRoleFrame {
        complete: complete && seen.len() == roles.len() && output.is_some(),
        explicit_output: seen.contains("audit")
            && (seen.contains("deliver") || (seen.contains("weigh") && seen.contains("persist"))),
        output,
    })
}
