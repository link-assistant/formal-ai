//! Review-gated bulk lexeme import learning report (issue #660).

use super::LearningReport;

pub const LEXEME_IMPORT_LEARNING_PATH: &str = "lexeme-import-learning-report.lino";
pub const LEXEME_IMPORT_LEARNING_TASK: &str = LEXEME_IMPORT_LEARNING_PATH;

pub static REPORT: LearningReport = LearningReport {
    head: "lexeme_import_learning_report",
    issue: "660",
    promotion_gate: Some("bulk_lexeme_import_integrity_and_dual_agent_cli_e2e_pass"),
    path: LEXEME_IMPORT_LEARNING_PATH,
    task: LEXEME_IMPORT_LEARNING_TASK,
    memory: include_str!("../../../data/meta/issue-660-lexeme-import-learning.lino"),
    subject: "bulk-import observations and amendments",
};

#[must_use]
pub fn is_lexeme_import_learning_task(prompt: &str) -> bool {
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
