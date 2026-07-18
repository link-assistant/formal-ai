//! Review-gated auto-learning report for workspace rewrites (issue #715).
//!
//! The derivation lives in [`super::learning_report`]; this module is the
//! identity it renders under. Promotion remains gated on normal-algorithm laws,
//! multilingual structural-slot tests, and a real Agent CLI replay against
//! client-owned file bytes.

use super::learning_report::LearningReport;

pub const CODE_REWRITE_LEARNING_PATH: &str = "code-rewrite-learning-report.lino";
pub const CODE_REWRITE_LEARNING_TASK: &str = "Use Formal AI auto-learning to inspect the persisted issue 715 workspace-rewrite failures as an associative links network, rank the observations and normal-algorithm amendments, keep promotion human-review gated, and write code-rewrite-learning-report.lino.";

pub static REPORT: LearningReport = LearningReport {
    head: "code_rewrite_learning_report",
    issue: "715",
    promotion_gate: Some("normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass"),
    path: CODE_REWRITE_LEARNING_PATH,
    task: CODE_REWRITE_LEARNING_TASK,
    memory: include_str!("../../data/meta/issue-715-code-rewrite-learning.lino"),
    subject: "workspace-rewrite observations and amendments",
};

#[must_use]
pub fn is_code_rewrite_learning_task(prompt: &str) -> bool {
    REPORT.matches(prompt)
}

#[must_use]
pub fn render_document() -> String {
    REPORT.render_document()
}

/// Derive a review artifact from any persisted rewrite-observation network.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    REPORT.render_document_from(memory_document)
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    REPORT.final_answer(document)
}
