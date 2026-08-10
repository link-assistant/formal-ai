//! Review-gated associative report for statement-level search fusion (#709).

use super::LearningReport;

pub const SEARCH_FUSION_LEARNING_PATH: &str = "search-fusion-learning-report.lino";
pub const SEARCH_FUSION_LEARNING_TASK: &str = SEARCH_FUSION_LEARNING_PATH;

pub static REPORT: LearningReport = LearningReport {
    head: "search_fusion_learning_report",
    issue: "709",
    promotion_gate: Some("issue_709_held_out_zero_failures_and_named_review"),
    path: SEARCH_FUSION_LEARNING_PATH,
    task: SEARCH_FUSION_LEARNING_TASK,
    memory: include_str!("../../../data/meta/issue-709-search-fusion-learning.lino"),
    subject: "search-fusion execution observations and reusable corrections",
};

#[must_use]
pub fn is_search_fusion_learning_task(prompt: &str) -> bool {
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
