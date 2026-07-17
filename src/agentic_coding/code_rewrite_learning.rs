//! Review-gated auto-learning report for workspace rewrites (issue #715).
//!
//! The production associative-memory adapter ranks persisted failure evidence
//! and linked architectural amendments. Promotion remains gated on normal-
//! algorithm laws, multilingual structural-slot tests, and a real Agent CLI
//! replay against client-owned file bytes.

use super::associative_learning;
use super::planner::{plan_document_recipe, AgenticPlan, DocumentRecipe};
use crate::protocol::ChatMessage;

pub const CODE_REWRITE_LEARNING_PATH: &str = "code-rewrite-learning-report.lino";
pub const CODE_REWRITE_LEARNING_TASK: &str = "Use Formal AI auto-learning to inspect the persisted issue 715 workspace-rewrite failures as an associative links network, rank the observations and normal-algorithm amendments, keep promotion human-review gated, and write code-rewrite-learning-report.lino.";

const REWRITE_MEMORY: &str = include_str!("../../data/meta/issue-715-code-rewrite-learning.lino");

#[must_use]
pub fn is_code_rewrite_learning_task(prompt: &str) -> bool {
    prompt.to_lowercase().contains(CODE_REWRITE_LEARNING_PATH)
}

pub(super) fn plan_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = render_document();
    let final_answer = final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: CODE_REWRITE_LEARNING_PATH,
            document,
            verify_command: format!("cat {CODE_REWRITE_LEARNING_PATH}"),
            final_answer,
        },
    )
}

#[must_use]
pub fn render_document() -> String {
    render_document_from(REWRITE_MEMORY)
}

/// Derive a review artifact from any persisted rewrite-observation network.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    associative_learning::render_document_from(memory_document)
        .replacen(
            "associative_learning_report\n",
            concat!(
                "code_rewrite_learning_report\n",
                "  issue \"715\"\n",
                "  decision \"awaiting_human_review\"\n",
                "  promotion_gate \"normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass\"\n",
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
        "Formal AI ranked {expressions} workspace-rewrite observations and amendments; the human-review-gated report is in {CODE_REWRITE_LEARNING_PATH}."
    )
}
