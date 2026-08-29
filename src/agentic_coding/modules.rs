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
mod code_artifact;
pub mod code_rewrite_learning;
mod code_task;
pub(crate) mod command_reroute;
mod comparison;
mod conversation_recall;
pub mod corpus;
pub mod diagram;
mod directory_listing;
mod document_recipe;
pub mod dreaming_audit;
pub mod driver;
mod evidence_record;
pub mod execution_learning;
pub mod explain;
pub mod external_benchmark_learning;
pub(crate) mod file_path_shape;
mod file_read;
mod formalization_recipe;
pub mod formalize;
mod general_execution;
pub mod general_planner;
pub mod google_trends_catalog;
pub mod google_trends_learning;
mod intent_router;
pub mod learning_report;
pub mod ledger;
pub(crate) mod lexicon;
mod local_search;
pub mod meaning_detail;
pub mod mutating_action;
pub(crate) mod narration;
mod note_composition;
pub mod planner;
pub mod procedure;
mod progress;
pub mod question_catalog;
pub mod rebuild_plan;
pub mod repair_strategy;
mod report_issue;
mod report_script;
pub mod routing_learning;
pub mod self_ast;
pub mod self_heal;
mod shell_command;
mod shell_command_policy;
mod shell_file_fallback;
pub mod source_links;
pub mod statement_audit;
mod structured_edit;
mod task_structure;
pub mod tool_result;
mod web_research;
mod workspace_change;
mod workspace_inspection;
mod write_request;

pub use associative_learning::{
    is_associative_learning_task, ASSOCIATIVE_LEARNING_PATH, ASSOCIATIVE_LEARNING_TASK,
};
pub use change_request::{is_change_request_task, CHANGE_PATH, CHANGE_TASK};
pub use code_rewrite_learning::{
    is_code_rewrite_learning_task, CODE_REWRITE_LEARNING_PATH, CODE_REWRITE_LEARNING_TASK,
};
pub use command_reroute::plan_symbolic_command_reroute;
pub use diagram::{is_diagram_task, DIAGRAM_PATH, DIAGRAM_TASK};
pub use dreaming_audit::{is_dreaming_audit_task, DREAMING_AUDIT_PATH, DREAMING_AUDIT_TASK};
pub use driver::{
    run_agentic_task, run_agentic_task_in, DriverOutcome, DriverToolStep, DRIVER_TOOLS,
};
pub use execution_learning::{
    is_execution_learning_task, EXECUTION_LEARNING_PATH, EXECUTION_LEARNING_TASK,
};
pub use explain::{is_explain_task, EXPLAIN_PATH, EXPLAIN_TASK};
pub use external_benchmark_learning::EXTERNAL_BENCHMARK_LEARNING_PATH;
pub(crate) use file_read::supplied_file_answer;
pub use formalize::{
    coverage_line, formalize_text_to_links, FormalizationSummary, FormalizedKnowledgeBase,
    CANONICAL_FISHERMAN_SYNOPSIS, FISHERMAN_DOC_ID, PRIMITIVE_KINDS,
};
pub use google_trends_catalog::{
    is_google_trends_catalog_task, GOOGLE_TRENDS_CATALOG_PATH, GOOGLE_TRENDS_CATALOG_TASK,
};
pub use google_trends_learning::{
    is_google_trends_learning_task, GOOGLE_TRENDS_LEARNING_PATH, GOOGLE_TRENDS_LEARNING_TASK,
};
pub use learning_report::context_hierarchy_learning::{
    is_context_hierarchy_learning_task, CONTEXT_HIERARCHY_LEARNING_PATH,
    CONTEXT_HIERARCHY_LEARNING_TASK,
};
pub use learning_report::handler_precedence_learning::{
    is_handler_precedence_learning_task, HANDLER_PRECEDENCE_LEARNING_PATH,
    HANDLER_PRECEDENCE_LEARNING_TASK,
};
pub use learning_report::hardcoded_language_learning::{
    is_hardcoded_language_learning_task, HARDCODED_LANGUAGE_LEARNING_PATH,
    HARDCODED_LANGUAGE_LEARNING_TASK,
};
pub use learning_report::lexeme_import_learning::{
    is_lexeme_import_learning_task, LEXEME_IMPORT_LEARNING_PATH, LEXEME_IMPORT_LEARNING_TASK,
};
pub use learning_report::search_fusion_learning::{
    is_search_fusion_learning_task, SEARCH_FUSION_LEARNING_PATH, SEARCH_FUSION_LEARNING_TASK,
};
pub use learning_report::self_hosting_learning::{
    is_self_hosting_learning_task, SELF_HOSTING_LEARNING_PATH, SELF_HOSTING_LEARNING_TASK,
};
pub use learning_report::{LearningReport, REPORTS};
pub use ledger::{is_ledger_task, LEDGER_PATH, LEDGER_TASK};
pub use meaning_detail::{
    concept_for_task, enrich_block, is_meaning_detail_task, MEANING_DETAIL_TASK, POTATO_DETAIL_TASK,
};
pub use planner::{
    plan_chat_step, AgenticPlan, PlannedToolCall, CANONICAL_SOURCE_URL, KB_PATH, SEARCH_QUERY,
};
pub use procedure::{compile_task as compile_procedure_task, COMPILED_PROCEDURE_PATH};
pub use question_catalog::{
    is_question_catalog_task, QUESTION_CATALOG_PATH, QUESTION_CATALOG_TASK,
};
pub use rebuild_plan::{is_rebuild_task, REBUILD_PATH, REBUILD_TASK};
pub use repair_strategy::{is_repair_strategy_task, REPAIR_STRATEGY_PATH, REPAIR_STRATEGY_TASK};
pub use routing_learning::{
    is_routing_learning_task, ROUTING_LEARNING_PATH, ROUTING_LEARNING_TASK,
};
pub use self_ast::{ast_census, is_self_ast_task, render_ast_document, AST_PATH, AST_TASK};
pub use self_heal::{is_self_heal_task, SELF_HEAL_PATH, SELF_HEAL_TASK};
pub(crate) use shell_command::semantic_shell_command_for_task;
pub use source_links::{is_source_links_task, SOURCE_LINKS_PATH, SOURCE_LINKS_TASK};
pub use statement_audit::{is_statement_audit_task, STATEMENT_AUDIT_COMMAND, STATEMENT_AUDIT_PATH};
