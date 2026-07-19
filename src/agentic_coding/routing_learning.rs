//! Review-gated auto-learning report for tool-routing observations (issue #712).
//!
//! The derivation lives in [`super::learning_report`]; this module is the
//! identity it renders under. The output is a proposal for human review: it
//! never mutates the seed lexicon automatically.

use super::learning_report::LearningReport;

pub const ROUTING_LEARNING_PATH: &str = "tool-routing-learning-report.lino";
pub const ROUTING_LEARNING_TASK: &str = "Use Formal AI auto-learning to inspect the persisted issue 712 tool-routing failures as an associative links network, rank the observations and semantic-frame amendments, keep the result human-review gated, and write tool-routing-learning-report.lino.";

pub static REPORT: LearningReport = LearningReport {
    head: "tool_routing_learning_report",
    issue: "712",
    promotion_gate: Some("reported_matrix_and_unseen_paraphrases_pass"),
    path: ROUTING_LEARNING_PATH,
    task: ROUTING_LEARNING_TASK,
    memory: include_str!("../../data/meta/issue-712-routing-learning.lino"),
    subject: "routing observations and amendments",
};

#[must_use]
pub fn is_routing_learning_task(prompt: &str) -> bool {
    REPORT.matches(prompt)
}

#[must_use]
pub fn render_document() -> String {
    REPORT.render_document()
}

/// Derive a review artifact from any persisted routing-observation network.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    REPORT.render_document_from(memory_document)
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    REPORT.final_answer(document)
}
