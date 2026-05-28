//! Issue #324 R4 — the program-modification step as a data-driven Links
//! Notation substitution pipeline.
//!
//! The long-horizon vision of issue #324 is *reason → plan in links → lower the
//! plan to Turing-complete substitution rules → compile to Rust/WASM →
//! execute*. This module implements the first runtime increment of that
//! pipeline: instead of hard-coding "a path-argument follow-up upgrades
//! `list_files` to `list_files_arg`" as a `match` arm, the upgrade is expressed
//! as a [`crate::substitution`] rule in [`data/seed/program-plan-rules.lino`].
//!
//! The flow is:
//!
//! 1. **Reason** — the intent formalizer extracts the base task plus any
//!    modifier slugs (e.g. `path_argument`) from the request prose.
//! 2. **Plan in Links Notation** — [`lower`] seeds a [`SubstitutionGraph`] with
//!    `request:task -> <base_task>` and one `request:modifier -> <slug>` link
//!    per detected modifier.
//! 3. **Lower via substitution rules** — the program-plan rule set is applied to
//!    a fixpoint by the same engine that powers the text-manipulation chain. A
//!    rule rewrites `request:task -> list_files` to `request:task ->
//!    list_files_arg` whenever `request:modifier -> path_argument` is present.
//! 4. **Compile / execute** — the resolved task slug feeds the existing template
//!    catalog (`program_spec`), which the engine renders and reports honestly.
//!
//! Adding a new modification (e.g. "sort descending", "count instead of list")
//! becomes *data* — a new rule in the `.lino` file — not new control flow. The
//! whole transformation is inspectable as Links Notation via
//! [`ProgramPlan::links_notation`].

use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::substitution::{
    CrudEvent, SubstitutionGraph, SubstitutionRuleSet, SubstitutionTraceReport,
};

/// Graph node naming the task currently planned.
pub const TASK_NODE: &str = "request:task";
/// Graph node naming a modifier requested over the task.
pub const MODIFIER_NODE: &str = "request:modifier";

/// The canonical program-plan substitution rules, in Links Notation.
pub const PROGRAM_PLAN_RULES_LINO: &str = crate::seed::PROGRAM_PLAN_RULES_LINO;

/// Parsed, cached program-plan rule set embedded at compile time.
///
/// Parsing once keeps the hot path (`write_program` formalization) cheap while
/// the rules themselves stay external data.
#[must_use]
pub fn rules() -> &'static SubstitutionRuleSet {
    static RULES: OnceLock<SubstitutionRuleSet> = OnceLock::new();
    RULES.get_or_init(|| {
        SubstitutionRuleSet::from_links_notation(PROGRAM_PLAN_RULES_LINO)
            .expect("embedded program-plan rules must parse")
    })
}

/// The result of lowering a `(base_task, modifiers)` request through the
/// substitution rules: an inspectable plan plus its rewrite trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramPlan {
    /// The task slug before any modification was applied.
    pub base_task: String,
    /// Modifier slugs detected in the request, in the order supplied.
    pub modifiers: Vec<String>,
    /// The task slug after the substitution rules reached a fixpoint.
    pub resolved_task: String,
    /// The final plan graph (links after rewriting).
    pub graph: SubstitutionGraph,
    /// The trace of every rule application performed.
    pub report: SubstitutionTraceReport,
}

impl ProgramPlan {
    /// `true` when a rule rewrote the task (the plan changed the base task).
    #[must_use]
    pub fn was_modified(&self) -> bool {
        self.resolved_task != self.base_task
    }

    /// Render the plan graph and its substitution trace as Links Notation so the
    /// solver can surface the reasoning transparently (issue #324 R6).
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        out.push_str("program_plan\n");
        let _ = writeln!(out, "  base_task {}", self.base_task);
        let _ = writeln!(out, "  resolved_task {}", self.resolved_task);
        for modifier in &self.modifiers {
            let _ = writeln!(out, "  modifier {modifier}");
        }
        for line in self.graph.links_notation().lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        for line in self.report.links_notation().lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        out.trim_end().to_owned()
    }
}

/// Lower a `(base_task, modifiers)` request using the embedded program-plan
/// rules.
#[must_use]
pub fn lower(base_task: &str, modifiers: &[String]) -> ProgramPlan {
    lower_with_rules(rules(), base_task, modifiers)
}

