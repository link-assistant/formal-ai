//! Review-gated auto-learning report for client-owned execution (issue #716).
//!
//! The derivation lives in [`super::learning_report`]; this module is the
//! identity it renders under. The artifact is intentionally a proposal:
//! promotion remains gated on protocol, presentation-variation, and real Agent
//! CLI checks.

use super::learning_report::LearningReport;

pub const EXECUTION_LEARNING_PATH: &str = "client-execution-learning-report.lino";
pub const EXECUTION_LEARNING_TASK: &str = "Use Formal AI auto-learning to inspect the persisted issue 716 client-execution failures as an associative links network, rank the observations and architectural amendments, keep promotion human-review gated, and write client-execution-learning-report.lino.";

pub static REPORT: LearningReport = LearningReport {
    head: "client_execution_learning_report",
    issue: "716",
    promotion_gate: Some("protocol_matrix_presentation_variations_and_agent_cli_e2e_pass"),
    path: EXECUTION_LEARNING_PATH,
    task: EXECUTION_LEARNING_TASK,
    memory: include_str!("../../data/meta/issue-716-execution-learning.lino"),
    subject: "client-execution observations and amendments",
};

#[must_use]
pub fn is_execution_learning_task(prompt: &str) -> bool {
    REPORT.matches(prompt)
}

#[must_use]
pub fn render_document() -> String {
    REPORT.render_document()
}

/// Derive a review artifact from any persisted execution-observation network.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    REPORT.render_document_from(memory_document)
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    REPORT.final_answer(document)
}
