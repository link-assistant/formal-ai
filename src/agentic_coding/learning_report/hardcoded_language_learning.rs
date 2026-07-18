//! Review-gated R379 learning report (issue #659).

use super::LearningReport;

pub const HARDCODED_LANGUAGE_LEARNING_PATH: &str = "hardcoded-language-learning-report.lino";
pub const HARDCODED_LANGUAGE_LEARNING_TASK: &str = HARDCODED_LANGUAGE_LEARNING_PATH;

pub static REPORT: LearningReport = LearningReport {
    head: "hardcoded_language_learning_report",
    issue: "659",
    promotion_gate: Some("hardcoded_language_fixture_context_gate_and_agent_cli_e2e_pass"),
    path: HARDCODED_LANGUAGE_LEARNING_PATH,
    task: HARDCODED_LANGUAGE_LEARNING_TASK,
    memory: include_str!("../../../data/meta/issue-659-hardcoded-language-learning.lino"),
    subject: "R379",
};

#[must_use]
pub fn is_hardcoded_language_learning_task(prompt: &str) -> bool {
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
