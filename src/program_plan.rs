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
//! becomes *data* — operation-vocabulary triggers plus a rule in the `.lino`
//! file — not new control flow. The whole transformation is inspectable as Links
//! Notation via
//! [`ProgramPlan::links_notation`].

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::substitution::{
    CrudEvent, LinkPattern, SubstitutionAction, SubstitutionGraph, SubstitutionRule,
    SubstitutionRuleSet, SubstitutionTraceReport,
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
        let mut set = SubstitutionRuleSet::from_links_notation(PROGRAM_PLAN_RULES_LINO)
            .expect("embedded program-plan rules must parse");
        let derived = derive_inverse_rules(
            &set.rules,
            &crate::seed::operation_vocabulary().inverse_pairs(),
        );
        set.rules.extend(derived);
        set.rules
            .sort_by(|left, right| left.order.cmp(&right.order).then(left.id.cmp(&right.id)));
        set
    })
}

/// Derive subtractive ("cancel") substitution rules from the additive base rules
/// plus the declared `(cancel_op, base_op)` inverse pairs.
///
/// This is the structural heart of the issue #386 architecture rethink: rather
/// than hand-writing a `request:task -> list_files_reverse_sort` ⇒ `list_files`
/// downgrade rule for every sorted variant, we *mirror* each additive rule. For
/// every base rule that fires on `request:modifier -> base_op` with a single-link
/// task rewrite, emit its inverse — fire on `request:modifier -> cancel_op` and
/// swap the rewrite's removed and added task links. "Cancel the sort" therefore
/// becomes the exact, automatically-maintained inverse of "sort", expressed as
/// data: a new cancellable operation needs only an `inverse` declaration in
/// `operation-vocabulary.lino`, never new control flow here.
fn derive_inverse_rules(
    base_rules: &[SubstitutionRule],
    inverse_pairs: &[(String, String)],
) -> Vec<SubstitutionRule> {
    let mut derived = Vec::new();
    for (cancel_op, base_op) in inverse_pairs {
        for rule in base_rules {
            // Only mirror a rule that fires on `request:modifier -> base_op`.
            let Some(condition_index) = rule.conditions.iter().position(|condition| {
                condition.literal_pair() == Some((MODIFIER_NODE, base_op.as_str()))
            }) else {
                continue;
            };
            // A well-defined inverse exists only for a single-link additive
            // rewrite (`remove one task link, add one task link`).
            let [action] = rule.actions.as_slice() else {
                continue;
            };
            let [added] = action.add.as_slice() else {
                continue;
            };
            // Keep every other condition; swap the cancelled modifier into place.
            let conditions = rule
                .conditions
                .iter()
                .enumerate()
                .map(|(index, condition)| {
                    if index == condition_index {
                        LinkPattern::parse(&format!("{MODIFIER_NODE} -> {cancel_op}"))
                            .expect("modifier condition pattern is well-formed")
                    } else {
                        condition.clone()
                    }
                })
                .collect();
            derived.push(SubstitutionRule {
                id: format!("{cancel_op}__{}", rule.id),
                order: rule.order,
                events: rule.events.clone(),
                conditions,
                actions: vec![SubstitutionAction {
                    remove: added.clone(),
                    add: vec![action.remove.clone()],
                }],
            });
        }
    }
    derived
}

