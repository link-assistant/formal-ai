//! Agent-CLI execution artifact for issue #686's associative auto-learning loop.
//!
//! The derivation lives in [`super::learning_report`]; this module is the
//! identity it renders under. It is the one report that proposes no promotion:
//! it demonstrates the loop rather than asking for a change, so it carries no
//! gate and no decision.

use super::learning_report::LearningReport;

pub const ASSOCIATIVE_LEARNING_PATH: &str = "associative-learning-report.lino";
pub const ASSOCIATIVE_LEARNING_TASK: &str =
    "Use Formal AI auto-learning to inspect the persisted issue 686 memory as an associative links network, perform bounded multi-hop recall, rank expressions by reads, writes, incoming links, and outgoing links, retain validation warnings, and write associative-learning-report.lino.";

pub static REPORT: LearningReport = LearningReport {
    head: "associative_learning_report",
    issue: "686",
    promotion_gate: None,
    path: ASSOCIATIVE_LEARNING_PATH,
    task: ASSOCIATIVE_LEARNING_TASK,
    memory: include_str!("../../data/meta/associative-learning-case.lino"),
    subject: "expressions",
};

#[must_use]
pub fn is_associative_learning_task(prompt: &str) -> bool {
    REPORT.matches(prompt)
}

#[must_use]
pub fn render_document() -> String {
    REPORT.render_document()
}

/// Derive an auditable learning report from any persisted memory document.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    REPORT.render_document_from(memory_document)
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    REPORT.final_answer(document)
}
