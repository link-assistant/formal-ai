// Every module and re-export of the agentic-coding capability.
//
// Issue #991: this list is `merge=union` (see `data/meta/merge-conflict-policy.lino`),
// so two branches that each add a capability produce a superset instead of a
// conflict. It lives apart from `mod.rs` because a union is only safe for a file
// whose every line is a list entry -- a union of two logic edits can compile and
// still be wrong. Regenerate with
// `rust-script scripts/normalize-ordered-lists.rs --write`.

pub mod algorithm_learning;
pub mod associative_learning;
mod capability_router;
pub mod change_request;
mod ci_workflow;
mod code_artifact;
pub mod code_rewrite_learning;
mod code_task;
pub(crate) mod command_reroute;
mod comparison;
mod contents_source;
mod conversation_recall;
pub mod corpus;
pub mod diagram;
mod directory_listing;
mod document_recipe;
pub mod dreaming_audit;
pub mod driver;
mod edit_scope;
mod evidence_record;
pub mod execution_learning;
pub mod explain;
pub mod external_benchmark_learning;
pub(crate) mod file_path_shape;
mod file_read;
mod file_summary;
mod final_result;
mod formalization_recipe;
pub mod formalize;
mod function_expectation;
mod general_execution;
pub mod general_planner;
mod git_commit;
pub mod google_trends_catalog;
pub mod google_trends_learning;
mod harness_envelope;
mod intent_router;
pub mod learning_report;
pub mod ledger;
pub(crate) mod lexicon;
mod line_range_move;
mod line_removal;
pub mod link_edit_rules;
mod literal_write_guard;
mod local_search;
mod markdown_section;
pub mod meaning_detail;
mod module_exports;
pub mod module_function;
pub mod mutating_action;
pub(crate) mod narration;
mod note_composition;
pub mod planner;
mod positional_edit;
mod prerequisite_recovery;
pub mod procedure;
mod progress;
pub mod question_catalog;
mod quote_nesting;
pub mod rebuild_plan;
pub mod repair_apply;
pub mod repair_loop;
pub mod repair_strategy;
mod replace_list;
mod report_issue;
mod report_script;
mod request_sequence;
pub mod requirement_extraction;
pub mod requirement_resolution;
pub mod restart_feedback;
pub mod routing_learning;
pub mod self_ast;
pub mod self_heal;
mod shell_command;
mod shell_command_policy;
mod shell_file_fallback;
pub mod source_links;
mod spelling;
mod stated_request;
pub mod statement_audit;
mod structured_document;
pub(crate) mod structured_edit;
pub mod system_diagram;
pub mod task_obligations;
mod task_structure;
mod test_assertion;
mod test_file_runner;
pub mod tool_result;
pub mod transcript_evidence;
mod web_research;
mod work_item_steps;
mod workspace_change;
mod workspace_computed_change;
mod workspace_inspection;
mod workspace_line_operation;
mod workspace_search;
mod workspace_setting;
mod write_request;

