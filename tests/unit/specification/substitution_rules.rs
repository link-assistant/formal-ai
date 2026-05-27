//! Link-pattern substitution rule tests.
//!
//! Issue #301 requires a link-cli-style `replace x y` primitive over link
//! patterns, composable `when ... do ...` rules, CRUD event attachment, and
//! deterministic trace links for every applied rule.

use formal_ai::{CrudEvent, FormalAiEngine, SubstitutionGraph, SubstitutionRuleSet};
use lino_objects_codec::format::parse_indented;

#[test]
fn single_replace_operates_over_link_patterns_with_variables() {
    let rules = SubstitutionRuleSet::from_links_notation(
        r#"
substitution_rules
  id "issue_301_single_replace"
  rule "rule_promote_kind"
    order "10"
    event "manual"
    replace "$node -> kind:cat"
      with "$node -> kind:mammal"
"#,
    )
    .expect("rule data should parse");
    let mut graph = SubstitutionGraph::new().with_link("entity:ada", "kind:cat");

    let report = graph.apply_rules(&rules, CrudEvent::Manual);

    assert!(!graph.contains_link("entity:ada", "kind:cat"));
    assert!(graph.contains_link("entity:ada", "kind:mammal"));
    assert_eq!(report.traces.len(), 1);
    assert_eq!(report.traces[0].rule_id, "rule_promote_kind");
    assert_eq!(
        report.traces[0].bindings.get("node").map(String::as_str),
        Some("entity:ada")
    );
    parse_indented(&rules.links_notation()).expect("rules must export valid Links Notation");
    parse_indented(&report.links_notation())
        .expect("trace report must export valid Links Notation");
}

#[test]
fn when_do_rules_compose_conditions_before_replacement() {
    let rules = SubstitutionRuleSet::from_links_notation(
        r#"
substitution_rules
  id "issue_301_when_do"
  rule "rule_open_task_ready"
    order "1"
    event "manual"
    when "$task -> kind:task"
    when "$task -> assignee:$person"
    replace "$task -> state:open"
      with "$task -> state:ready"
      with "$task -> routed_to:$person"
"#,
    )
    .expect("conditional rule data should parse");
    let mut graph = SubstitutionGraph::new()
        .with_link("task:7", "kind:task")
        .with_link("task:7", "assignee:ada")
        .with_link("task:7", "state:open");

    let report = graph.apply_rules(&rules, CrudEvent::Manual);

    assert!(graph.contains_link("task:7", "state:ready"));
    assert!(graph.contains_link("task:7", "routed_to:ada"));
    assert!(!graph.contains_link("task:7", "state:open"));
    assert_eq!(
        report.traces[0].bindings.get("person").map(String::as_str),
        Some("ada")
    );
}

