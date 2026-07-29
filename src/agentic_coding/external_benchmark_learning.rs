//! Review-gated auto-learning report for real external benchmark failures
//! (issue #698).
//!
//! The shared learning renderer ranks persisted observations and amendments as
//! a links network. This module supplies only the report identity and initial
//! audited evidence; live benchmark runs use the same descriptor with memory
//! derived from their actual failed case outcomes.

use super::learning_report::LearningReport;

pub const EXTERNAL_BENCHMARK_LEARNING_PATH: &str = "external-benchmark-learning-report.lino";

/// Resolve the auto-learning replay task from the shared Links seed.
#[must_use]
pub fn task() -> String {
    crate::external_benchmarks::vocabulary::text("external_benchmark_learning_task")
}

pub static REPORT: LearningReport = LearningReport {
    head: "external_benchmark_learning_report",
    issue: "698",
    promotion_gate: Some("external_benchmark_ratchet_and_agent_cli_e2e_pass"),
    path: EXTERNAL_BENCHMARK_LEARNING_PATH,
    task: EXTERNAL_BENCHMARK_LEARNING_PATH,
    memory: include_str!("../../data/meta/issue-698-external-benchmark-learning.lino"),
    subject: "external benchmark failure observations and evaluator-boundary amendments",
};

#[must_use]
pub fn render_document() -> String {
    REPORT.render_document()
}

/// Derive a review artifact from any persisted benchmark-failure network.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    REPORT.render_document_from(memory_document)
}