/// Program-modifier slugs declared by the rule data.
///
/// A slug is considered a program modifier when a program-plan rule has a
/// literal `request:modifier -> <slug>` condition. Intent recognition combines
/// this set with `data/seed/operation-vocabulary.lino`, so adding a modifier is
/// seed data plus a substitution rule rather than a Rust allowlist entry.
#[must_use]
pub(crate) fn modifier_slugs() -> &'static BTreeSet<String> {
    static MODIFIER_SLUGS: OnceLock<BTreeSet<String>> = OnceLock::new();
    MODIFIER_SLUGS.get_or_init(|| {
        rules()
            .rules
            .iter()
            .flat_map(|rule| &rule.conditions)
            .filter_map(|condition| condition.literal_pair())
            .filter(|(from, _)| *from == MODIFIER_NODE)
            .map(|(_, to)| to.to_owned())
            .collect()
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
        // 4 additive base rules + 2 subtractive rules derived from the
        // cancel_reverse_sort↔reverse_sort inverse declaration (issue #386).
        assert_eq!(parsed.rules.len(), 6);
        assert_eq!(parsed.rules[0].id, "path_argument_list_files");
        let ids: BTreeSet<&str> = parsed.rules.iter().map(|rule| rule.id.as_str()).collect();
        assert!(ids.contains("cancel_reverse_sort__reverse_sort_list_files"));
        assert!(ids.contains("cancel_reverse_sort__reverse_sort_list_files_arg"));
    }

    #[test]
    fn modifier_slugs_are_discovered_from_rule_conditions() {
        let slugs = modifier_slugs();
        assert!(slugs.contains("path_argument"));
        assert!(slugs.contains("reverse_sort"));
        // The derived subtractive rules contribute the cancel modifier, so intent
        // recognition treats "cancel sort" as a first-class program modifier.
        assert!(slugs.contains("cancel_reverse_sort"));
    }

    #[test]
    fn cancel_reverse_sort_downgrades_sorted_path_argument_variant() {
        // Issue #386: turn 7 context task is `list_files_arg_reverse_sort`; the
        // cancel follow-up must remove the sort and fall back to `list_files_arg`.
        let plan = lower(
            "list_files_arg_reverse_sort",
            &modifiers(&["cancel_reverse_sort"]),
        );
        assert_eq!(plan.resolved_task, "list_files_arg");
        assert!(plan.was_modified());
        assert_eq!(plan.report.applied_count(), 1);
        assert!(plan.graph.contains_link(TASK_NODE, "list_files_arg"));
        assert!(!plan
            .graph
            .contains_link(TASK_NODE, "list_files_arg_reverse_sort"));
    }

    #[test]
    fn cancel_reverse_sort_downgrades_plain_sorted_variant() {
        let plan = lower(
            "list_files_reverse_sort",
            &modifiers(&["cancel_reverse_sort"]),
        );
        assert_eq!(plan.resolved_task, "list_files");
        assert!(plan.was_modified());
        assert_eq!(plan.report.applied_count(), 1);
    }

    #[test]
    fn cancel_reverse_sort_on_unsorted_task_is_noop() {
        // Cancelling a sort that was never applied leaves the task untouched.
        let plan = lower("list_files_arg", &modifiers(&["cancel_reverse_sort"]));
        assert_eq!(plan.resolved_task, "list_files_arg");
        assert!(!plan.was_modified());
        assert_eq!(plan.report.applied_count(), 0);
    }

    #[test]
    fn cancel_reverse_sort_is_the_exact_inverse_of_reverse_sort() {
        // Applying reverse_sort then cancel_reverse_sort round-trips back to the
        // original task — the derived rules are genuine inverses, not approximations.
        for base in ["list_files", "list_files_arg"] {
            let sorted = lower(base, &modifiers(&["reverse_sort"])).resolved_task;
            let restored = lower(&sorted, &modifiers(&["cancel_reverse_sort"])).resolved_task;
            assert_eq!(restored, base, "round-trip through {sorted} must restore {base}");
        }
    }

    #[test]
    fn derived_inverse_rules_are_pure_data_from_declarations() {
        // Proves R-h/R-m: a brand-new cancellable operation produces working
        // subtractive rules with no code change here — only an `inverse`
        // declaration plus the additive rule it mirrors.
        let base = SubstitutionRuleSet::from_links_notation(concat!(
            "substitution_rules\n",
            "  id \"demo\"\n",
            "  rule \"shout_greeting\"\n",
            "    order \"5\"\n",
            "    event \"manual\"\n",
            "    when \"request:modifier -> shout\"\n",
            "    replace \"request:task -> greeting\"\n",
            "      with \"request:task -> greeting_shout\"",
        ))
        .expect("demo rules parse");
        let derived = derive_inverse_rules(
            &base.rules,
            &[(String::from("calm"), String::from("shout"))],
        );
        assert_eq!(derived.len(), 1);
        let rule = &derived[0];
        assert_eq!(rule.id, "calm__shout_greeting");
        assert_eq!(rule.order, 5);
        assert_eq!(
            rule.conditions[0].literal_pair(),
            Some(("request:modifier", "calm"))
        );
        assert_eq!(
            rule.actions[0].remove.literal_pair(),
            Some(("request:task", "greeting_shout"))
        );
        assert_eq!(
            rule.actions[0].add[0].literal_pair(),
            Some(("request:task", "greeting"))
        );
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
    fn reverse_sort_upgrades_list_files() {
        let plan = lower("list_files", &modifiers(&["reverse_sort"]));
        assert_eq!(plan.resolved_task, "list_files_reverse_sort");
        assert!(plan.was_modified());
        assert_eq!(plan.report.applied_count(), 1);
        assert!(plan
            .graph
            .contains_link(TASK_NODE, "list_files_reverse_sort"));
        assert!(!plan.graph.contains_link(TASK_NODE, "list_files"));
    }

    #[test]
    fn path_argument_and_reverse_sort_compose() {
        let plan = lower("list_files", &modifiers(&["path_argument", "reverse_sort"]));
        assert_eq!(plan.resolved_task, "list_files_arg_reverse_sort");
        assert!(plan.was_modified());
        assert_eq!(plan.report.applied_count(), 2);
        assert!(plan
            .graph
            .contains_link(TASK_NODE, "list_files_arg_reverse_sort"));
        assert!(!plan.graph.contains_link(TASK_NODE, "list_files"));
    }

    #[test]
    fn reverse_sort_composes_with_existing_path_argument_variant() {
        let plan = lower("list_files_arg", &modifiers(&["reverse_sort"]));
        assert_eq!(plan.resolved_task, "list_files_arg_reverse_sort");
        assert!(plan.was_modified());
        assert_eq!(plan.report.applied_count(), 1);
    }

    #[test]
    fn path_argument_composes_with_existing_reverse_sort_variant() {
        let plan = lower("list_files_reverse_sort", &modifiers(&["path_argument"]));
        assert_eq!(plan.resolved_task, "list_files_arg_reverse_sort");
        assert!(plan.was_modified());
        assert_eq!(plan.report.applied_count(), 1);
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
        let plan = lower("list_files", &modifiers(&["path_argument", "reverse_sort"]));
        let notation = plan.links_notation();
        assert!(notation.contains("program_plan"));
        assert!(notation.contains("base_task list_files"));
        assert!(notation.contains("resolved_task list_files_arg_reverse_sort"));
        assert!(notation.contains("modifier path_argument"));
        assert!(notation.contains("modifier reverse_sort"));
        // The rewrite trace is embedded so the reasoning is inspectable.
        assert!(notation.contains("path_argument_list_files"));
        assert!(notation.contains("reverse_sort_list_files_arg"));
    }
}
