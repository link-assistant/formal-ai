//! Review-gated auto-learning report for tool-routing observations (issue #712).
//!
//! The report is derived through the production associative-memory adapter. It
//! ranks persisted observations and amendments by actual usage, mutation, and
//! link degree, while preserving their evidence links. The output is a proposal
//! for human review: it never mutates the seed lexicon automatically.

use super::associative_learning;

pub const ROUTING_LEARNING_PATH: &str = "tool-routing-learning-report.lino";
pub const ROUTING_LEARNING_TASK: &str = "Use Formal AI auto-learning to inspect the persisted issue 712 tool-routing failures as an associative links network, rank the observations and semantic-frame amendments, keep the result human-review gated, and write tool-routing-learning-report.lino.";

const ROUTING_MEMORY: &str = include_str!("../../data/meta/issue-712-routing-learning.lino");

#[must_use]
pub fn is_routing_learning_task(prompt: &str) -> bool {
    prompt.to_lowercase().contains(ROUTING_LEARNING_PATH)
}

#[must_use]
pub fn render_document() -> String {
    render_document_from(ROUTING_MEMORY)
}

/// Derive a review artifact from any persisted routing-observation network.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    let learned = associative_learning::render_document_from(memory_document);
    learned
        .replacen(
            "associative_learning_report\n",
            concat!(
                "tool_routing_learning_report\n",
                "  issue \"712\"\n",
                "  decision \"awaiting_human_review\"\n",
                "  promotion_gate \"reported_matrix_and_unseen_paraphrases_pass\"\n",
            ),
            1,
        )
        .replacen("  issue \"686\"\n", "", 1)
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    let expressions = document
        .lines()
        .filter(|line| line.trim_start().starts_with("learned_expression_"))
        .count();
    format!(
        "Formal AI ranked {expressions} routing observations and amendments; the human-review-gated report is in {ROUTING_LEARNING_PATH}."
    )
}
