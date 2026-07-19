//! Review-gated auto-learning report for the release self-hosting metric (issue #657).
//!
//! This report is what the generalization above is *for*: it is a new
//! auto-learning subject that cost a descriptor and a persisted network, with no
//! new renderer, no new planner branch, and no copy of the identity patch the
//! other reports used to carry.
//!
//! Its promotion gate is the metric's own: the ratchet only means something if
//! the fixture proves an exact share and the recorded baseline stays honest, so
//! a change to how authorship is attributed waits on both.

use super::LearningReport;

pub const SELF_HOSTING_LEARNING_PATH: &str = "self-hosting-learning-report.lino";
pub const SELF_HOSTING_LEARNING_TASK: &str = "Use Formal AI auto-learning to inspect the persisted issue 657 self-hosting attribution observations as an associative links network, rank the observations and attribution amendments, keep promotion human-review gated, and write self-hosting-learning-report.lino.";

pub static REPORT: LearningReport = LearningReport {
    head: "self_hosting_learning_report",
    issue: "657",
    promotion_gate: Some("metric_fixture_exact_share_and_honest_ledger_ratchet_pass"),
    path: SELF_HOSTING_LEARNING_PATH,
    task: SELF_HOSTING_LEARNING_TASK,
    memory: include_str!("../../../data/meta/issue-657-self-hosting-learning.lino"),
    subject: "self-hosting attribution observations and amendments",
};

#[must_use]
pub fn is_self_hosting_learning_task(prompt: &str) -> bool {
    REPORT.matches(prompt)
}

#[must_use]
pub fn render_document() -> String {
    REPORT.render_document()
}

/// Derive a review artifact from any persisted attribution-observation network.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    REPORT.render_document_from(memory_document)
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    REPORT.final_answer(document)
}