pub use associative_learning::{
    ASSOCIATIVE_LEARNING_PATH, ASSOCIATIVE_LEARNING_TASK, is_associative_learning_task,
};
pub use change_request::{CHANGE_PATH, CHANGE_TASK, is_change_request_task};
pub use ci_workflow::render_with as render_ci_workflow;
pub use code_rewrite_learning::{
    CODE_REWRITE_LEARNING_PATH, CODE_REWRITE_LEARNING_TASK, is_code_rewrite_learning_task,
};
pub use command_reroute::plan_symbolic_command_reroute;
pub use diagram::{DIAGRAM_PATH, DIAGRAM_TASK, is_diagram_task};
pub use dreaming_audit::{DREAMING_AUDIT_PATH, DREAMING_AUDIT_TASK, is_dreaming_audit_task};
pub use driver::{
    CORE_RECIPE_TOOLS, DRIVER_TOOLS, DriverOutcome, DriverToolStep, run_agentic_task,
    run_agentic_task_in, run_agentic_task_with_tools,
};
pub use execution_learning::{
    EXECUTION_LEARNING_PATH, EXECUTION_LEARNING_TASK, is_execution_learning_task,
};
pub use explain::{EXPLAIN_PATH, EXPLAIN_TASK, is_explain_task};
pub use external_benchmark_learning::EXTERNAL_BENCHMARK_LEARNING_PATH;
pub(crate) use file_read::supplied_file_answer;
pub use formalize::{
    CANONICAL_FISHERMAN_SYNOPSIS, FISHERMAN_DOC_ID, FormalizationSummary, FormalizedKnowledgeBase,
    PRIMITIVE_KINDS, coverage_line, formalize_text_to_links,
};
pub use google_trends_catalog::{
    GOOGLE_TRENDS_CATALOG_PATH, GOOGLE_TRENDS_CATALOG_TASK, is_google_trends_catalog_task,
};
pub use google_trends_learning::{
    GOOGLE_TRENDS_LEARNING_PATH, GOOGLE_TRENDS_LEARNING_TASK, is_google_trends_learning_task,
};
pub use learning_report::context_hierarchy_learning::{
    CONTEXT_HIERARCHY_LEARNING_PATH, CONTEXT_HIERARCHY_LEARNING_TASK,
    is_context_hierarchy_learning_task,
};
pub use learning_report::handler_precedence_learning::{
    HANDLER_PRECEDENCE_LEARNING_PATH, HANDLER_PRECEDENCE_LEARNING_TASK,
    is_handler_precedence_learning_task,
};
pub use learning_report::hardcoded_language_learning::{
    HARDCODED_LANGUAGE_LEARNING_PATH, HARDCODED_LANGUAGE_LEARNING_TASK,
    is_hardcoded_language_learning_task,
};
pub use learning_report::lexeme_import_learning::{
    LEXEME_IMPORT_LEARNING_PATH, LEXEME_IMPORT_LEARNING_TASK, is_lexeme_import_learning_task,
};
pub use learning_report::search_fusion_learning::{
    SEARCH_FUSION_LEARNING_PATH, SEARCH_FUSION_LEARNING_TASK, is_search_fusion_learning_task,
};
pub use learning_report::self_hosting_learning::{
    SELF_HOSTING_LEARNING_PATH, SELF_HOSTING_LEARNING_TASK, is_self_hosting_learning_task,
};
pub use learning_report::{LearningReport, REPORTS};
pub use ledger::{LEDGER_PATH, LEDGER_TASK, is_ledger_task};
pub use link_edit_rules::{
    LinkEditError, LinkEditReport, LinkEditRule, RuleDocument, RuleShape, apply_link_edit,
    insert_members_via_links, parse_rule_document, rule_shapes,
};
pub use meaning_detail::{
    MEANING_DETAIL_TASK, POTATO_DETAIL_TASK, concept_for_task, enrich_block, is_meaning_detail_task,
};
pub use planner::{
    AgenticPlan, CANONICAL_SOURCE_URL, KB_PATH, PlannedToolCall, SEARCH_QUERY, plan_chat_step,
};
pub use procedure::{COMPILED_PROCEDURE_PATH, compile_task as compile_procedure_task};
pub use question_catalog::{
    QUESTION_CATALOG_PATH, QUESTION_CATALOG_TASK, is_question_catalog_task,
};
pub use rebuild_plan::{REBUILD_PATH, REBUILD_TASK, is_rebuild_task};
pub use repair_strategy::{REPAIR_STRATEGY_PATH, REPAIR_STRATEGY_TASK, is_repair_strategy_task};
pub use requirement_resolution::{RequirementTarget, resolve_in, resolve_requirement_target};
pub use routing_learning::{
    ROUTING_LEARNING_PATH, ROUTING_LEARNING_TASK, is_routing_learning_task,
};
pub use self_ast::{AST_PATH, AST_TASK, ast_census, is_self_ast_task, render_ast_document};
pub use self_heal::{SELF_HEAL_PATH, SELF_HEAL_TASK, is_self_heal_task};
pub(crate) use shell_command::semantic_shell_command_for_task;
pub(crate) use shell_command::shell_command_for_task as repository_shell_command;
pub(crate) use shell_command_policy::{
    is_prose_word as repository_prose_word,
    named_shell_command_in_sentence as repository_named_command,
};
pub use source_links::{SOURCE_LINKS_PATH, SOURCE_LINKS_TASK, is_source_links_task};
pub use statement_audit::{STATEMENT_AUDIT_COMMAND, STATEMENT_AUDIT_PATH, is_statement_audit_task};
pub(crate) use workspace_inspection::asks_about_the_workspace as repository_inspection;
