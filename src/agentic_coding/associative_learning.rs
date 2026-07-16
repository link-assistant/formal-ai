//! Agent-CLI execution artifact for issue #686's associative auto-learning loop.
//!
//! The report is computed from a persisted `demo_memory` document by the same
//! `AssociativeMemory::from_memory_events` adapter used by dreaming. It therefore
//! exercises parsing, qualifier preservation, alias/reference normalization,
//! validation, multi-hop recall, and four-signal retention instead of returning a
//! canned checklist.

use std::fmt::Write as _;

use super::planner::{plan_document_recipe, AgenticPlan, DocumentRecipe};
use crate::associative_persistence::AssociativeMemory;
use crate::memory::parse_links_notation;
use crate::protocol::ChatMessage;

pub const ASSOCIATIVE_LEARNING_PATH: &str = "associative-learning-report.lino";
pub const ASSOCIATIVE_LEARNING_TASK: &str =
    "Use Formal AI auto-learning to inspect the persisted issue 686 memory as an associative links network, perform bounded multi-hop recall, rank expressions by reads, writes, incoming links, and outgoing links, retain validation warnings, and write associative-learning-report.lino.";

const EMBEDDED_CASE: &str = include_str!("../../data/meta/associative-learning-case.lino");

#[must_use]
pub fn is_associative_learning_task(prompt: &str) -> bool {
    prompt.to_lowercase().contains(ASSOCIATIVE_LEARNING_PATH)
}

pub(super) fn plan_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = render_document();
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: ASSOCIATIVE_LEARNING_PATH,
            verify_command: format!("cat {ASSOCIATIVE_LEARNING_PATH}"),
            final_answer: final_answer(&document),
            document,
        },
    )
}

#[must_use]
pub fn render_document() -> String {
    render_document_from(EMBEDDED_CASE)
}

/// Derive an auditable learning report from any persisted memory document.
#[must_use]
pub fn render_document_from(memory_document: &str) -> String {
    let events = parse_links_notation(memory_document);
    let mut memory = AssociativeMemory::from_memory_events(&events);
    let seed = memory
        .retention_ranking()
        .first()
        .cloned()
        .unwrap_or_default();
    let recalled = memory.recall_related(&seed, 2);
    let ranking = memory.retention_ranking();
    let warning_count = memory
        .expressions()
        .values()
        .map(|expression| expression.validation_issues.len())
        .sum::<usize>();

    let mut out = String::from("associative_learning_report\n");
    out.push_str("  record_type \"agent_cli_auto_learning\"\n");
    out.push_str("  issue \"686\"\n");
    out.push_str("  substrate \"links_network\"\n");
    out.push_str("  retention_formula \"reads + writes + incoming_links + outgoing_links\"\n");
    let _ = writeln!(out, "  expression_count \"{}\"", memory.len());
    let _ = writeln!(out, "  validation_warning_count \"{warning_count}\"");
    field(&mut out, 2, "multi_hop_seed", &seed);
    field(&mut out, 2, "multi_hop_recall", &recalled.join("|"));
    for (index, id) in ranking.iter().enumerate() {
        let Some(expression) = memory.get(id) else {
            continue;
        };
        let _ = writeln!(out, "  learned_expression_{:02}", index + 1);
        field(&mut out, 4, "id", id);
        field(&mut out, 4, "text", &expression.text);
        field(&mut out, 4, "reads", &expression.reads.to_string());
        field(&mut out, 4, "writes", &expression.writes.to_string());
        field(
            &mut out,
            4,
            "incoming_links",
            &memory.in_degree(id).to_string(),
        );
        field(
            &mut out,
            4,
            "outgoing_links",
            &memory.out_degree(id).to_string(),
        );
        field(
            &mut out,
            4,
            "retention_score",
            &memory.retention_score(id).to_string(),
        );
        field(
            &mut out,
            4,
            "qualifiers",
            &expression
                .qualifiers
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("|"),
        );
        field(
            &mut out,
            4,
            "validation",
            if expression.validation_issues.is_empty() {
                "aligned"
            } else {
                "retained_with_warning"
            },
        );
    }
    out
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    let expressions = document
        .lines()
        .filter(|line| line.trim_start().starts_with("learned_expression_"))
        .count();
    format!(
        "Formal AI persisted and ranked {expressions} expressions through the associative auto-learning pipeline; the reproducible Links Notation report is in {ASSOCIATIVE_LEARNING_PATH}."
    )
}

fn field(out: &mut String, indent: usize, name: &str, value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let _ = writeln!(out, "{}{name} \"{escaped}\"", " ".repeat(indent));
}
