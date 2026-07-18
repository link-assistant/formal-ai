//! One descriptor for every review-gated auto-learning report.
//!
//! Each of these reports is the same derivation: rank a persisted observation
//! network by the associative-memory adapter's four retention signals, keep the
//! evidence links, and hand a human a proposal rather than a promotion. Only the
//! *identity* of the report differs — which issue it answers, what it calls
//! itself, and which gate its promotion waits on.
//!
//! That identity used to be spliced in after the fact. The renderer hardcoded
//! issue #686's head and issue number, so every other report re-derived itself by
//! string-patching them back out:
//!
//! ```text
//! associative_learning::render_document_from(memory)
//!     .replacen("associative_learning_report\n", "code_rewrite_learning_report\n  issue \"715\"\n…", 1)
//!     .replacen("  issue \"686\"\n", "", 1)
//! ```
//!
//! Four modules carried a copy of that, which made the generic renderer generic
//! in name only: it could not name a report, so each caller had to un-name it
//! first. A fifth report meant a fifth copy of the hack, and the patch was
//! silent — `replacen` that matches nothing returns the string unchanged, so a
//! report could quietly keep claiming issue #686.
//!
//! A [`LearningReport`] carries that identity instead, and the shared renderer
//! emits it directly. Adding a report is now a row in [`REPORTS`], which is what
//! "the same derivation with a different identity" should have cost all along.

use std::fmt::Write as _;

use super::planner::{plan_document_recipe, AgenticPlan, DocumentRecipe};
use super::{associative_learning, code_rewrite_learning, execution_learning, routing_learning};
use crate::associative_persistence::AssociativeMemory;
use crate::links_format::format_lino_value_verbatim;
use crate::memory::parse_links_notation;
use crate::protocol::ChatMessage;

/// Every auto-learning report, in routing order.
///
/// A report is reachable because it is listed here, not because someone
/// remembered to add a branch to the planner.
pub static REPORTS: &[&LearningReport] = &[
    &associative_learning::REPORT,
    &routing_learning::REPORT,
    &code_rewrite_learning::REPORT,
    &execution_learning::REPORT,
    &self_hosting_learning::REPORT,
    &hardcoded_language_learning::REPORT,
];

pub mod hardcoded_language_learning;
pub mod self_hosting_learning;

/// Resolve the report a task asks for, if any.
#[must_use]
pub fn route(task: &str) -> Option<&'static LearningReport> {
    REPORTS.iter().copied().find(|report| report.matches(task))
}

/// The identity of one auto-learning report.
///
/// The derivation is shared; these fields are the whole of what varies.
pub struct LearningReport {
    /// The report's own head, e.g. `code_rewrite_learning_report`.
    pub head: &'static str,
    /// The issue the report answers.
    pub issue: &'static str,
    /// What promotion waits on, or `None` for a report that proposes no
    /// promotion. `Some` also carries the `awaiting_human_review` decision:
    /// a gate with no decision would state a condition and never apply it.
    pub promotion_gate: Option<&'static str>,
    /// The workspace artifact the agent is asked to write.
    pub path: &'static str,
    /// The task phrasing that routes to this report.
    pub task: &'static str,
    /// The persisted observation network the report ranks.
    pub memory: &'static str,
    /// How the final answer names what was ranked, e.g. `workspace-rewrite
    /// observations and amendments`.
    pub subject: &'static str,
}

impl LearningReport {
    /// Render this report from its own embedded memory.
    #[must_use]
    pub fn render_document(&self) -> String {
        self.render_document_from(self.memory)
    }

    /// Derive the review artifact from any persisted observation network.
    ///
    /// Taking the memory as an argument is what lets a test rank a fixture
    /// network through the production adapter rather than a stub of it.
    #[must_use]
    pub fn render_document_from(&self, memory_document: &str) -> String {
        render(self, memory_document)
    }

    /// The prompt routes here when it names the artifact it wants.
    #[must_use]
    pub fn matches(&self, prompt: &str) -> bool {
        prompt.to_lowercase().contains(self.path)
    }

    /// Plan the write-then-verify steps that produce this report.
    pub(super) fn plan_step(&self, messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
        let document = self.render_document();
        let final_answer = self.final_answer(&document);
        plan_document_recipe(
            messages,
            tool_names,
            DocumentRecipe {
                path: self.path,
                verify_command: format!("cat {}", self.path),
                final_answer,
                document,
            },
        )
    }

    /// Report what was ranked, and where the proposal is.
    #[must_use]
    pub fn final_answer(&self, document: &str) -> String {
        let expressions = document
            .lines()
            .filter(|line| line.trim_start().starts_with("learned_expression_"))
            .count();
        let Self { subject, path, .. } = self;
        if self.promotion_gate.is_some() {
            format!(
                "Formal AI ranked {expressions} {subject}; the human-review-gated report is in {path}."
            )
        } else {
            format!(
                "Formal AI persisted and ranked {expressions} {subject} through the associative auto-learning pipeline; the reproducible Links Notation report is in {path}."
            )
        }
    }
}

/// Derive an auditable learning report from a persisted memory document.
///
/// The ranking is computed by the same `AssociativeMemory::from_memory_events`
/// adapter that dreaming uses, so this exercises parsing, qualifier
/// preservation, alias/reference normalization, validation, multi-hop recall,
/// and four-signal retention rather than returning a canned checklist.
fn render(report: &LearningReport, memory_document: &str) -> String {
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

    let mut out = String::new();
    let _ = writeln!(out, "{}", report.head);
    field(&mut out, 2, "issue", report.issue);
    if let Some(gate) = report.promotion_gate {
        field(&mut out, 2, "decision", "awaiting_human_review");
        field(&mut out, 2, "promotion_gate", gate);
    }
    out.push_str("  record_type \"agent_cli_auto_learning\"\n");
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

/// Write one field the way Links Notation quotes.
///
/// This delegates rather than escaping in place: the notation *doubles* a
/// delimiter and has no backslash escape, so the C-style escape this used to
/// hand-roll wrote a value the grammar could not read — a report whose `text`
/// carried a quote, which is to say a report about real code, was rejected
/// outright.
///
/// The verbatim quoter is the right one here: nothing reads this report back
/// through the line-based `seed::parser`, so its values keep their newlines
/// instead of being flattened into escapes the grammar would hand back
/// literally.
fn field(out: &mut String, indent: usize, name: &str, value: &str) {
    let _ = writeln!(
        out,
        "{}{name} {}",
        " ".repeat(indent),
        format_lino_value_verbatim(value)
    );
}
