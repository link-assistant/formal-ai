//! Review-gated auto-learning report for client-owned execution (issue #716).
//!
//! The production associative-memory adapter ranks persisted observations and
//! their evidence-linked amendments. The resulting artifact is intentionally a
//! proposal: promotion remains gated on protocol, presentation-variation, and
//! real Agent CLI checks.

use super::associative_learning;
use super::planner::{plan_document_recipe, AgenticPlan, DocumentRecipe};
use crate::protocol::ChatMessage;

pub const EXECUTION_LEARNING_PATH: &str = "client-execution-learning-report.lino";
pub const EXECUTION_LEARNING_TASK: &str = "Use Formal AI auto-learning to inspect the persisted issue 716 client-execution failures as an associative links network, rank the observations and architectural amendments, keep promotion human-review gated, and write client-execution-learning-report.lino.";

const EXECUTION_MEMORY: &str = include_str!("../../data/meta/issue-716-execution-learning.lino");

#[must_use]
pub fn is_execution_learning_task(prompt: &str) -> bool {
    prompt.to_lowercase().contains(EXECUTION_LEARNING_PATH)
}

pub(super) fn plan_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = render_document();
    let final_answer = final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: EXECUTION_LEARNING_PATH,
            document,
            verify_command: format!("cat {EXECUTION_LEARNING_PATH}"),
            final_answer,
        },
    )
}

#[must_use]
pub fn render_document() -> String {
    render_document_from(EXECUTION_MEMORY)
}

/// Derive a review artifact from any persisted execution-observation network.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    associative_learning::render_document_from(memory_document)
        .replacen(
            "associative_learning_report\n",
            concat!(
                "client_execution_learning_report\n",
                "  issue \"716\"\n",
                "  decision \"awaiting_human_review\"\n",
                "  promotion_gate \"protocol_matrix_presentation_variations_and_agent_cli_e2e_pass\"\n",
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
        "Formal AI ranked {expressions} client-execution observations and amendments; the human-review-gated report is in {EXECUTION_LEARNING_PATH}."
    )
}
