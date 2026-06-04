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
        assert_eq!(
            restored, base,
            "round-trip through {sorted} must restore {base}"
        );
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
