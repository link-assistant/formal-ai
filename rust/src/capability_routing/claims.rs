//! Claim routing (issue #1175 R3): which structural evidence a handler's claim
//! needs, consulted before the handler runs.
//!
//! The `claim` rows of `data/seed/capability-routing.lino` name a handler (the
//! native dispatch name, and the browser worker's function as
//! `browser_handler`) and the evidence kinds any one of which admits it. The
//! router reads the prompt's structure through the evidence predicate each
//! kind names — the verb's object phrase, a parse as command arguments, a
//! repository subject — and a handler whose row admits on none of them is not
//! offered the prompt at all, so it cannot claim on a surface word. A handler
//! with no row is admitted as before; the rows are where the issue #1175
//! classes of misroute live, and new rows are added there, not in code.
//!
//! Mirrored by `claimRouteAdmits` in `js/worker/formal_ai_worker_dispatch.js`.

use std::sync::OnceLock;

use crate::seed::parser::parse_lino;

/// One `claim` row: a handler and the evidence kinds that admit it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRow {
    pub handler: String,
    pub browser_handler: String,
    pub admits_on: Vec<String>,
    pub because: String,
}

/// The evidence kinds the router can read, in the order the code knows them.
///
/// A row naming a kind outside this list is a seed error the unit suite
/// reports, never a silent admission.
pub const CLAIM_EVIDENCE_KINDS: &[&str] = &[
    "object_phrase_artifact",
    "approval_of_a_proposal",
    "shell_command_shape",
    "semantic_shell_task",
    "repository_subject",
];

/// Parse the `claim` rows of a capability-routing document.
#[must_use]
pub fn claim_rows_from(text: &str) -> Vec<ClaimRow> {
    let tree = parse_lino(text);
    let mut rows = Vec::new();
    for document in &tree.children {
        for record in &document.children {
            if record.name != "claim" {
                continue;
            }
            rows.push(ClaimRow {
                handler: record.find_child_value("handler").to_owned(),
                browser_handler: record.find_child_value("browser_handler").to_owned(),
                admits_on: record
                    .children
                    .iter()
                    .filter(|child| child.name == "admits_on" && !child.id.is_empty())
                    .map(|child| child.id.clone())
                    .collect(),
                because: record.find_child_value("because").to_owned(),
            });
        }
    }
    rows
}

/// The shipped `claim` rows.
#[must_use]
pub fn claim_rows() -> &'static [ClaimRow] {
    static ROWS: OnceLock<Vec<ClaimRow>> = OnceLock::new();
    ROWS.get_or_init(|| claim_rows_from(super::CAPABILITY_ROUTING_LINO))
}

/// Whether the prompt carries evidence of `kind`; `None` for an unknown kind.
#[must_use]
pub fn claim_evidence_holds(kind: &str, prompt: &str, normalized: &str) -> Option<bool> {
    let canonical = crate::engine::normalize_prompt(prompt);
    let canonical = if canonical.is_empty() {
        normalized
    } else {
        canonical.as_str()
    };
    let holds = match kind {
        "object_phrase_artifact" => crate::solver_handlers::software_project_claims(canonical),
        "approval_of_a_proposal" => {
            crate::solver_handlers::software_project_approval_claims(canonical)
        }
        "shell_command_shape" => crate::solver_terminal::names_terminal_command(prompt),
        "semantic_shell_task" => {
            crate::agentic_coding::semantic_shell_command_for_task(prompt).is_some()
        }
        "repository_subject" => {
            let rules = crate::history_context::HistoryRules::load(None);
            !crate::history_context::lineage_subjects(prompt, &rules).is_empty()
                || crate::history_context::status_subject(prompt, &rules).is_some()
                || !crate::history_context::definition_subjects(prompt, &rules).is_empty()
        }
        _ => return None,
    };
    Some(holds)
}

/// Whether the router offers `prompt` to `handler`.
///
/// A handler with no `claim` row is admitted; a handler with one is admitted
/// when any evidence kind its row lists holds.
#[must_use]
pub fn claim_admitted(handler: &str, prompt: &str, normalized: &str) -> bool {
    let Some(row) = claim_rows().iter().find(|row| row.handler == handler) else {
        return true;
    };
    row.admits_on
        .iter()
        .any(|kind| claim_evidence_holds(kind, prompt, normalized) == Some(true))
}
