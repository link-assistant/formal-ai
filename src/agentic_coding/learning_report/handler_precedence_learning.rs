//! Review-gated auto-learning report for specialized-handler precedence (issue #663).
//!
//! Issue #663 retired the `SPECIALIZED_HANDLERS` constant into
//! `data/seed/handler-precedence.lino`. This report is the auto-learning side of
//! that move: it observes the precedence rationale (`#395`, `#423`, `#425`,
//! `#552`, http_fetch-first, incompatible_units-last) as an associative links
//! network and ranks it into a human-review-gated proposal. Like every report in
//! [`super`], it never mutates the seed automatically — a reviewer adopts it.

use super::LearningReport;

pub const HANDLER_PRECEDENCE_LEARNING_PATH: &str = "handler-precedence-learning-report.lino";
pub const HANDLER_PRECEDENCE_LEARNING_TASK: &str = "Use Formal AI auto-learning to read the persisted issue 663 specialized-handler precedence rationale as an associative links network, rank the ordering observations and the precedence-is-data amendment, keep the result human-review gated, and write handler-precedence-learning-report.lino.";

pub static REPORT: LearningReport = LearningReport {
    head: "handler_precedence_learning_report",
    issue: "663",
    promotion_gate: Some("routing_precedence_from_seed_and_parity_fixture_pass"),
    path: HANDLER_PRECEDENCE_LEARNING_PATH,
    task: HANDLER_PRECEDENCE_LEARNING_TASK,
    memory: include_str!("../../../data/meta/issue-663-handler-precedence-learning.lino"),
    subject: "handler-precedence observations and the precedence-is-data amendment",
};

#[must_use]
pub fn is_handler_precedence_learning_task(prompt: &str) -> bool {
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
