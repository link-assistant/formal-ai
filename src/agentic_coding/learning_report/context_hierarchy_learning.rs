//! Review-gated auto-learning report for nested context resolution (issue #702).
//!
//! The persisted observations capture the failure modes that the generalized
//! hierarchy replaces: immediate-turn lookup, duplicated reverse scans, a
//! fixed inheritance depth, and an implicit outside-world boundary. The shared
//! learning renderer ranks those observations and the resulting amendments as
//! one associative links network. Promotion remains gated on the runtime and
//! Rust/browser parity fixtures.

use super::LearningReport;

pub const CONTEXT_HIERARCHY_LEARNING_PATH: &str = "context-hierarchy-learning-report.lino";
pub const CONTEXT_HIERARCHY_LEARNING_TASK: &str = CONTEXT_HIERARCHY_LEARNING_PATH;

pub static REPORT: LearningReport = LearningReport {
    head: "context_hierarchy_learning_report",
    issue: "702",
    promotion_gate: Some("nested_context_runtime_and_parity_fixtures_pass"),
    path: CONTEXT_HIERARCHY_LEARNING_PATH,
    task: CONTEXT_HIERARCHY_LEARNING_TASK,
    memory: include_str!("../../../data/meta/issue-702-context-hierarchy-learning.lino"),
    subject: "nested-context observations and hierarchy amendments",
};

#[must_use]
pub fn is_context_hierarchy_learning_task(prompt: &str) -> bool {
    REPORT.matches(prompt)
}

#[must_use]
pub fn render_document() -> String {
    REPORT.render_document()
}

#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    REPORT.render_document_from(memory_document)
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    REPORT.final_answer(document)
}