#[test]
fn crud_attached_rules_fire_on_matching_event_and_emit_trace_links() {
    let rules = SubstitutionRuleSet::from_links_notation(
        r#"
substitution_rules
  id "issue_301_crud_attached"
  rule "rule_created_prompt_answer"
    order "1"
    event "create"
    when "$turn -> prompt:Hi"
    replace "$turn -> response:missing"
      with "$turn -> intent:greeting"
      with "$turn -> answer:Hi, how may I help you?"
"#,
    )
    .expect("CRUD-attached rule data should parse");
    let mut graph = SubstitutionGraph::new().with_link("turn:1", "response:missing");

    let report = graph.create_link("turn:1", "prompt:Hi", &rules);

    assert!(graph.contains_link("turn:1", "intent:greeting"));
    assert!(graph.contains_link("turn:1", "answer:Hi, how may I help you?"));
    assert!(!graph.contains_link("turn:1", "response:missing"));
    assert_eq!(report.traces.len(), 1);
    assert!(report
        .trace_link_records()
        .iter()
        .any(|record| record.record_type == "SubstitutionTraceLink"));
    assert!(report
        .links_notation()
        .contains("rule_id \"rule_created_prompt_answer\""));

    let lifecycle_rules = SubstitutionRuleSet::from_links_notation(
        r#"
substitution_rules
  id "issue_301_crud_lifecycle"
  rule "rule_read_records_observation"
    order "1"
    event "read"
    when "$task -> audit:updated"
    replace "$task -> observed:missing"
      with "$task -> observed:read"
  rule "rule_update_records_audit"
    order "2"
    event "update"
    when "$task -> state:open"
    replace "$task -> audit:missing"
      with "$task -> audit:updated"
  rule "rule_delete_records_tombstone"
    order "3"
    event "delete"
    when "$task -> watch:yes"
    replace "$task -> tombstone:missing"
      with "$task -> tombstone:deleted"
"#,
    )
    .expect("CRUD lifecycle rule data should parse");
    let mut lifecycle = SubstitutionGraph::new()
        .with_link("task:1", "state:draft")
        .with_link("task:1", "audit:missing")
        .with_link("task:1", "observed:missing")
        .with_link("task:1", "watch:yes")
        .with_link("task:1", "tombstone:missing");

    let update_report = lifecycle.update_link(
        "task:1",
        "state:draft",
        "task:1",
        "state:open",
        &lifecycle_rules,
    );
    let (exists, read_report) = lifecycle.read_link("task:1", "audit:updated", &lifecycle_rules);
    let delete_report = lifecycle.delete_link("task:1", "state:open", &lifecycle_rules);

    assert!(exists);
    assert!(lifecycle.contains_link("task:1", "audit:updated"));
    assert!(lifecycle.contains_link("task:1", "observed:read"));
    assert!(lifecycle.contains_link("task:1", "tombstone:deleted"));
    assert_eq!(update_report.traces[0].event, CrudEvent::Update);
    assert_eq!(read_report.traces[0].event, CrudEvent::Read);
    assert_eq!(delete_report.traces[0].event, CrudEvent::Delete);
}

#[test]
fn deterministic_rule_order_and_termination_guard_are_visible() {
    let rules = SubstitutionRuleSet::from_links_notation(
        r#"
substitution_rules
  id "issue_301_determinism"
  rule "rule_a_to_b"
    order "1"
    event "manual"
    replace "$node -> flag:a"
      with "$node -> flag:b"
  rule "rule_b_to_a"
    order "2"
    event "manual"
    replace "$node -> flag:b"
      with "$node -> flag:a"
"#,
    )
    .expect("looping rule data should parse");
    let mut first = SubstitutionGraph::new().with_link("node:1", "flag:a");
    let mut second = SubstitutionGraph::new().with_link("node:1", "flag:a");

    let first_report = first.apply_rules_with_limit(&rules, CrudEvent::Manual, 3);
    let second_report = second.apply_rules_with_limit(&rules, CrudEvent::Manual, 3);

    assert_eq!(first.links(), second.links());
    assert_eq!(
        first_report
            .traces
            .iter()
            .map(|trace| trace.rule_id.as_str())
            .collect::<Vec<_>>(),
        vec!["rule_a_to_b", "rule_b_to_a", "rule_a_to_b"]
    );
    assert_eq!(
        first_report.traces, second_report.traces,
        "same input graph and ordered rules must produce the same trace"
    );
    assert!(first_report.terminated_by_guard);
    assert_eq!(first_report.applied_count(), 3);
}

#[test]
fn rust_handler_behavior_can_be_expressed_as_substitution_rule_data() {
    let rules = SubstitutionRuleSet::from_links_notation(
        r#"
substitution_rules
  id "issue_301_behavior_as_data"
  rule "rule_greeting_as_data"
    order "1"
    event "create"
    when "$turn -> role:user"
    when "$turn -> prompt:Hi"
    replace "$turn -> response:missing"
      with "$turn -> intent:greeting"
      with "$turn -> answer:Hi, how may I help you?"
"#,
    )
    .expect("behavior rule data should parse");
    let mut graph = SubstitutionGraph::new()
        .with_link("turn:1", "role:user")
        .with_link("turn:1", "response:missing");

    let report = graph.create_link("turn:1", "prompt:Hi", &rules);
    let rust_handler_answer = FormalAiEngine.answer("Hi");

    assert_eq!(rust_handler_answer.intent, "greeting");
    assert!(graph.contains_link("turn:1", "intent:greeting"));
    assert!(graph.contains_link("turn:1", &format!("answer:{}", rust_handler_answer.answer)));
    assert_eq!(report.traces[0].rule_id, "rule_greeting_as_data");
}
