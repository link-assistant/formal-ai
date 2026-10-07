//! The named route arms of the agentic planner cascade and their join with
//! the precedence the seed declares (plan 10 leaf 18, issue #1138).
//!
//! Moved out of `planner.rs` whole, so the cascade file keeps room for its
//! arms; the arms themselves still run in `planner.rs`.

/// The route arms of the cascade below, named and in run order.
///
/// Plan 10 leaf 18 (issue #1138): precedence is behaviour, so it is named in
/// `data/seed/planner-precedence.lino` where an edit is reviewable, and the
/// cascade here joins that order exactly as
/// `solver_dispatch::specialized_handlers` joins `handler-precedence.lino`
/// (issue #663). Each entry is `(function, arm)`; the arms stay heterogeneous
/// code — only their names and order are data.
pub(crate) const PLANNER_ROUTE_ARMS: &[(&str, &str)] = &[
    ("plan_chat_step_routes", "conversation_control_decline"),
    ("plan_chat_step_routes", "computer_use"),
    ("plan_chat_step_routes", "authoritative_literal_write"),
    ("plan_chat_step_routes", "program_contract"),
    ("plan_chat_step_routes", "evidence_record"),
    ("plan_settled_routes", "git_commit"),
    ("plan_settled_routes", "workspace_change"),
    ("plan_settled_routes", "module_function"),
    ("plan_settled_routes", "generated_source"),
    ("plan_settled_routes", "structured_edit"),
    ("plan_settled_routes", "structured_document"),
    ("plan_settled_routes", "statement_audit"),
    ("plan_settled_routes", "task_obligations"),
    ("plan_settled_routes", "literal_write"),
    ("plan_settled_routes", "algorithm_learning"),
    ("plan_settled_routes", "procedure"),
    ("plan_settled_routes", "learning_report"),
    ("plan_settled_routes", "code_artifact"),
    ("plan_settled_routes", "self_heal"),
    ("plan_settled_routes", "dreaming_audit"),
    ("plan_settled_routes", "self_ast"),
    ("plan_settled_routes", "source_links"),
    ("plan_settled_routes", "learning_ledger"),
    ("plan_settled_routes", "explain"),
    ("plan_settled_routes", "change_request"),
    ("plan_settled_routes", "repair_strategy"),
    ("plan_settled_routes", "rebuild_plan"),
    ("plan_settled_routes", "google_trends_learning"),
    ("plan_settled_routes", "google_trends_catalog"),
    ("plan_settled_routes", "question_catalog"),
    ("plan_settled_routes", "file_analysis"),
    ("plan_settled_routes", "report_flow"),
    ("plan_settled_routes", "conversation_recall"),
    ("plan_settled_routes", "follow_up_answer"),
    ("plan_settled_routes", "contextual_reference_clarification"),
    ("plan_settled_routes", "definition_followup"),
    ("plan_settled_routes", "intent_edit"),
    ("plan_settled_routes", "typed_file_read"),
    ("plan_settled_routes", "local_search"),
    ("plan_settled_routes", "comparison"),
    ("plan_settled_routes", "named_capability_table"),
    ("plan_settled_routes", "shell_command"),
    ("plan_settled_routes", "file_read"),
    ("plan_settled_routes", "formalization_recipe"),
    ("plan_settled_routes", "meaning_detail"),
    ("plan_settled_routes", "diagram"),
    ("plan_settled_routes", "workspace_inspection"),
    ("plan_settled_routes", "task_structure"),
    ("plan_settled_routes", "capability_table_named_or_local"),
    ("plan_settled_routes", "positional_edit_decline"),
    ("plan_settled_routes", "web_research_query"),
    ("plan_settled_routes", "intent_web_search"),
    ("plan_settled_routes", "code_search_fallback"),
    ("plan_settled_routes", "research_continuation"),
    ("plan_settled_routes", "latest_turn_answer"),
    ("plan_settled_routes", "note_composition"),
    ("plan_settled_routes", "general_change_fallback"),
    ("plan_settled_routes", "web_research_final"),
    ("plan_settled_routes", "capability_table_open_web"),
];

/// The coded arms with the seed's declared precedence joined against them.
///
/// Panics unless `planner-precedence.lino` names exactly these arms in exactly
/// this order — every arm present once, where the cascade runs it — so a seed
/// edit can never silently drop, duplicate, or reorder a route. Public so the
/// specification tests can pin the same join.
pub fn checked_route_precedence() -> &'static [&'static str] {
    static CELL: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    CELL.get_or_init(|| {
        let declared = crate::seed::planner_precedence();
        assert_eq!(
            declared.len(),
            PLANNER_ROUTE_ARMS.len(),
            "planner-precedence.lino lists {} arms but the cascade runs {}; \
             the seed must name every route arm exactly once",
            declared.len(),
            PLANNER_ROUTE_ARMS.len(),
        );
        let mut seen = std::collections::BTreeSet::new();
        for (index, (function, name)) in PLANNER_ROUTE_ARMS.iter().enumerate() {
            assert!(
                seen.insert(*name),
                "planner-precedence.lino names arm `{name}` more than once"
            );
            let declared_name = declared[index].as_str();
            assert_eq!(
                *name, declared_name,
                "planner-precedence.lino declares arm `{declared_name}` where the cascade \
                 runs `{function}`'s `{name}`; the seed must be an exact ordered permutation \
                 of the coded arms"
            );
        }
        declared.iter().map(String::as_str).collect()
    })
}