/// Lower a request through an explicit rule set.
///
/// Exposed so callers (and tests) can prove the pipeline is genuinely
/// data-driven: a new modifier rule changes behavior with no code change.
#[must_use]
pub fn lower_with_rules(
    rules: &SubstitutionRuleSet,
    base_task: &str,
    modifiers: &[String],
) -> ProgramPlan {
    let mut graph = SubstitutionGraph::new().with_link(TASK_NODE, base_task);
    for modifier in modifiers {
        graph.insert_link(MODIFIER_NODE, modifier);
    }
    let report = graph.apply_rules(rules, CrudEvent::Manual);
    let resolved_task = resolved_task_from_graph(&graph).unwrap_or_else(|| base_task.to_owned());
    ProgramPlan {
        base_task: base_task.to_owned(),
        modifiers: modifiers.to_vec(),
        resolved_task,
        graph,
        report,
    }
}

/// Convenience wrapper returning only the resolved task slug — a drop-in for the
/// former `path_argument_task_variant` hard-coded mapping.
#[must_use]
pub fn resolve_task(base_task: &str, modifiers: &[String]) -> String {
    lower(base_task, modifiers).resolved_task
}

fn resolved_task_from_graph(graph: &SubstitutionGraph) -> Option<String> {
    graph
        .links()
        .into_iter()
        .find(|link| link.from == TASK_NODE)
        .map(|link| link.to)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modifiers(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn embedded_rules_parse() {
        let parsed = rules();
        assert_eq!(parsed.id, "program_plan_rules");
        assert_eq!(parsed.rules.len(), 1);
        assert_eq!(parsed.rules[0].id, "path_argument_list_files");
    }

    #[test]
    fn path_argument_upgrades_list_files() {
        let plan = lower("list_files", &modifiers(&["path_argument"]));
        assert_eq!(plan.resolved_task, "list_files_arg");
        assert!(plan.was_modified());
        assert_eq!(plan.report.applied_count(), 1);
        assert!(plan.graph.contains_link(TASK_NODE, "list_files_arg"));
        assert!(!plan.graph.contains_link(TASK_NODE, "list_files"));
    }

    #[test]
    fn no_modifier_leaves_task_unchanged() {
        let plan = lower("list_files", &[]);
        assert_eq!(plan.resolved_task, "list_files");
        assert!(!plan.was_modified());
        assert_eq!(plan.report.applied_count(), 0);
    }

    #[test]
    fn path_argument_on_already_upgraded_task_is_idempotent() {
        // Matches the former `list_files_arg -> list_files_arg` behavior: the
        // rule only fires on the base task, so an already-upgraded task is left
        // untouched and the resolved slug stays stable.
        let plan = lower("list_files_arg", &modifiers(&["path_argument"]));
        assert_eq!(plan.resolved_task, "list_files_arg");
        assert!(!plan.was_modified());
    }

    #[test]
    fn unknown_task_with_modifier_is_unchanged() {
        let plan = lower("hello_world", &modifiers(&["path_argument"]));
        assert_eq!(plan.resolved_task, "hello_world");
        assert!(!plan.was_modified());
    }

    #[test]
    fn pipeline_is_data_driven() {
        // Proves the generality claim: a brand-new modifier/task-variant rewrite
        // is pure data — no code in this module changes to support it.
        let extra = concat!(
            "substitution_rules\n",
            "  id \"custom_rules\"\n",
            "  rule \"count_instead_of_list\"\n",
            "    order \"1\"\n",
            "    event \"manual\"\n",
            "    when \"request:modifier -> count_only\"\n",
            "    replace \"request:task -> list_files\"\n",
            "      with \"request:task -> count_files\"",
        );
        let custom = SubstitutionRuleSet::from_links_notation(extra).expect("custom rules parse");
        let plan = lower_with_rules(&custom, "list_files", &modifiers(&["count_only"]));
        assert_eq!(plan.resolved_task, "count_files");
        assert!(plan.was_modified());
    }

    #[test]
    fn links_notation_surfaces_plan_and_trace() {
        let plan = lower("list_files", &modifiers(&["path_argument"]));
        let notation = plan.links_notation();
        assert!(notation.contains("program_plan"));
        assert!(notation.contains("base_task list_files"));
        assert!(notation.contains("resolved_task list_files_arg"));
        assert!(notation.contains("modifier path_argument"));
        // The rewrite trace is embedded so the reasoning is inspectable.
        assert!(notation.contains("path_argument_list_files"));
    }
}
